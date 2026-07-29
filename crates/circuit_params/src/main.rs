//! Computes per-component sizes for the CANONICAL preprocessed trace config, across a range of
//! verified trace sizes.
//!
//! By default, writes a human-readable report of two circuits per run: the leaf-prover verifier
//! circuit (which verifies one Cairo proof), reported for every trace size, and the multiverifier
//! circuit (which verifies two proofs of that leaf verifier circuit), reported once for the largest
//! trace size. Each line gives every component's padded log size and usage.
//!
//! With `--registry`, instead writes a JSON circuit registry: it computes
//! shared target component sizes (the elementwise max over every leaf circuit in the range and the
//! multiverifier circuit), then the circuit hash of the leaf circuit for each trace size and of
//! the multiverifier circuit once (for the largest trace size), all padded to those targets.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use circuit_cairo_verifier::privacy::get_pcs_config;
use circuit_cairo_verifier::statement::MEMORY_VALUES_LIMBS;
use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;
use circuit_common::finalize::{
    ComponentSizes, compute_padded_sizes, pad_to_targets, raw_component_sizes,
};
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_multiverifier::verify::build_multiverifier_context;
use circuit_prover::circuit_hash::preprocessed_circuit_hash;
use circuit_registry::{
    CircuitProofConfig, CircuitRegistry, DigestHex, LeafVerifier, Multiverifier,
};
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use clap::Parser;
use leaf_prover::prove_leaf::leaf_verifier_config;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_utils::binary_utils::run_binary;

#[derive(Parser)]
struct Args {
    /// Path to write the output to. If omitted, prints to stdout.
    #[clap(long)]
    output_path: Option<PathBuf>,
    /// Smallest verified trace log size to measure (inclusive). A canonical Cairo trace commits
    /// its preprocessed sequence columns at `MAX_SEQUENCE_LOG_SIZE = 25`, so a real canonical
    /// leaf proof has `log_trace_size >= 25`.
    #[clap(long)]
    min_trace_log_size: u32,
    /// Largest verified trace log size to measure (inclusive).
    #[clap(long)]
    max_trace_log_size: u32,
    /// Log blowup factor (1, 2, or 3) of both the verified Cairo proof and the circuit proofs,
    /// passed to `get_pcs_config`.
    #[clap(long, default_value_t = 1)]
    log_blowup_factor: u32,
    /// Output a JSON circuit registry: a proof-config map, the leaf verifiers (one per trace
    /// size) and the multiverifier, each with its circuit hash. All circuits are padded to the
    /// shared target component sizes (the elementwise max over every leaf circuit in the range
    /// and the multiverifier circuit). If omitted, prints human-readable per-component sizes.
    #[clap(long)]
    registry: bool,
}

/// The fraction (as a percentage) of the padded (power-of-two) component that is actually used.
fn usage_percent(size: usize, padded_size: usize) -> f64 {
    100.0 * size as f64 / padded_size as f64
}

