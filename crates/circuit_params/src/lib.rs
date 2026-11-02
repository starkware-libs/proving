//! Building blocks for the circuits of one registry: a leaf (cairo-verifier) circuit per
//! verified Cairo trace log size in a range, plus the multiverifier that verifies their proofs —
//! all padded to a shared component-size target (the elementwise max over them), so one
//! circuit shape verifies proofs of any of them.
//!
//! The `circuit-params` binary uses these to emit the registry (see `main.rs`); tests use them to
//! derive its circuits' shape without committing anything.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use applicative_unpacking_verifier::unpacking_verifier::{
    UnpackingCircuitParams, build_applicative_unpacking_verifier_context,
};
use cairo_program_runner_lib::compiled_program_hash;
use cairo_program_runner_lib::types::HashFunc;
use circuit_cairo_verifier::all_components::all_components;
use circuit_cairo_verifier::statement::MEMORY_VALUES_LIMBS;
use circuit_cairo_verifier::utils::load_program;
use circuit_cairo_verifier::verify::{NON_QUERY_INFO_LEAK, build_cairo_verifier_circuit};
use circuit_common::finalize::{
    ComponentSizes, compute_padded_sizes, pad_to_targets, raw_component_sizes,
};
use circuit_common::preprocessed::{
    PreprocessedCircuit, layout_and_pcs_config, layout_from_component_sizes,
};
use circuit_prover::circuit_hash::circuit_hash_and_preprocessed_root;
use circuit_registry::{
    CircuitProofConfig, CircuitRegistry, DigestHex, LeafVerifier, LogSizes, Multiverifier,
    UnpackingVerifier,
};
use circuit_verifier::multiverifier::{
    SharedConfig, build_multiverifier_context_from_shared_config, shared_config,
};
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use leaf_prover::verifier_config::leaf_verifier_config;
use starknet_types_core::felt::Felt;
use stwo::core::fri::{FriConfig, FriParams};
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_prover::prover::ProverParameters;
use stwo_cairo_prover::witness::prelude::QM31;
use stwo_cairo_prover::witness::preprocessed_trace::generate_preprocessed_commitment_root;
use stwo_cairo_utils::file_utils::repo_root;

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
    /// The root of the Cairo preprocessed trace for each supported trace size.
    pub cairo_roots_cache: BTreeMap<u32, HashValue<QM31>>,
    /// The program every leaf proof of these circuits attests to.
    pub leaf_program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    /// The set of components used by the program. The leaf verifier circuit will assume the Cairo
    /// AIR contains exactly this set of components.
    pub cairo_component_names: Vec<String>,
    /// Prover parameters used to create the verified Cairo proofs.
    pub cairo_prover_params: ProverParameters,
    /// FRI params used to prove the leaf circuits
    pub circuit_fri_params: FriParams,
    /// Whether to add zk blinding to the circuit.
    pub add_zk_blinding: bool,
}

impl CircuitsBuilder {
    /// The verified proofs' PCS config at `trace_log_size`: `cairo_fri_params`, lifted to that
    /// trace.
    pub fn cairo_pcs_config(&self, trace_log_size: u32) -> PcsConfig {
        PcsConfig::from_fri_config(FriConfig {
            params: self.cairo_prover_params.fri_params,
            trace_log_size,
        })
    }

    /// Returns the preprocessed root of a verified Cairo proof of `trace_log_size` — the Cairo
    /// AIR's preprocessed trace, committed at `trace_log_size + log_blowup_factor`, which the leaf
    /// circuit then verifies against as a constant.
    pub fn cairo_preprocessed_root(&mut self, trace_log_size: u32) -> HashValue<QM31> {
        self.cairo_roots_cache
            .entry(trace_log_size)
            .or_insert_with(|| {
                let log_blowup_factor = self.cairo_prover_params.fri_params.log_blowup_factor;
                generate_preprocessed_commitment_root::<Blake2sMerkleChannel>(
                    log_blowup_factor,
                    self.cairo_prover_params.preprocessed_trace_variant,
                    trace_log_size,
                )
                .into()
            })
            .clone()
    }

