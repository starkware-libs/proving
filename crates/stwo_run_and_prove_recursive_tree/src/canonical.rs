//! One-time setup of the single circuit shape that verifies every layer of the recursive tree.
//!
//! Because the leaf cairo-verifier circuit and the multiverifier circuit are both padded to the
//! same target component sizes — taken from the circuit registry — they share the
//! same preprocessed-trace layout and `trace_log_size`. As a result a *single* multiverifier
//! circuit shape can verify both leaf proofs (layer 1) and multiverifier proofs (every layer
//! above), with one [`SharedConfig`] and one preprocessed root. This is what makes the reduction
//! homogeneous and lets an unpaired entry be carried up to a higher layer unchanged.

use circuit_common::finalize::{ComponentSizes, pad_to_targets};
use circuit_common::preprocessed::{PreprocessedCircuit, layout_from_component_sizes};
use circuit_multiverifier::verify::{
    SharedConfig, build_multiverifier_context_from_shared_config, shared_config,
};
use circuit_prover::circuit_hash::preprocessed_circuit_hash;
use circuit_prover::prover::{BaseColumnPool, SimdBackend};
use circuit_registry::{CircuitRegistry, DigestHex};
use stwo::core::pcs::PcsConfig;
use tracing::{Level, info, span};

use crate::RecursiveTreeError;

/// Everything that is identical for every node of the tree. Built once at startup and threaded by
/// reference into every `reduce_pair` call.
pub struct CanonicalCircuit {
    /// The preprocessed multiverifier circuit — the shape every layer's proof is generated
    /// against.
    pub preprocessed_multiverifier: PreprocessedCircuit,
    /// Config shared by all proofs being verified by the multiverifier. Its `proof_config` is also
    /// used to deserialize leaf / intermediate proofs from disk.
    pub shared_config: SharedConfig,
    /// The registry's padding target, applied to both the leaf and multiverifier circuits so one
    /// proof shape verifies every layer.
    pub target_sizes: ComponentSizes,
    /// Reused across all `prove_circuit_assignment` calls.
    pub base_column_pool: BaseColumnPool<SimdBackend>,
}

impl CanonicalCircuit {
    /// Builds the canonical circuit shape and all the configuration derived from it.
    ///
    /// `registry` is the only input: its multiverifier's config supplies the FRI config the leaf
    /// circuit proofs were produced with and the padding target every circuit here is built to,
    /// and its entry supplies the hash the built multiverifier is checked against.
    pub fn build(registry: &CircuitRegistry) -> Result<Self, RecursiveTreeError> {
        let _span = span!(Level::INFO, "CanonicalCircuit::build").entered();

        let multiverifier_entry = registry.multiverifier()?;
        let circuit_proof_config = registry.config(&multiverifier_entry.config)?;
        let target_sizes = circuit_proof_config.target_sizes();

        // 1. The shared config for verifying a child circuit proof — a proof the multiverifier
        //    verifies, a leaf proof at layer 1 and a multiverifier proof above: the layout every
        //    circuit in the registry has, derived from the shared target.
        let preprocessed_column_log_sizes = layout_from_component_sizes(&target_sizes);
        let trace_log_size =
            *preprocessed_column_log_sizes.values().max().expect("the layout is non-empty");
        let circuit_pcs_config =
            PcsConfig::from_fri_and_trace_size(circuit_proof_config.fri_config, trace_log_size);
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
            || preprocessed_multiverifier.trace_log_size != trace_log_size
        {
            return Err(RecursiveTreeError::PaddingParity);
        }

        // 4. The built multiverifier must hash to the registry's entry — the trust anchor every
        //    internal node is held to. Catches registry/circuit drift before any proving.
        // TODO(yairv): consider reading the hash off the first fold's proof instead (the prover
        // computes it anyway), trading this preprocessed-trace commitment for a later failure.
        let circuit_hash = DigestHex::from(
            preprocessed_circuit_hash(
                &preprocessed_multiverifier,
                circuit_pcs_config.fri_config.log_blowup_factor,
            )
            .0,
        );
        if circuit_hash != multiverifier_entry.circuit_hash {
            return Err(RecursiveTreeError::MultiverifierCircuitHash {
                expected: multiverifier_entry.circuit_hash,
                got: circuit_hash,
            });
        }

        info!(
            trace_log_size = preprocessed_multiverifier.trace_log_size,
            "Canonical multiverifier circuit ready."
        );
        Ok(Self {
            preprocessed_multiverifier,
            shared_config,
            target_sizes,
            base_column_pool: BaseColumnPool::new(),
        })
    }
}