/// Non-padded row counts and padded sizes of a circuit context's AIR components.
fn component_sizes(context: &FinalizedContext<NoValue>) -> (ComponentSizes, ComponentSizes) {
    (raw_component_sizes(context), compute_padded_sizes(context))
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

/// Builds the leaf-prover verifier circuit topology (with the CANONICAL preprocessed trace and its
/// component set) for a verified Cairo proof whose trace has the given log size. The result is a
/// `NoValue` context (topology only; no witness values).
fn build_leaf_verifier_context(
    trace_log_size: u32,
    log_blowup_factor: u32,
) -> FinalizedContext<NoValue> {
    // The Cairo-proof PCS config the leaf prover uses (canonical preprocessed).
    let pcs_config = get_pcs_config(trace_log_size, log_blowup_factor);

    // TODO(ilya): Use a real program for the circuit construction.
    // Pass a dummy program for the circuit construction.
    let program: Arc<[[M31; MEMORY_VALUES_LIMBS]]> =
        std::iter::repeat_n([M31::from(0u32); MEMORY_VALUES_LIMBS], 128).collect();

    let verifier_config = leaf_verifier_config(
        PreProcessedTraceVariant::Canonical,
        &pcs_config,
        program,
        [0u32; 8].into(),
    );

    build_cairo_verifier_circuit(&verifier_config)
}

/// Builds the multiverifier circuit that verifies two proofs of the leaf verifier circuit for the
/// given verified trace log size.
fn build_multiverifier_context_for_trace(
    trace_log_size: u32,
    log_blowup_factor: u32,
) -> FinalizedContext<NoValue> {
    let mut leaf_context = build_leaf_verifier_context(trace_log_size, log_blowup_factor);

    // The multiverifier verifies proofs of the (preprocessed) leaf circuit, proven at the leaf
    // circuit's own trace log size.
    let preprocessed_leaf = PreprocessedCircuit::preprocess_circuit(&mut leaf_context);
    let multiverifier_pcs_config =
        get_pcs_config(preprocessed_leaf.trace_log_size, log_blowup_factor);
    build_multiverifier_context(&preprocessed_leaf, multiverifier_pcs_config)
}

/// Pads `context` to the shared `target_sizes` and preprocesses it.
fn padded_preprocessed_circuit(
    mut context: FinalizedContext<NoValue>,
    target_sizes: &ComponentSizes,
) -> PreprocessedCircuit {
    pad_to_targets(&mut context, target_sizes.clone());
    PreprocessedCircuit::preprocess_circuit(&mut context)
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

    // Build every leaf circuit in the range once.
    let leaf_contexts: Vec<(u32, FinalizedContext<NoValue>)> = (args.min_trace_log_size
        ..=args.max_trace_log_size)
        .map(|trace_log_size| {
            (trace_log_size, build_leaf_verifier_context(trace_log_size, args.log_blowup_factor))
        })
        .collect();
    // Build the mv context for the largest trace size.
    let multiverifier_context =
        build_multiverifier_context_for_trace(args.max_trace_log_size, args.log_blowup_factor);

    let output = if args.registry {
        // Currently a single log blowup factor is used across the system.
        let circuit_log_blowup_factor = args.log_blowup_factor;

        // Target sizes: the elementwise max of the component sizes over every leaf (cairo
        // verifier) circuit in the range and the multiverifier.
        let target_sizes = leaf_contexts
            .iter()
            .map(|(_, context)| compute_padded_sizes(context))
            .fold(compute_padded_sizes(&multiverifier_context), |max_sizes, sizes| {
                max_sizes.elementwise_max(&sizes)
            });

        let preprocessed_multiverifier =
            padded_preprocessed_circuit(multiverifier_context, &target_sizes);

        // All circuits are padded to `target_sizes` and proven with
        // `circuit_log_blowup_factor`, so they share a single config.
        const CONFIG_ID: &str = "default";
        let circuit_proof_configs = BTreeMap::from([(
            CONFIG_ID.to_string(),
            CircuitProofConfig {
                log_blowup_factor: circuit_log_blowup_factor,
                component_log_sizes: (&target_sizes).into(),
            },
        )]);
        let leaf_verifiers = leaf_contexts
            .into_iter()
            .map(|(trace_log_size, context)| LeafVerifier {
                config: CONFIG_ID.to_string(),
                trace_log_size,
                log_blowup_factor: args.log_blowup_factor,
                circuit_hash: circuit_hash_hex(
                    &padded_preprocessed_circuit(context, &target_sizes),
                    circuit_log_blowup_factor,
                ),
            })
            .collect::<Vec<_>>();

        // The multiverifier verifies two proofs of the leaf circuit, hence
        // `input_configs = [CONFIG_ID, CONFIG_ID]`.
        let multiverifiers = vec![Multiverifier {
            config: CONFIG_ID.to_string(),
            input_configs: [CONFIG_ID.to_string(), CONFIG_ID.to_string()],
            circuit_hash: circuit_hash_hex(&preprocessed_multiverifier, circuit_log_blowup_factor),
        }];

        let registry = CircuitRegistry { circuit_proof_configs, leaf_verifiers, multiverifiers };
        serde_json::to_string_pretty(&registry).map_err(|err| err.to_string())?
    } else {
        // The leaf verifier circuit's size grows with the verified trace size, so it's reported
        // for every trace log size in the range. The multiverifier
        // verifies proofs of the leaf circuit; we only report it for the largest leaf
        // (`max_trace_log_size`), which bounds the multiverifier size across the range.
        let leaf_lines: Vec<String> = leaf_contexts
            .iter()
            .map(|(trace_log_size, context)| {
                let (raw, padded) = component_sizes(context);
                format!("{}: {}", trace_log_size, format_sizes(&raw, &padded))
            })
            .collect();
        let leaf_section = format!("leaf:\n{}", leaf_lines.join("\n"));

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