    /// Builds the leaf verifier circuit topology for a verified Cairo proof of `trace_log_size`,
    /// with `preprocessed_root` baked in.
    pub fn build_leaf_context(
        &self,
        trace_log_size: u32,
        preprocessed_root: HashValue<QM31>,
    ) -> FinalizedContext<NoValue> {
        let verifier_config = leaf_verifier_config(
            self.cairo_prover_params.preprocessed_trace_variant,
            &self.cairo_pcs_config(trace_log_size),
            &self.cairo_component_names,
            self.leaf_program.clone(),
            preprocessed_root,
            self.add_zk_blinding.then_some(self.circuit_fri_params.n_queries + NON_QUERY_INFO_LEAK),
        );

        build_cairo_verifier_circuit(&verifier_config)
    }
}

pub fn circuits_builder_from_definition(definition: &RegistryDefinition) -> CircuitsBuilder {
    // The definition's params, recorded in the registry for the proving binaries to run with.
    let cairo_prover_params: ProverParameters = definition.cairo_prover_params;
    let circuit_fri_params: FriParams = definition.circuit_fri_params;

    definition.check_id_to_big_components();

    let program = load_program(&definition.program);

    CircuitsBuilder {
        cairo_roots_cache: Default::default(),
        leaf_program: program,
        cairo_component_names: definition.cairo_components.clone(),
        cairo_prover_params,
        circuit_fri_params,
        add_zk_blinding: definition.add_zk_blinding,
    }
}

/// Get the sizes of the Cairo verifying circuits for the given Cairo trace sizes.
///
/// Each leaf is dropped as soon as its sizes are read, so only one is ever in memory.
pub fn leaf_sizes(
    circuits_builder: &mut CircuitsBuilder,
    trace_log_sizes: impl IntoIterator<Item = u32>,
) -> BTreeMap<u32, (ComponentSizes, ComponentSizes)> {
    trace_log_sizes
        .into_iter()
        .map(|trace_log_size| {
            let cairo_root = circuits_builder.cairo_preprocessed_root(trace_log_size);
            let context = circuits_builder.build_leaf_context(trace_log_size, cairo_root);
            (trace_log_size, raw_and_padded_component_sizes(&context))
        })
        .collect()
}

pub fn create_registry_from_definition(definition: &RegistryDefinition) -> CircuitRegistry {
    let mut circuits_builder = circuits_builder_from_definition(definition);
    let trace_log_sizes = definition.min_trace_log_size..=definition.max_trace_log_size;

    let leaf_sizes = leaf_sizes(&mut circuits_builder, trace_log_sizes.clone());

    let circuit_log_blowup_factor = circuits_builder.circuit_fri_params.log_blowup_factor;

    let leaves_max_sizes = leaf_sizes
        .values()
        .map(|(_, padded)| padded.clone())
        .reduce(|max_sizes, sizes| max_sizes.elementwise_max(&sizes))
        .expect("the trace range is non-empty");
    let (target_sizes, preprocessed_multiverifier) = padded_shared_target(
        leaves_max_sizes,
        circuits_builder.circuit_fri_params,
        definition.pad_to_component_log_sizes.as_ref(),
    );

    // All circuits are padded to `target_sizes` and proven with
    // `circuit_log_blowup_factor`, so they share a single config.
    const CONFIG_ID: &str = "default";
    let circuit_proof_config =
        CircuitProofConfig::new(circuits_builder.circuit_fri_params, &target_sizes);
    let circuit_proof_configs =
        BTreeMap::from([(CONFIG_ID.to_string(), circuit_proof_config.clone())]);
    // Rebuild each leaf circuit, pad it to the shared target and record the hashes
    // that identify it.
    let leaf_verifiers = trace_log_sizes
        .clone()
        .map(|trace_log_size| {
            let cairo_root = circuits_builder.cairo_preprocessed_root(trace_log_size);
            let context = circuits_builder.build_leaf_context(trace_log_size, cairo_root);
            let (circuit_hash, preprocessed_root) = circuit_hash_and_preprocessed_root(
                &padded_preprocessed_circuit(context, &target_sizes),
                circuit_log_blowup_factor,
            );
            LeafVerifier {
                config: CONFIG_ID.to_string(),
                trace_log_size,
                circuit_hash: DigestHex::from(circuit_hash.0),
                preprocessed_root: DigestHex::from(preprocessed_root.0),
                zk_blinding: definition.add_zk_blinding,
            }
        })
        .collect::<Vec<_>>();

    // The multiverifier verifies two proofs of the leaf circuit, hence
    // `input_configs = [CONFIG_ID, CONFIG_ID]`.
    let (mv_circuit_hash, mv_preprocessed_root) =
        circuit_hash_and_preprocessed_root(&preprocessed_multiverifier, circuit_log_blowup_factor);
    let multiverifiers = vec![Multiverifier {
        config: CONFIG_ID.to_string(),
        input_configs: [CONFIG_ID.to_string(), CONFIG_ID.to_string()],
        circuit_hash: DigestHex::from(mv_circuit_hash.0),
        preprocessed_root: DigestHex::from(mv_preprocessed_root.0),
    }];

    // The unpacking verifier's whitelists are the hashes computed above, baked in as circuit
    // constants — so it is built last and its own hash lives in no whitelist. It must fit the
    // target those circuits fixed: it never grows it.
    let unpacking_verifiers = match definition.aggregator_wrapper_program_hash() {
        Some(aggregator_wrapper_program_hash) => {
            let leaf_set: Vec<HashValue<QM31>> = leaf_verifiers
                .iter()
                .map(|verifier| HashValue::from(verifier.circuit_hash.0))
                .collect();
            let internal_node_set = [HashValue::from(multiverifiers[0].circuit_hash.0)];
            let unpacking_context = unpacking_verifier_context_for_params(
                &circuit_proof_config,
                &UnpackingCircuitParams {
                    leaf_circuit_whitelist: &leaf_set,
                    internal_node_circuit_whitelist: &internal_node_set,
                    aggregator_wrapper_program_hash,
                },
            );
            assert_eq!(
                target_sizes.elementwise_max(&compute_padded_sizes(&unpacking_context)),
                target_sizes,
                "the unpacking verifier does not fit the shared target"
            );
            let (circuit_hash, preprocessed_root) = circuit_hash_and_preprocessed_root(
                &padded_preprocessed_circuit(unpacking_context, &target_sizes),
                circuit_log_blowup_factor,
            );
            vec![UnpackingVerifier {
                config: CONFIG_ID.to_string(),
                input_config: CONFIG_ID.to_string(),
                preprocessed_root: DigestHex::from(preprocessed_root.0),
                circuit_hash: DigestHex::from(circuit_hash.0),
            }]
        }
        None => vec![],
    };

    CircuitRegistry {
        cairo_prover_params: circuits_builder.cairo_prover_params,
        cairo_components: circuits_builder.cairo_component_names,
        circuit_proof_configs,
        leaf_verifiers,
        multiverifiers,
        unpacking_verifiers,
    }
}

