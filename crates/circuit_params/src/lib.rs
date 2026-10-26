//! Building blocks for the circuits of one registry, listed in `main.rs`.
//!
//! The `circuit-params` binary uses these to emit the registry; tests use them to derive its
//! circuits' shape without committing anything.

use std::path::Path;
use std::sync::Arc;

use applicative_unpacking_verifier::unpacking_verifier::{
    UnpackingCircuitParams, build_applicative_unpacking_verifier_context,
};
use cairo_program_runner_lib::compiled_program_hash;
use cairo_program_runner_lib::types::HashFunc;
use circuit_cairo_verifier::statement::MEMORY_VALUES_LIMBS;
use circuit_cairo_verifier::verify::{NON_QUERY_INFO_LEAK, build_cairo_verifier_circuit};
use circuit_common::finalize::{
    ComponentSizes, compute_padded_sizes, pad_to_targets, raw_component_sizes,
};
use circuit_common::preprocessed::{
    PreprocessedCircuit, layout_and_pcs_config, layout_from_component_sizes,
};
use circuit_multiverifier::verify::{
    SharedConfig, build_multiverifier_context_from_shared_config, shared_config,
};
use circuit_registry::{CircuitProofConfig, LogSizes};
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use leaf_prover::prove_leaf::leaf_verifier_config;
use starknet_types_core::felt::Felt;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_prover::prover::ProverParameters;
use stwo_cairo_prover::witness::prelude::QM31;
use stwo_cairo_prover::witness::preprocessed_trace::generate_preprocessed_commitment_root;

/// Reads a JSON file.
pub fn read_params<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    let json = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Cannot read {}: {err}", path.display()));
    serde_json::from_str(&json)
        .unwrap_or_else(|err| panic!("Cannot parse {}: {err}", path.display()))
}

/// Non-padded row counts and padded sizes of a circuit context's AIR components.
pub fn raw_and_padded_component_sizes(
    context: &FinalizedContext<NoValue>,
) -> (ComponentSizes, ComponentSizes) {
    (raw_component_sizes(context), compute_padded_sizes(context))
}

/// Builds the circuits of one registry definition: holds the definition's inputs (program, prover
/// params), so that a leaf verifier circuit can be specified and built from just the trace log
/// size of the Cairo proof it verifies.
pub struct CircuitsBuilder {
    /// Preprocessed-trace variant of the verified Cairo proofs.
    pub cairo_preprocessed_trace_variant: PreProcessedTraceVariant,
    /// The program every leaf proof of these circuits attests to.
    pub leaf_program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    /// FRI config of the verified Cairo proofs.
    pub cairo_fri_config: FriConfig,
    /// FRI config used to prove the leaf circuits
    pub circuit_fri_config: FriConfig,
    /// Whether to add zk blinding to the circuit.
    pub add_zk_blinding: bool,
}

impl CircuitsBuilder {
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
            self.cairo_preprocessed_trace_variant,
            trace_log_size + log_blowup_factor,
        )
        .into()
    }

    /// Builds the leaf verifier circuit topology for a verified Cairo proof of `trace_log_size`,
    /// with `preprocessed_root` baked in.
    pub fn build_leaf_context(
        &self,
        trace_log_size: u32,
        preprocessed_root: HashValue<QM31>,
    ) -> FinalizedContext<NoValue> {
        let verifier_config = leaf_verifier_config(
            self.cairo_preprocessed_trace_variant,
            &self.cairo_pcs_config(trace_log_size),
            self.leaf_program.clone(),
            preprocessed_root,
            self.add_zk_blinding.then_some(self.circuit_fri_config.n_queries + NON_QUERY_INFO_LEAK),
        );

        build_cairo_verifier_circuit(&verifier_config)
    }
}

/// The config every circuit proven under `config` is built and verified against (see
/// [`layout_and_pcs_config`]).
pub fn shared_config_from_circuit_config(config: &CircuitProofConfig) -> SharedConfig {
    let (preprocessed_column_log_sizes, pcs_config) =
        layout_and_pcs_config(&config.target_sizes(), config.fri_config);
    shared_config(preprocessed_column_log_sizes, pcs_config)
}

