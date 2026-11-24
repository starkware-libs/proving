//! One-time setup of the single circuit shape that verifies every layer of the recursive tree.
//!
//! Because the leaf cairo-verifier circuit and the multiverifier circuit are both padded to the
//! same target component sizes — taken from the circuit registry — they share the
//! same preprocessed-trace layout and `trace_log_size`. As a result a *single* multiverifier
//! circuit shape can verify both leaf proofs (layer 1) and multiverifier proofs (every layer
//! above), with one [`SharedConfig`] and one preprocessed root. This is what makes the reduction
//! homogeneous and lets an unpaired entry be carried up to a higher layer unchanged.

use circuit_common::finalize::{ComponentSizes, pad_to_targets};
use circuit_common::preprocessed::{PreprocessedCircuit, layout_and_pcs_config};
use circuit_prover::prover::{BaseColumnPool, SimdBackend};
use circuit_registry::{CircuitRegistry, DigestHex};
use circuit_verifier::multiverifier::{
    SharedConfig, build_multiverifier_context_from_shared_config, shared_config,
};
use starknet_types_core::felt::Felt as FieldElement252;
use stwo::core::pcs::PcsConfig;
use tracing::{Level, info, span};

use crate::RecursiveTreeError;

/// Everything that is identical for every node of the tree. Built once at startup and threaded by
/// reference into every `reduce_pair` call.
pub struct CanonicalCircuit {
    /// The preprocessed multiverifier circuit — the shape every layer's proof is generated
    /// against.
    pub preprocessed_multiverifier: PreprocessedCircuit,
    /// Config shared by all proofs being verified by the multiverifier. Its `proof_config` also
    /// deserializes the leaf proofs.
    pub shared_config: SharedConfig,
    /// The registry's padding target, applied to both the leaf and multiverifier circuits so one
    /// proof shape verifies every layer.
    pub target_sizes: ComponentSizes,
    /// The registry's Blake2s identity of the multiverifier circuit, which every Blake2s fold
    /// proof is held to.
    pub multiverifier_circuit_hash: DigestHex,
    /// How a root is proven for a Cairo circuit verifier. `None` when the registry's multiverifier
    /// has no `cairo_verifier_root`, and then a tree cannot be folded into such a root.
    pub cairo_verifier_root: Option<CairoVerifierRootConfig>,
    /// Reused across all `prove_circuit_assignment` calls.
    pub base_column_pool: BaseColumnPool<SimdBackend>,
}

/// A root for a Cairo circuit verifier: the same multiverifier circuit, proven under the
/// registry's `cairo_verifier_root` config.
pub struct CairoVerifierRootConfig {
    /// The PCS config the root is proven with, which both builds of the Cairo circuit verifier
    /// pin.
    pub pcs_config: PcsConfig,
    /// The Blake2s identity a root for the default build is held to.
    pub blake_circuit_hash: DigestHex,
    /// The felt identity a root for the `poseidon252_verifier` build is held to.
    pub poseidon_circuit_hash: FieldElement252,
}

impl CanonicalCircuit {
    /// Builds the canonical circuit shape and all the configuration derived from it.
    ///
    /// `registry` is the only input: its multiverifier's config supplies the FRI params the leaf
    /// circuit proofs were produced with and the padding target every circuit here is built to,
    /// and its entry supplies the hashes the fold proofs are checked against and, when it has a
    /// root for a Cairo circuit verifier, that root's config.
    pub fn build(registry: &CircuitRegistry) -> Result<Self, RecursiveTreeError> {
        let _span = span!(Level::INFO, "CanonicalCircuit::build").entered();

        let multiverifier_entry = registry.multiverifier()?;
        let circuit_proof_config = registry.config(&multiverifier_entry.config)?;
        let target_sizes = circuit_proof_config.target_sizes();

        // 1. The shared config for verifying a child circuit proof — a proof the multiverifier
        //    verifies, a leaf proof at layer 1 and a multiverifier proof above: the layout every
        //    circuit in the registry has, derived from the shared target.
        let (preprocessed_column_log_sizes, circuit_pcs_config) =
            layout_and_pcs_config(&target_sizes, circuit_proof_config.fri_params);
        let trace_log_size =
            *preprocessed_column_log_sizes.values().max().expect("the layout is non-empty");
        let shared_config = shared_config(preprocessed_column_log_sizes, circuit_pcs_config);

        // 2. The multiverifier circuit shape, padded to the registry's target.
        let mut multiverifier_context =
            build_multiverifier_context_from_shared_config(&shared_config);
        pad_to_targets(&mut multiverifier_context, &target_sizes);
        let preprocessed_multiverifier =
            PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);

        // 3. Homogeneity check: the multiverifier must itself have the layout it was built to
        //    verify — otherwise one `proof_config` / `preprocessed_column_log_sizes` cannot verify
        //    BOTH a leaf child proof (layer 1) and a multiverifier child proof (every layer above).
        if preprocessed_multiverifier.preprocessed_trace.log_sizes()
            != shared_config.preprocessed_column_log_sizes
            || preprocessed_multiverifier.trace_log_size() != trace_log_size
        {
            return Err(RecursiveTreeError::PaddingParity);
        }
        info!(
            trace_log_size = preprocessed_multiverifier.trace_log_size(),
            "Canonical multiverifier circuit ready."
        );

        // 4. A root for a Cairo circuit verifier proves the same padded circuit, so its config must
        //    have the multiverifier's sizes. Only its FRI params may differ.
        let cairo_verifier_root = match &multiverifier_entry.cairo_verifier_root {
            Some(root) => {
                let root_proof_config = registry.config(&root.config)?;
                if root_proof_config.component_log_sizes != circuit_proof_config.component_log_sizes
                {
                    return Err(RecursiveTreeError::CairoVerifierRootSizes {
                        expected: circuit_proof_config.component_log_sizes.clone(),
                        got: root_proof_config.component_log_sizes.clone(),
                    });
                }
                let (_, pcs_config) =
                    layout_and_pcs_config(&target_sizes, root_proof_config.fri_params);
                Some(CairoVerifierRootConfig {
                    pcs_config,
                    blake_circuit_hash: root.blake_circuit_hash,
                    poseidon_circuit_hash: root.poseidon_circuit_hash,
                })
            }
            None => None,
        };

        Ok(Self {
            preprocessed_multiverifier,
            shared_config,
            target_sizes,
            multiverifier_circuit_hash: multiverifier_entry.circuit_hash,
            cairo_verifier_root,
            base_column_pool: BaseColumnPool::new(),
        })
    }
}
