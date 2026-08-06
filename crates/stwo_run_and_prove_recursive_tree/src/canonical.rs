//! One-time setup of the single circuit shape that verifies every layer of the recursive tree.
//!
//! Because the leaf cairo-verifier circuit and the multiverifier circuit are both padded to the
//! same target component sizes — taken from the circuit registry — they share the
//! same preprocessed-trace layout and `trace_log_size`. As a result a *single* multiverifier
//! circuit shape can verify both leaf proofs (layer 1) and multiverifier proofs (every layer
//! above), with one [`SharedConfig`] and one preprocessed root. This is what makes the reduction
//! homogeneous and lets an unpaired entry be carried up to a higher layer unchanged.

use std::path::PathBuf;

use circuit_cairo_verifier::utils::load_program;
use circuit_cairo_verifier::verify::{
    CairoVerifierConfig, build_cairo_verifier_circuit, get_preprocessed_root,
};
use circuit_common::finalize::{ComponentSizes, pad_to_targets};
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_multiverifier::verify::{SharedConfig, build_multiverifier_context};
use circuit_prover::circuit_hash::preprocessed_circuit_hash;
use circuit_prover::prover::{BaseColumnPool, SimdBackend};
use circuit_registry::{CircuitRegistry, DigestHex};
use circuit_verifier::statement::circuit_verifier_proof_config;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use leaf_prover::prove_leaf::leaf_verifier_config;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
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
    /// The family's padding target, applied to both the leaf and multiverifier circuits so one
    /// proof shape verifies every layer.
    pub target_sizes: ComponentSizes,
    /// Reused across all `prove_circuit_assignment` calls.
    pub base_column_pool: BaseColumnPool<SimdBackend>,
}

impl CanonicalCircuit {
    /// Builds the canonical circuit shape and all the configuration derived from it.
    ///
    /// `circuit_pcs_config` is the PCS config the leaf circuit proofs were produced with (the
    /// leaf prover's `circuit_prover_params_json`). Its `lifting_log_size` is overridden to the
    /// target-padded circuit's `trace_log_size + log_blowup_factor` — the only valid value for
    /// the canonical circuit shape.
    ///
    /// `registry` supplies the padding target every circuit here is built to, and the hash the
    /// built multiverifier is checked against.
    pub fn build(
        circuit_pcs_config: PcsConfig,
        registry: &CircuitRegistry,
    ) -> Result<Self, RecursiveTreeError> {
        let _span = span!(Level::INFO, "CanonicalCircuit::build").entered();

        let multiverifier_entry = registry.multiverifier()?;
        let target_sizes = registry.config(&multiverifier_entry.config)?.target_sizes();

        // 1. The leaf cairo-verifier circuit (padded + preprocessed). Its preprocessed trace gives
        //    the column count / log sizes of the circuit proofs the multiverifier verifies — its
        //    children in the tree, a leaf proof at layer 1 and a multiverifier proof above. Built
        //    at the largest trace size the registry covers, but any other would have done: every
        //    leaf of the family shares that layout.
        let preprocessed_leaf =
            build_preprocessed_leaf_circuit(registry.max_leaf_trace_log_size()?, &target_sizes);

        let circuit_pcs_config = PcsConfig::from_fri_and_trace_size(
            circuit_pcs_config.fri_config,
            preprocessed_leaf.trace_log_size,
        );

        // 2. The shared config for verifying a child circuit proof.
        let preprocessed_column_log_sizes = preprocessed_leaf.preprocessed_trace.log_sizes();
        let proof_config =
            circuit_verifier_proof_config(&preprocessed_column_log_sizes, &circuit_pcs_config);
        let shared_config = SharedConfig {
            pcs_config: circuit_pcs_config,
            proof_config,
            preprocessed_column_log_sizes,
        };

        // 3. The multiverifier circuit shape, padded to the same target as the leaf circuit.
        let mut multiverifier_context =
            build_multiverifier_context(&preprocessed_leaf, circuit_pcs_config);
        pad_to_targets(&mut multiverifier_context, &target_sizes);
        let preprocessed_multiverifier =
            PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);

