//! Building blocks for the circuits of one registry: a leaf (cairo-verifier) circuit per
//! verified Cairo trace log size in a range, plus the multiverifier that verifies their proofs —
//! all padded to a shared component-size target (the elementwise max over them), so one
//! circuit shape verifies proofs of any of them.
//!
//! The `circuit-params` binary uses these to emit the registry (see `main.rs`).

use std::path::Path;
use std::sync::Arc;

use circuit_cairo_verifier::statement::MEMORY_VALUES_LIMBS;
use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;
use circuit_common::finalize::{
    ComponentSizes, compute_padded_sizes, pad_to_targets, raw_component_sizes,
};
use circuit_common::preprocessed::{PreprocessedCircuit, layout_from_component_sizes};
use circuit_multiverifier::verify::{
    build_multiverifier_context, build_multiverifier_context_from_shared_config, shared_config,
};
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use leaf_prover::prove_leaf::leaf_verifier_config;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_prover::witness::prelude::QM31;
use stwo_cairo_prover::witness::preprocessed_trace::generate_preprocessed_commitment_root;

/// A stand-in for the Cairo preprocessed root, for the passes that only read component sizes: the
/// root ends up in the circuit's constant VALUES, never in its topology, and committing the real
/// one is the most expensive thing here.
pub const DUMMY_PREPROCESSED_ROOT: [u32; 8] = [0; 8];

/// Reads a prover params JSON file.
pub fn read_params<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    let json = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Cannot read prover params from {}: {err}", path.display()));
    serde_json::from_str(&json)
        .unwrap_or_else(|err| panic!("Cannot parse prover params from {}: {err}", path.display()))
}

/// Non-padded row counts and padded sizes of a circuit context's AIR components.
pub fn component_sizes(context: &FinalizedContext<NoValue>) -> (ComponentSizes, ComponentSizes) {
    (raw_component_sizes(context), compute_padded_sizes(context))
}

/// Builds the circuits of one registry definition: holds the definition's inputs (program, prover
/// params), so that a leaf verifier circuit can be specified and built from just the trace log
/// size of the Cairo proof it verifies.
pub struct CircuitBuilder {
    pub preprocessed_trace: PreProcessedTraceVariant,
    /// The program every leaf proof of these circuits attests to.
    pub program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    /// FRI config of the verified Cairo proofs.
    pub cairo_fri_config: FriConfig,
}

impl CircuitBuilder {
    /// The verified proofs' PCS config at `trace_log_size`: `cairo_fri_config`, lifted to that
    /// trace.
    pub fn cairo_pcs_config(&self, trace_log_size: u32) -> PcsConfig {
        PcsConfig::from_fri_and_trace_size(self.cairo_fri_config, trace_log_size)
    }

    /// Returns the preprocessed root of a verified Cairo proof of `trace_log_size` — the Cairo
    /// AIR's preprocessed trace, committed at `trace_log_size + log_blowup_factor`, which the leaf
    /// circuit then verifies against as a constant.
    pub fn cairo_preprocessed_root(&self, trace_log_size: u32) -> HashValue<QM31> {
        let log_blowup_factor = self.cairo_fri_config.log_blowup_factor;
        generate_preprocessed_commitment_root::<Blake2sM31MerkleChannel>(
            log_blowup_factor,
            self.preprocessed_trace,
            trace_log_size + log_blowup_factor,
        )
        .into()
    }

    /// Builds the leaf verifier circuit topology for a verified Cairo proof of `trace_log_size`,
    /// with `preprocessed_root` baked in.
    pub fn build_context(
        &self,
        trace_log_size: u32,
        preprocessed_root: HashValue<QM31>,
    ) -> FinalizedContext<NoValue> {
        let verifier_config = leaf_verifier_config(
            self.preprocessed_trace,
            &self.cairo_pcs_config(trace_log_size),
            self.program.clone(),
            preprocessed_root,
        );

        build_cairo_verifier_circuit(&verifier_config)
    }

    /// Builds the multiverifier over the *default-padded* leaf circuit for `trace_log_size` —
    /// only for the sizes report; the registry pads the leaf to the shared target first (see
    /// [`shared_target_fixpoint`]).
    pub fn build_multiverifier_context_for_trace(
        &self,
        trace_log_size: u32,
        circuit_fri_config: FriConfig,
    ) -> FinalizedContext<NoValue> {
        let mut leaf_context = self.build_context(trace_log_size, DUMMY_PREPROCESSED_ROOT.into());

        let preprocessed_leaf = PreprocessedCircuit::preprocess_circuit(&mut leaf_context);
        build_multiverifier_context(
            &preprocessed_leaf,
            PcsConfig::from_fri_and_trace_size(
                circuit_fri_config,
                preprocessed_leaf.trace_log_size,
            ),
        )
    }
}

/// Pads `context` to the shared `target_sizes` and preprocesses it.
pub fn padded_preprocessed_circuit(
    mut context: FinalizedContext<NoValue>,
    target_sizes: &ComponentSizes,
) -> PreprocessedCircuit {
    pad_to_targets(&mut context, target_sizes);
    PreprocessedCircuit::preprocess_circuit(&mut context)
}

/// Shared target sizes for a set of circuits, with the multiverifier padded to them.
///
/// The target and the multiverifier's sizes are a fixpoint of each other: the multiverifier
/// verifies proofs of the TARGET-PADDED leaf circuit, and the target is the elementwise max over
/// every leaf circuit and that very multiverifier. Starts from the leaves' max (`target_sizes`)
/// and iterates — a larger target can grow the multiverifier, growing the target again.
///
/// The verified proofs' layout is derived from the target alone (`layout_from_component_sizes`),
/// exactly as the recursive tree builds its multiverifier.
pub fn shared_target_fixpoint(
    mut target_sizes: ComponentSizes,
    circuit_fri_config: FriConfig,
) -> (ComponentSizes, PreprocessedCircuit) {
    loop {
        let preprocessed_column_log_sizes = layout_from_component_sizes(&target_sizes);
        let trace_log_size =
            *preprocessed_column_log_sizes.values().max().expect("the layout is non-empty");
        let multiverifier_context = build_multiverifier_context_from_shared_config(&shared_config(
            preprocessed_column_log_sizes,
            PcsConfig::from_fri_and_trace_size(circuit_fri_config, trace_log_size),
        ));
        let grown_sizes =
            target_sizes.elementwise_max(&compute_padded_sizes(&multiverifier_context));
        if grown_sizes == target_sizes {
            let preprocessed_multiverifier =
                padded_preprocessed_circuit(multiverifier_context, &target_sizes);
            return (target_sizes, preprocessed_multiverifier);
        }
        target_sizes = grown_sizes;
    }
}