/// The config every circuit proven under `config` is built and verified against (see
/// [`layout_and_pcs_config`]).
pub fn shared_config_from_circuit_config(config: &CircuitProofConfig) -> SharedConfig {
    let (preprocessed_column_log_sizes, pcs_config) =
        layout_and_pcs_config(&config.target_sizes(), config.fri_params);
    shared_config(preprocessed_column_log_sizes, pcs_config)
}

/// The multiverifier that verifies proofs of circuits proven under `config`.
///
/// It depends only on the config's padded sizes and FRI params, so no leaf circuit is built here;
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
/// production target, so the multiverifier — a function of the target and FRI params alone — is
/// the production one). `pad_to` must then be the exact final target: a fixpoint dominating the
/// registry's own leaf circuits (asserted, so the shape cannot silently diverge).
pub fn padded_shared_target(
    leaves_max_sizes: ComponentSizes,
    circuit_fri_params: FriParams,
    pad_to: Option<&LogSizes>,
) -> (ComponentSizes, PreprocessedCircuit) {
    let (target_sizes, multiverifier_context) =
        padded_shared_target_context(leaves_max_sizes, circuit_fri_params, pad_to);
    let preprocessed_multiverifier =
        padded_preprocessed_circuit(multiverifier_context, &target_sizes);
    (target_sizes, preprocessed_multiverifier)
}

/// [`padded_shared_target`] without materializing the multiverifier's preprocessed trace — for
/// consumers that need only the target sizes.
pub fn padded_shared_target_sizes(
    leaves_max_sizes: ComponentSizes,
    circuit_fri_params: FriParams,
    pad_to: Option<&LogSizes>,
) -> ComponentSizes {
    padded_shared_target_context(leaves_max_sizes, circuit_fri_params, pad_to).0
}

