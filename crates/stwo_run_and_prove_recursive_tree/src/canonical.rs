//! One-time setup of the single circuit shape that verifies every layer of the recursive tree.
//!
//! Because the leaf cairo-verifier circuit and the multiverifier circuit are both padded to the
//! same [`TARGET_PADDING_SIZES`], they share the same preprocessed-trace
//! layout and `trace_log_size`. As a result a *single* multiverifier circuit shape can verify both
//! leaf proofs (layer 1) and multiverifier proofs (every layer above), with one [`SharedConfig`]
//! and one preprocessed root. This is what makes the reduction homogeneous and lets an unpaired
//! entry be carried up to a higher layer unchanged.

use std::path::PathBuf;

use circuit_cairo_verifier::privacy::get_pcs_config;
use circuit_cairo_verifier::utils::load_program;
use circuit_cairo_verifier::verify::{
    CairoVerifierConfig, build_cairo_verifier_circuit, get_preprocessed_root,
};
use circuit_common::finalize::{ComponentSizes, pad_to_targets};
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_multiverifier::verify::{SharedConfig, build_multiverifier_context};
use circuit_prover::prover::{BaseColumnPool, SimdBackend};
use circuit_verifier::statement::circuit_verifier_proof_config;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use leaf_prover::prove_leaf::leaf_verifier_config;
use stwo::core::pcs::PcsConfig;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use tracing::{Level, info, span};

use crate::RecursiveTreeError;

/// Log blowup factor of the outer circuit proof.
pub const MULTIVERIFIER_LOG_BLOWUP_FACTOR: u32 = 1;

/// The circuit trace log size every layer's proof reaches (see [`TARGET_PADDING_SIZES`]).
pub const MULTIVERIFIER_TRACE_LOG_SIZE: u32 = 23;

/// PCS config for proving each layer. MUST equal the config the leaf circuit proofs were produced
/// with: this constant is the single source of truth for the tree's circuit PCS shape, and the
/// backend must pass it (via the leaf prover's `circuit_prover_params_json`) when producing
/// leaves.
// TODO(yairv): Consider taking this from the backend via configuration (alongside the leaf
// bootloader program), so the backend passes one config to both `leaf_prover` and the recursive
// tree.
pub const MULTIVERIFIER_PCS_CONFIG: PcsConfig =
    get_pcs_config(MULTIVERIFIER_TRACE_LOG_SIZE, MULTIVERIFIER_LOG_BLOWUP_FACTOR);

/// Common per-component padding target applied to BOTH the leaf cairo-verifier circuit and the
/// multiverifier circuit, so they share one preprocessed-trace layout and a single proof shape
/// verifies every layer.
// TODO(Yair): Update according to the maximum component sizes possible in the given circuits to
// be verified.
pub const TARGET_PADDING_SIZES: ComponentSizes = ComponentSizes {
    eq: 1 << 20,
    qm31_ops: 1 << 23,
    m31_to_u32: 1 << 20,
    triple_xor: 1 << 19,
    blake_g_gate: 1 << 23,
};

/// Everything that is identical for every node of the tree. Built once at startup and threaded by
/// reference into every `reduce_pair` call.
pub struct CanonicalCircuit {
    /// The preprocessed multiverifier circuit — the shape every layer's proof is generated
    /// against.
    pub preprocessed_multiverifier: PreprocessedCircuit,
    /// Config shared by all proofs being verified by the multiverifier. Its `proof_config` is also
    /// used to deserialize leaf / intermediate proofs from disk.
    pub shared_config: SharedConfig,
    /// Reused across all `prove_circuit_assignment` calls.
    pub base_column_pool: BaseColumnPool<SimdBackend>,
}

impl CanonicalCircuit {
    /// Builds the canonical circuit shape and all the configuration derived from it.
    pub fn build() -> Result<Self, RecursiveTreeError> {
        let _span = span!(Level::INFO, "CanonicalCircuit::build").entered();

        // 1. The leaf cairo-verifier circuit (padded + preprocessed). Its preprocessed trace gives
        //    the column count / log sizes that describe a *child* circuit proof.
        let preprocessed_leaf = build_preprocessed_leaf_circuit();

        // 2. The shared config for verifying a child circuit proof.
        let preprocessed_column_log_sizes = preprocessed_leaf.preprocessed_trace.log_sizes();
        let proof_config = circuit_verifier_proof_config(
            &preprocessed_column_log_sizes,
            &MULTIVERIFIER_PCS_CONFIG,
        );
        let shared_config = SharedConfig {
            pcs_config: MULTIVERIFIER_PCS_CONFIG,
            proof_config,
            preprocessed_column_log_sizes,
        };

        // 3. The multiverifier circuit shape, padded to the same target as the leaf circuit.
        let mut multiverifier_context =
            build_multiverifier_context(&preprocessed_leaf, MULTIVERIFIER_PCS_CONFIG);
        pad_to_targets(&mut multiverifier_context, TARGET_PADDING_SIZES);
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

        info!(
            trace_log_size = preprocessed_multiverifier.trace_log_size,
            "Canonical multiverifier circuit ready."
        );
        Ok(Self {
            preprocessed_multiverifier,
            shared_config,
            base_column_pool: BaseColumnPool::new(),
        })
    }
}

