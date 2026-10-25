//! Builds the circuits a registry definition determines, and reports or records them.
//!
//! A circuit registry specifies which circuits may appear in a recursive proof tree:
//! 1. a leaf (cairo-verifier) circuit per verified Cairo trace log size in the definition's range,
//! 2. the multiverifier, which verifies two proofs of the registry's circuits,
//! 3. the applicative unpacking verifier, when the definition specifies the aggregator wrapper.
//!
//! All are padded to a shared component-size target (the elementwise max over them), so one
//! circuit shape verifies proofs of any of them.
//!
//! The circuits, hence their identities, are functions of the verified Cairo proofs' prover params,
//! the circuit proofs' prover params, and the verified program. The params are taken from the
//! definition and recorded in the registry, which is then the only configuration the proving
//! binaries need.
//!
//! By default, writes a human-readable report: each component's padded log size and usage, per
//! trace size for the leaf circuit and once (at the largest size) for the multiverifier.
//!
//! With `--registry`, instead writes the registry JSON: the params above, the shared target sizes
//! and every circuit's hash — everything a binary proving these circuits needs.

use std::path::PathBuf;
use std::process::ExitCode;

<<<<<<< HEAD
use circuit_common::finalize::ComponentSizes;
||||||| 0dbaec2ac
use circuit_cairo_verifier::utils::load_program;
use circuit_common::finalize::ComponentSizes;
=======
use applicative_unpacking_verifier::unpacking_verifier::UnpackingCircuitParams;
use circuit_cairo_verifier::utils::load_program;
use circuit_common::finalize::{ComponentSizes, compute_padded_sizes};
>>>>>>> a4b96ad32727e219b4a5a2c022f5ab419819045f
use circuit_params::{
<<<<<<< HEAD
    RegistryDefinition, circuits_builder_from_definition, create_registry_from_definition,
    leaf_sizes, multiverifier_context_for_sizes, raw_and_padded_component_sizes, read_params,
||||||| 0dbaec2ac
    CircuitsBuilder, RegistryDefinition, multiverifier_context_for_sizes,
    padded_preprocessed_circuit, padded_shared_target, raw_and_padded_component_sizes, read_params,
=======
    CircuitsBuilder, RegistryDefinition, multiverifier_context_from_circuit_config,
    padded_preprocessed_circuit, padded_shared_target, raw_and_padded_component_sizes, read_params,
    unpacking_verifier_context_for_params,
>>>>>>> a4b96ad32727e219b4a5a2c022f5ab419819045f
};
<<<<<<< HEAD
||||||| 0dbaec2ac
use circuit_prover::circuit_hash::circuit_hash_and_preprocessed_root;
use circuit_registry::{
    CircuitProofConfig, CircuitRegistry, DigestHex, LeafVerifier, Multiverifier,
};
use circuits::blake::HashValue;
=======
use circuit_prover::circuit_hash::circuit_hash_and_preprocessed_root;
use circuit_registry::{
    CircuitProofConfig, CircuitRegistry, DigestHex, LeafVerifier, Multiverifier, UnpackingVerifier,
};
use circuits::blake::HashValue;
>>>>>>> a4b96ad32727e219b4a5a2c022f5ab419819045f
use clap::Parser;
use stwo::core::fri::FriParams;
use stwo_cairo_utils::binary_utils::run_binary;

#[derive(Parser)]
struct Args {
    /// The registry definition JSON (see [`RegistryDefinition`]); the paths inside are resolved
    /// against the working directory, so run from the repo root for the committed definitions.
    #[clap(long)]
    definition: PathBuf,
    /// Write the registry JSON instead of the human-readable sizes report.
    #[clap(long)]
    registry: bool,
    /// Path to write the output to. If omitted, prints to stdout.
    #[clap(long)]
    output_path: Option<PathBuf>,
}

/// The fraction (as a percentage) of the padded (power-of-two) component that is actually used.
fn usage_percent(size: usize, padded_size: usize) -> f64 {
    100.0 * size as f64 / padded_size as f64
}

/// One line with each component's padded log size and usage (fraction of the padded power-of-two
/// that the non-padded rows fill).
fn format_sizes(raw: &ComponentSizes, padded: &ComponentSizes) -> String {
    let component = |name: &str, raw_size: usize, padded_size: usize| {
        format!(
            "{name}:(log: {}, usage = {:.0}%)",
            padded_size.ilog2(),
            usage_percent(raw_size, padded_size),
        )
    };
    format!(
        "{} {} {} {} {}",
        component("eq", raw.eq, padded.eq),
        component("qm31_ops", raw.qm31_ops, padded.qm31_ops),
        component("m31_to_u32", raw.m31_to_u32, padded.m31_to_u32),
        component("triple_xor", raw.triple_xor, padded.triple_xor),
        component("blake_g_gate", raw.blake_g_gate, padded.blake_g_gate),
    )
}

