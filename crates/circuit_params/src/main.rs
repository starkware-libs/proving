//! Builds the circuits a registry definition determines, and reports or records them.
//!
//! A circuit registry specifies which circuits may appear in a recursive proof tree. Currently
//! this utility generates a leaf (cairo-verifier) circuit per verified Cairo trace log size in the
//! definition's range, plus the multiverifier that verifies their proofs — all padded to a shared
//! component-size target (the elementwise max over them), so one circuit shape verifies proofs of
//! any of them.
//!
//! The circuits, hence their identities, are functions of the verified Cairo proofs' prover params,
//! the circuit proofs' prover params, and the verified program. The params are taken from files
//! here and recorded in the registry, which is then the only configuration the proving binaries
//! need.
//!
//! By default, writes a human-readable report: each component's padded log size and usage, per
//! trace size for the leaf circuit and once (at the largest size) for the multiverifier.
//!
//! With `--registry`, instead writes the registry JSON: the params above, the shared target sizes
//! and every circuit's hash — everything a binary proving these circuits needs. Only this mode
//! commits the real Cairo preprocessed root per trace size (it is baked into the leaf circuit, so
//! the hashes depend on it).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use circuit_cairo_verifier::utils::load_program;
use circuit_common::finalize::ComponentSizes;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_params::{
    CircuitBuilder, DUMMY_PREPROCESSED_ROOT, RegistryDefinition, component_sizes,
    padded_preprocessed_circuit, padded_shared_target, read_params,
};
use circuit_prover::circuit_hash::preprocessed_circuit_hash;
use circuit_registry::{
    CircuitProofConfig, CircuitRegistry, DigestHex, LeafVerifier, Multiverifier,
};
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use clap::Parser;
use stwo::core::fri::FriConfig;
use stwo_cairo_prover::prover::ProverParameters;
use stwo_cairo_utils::binary_utils::run_binary;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

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

/// A circuit's preprocessed-trace layout: its `trace_log_size` and every preprocessed column's log
/// size. Circuits verified by one verifier circuit must share it.
fn preprocessed_layout(
    circuit: &PreprocessedCircuit,
) -> (u32, OrderedHashMap<PreProcessedColumnId, u32>) {
    (circuit.trace_log_size, circuit.preprocessed_trace.log_sizes())
}

/// The circuit hash of a padded, preprocessed circuit, as eight little-endian Blake2s words.
fn circuit_hash_hex(
    preprocessed_circuit: &PreprocessedCircuit,
    circuit_log_blowup_factor: u32,
) -> DigestHex {
    DigestHex::from(preprocessed_circuit_hash(preprocessed_circuit, circuit_log_blowup_factor).0)
}

fn main() -> ExitCode {
    run_binary(run, "circuit_params")
}

fn run() -> Result<(), String> {
    let args = Args::parse();

    let definition: RegistryDefinition = read_params(&args.definition);
    let program = load_program(&definition.program);

    // The definition's params, each read from the file the proving binaries run with.
    let cairo_params: ProverParameters = definition.cairo_params();
    let circuit_fri_config: FriConfig = definition.circuit_fri_config();

    let circuit_builder = CircuitBuilder {
        preprocessed_trace: cairo_params.preprocessed_trace,
        program,
        cairo_fri_config: cairo_params.fri_config,
    };
    let trace_log_sizes = definition.min_trace_log_size..=definition.max_trace_log_size;

    // Pass 1: every leaf circuit's component sizes. Built with a dummy Cairo root and dropped as
    // soon as its sizes are read, so only one leaf circuit is ever in memory.
    let leaf_sizes: BTreeMap<u32, (ComponentSizes, ComponentSizes)> = trace_log_sizes
        .clone()
        .map(|trace_log_size| {
            let context =
                circuit_builder.build_context(trace_log_size, DUMMY_PREPROCESSED_ROOT.into());
            (trace_log_size, component_sizes(&context))
        })
        .collect();

    let output = if args.registry {
        let circuit_log_blowup_factor = circuit_fri_config.log_blowup_factor;

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

        // Homogeneity: padded to the shared target, every circuit in the registry must have the
        // multiverifier's preprocessed-trace layout — the layout it verifies (each leaf is
        // asserted below).
        let shared_layout = preprocessed_layout(&preprocessed_multiverifier);

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
        let leaf_verifier = |trace_log_size: u32, padded_leaf: &PreprocessedCircuit| {
            assert_eq!(
                preprocessed_layout(padded_leaf),
                shared_layout,
                "the leaf circuit for trace log size {trace_log_size} does not have the shared \
                 preprocessed layout"
            );
            LeafVerifier {
                config: CONFIG_ID.to_string(),
                trace_log_size,
                circuit_hash: circuit_hash_hex(padded_leaf, circuit_log_blowup_factor),
            }
        };
        // Pass 2: each leaf's identity — its circuit hash, which needs the real Cairo root. Both
        // the commitment and the rebuilt circuit are dropped before moving to the next trace size.
        let leaf_verifiers = trace_log_sizes
            .clone()
            .map(|trace_log_size| {
                let context = circuit_builder.build_context(
                    trace_log_size,
                    circuit_builder.cairo_preprocessed_root(trace_log_size),
                );
                leaf_verifier(trace_log_size, &padded_preprocessed_circuit(context, &target_sizes))
            })
            .collect::<Vec<_>>();

        // The multiverifier verifies two proofs of the leaf circuit, hence
        // `input_configs = [CONFIG_ID, CONFIG_ID]`.
        let multiverifiers = vec![Multiverifier {
            config: CONFIG_ID.to_string(),
            input_configs: [CONFIG_ID.to_string(), CONFIG_ID.to_string()],
            circuit_hash: circuit_hash_hex(&preprocessed_multiverifier, circuit_log_blowup_factor),
        }];

        let registry = CircuitRegistry {
            cairo_prover_params: cairo_params,
            circuit_proof_configs,
            leaf_verifiers,
            multiverifiers,
        };
        serde_json::to_string_pretty(&registry).map_err(|err| err.to_string())?
    } else {
        // The leaf verifier circuit's size grows with the verified trace size, so it's reported
        // for every trace log size in the range. The multiverifier
        // verifies proofs of the leaf circuit; we only report it for the largest leaf
        // (`max_trace_log_size`), which bounds the multiverifier size across the range.
        let leaf_lines: Vec<String> = leaf_sizes
            .iter()
            .map(|(trace_log_size, (raw, padded))| {
                format!("{}: {}", trace_log_size, format_sizes(raw, padded))
            })
            .collect();
        let leaf_section = format!("leaf:\n{}", leaf_lines.join("\n"));

        let multiverifier_context = circuit_builder.build_multiverifier_context_for_trace(
            definition.max_trace_log_size,
            circuit_fri_config,
        );
        let (mv_raw, mv_padded) = component_sizes(&multiverifier_context);
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