/// The cairo-verifier (leaf) circuit padded to [`TARGET_PADDING_SIZES`] and preprocessed.
fn build_preprocessed_leaf_circuit() -> PreprocessedCircuit {
    let mut leaf_context = build_unpadded_leaf_context();
    pad_to_targets(&mut leaf_context, TARGET_PADDING_SIZES);
    PreprocessedCircuit::preprocess_circuit(&mut leaf_context)
}

/// The unpadded cairo-verifier (leaf) circuit context, shaped exactly as `leaf_prover` shapes it
/// when proving the leaf simple bootloader (see `leaf_prover::prove_leaf`).
pub fn build_unpadded_leaf_context() -> FinalizedContext<NoValue> {
    build_cairo_verifier_circuit(&leaf_cairo_verifier_config())
}

/// Path of the compiled leaf simple bootloader — the program every leaf proof attests to.
/// TEMPORARY: read from this crate's `test_data`; production will receive it via configuration.
fn leaf_bootloader_program_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/leaf_simple_bootloader_compiled.json")
}

/// The slice of the leaf's Cairo prover parameters the canonical circuit shape depends on.
#[derive(serde::Deserialize)]
struct LeafCairoProverParams {
    /// PCS config of the *inner* Cairo proof the leaf circuit verifies.
    pcs_config: PcsConfig,
    preprocessed_trace: PreProcessedTraceVariant,
}

/// The Cairo prover parameters the leaf's inner proof is produced with — loaded from the same
/// file the golden e2e proves leaves with, so the canonical circuit shape cannot drift from it.
/// TEMPORARY: read from `leaf_prover`'s test data; production will receive it via configuration
/// (see the TODO on [`MULTIVERIFIER_PCS_CONFIG`]).
fn leaf_cairo_prover_params() -> LeafCairoProverParams {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../leaf_prover/tests/data/cairo_prover_params_canonical_small.json");
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

/// The inner canonical-small cairo proof's trace log size (its widest column, 2^20), mirroring
/// `circuit_cairo_verifier::privacy`'s `PRIVACY_CAIRO_TRACE_LOG_SIZE`. The leaf verifier config's
/// `lifting_log_size` is this plus the log blowup factor.
const LEAF_CAIRO_TRACE_LOG_SIZE: u32 = 20;

/// The [`CairoVerifierConfig`] of the leaf circuit (via `leaf_prover`'s shared builder), for the
/// leaf simple bootloader under the canonical-small test setup.
fn leaf_cairo_verifier_config() -> CairoVerifierConfig {
    let leaf_cairo_params = leaf_cairo_prover_params();
    // The circuit verifier's `ProofConfig` requires the explicit
    // `lifting_log_size = trace_log_size + log_blowup_factor` (the params file stores `0`,
    // meaningful only to the prover); mirror `prove_leaf`, which overrides it from the proof.
    let mut pcs_config = leaf_cairo_params.pcs_config;
    pcs_config.lifting_log_size =
        LEAF_CAIRO_TRACE_LOG_SIZE + pcs_config.fri_config.log_blowup_factor;
    leaf_verifier_config(
        leaf_cairo_params.preprocessed_trace,
        &pcs_config,
        load_program(&leaf_bootloader_program_path()),
        get_preprocessed_root(pcs_config.lifting_log_size),
    )
}

/// Builds the multiverifier circuit topology (structure-only) from a leaf circuit padded only with
/// the *default* next-power-of-two padding (NOT [`TARGET_PADDING_SIZES`]), and applies no target
/// padding to the multiverifier itself. Exposed for the regression test that derives and locks
/// [`TARGET_PADDING_SIZES`] — mirroring `circuit_multiverifier::verify_test`'s `None`-padding path,
/// which must not depend on the very constant being derived.
#[cfg(test)]
pub fn build_unpadded_multiverifier_context() -> FinalizedContext<NoValue> {
    let mut leaf_context = build_unpadded_leaf_context();
    let preprocessed_leaf = PreprocessedCircuit::preprocess_circuit(&mut leaf_context);
    build_multiverifier_context(&preprocessed_leaf, MULTIVERIFIER_PCS_CONFIG)
}