fn main() -> ExitCode {
    run_binary(run, "circuit_params")
}

fn run() -> Result<(), String> {
    let args = Args::parse();

    let definition: RegistryDefinition = read_params(&args.definition);
<<<<<<< HEAD
||||||| 0dbaec2ac
    let program = load_program(&definition.program);

    // The definition's params, each read from the file the proving binaries run with.
    let cairo_params: ProverParameters = definition.cairo_params();
    let circuit_fri_config: FriConfig = definition.circuit_fri_config();

    let circuits_builder = CircuitsBuilder {
        cairo_preprocessed_trace_variant: cairo_params.preprocessed_trace,
        leaf_program: program,
        cairo_fri_config: cairo_params.fri_config,
        circuit_fri_config,
        add_zk_blinding: definition.add_zk_blinding,
    };
    let trace_log_sizes = definition.min_trace_log_size..=definition.max_trace_log_size;

    // The verified Cairo proofs' preprocessed roots, committed once per trace size and baked into
    // the leaf circuit both passes build.
    let cairo_roots: BTreeMap<u32, HashValue<QM31>> = trace_log_sizes
        .clone()
        .map(|trace_log_size| {
            (trace_log_size, circuits_builder.cairo_preprocessed_root(trace_log_size))
        })
        .collect();

    // Pass 1: every leaf circuit's component sizes. Each leaf is dropped as soon as its sizes are
    // read, so only one is ever in memory.
    let leaf_sizes: BTreeMap<u32, (ComponentSizes, ComponentSizes)> = trace_log_sizes
        .clone()
        .map(|trace_log_size| {
            let context = circuits_builder
                .build_leaf_context(trace_log_size, cairo_roots[&trace_log_size].clone());
            (trace_log_size, raw_and_padded_component_sizes(&context))
        })
        .collect();
=======
    let program = load_program(&definition.program);

    // The definition's params, recorded in the registry for the proving binaries to run with.
    let cairo_params: ProverParameters = definition.cairo_prover_params;
    let circuit_fri_config: FriConfig = definition.circuit_fri_config;

    let circuits_builder = CircuitsBuilder {
        cairo_preprocessed_trace_variant: cairo_params.preprocessed_trace_variant,
        leaf_program: program,
        cairo_fri_config: cairo_params.fri_config,
        circuit_fri_config,
        add_zk_blinding: definition.add_zk_blinding,
    };
    let trace_log_sizes = definition.min_trace_log_size..=definition.max_trace_log_size;

    // The verified Cairo proofs' preprocessed roots, committed once per trace size and baked into
    // the leaf circuit both passes build.
    let cairo_roots: BTreeMap<u32, HashValue<QM31>> = trace_log_sizes
        .clone()
        .map(|trace_log_size| {
            (trace_log_size, circuits_builder.cairo_preprocessed_root(trace_log_size))
        })
        .collect();

    // Pass 1: every leaf circuit's component sizes. Each leaf is dropped as soon as its sizes are
    // read, so only one is ever in memory.
    let leaf_sizes: BTreeMap<u32, (ComponentSizes, ComponentSizes)> = trace_log_sizes
        .clone()
        .map(|trace_log_size| {
            let context = circuits_builder
                .build_leaf_context(trace_log_size, cairo_roots[&trace_log_size].clone());
            (trace_log_size, raw_and_padded_component_sizes(&context))
        })
        .collect();
>>>>>>> a4b96ad32727e219b4a5a2c022f5ab419819045f

    let output = if args.registry {
        let registry = create_registry_from_definition(&definition);

<<<<<<< HEAD
||||||| 0dbaec2ac
        let leaves_max_sizes = leaf_sizes
            .values()
            .map(|(_, padded)| padded.clone())
            .reduce(|max_sizes, sizes| max_sizes.elementwise_max(&sizes))
            .expect("the trace range is non-empty");
        let (target_sizes, preprocessed_multiverifier) = padded_shared_target(
            leaves_max_sizes,
            circuit_fri_config,
            definition.pad_to_component_log_sizes.as_ref(),
        );

        // All circuits are padded to `target_sizes` and proven with
        // `circuit_log_blowup_factor`, so they share a single config.
        const CONFIG_ID: &str = "default";
        let circuit_proof_configs = BTreeMap::from([(
            CONFIG_ID.to_string(),
            CircuitProofConfig {
                fri_config: circuit_fri_config,
                component_log_sizes: (&target_sizes).into(),
            },
        )]);
        // Pass 2: rebuild each leaf circuit, pad it to the shared target and record the hashes
        // that identify it.
        let leaf_verifiers = trace_log_sizes
            .clone()
            .map(|trace_log_size| {
                let context = circuits_builder
                    .build_leaf_context(trace_log_size, cairo_roots[&trace_log_size].clone());
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
        let (mv_circuit_hash, mv_preprocessed_root) = circuit_hash_and_preprocessed_root(
            &preprocessed_multiverifier,
            circuit_log_blowup_factor,
        );
        let multiverifiers = vec![Multiverifier {
            config: CONFIG_ID.to_string(),
            input_configs: [CONFIG_ID.to_string(), CONFIG_ID.to_string()],
            circuit_hash: DigestHex::from(mv_circuit_hash.0),
            preprocessed_root: DigestHex::from(mv_preprocessed_root.0),
        }];

        let registry = CircuitRegistry {
            cairo_prover_params: cairo_params,
            circuit_proof_configs,
            leaf_verifiers,
            multiverifiers,
        };
=======
        let leaves_max_sizes = leaf_sizes
            .values()
            .map(|(_, padded)| padded.clone())
            .reduce(|max_sizes, sizes| max_sizes.elementwise_max(&sizes))
            .expect("the trace range is non-empty");
        let (target_sizes, preprocessed_multiverifier) = padded_shared_target(
            leaves_max_sizes,
            circuit_fri_config,
            definition.pad_to_component_log_sizes.as_ref(),
        );

        // All circuits are padded to `target_sizes` and proven with
        // `circuit_log_blowup_factor`, so they share a single config.
        const CONFIG_ID: &str = "default";
        let circuit_proof_config = CircuitProofConfig::new(circuit_fri_config, &target_sizes);
        let circuit_proof_configs =
            BTreeMap::from([(CONFIG_ID.to_string(), circuit_proof_config.clone())]);
        // Pass 2: rebuild each leaf circuit, pad it to the shared target and record the hashes
        // that identify it.
        let leaf_verifiers = trace_log_sizes
            .clone()
            .map(|trace_log_size| {
                let context = circuits_builder
                    .build_leaf_context(trace_log_size, cairo_roots[&trace_log_size].clone());
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
        let (mv_circuit_hash, mv_preprocessed_root) = circuit_hash_and_preprocessed_root(
            &preprocessed_multiverifier,
            circuit_log_blowup_factor,
        );
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

        let registry = CircuitRegistry {
            cairo_prover_params: cairo_params,
            circuit_proof_configs,
            leaf_verifiers,
            multiverifiers,
            unpacking_verifiers,
        };
>>>>>>> a4b96ad32727e219b4a5a2c022f5ab419819045f
        serde_json::to_string_pretty(&registry).map_err(|err| err.to_string())?
    } else {
<<<<<<< HEAD
        let circuit_fri_params: FriParams = definition.circuit_fri_params();

        let mut circuits_builder = circuits_builder_from_definition(&definition);
        let trace_log_sizes = definition.min_trace_log_size..=definition.max_trace_log_size;

        let leaf_sizes = leaf_sizes(&mut circuits_builder, trace_log_sizes);

        // The leaf verifier circuit's size grows with the verified trace size, so it's reported
        // for every trace log size in the range. The multiverifier
        // verifies proofs of the leaf circuit; we only report it for the largest leaf
        // (`max_trace_log_size`), which bounds the multiverifier size across the range.
||||||| 0dbaec2ac
        // The leaf verifier circuit's size grows with the verified trace size, so it's reported
        // for every trace log size in the range. The multiverifier
        // verifies proofs of the leaf circuit; we only report it for the largest leaf
        // (`max_trace_log_size`), which bounds the multiverifier size across the range.
=======
>>>>>>> a4b96ad32727e219b4a5a2c022f5ab419819045f
        let leaf_lines: Vec<String> = leaf_sizes
            .iter()
            .map(|(trace_log_size, (raw, padded))| {
                format!("{}: {}", trace_log_size, format_sizes(raw, padded))
            })
            .collect();
        let leaf_section = format!("leaf:\n{}", leaf_lines.join("\n"));

        let (_, largest_leaf_padded_sizes) = &leaf_sizes[&definition.max_trace_log_size];
<<<<<<< HEAD
        let multiverifier_context =
            multiverifier_context_for_sizes(largest_leaf_padded_sizes, circuit_fri_params);
||||||| 0dbaec2ac
        let multiverifier_context =
            multiverifier_context_for_sizes(largest_leaf_padded_sizes, circuit_fri_config);
=======
        let config = CircuitProofConfig::new(circuit_fri_config, largest_leaf_padded_sizes);
        let multiverifier_context = multiverifier_context_from_circuit_config(&config);
>>>>>>> a4b96ad32727e219b4a5a2c022f5ab419819045f
        let (mv_raw, mv_padded) = raw_and_padded_component_sizes(&multiverifier_context);
        let multiverifier_line = format!("multiverifier:\n{}", format_sizes(&mv_raw, &mv_padded));
        format!("{leaf_section}\n\n{multiverifier_line}")
    };

    match args.output_path {
        Some(path) => std::fs::write(&path, format!("{output}\n"))
            .map_err(|err| format!("Cannot write output to {}: {err}", path.display()))?,
        None => println!("{output}"),
    }
    Ok(())
}
