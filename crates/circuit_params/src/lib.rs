//! Building blocks for the circuits of one registry: a leaf (cairo-verifier) circuit per
//! verified Cairo trace log size in a range, plus the multiverifier that verifies their proofs —
//! all padded to a shared component-size target (the elementwise max over them), so one
//! circuit shape verifies proofs of any of them.
//!
//! The `circuit-params` binary uses these to emit the registry (see `main.rs`); tests use them to
//! derive its circuits' shape (e.g. [`RegistryDefinition::shared_target_sizes`]) without
//! committing
//! anything.

use std::path::Path;
use std::sync::Arc;

use circuit_cairo_verifier::statement::MEMORY_VALUES_LIMBS;
use circuit_cairo_verifier::utils::load_program;
use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;
use circuit_common::finalize::{
    ComponentSizes, compute_padded_sizes, pad_to_targets, raw_component_sizes,
};
use circuit_common::preprocessed::{PreprocessedCircuit, layout_from_component_sizes};
use circuit_multiverifier::verify::{
    build_multiverifier_context, build_multiverifier_context_from_shared_config, shared_config,
};
use circuit_registry::LogSizes;
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use leaf_prover::prove_leaf::leaf_verifier_config;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_prover::prover::ProverParameters;
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

/// Runs [`shared_target_fixpoint`] from the leaves' max, raised to `pad_to` when given: that lets
/// a cheap registry adopt a larger one's circuit shape (e.g. small Cairo proofs padded to the
/// production target, so the multiverifier — a function of the target and FRI config alone — is
/// the production one). `pad_to` must then be the exact final target: a fixpoint dominating the
/// registry's own leaf circuits (asserted, so the shape cannot silently diverge).
pub fn padded_shared_target(
    leaves_max_sizes: ComponentSizes,
    circuit_fri_config: FriConfig,
    pad_to: Option<&LogSizes>,
) -> (ComponentSizes, PreprocessedCircuit) {
    let (target_sizes, multiverifier_context) =
        padded_shared_target_context(leaves_max_sizes, circuit_fri_config, pad_to);
    let preprocessed_multiverifier =
        padded_preprocessed_circuit(multiverifier_context, &target_sizes);
    (target_sizes, preprocessed_multiverifier)
}

/// [`padded_shared_target`] without materializing the multiverifier's preprocessed trace — for
/// consumers that need only the target sizes.
pub fn padded_shared_target_sizes(
    leaves_max_sizes: ComponentSizes,
    circuit_fri_config: FriConfig,
    pad_to: Option<&LogSizes>,
) -> ComponentSizes {
    padded_shared_target_context(leaves_max_sizes, circuit_fri_config, pad_to).0
}

/// The converged target sizes and the multiverifier context built at them (not yet padded).
fn padded_shared_target_context(
    leaves_max_sizes: ComponentSizes,
    circuit_fri_config: FriConfig,
    pad_to: Option<&LogSizes>,
) -> (ComponentSizes, FinalizedContext<NoValue>) {
    let start = match pad_to {
        Some(pad_to) => leaves_max_sizes.elementwise_max(&pad_to.into()),
        None => leaves_max_sizes,
    };
    let (target_sizes, multiverifier_context) = shared_target_fixpoint(start, circuit_fri_config);
    if let Some(pad_to) = pad_to {
        assert_eq!(
            &LogSizes::from(&target_sizes),
            pad_to,
            "the pad-to target must be a fixpoint dominating the registry's leaf circuits"
        );
    }
    (target_sizes, multiverifier_context)
}

/// Shared target sizes for a set of circuits, and the corresponding multiverifier — returned raw
/// so the caller chooses whether to pad and preprocess it ([`padded_preprocessed_circuit`]) or
/// only read its sizes.
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
) -> (ComponentSizes, FinalizedContext<NoValue>) {
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
            return (target_sizes, multiverifier_context);
        }
        target_sizes = grown_sizes;
    }
}

/// Inputs for the supported-circuits registry generator.
#[derive(serde::Deserialize)]
pub struct RegistryDefinition {
    /// The verified Cairo proofs' prover params. Shape the leaf circuits (via the FRI config and
    /// preprocessed trace variant) and are recorded in the registry for the leaf prover to run
    /// with.
    pub cairo_prover_params_json: std::path::PathBuf,
    /// The circuit proofs' FRI config, recorded in the registry for the leaf prover and the
    /// recursive tree to run with. Their lifting size is not configurable: each circuit is built
    /// as if lifted to `trace_size + blowup`.
    pub circuit_fri_config_json: std::path::PathBuf,
    /// The compiled program the leaf circuit verifies (the leaf prover's `--program`).
    pub program: std::path::PathBuf,
    /// Smallest verified trace log size (inclusive). Bounded below by the preprocessed-trace
    /// variant's sequence-column log height (20 for canonical_small, 25 for canonical).
    pub min_trace_log_size: u32,
    /// Largest verified trace log size (inclusive).
    pub max_trace_log_size: u32,
    /// Pads the shared target to another registry's shape (see [`padded_shared_target`]).
    pub pad_to_component_log_sizes: Option<LogSizes>,
}

impl RegistryDefinition {
    /// Loads `circuit_registry_definitions/<name>/definition.json`, resolving its paths against
    /// `repo_root`.
    pub fn load(repo_root: &Path, name: &str) -> Self {
        let mut definition: RegistryDefinition = read_params(
            &repo_root.join(format!("circuit_registry_definitions/{name}/definition.json")),
        );
        for path in [
            &mut definition.cairo_prover_params_json,
            &mut definition.circuit_fri_config_json,
            &mut definition.program,
        ] {
            *path = repo_root.join(&*path);
        }
        definition
    }

    pub fn cairo_params(&self) -> ProverParameters {
        read_params(&self.cairo_prover_params_json)
    }

    pub fn circuit_fri_config(&self) -> FriConfig {
        read_params(&self.circuit_fri_config_json)
    }

    /// The registry's shared padding target and the multiverifier padded to it, from the
    /// definition alone: the elementwise max over the leaf circuits of the trace range, closed
    /// under the multiverifier fixpoint. Builds circuit topologies with a dummy Cairo root (sizes
    /// are root-independent), one at a time — no commitment and no preprocessed-trace values, so
    /// this is the cheap part of registry generation.
    pub fn shared_target_sizes(&self) -> ComponentSizes {
        let cairo_params = self.cairo_params();
        let circuit_builder = CircuitBuilder {
            preprocessed_trace: cairo_params.preprocessed_trace,
            program: load_program(&self.program),
            cairo_fri_config: cairo_params.fri_config,
        };
        let leaves_max_sizes = (self.min_trace_log_size..=self.max_trace_log_size)
            .map(|trace_log_size| {
                let context =
                    circuit_builder.build_context(trace_log_size, DUMMY_PREPROCESSED_ROOT.into());
                compute_padded_sizes(&context)
            })
            .reduce(|a, b| a.elementwise_max(&b))
            .expect("the trace range is non-empty");
        padded_shared_target_sizes(
            leaves_max_sizes,
            self.circuit_fri_config(),
            self.pad_to_component_log_sizes.as_ref(),
        )
    }
}
