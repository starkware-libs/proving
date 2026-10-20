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
//! and every circuit's hash — everything a binary proving these circuits needs.

use std::path::PathBuf;
use std::process::ExitCode;

use circuit_common::finalize::ComponentSizes;
use circuit_params::{
    RegistryDefinition, circuits_builder_from_definition, create_registry_from_definition,
    leaf_sizes, multiverifier_context_for_sizes, raw_and_padded_component_sizes, read_params,
};
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

    let output = if args.registry {
        let registry = create_registry_from_definition(&definition);

        serde_json::to_string_pretty(&registry).map_err(|err| err.to_string())?
    } else {
        let circuit_fri_params: FriParams = definition.circuit_fri_params();

        let mut circuits_builder = circuits_builder_from_definition(&definition);
        let trace_log_sizes = definition.min_trace_log_size..=definition.max_trace_log_size;

        let leaf_sizes = leaf_sizes(&mut circuits_builder, trace_log_sizes);

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

        let (_, largest_leaf_padded_sizes) = &leaf_sizes[&definition.max_trace_log_size];
        let multiverifier_context =
            multiverifier_context_for_sizes(largest_leaf_padded_sizes, circuit_fri_params);
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