        // 4. Homogeneity check: the leaf and multiverifier circuits must share the SAME
        //    preprocessed trace layout (column ids + per-column log sizes + overall trace_log_size)
        //    so that one `proof_config` / `preprocessed_column_log_sizes` verifies BOTH a leaf
        //    child proof and a multiverifier child proof.
        if preprocessed_leaf.preprocessed_trace.log_sizes()
            != preprocessed_multiverifier.preprocessed_trace.log_sizes()
            || preprocessed_leaf.trace_log_size != preprocessed_multiverifier.trace_log_size
        {
            return Err(RecursiveTreeError::PaddingParity);
        }

        // 5. The built multiverifier must hash to the registry's entry — the trust anchor every
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

/// The cairo-verifier (leaf) circuit for a Cairo proof of `trace_log_size`, padded to
/// `target_sizes` and preprocessed.
fn build_preprocessed_leaf_circuit(
    trace_log_size: u32,
    target_sizes: &ComponentSizes,
) -> PreprocessedCircuit {
    let mut leaf_context = build_unpadded_leaf_context(trace_log_size);
    pad_to_targets(&mut leaf_context, target_sizes);
    PreprocessedCircuit::preprocess_circuit(&mut leaf_context)
}

/// The unpadded cairo-verifier (leaf) circuit context for a Cairo proof of `trace_log_size`, shaped
/// exactly as `leaf_prover` shapes it when proving the leaf simple bootloader (see
/// `leaf_prover::prove_leaf`).
pub fn build_unpadded_leaf_context(trace_log_size: u32) -> FinalizedContext<NoValue> {
    build_cairo_verifier_circuit(&leaf_cairo_verifier_config(trace_log_size))
}

/// Path of the compiled leaf simple bootloader — the program every leaf proof attests to.
/// TEMPORARY: read from this crate's `test_data`; production will receive it via configuration.
fn leaf_bootloader_program_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/leaf_simple_bootloader_compiled.json")
}

/// The slice of the leaf's Cairo prover parameters the canonical circuit shape depends on.
#[derive(serde::Deserialize)]
struct LeafCairoProverParams {
    /// FRI config of the *inner* Cairo proof the leaf circuit verifies.
    fri_config: FriConfig,
    preprocessed_trace: PreProcessedTraceVariant,
}

/// The Cairo prover parameters the leaf's inner proof is produced with — loaded from the same
/// file the golden e2e proves leaves with, so the canonical circuit shape cannot drift from it.
/// TEMPORARY: read from `leaf_prover`'s test data; production will receive it via configuration
/// (like the circuit prover params file).
fn leaf_cairo_prover_params() -> LeafCairoProverParams {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../leaf_prover/tests/data/cairo_prover_params_canonical_small.json");
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

/// The [`CairoVerifierConfig`] of the leaf circuit that verifies a Cairo proof of `trace_log_size`
/// (via `leaf_prover`'s shared builder), for the leaf simple bootloader under the canonical-small
/// test setup.
fn leaf_cairo_verifier_config(trace_log_size: u32) -> CairoVerifierConfig {
    let leaf_cairo_params = leaf_cairo_prover_params();
    // The circuit verifier's `ProofConfig` requires the explicit
    // `lifting_log_size = trace_log_size + log_blowup_factor` (the params file stores `0`,
    // meaningful only to the prover); mirror `prove_leaf`, which overrides it from the proof.
    let lifting_log_size = trace_log_size + leaf_cairo_params.fri_config.log_blowup_factor;
    leaf_verifier_config(
        leaf_cairo_params.preprocessed_trace,
        &PcsConfig { fri_config: leaf_cairo_params.fri_config, lifting_log_size },
        load_program(&leaf_bootloader_program_path()),
        get_preprocessed_root(lifting_log_size),
    )
}