/// The multiverifier that verifies proofs of circuits proven under `config`.
///
/// It depends only on the config's padded sizes and FRI config, so no leaf circuit is built here;
/// the recursive tree builds its multiverifier the same way.
pub fn multiverifier_context_from_circuit_config(
    config: &CircuitProofConfig,
) -> FinalizedContext<NoValue> {
    build_multiverifier_context_from_shared_config(&shared_config_from_circuit_config(config))
}

/// The unpacking verifier built under `config`. It must fit the config's target.
pub fn unpacking_verifier_context_for_params(
    config: &CircuitProofConfig,
    params: &UnpackingCircuitParams<'_>,
) -> FinalizedContext<NoValue> {
    build_applicative_unpacking_verifier_context(&shared_config_from_circuit_config(config), params)
}

/// Pads `context` to the shared `target_sizes` and preprocesses it.
///
/// Homogeneity: every circuit padded to one target must come out with that target's preprocessed
/// layout — the layout the multiverifier verifies — so a registry's circuits cannot silently
/// diverge in shape.
pub fn padded_preprocessed_circuit(
    mut context: FinalizedContext<NoValue>,
    target_sizes: &ComponentSizes,
) -> PreprocessedCircuit {
    pad_to_targets(&mut context, target_sizes);
    let padded = PreprocessedCircuit::preprocess_circuit(&mut context);
    assert_eq!(
        padded.preprocessed_trace.log_sizes(),
        layout_from_component_sizes(target_sizes),
        "the padded circuit does not have the target's preprocessed layout"
    );
    padded
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
/// verifies proofs of TARGET-PADDED circuits, and the target is the elementwise max over it and
/// the leaf circuits. Iterates from the leaves' max, since a larger target can grow it.
pub fn shared_target_fixpoint(
    mut target_sizes: ComponentSizes,
    circuit_fri_config: FriConfig,
) -> (ComponentSizes, FinalizedContext<NoValue>) {
    loop {
        let multiverifier_context = multiverifier_context_from_circuit_config(
            &CircuitProofConfig::new(circuit_fri_config, &target_sizes),
        );
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
    pub cairo_prover_params: ProverParameters,
    /// The circuit proofs' FRI config, recorded in the registry for the leaf prover and the
    /// recursive tree to run with. Their lifting size is not configurable: each circuit is built
    /// as if lifted to `trace_size + blowup`.
    pub circuit_fri_config: FriConfig,
    /// The compiled program the leaf circuit verifies (the leaf prover's `--program`).
    pub program: std::path::PathBuf,
    /// Smallest verified trace log size (inclusive). Bounded below by the preprocessed-trace
    /// variant's sequence-column log height (20 for canonical_small, 25 for canonical).
    pub min_trace_log_size: u32,
    /// Largest verified trace log size (inclusive).
    pub max_trace_log_size: u32,
    /// Pads the shared target to another registry's shape (see [`padded_shared_target`]).
    pub pad_to_component_log_sizes: Option<LogSizes>,
    /// Whether to add ZK blinding to the leaf circuits
    pub add_zk_blinding: bool,
    /// The compiled aggregator wrapper program, for registries of the applicative flow: its
    /// (blake) program hash is baked into the unpacking verifier. Absent for registries without
    /// that flow, which then have no unpacking verifier.
    pub aggregator_wrapper_program: Option<std::path::PathBuf>,
}

impl RegistryDefinition {
    /// Loads `circuit_registry_definitions/<name>/definition.json`, resolving its program paths
    /// against `repo_root`.
    pub fn load(repo_root: &Path, name: &str) -> Self {
        let mut definition: RegistryDefinition = read_params(
            &repo_root.join(format!("circuit_registry_definitions/{name}/definition.json")),
        );
        definition.program = repo_root.join(&definition.program);
        if let Some(wrapper) = &mut definition.aggregator_wrapper_program {
            *wrapper = repo_root.join(&*wrapper);
        }
        definition
    }

    /// The (blake) program hash of the definition's aggregator wrapper, baked into the unpacking
    /// verifier, or `None` when the definition names none.
    pub fn aggregator_wrapper_program_hash(&self) -> Option<Felt> {
        self.aggregator_wrapper_program.as_deref().map(blake_program_hash)
    }
}

/// The (blake) program hash of the compiled program at `path`.
fn blake_program_hash(path: &Path) -> Felt {
    compiled_program_hash(path, HashFunc::Blake)
        .unwrap_or_else(|err| panic!("Cannot hash {}: {err}", path.display()))
}