/// The converged target sizes and the multiverifier context built at them (not yet padded).
fn padded_shared_target_context(
    leaves_max_sizes: ComponentSizes,
    circuit_fri_params: FriParams,
    pad_to: Option<&LogSizes>,
) -> (ComponentSizes, FinalizedContext<NoValue>) {
    let start = match pad_to {
        Some(pad_to) => leaves_max_sizes.elementwise_max(&pad_to.to_component_sizes()),
        None => leaves_max_sizes,
    };
    let (target_sizes, multiverifier_context) = shared_target_fixpoint(start, circuit_fri_params);
    if let Some(pad_to) = pad_to {
        assert_eq!(
            &LogSizes::of(&target_sizes),
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
    circuit_fri_params: FriParams,
) -> (ComponentSizes, FinalizedContext<NoValue>) {
    loop {
        let multiverifier_context = multiverifier_context_from_circuit_config(
            &CircuitProofConfig::new(circuit_fri_params, &target_sizes),
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
    /// The verified Cairo proofs' prover params. Shape the leaf circuits (via the FRI params and
    /// preprocessed trace variant) and are recorded in the registry for the leaf prover to run
    /// with.
    ///
    /// Must set `opt_n_id_to_big_components` to `n`, with [`Self::cairo_components`] listing
    /// exactly the first `n` `memory_id_to_big` components, in any order.
    pub cairo_prover_params: ProverParameters,
    /// The circuit proofs' FRI params, recorded in the registry for the leaf prover and the
    /// recursive tree to run with. Their lifting size is not configurable: each circuit is built
    /// as if lifted to `trace_size + blowup`.
    pub circuit_fri_params: FriParams,
    /// The compiled program the leaf circuit verifies (the leaf prover's `--program`).
    pub program: std::path::PathBuf,
    /// The set of components used by the program. The circuit will assume the Cairo AIR contains
    /// exactly this set of components.
    pub cairo_components: Vec<String>,
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
    /// Loads the definition at `relative_definition_path`. `relative_definition_path`
    /// and the program paths inside the definition are assumed to be relative to `repo_root`.
    pub fn load(repo_root: &Path, relative_definition_path: &str) -> Self {
        let mut definition: RegistryDefinition =
            read_params(&repo_root.join(relative_definition_path));
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

    /// Asserts that [`Self::cairo_components`] lists exactly the first
    /// `opt_n_id_to_big_components` `memory_id_to_big` components: the prover fills those, and the
    /// leaf circuit verifies exactly the listed ones.
    fn check_id_to_big_components(&self) {
        let n_id_to_big_components = self.cairo_prover_params.opt_n_id_to_big_components.expect(
            "The definition's prover params must set opt_n_id_to_big_components: the leaf circuit \
             verifies a fixed set of memory_id_to_big components",
        );
        let listed_id_to_big_components: BTreeSet<&str> = self
            .cairo_components
            .iter()
            .map(String::as_str)
            .filter(|name| name.starts_with("memory_id_to_big"))
            .collect();
        let all_id_to_big_components: Vec<&str> = all_components::<NoValue>()
            .into_keys()
            .filter(|name| name.starts_with("memory_id_to_big"))
            .collect();
        let first_n_id_to_big_components: Option<BTreeSet<&str>> = all_id_to_big_components
            .get(..n_id_to_big_components)
            .map(|names| names.iter().copied().collect());
        assert!(
            first_n_id_to_big_components.as_ref() == Some(&listed_id_to_big_components),
            "The definition lists the memory_id_to_big components \
             {listed_id_to_big_components:?}, but its prover params set \
             opt_n_id_to_big_components to {n_id_to_big_components}"
        );
    }
}

/// The (blake) program hash of the compiled program at `path`.
fn blake_program_hash(path: &Path) -> Felt {
    compiled_program_hash(path, HashFunc::Blake)
        .unwrap_or_else(|err| panic!("Cannot hash {}: {err}", path.display()))
}

/// Regenerates the given registry and asserts it matches the committed. Run with `FIX=1`
/// to regenerate.
pub fn assert_committed_registry_is_up_to_date(definition_path: &str, committed_path: &str) {
    let definition = RegistryDefinition::load(&repo_root(), definition_path);
    let absolute_committed_path = repo_root().join(committed_path);

    let registry = create_registry_from_definition(&definition);
    let registry_serialized = serde_json::to_string_pretty(&registry).unwrap() + "\n";
    if std::env::var("FIX") == Ok("1".to_string()) {
        fs::write(absolute_committed_path, registry_serialized).unwrap();
    } else {
        let existing_registry = fs::read_to_string(absolute_committed_path).unwrap();
        assert_eq!(
            existing_registry, registry_serialized,
            "{committed_path} does not match this run's output; run with FIX=1 to regenerate"
        );
    }
}
