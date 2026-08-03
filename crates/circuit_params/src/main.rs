//! Describes the leaf verifier and multiverifier circuits of one *circuit family*.
//!
//! A circuit family is the set of verifier circuits one recursive proof tree folds together: a
//! leaf (cairo-verifier) circuit per verified Cairo trace log size in a range, plus the
//! multiverifier that verifies their proofs — all padded to a shared component-size target (the
//! elementwise max over the family), so one circuit shape verifies proofs of any of them.
//!
//! The circuits, hence their identities, are functions of the verified Cairo proofs' prover
//! params, the circuit proofs' prover params, and the verified program — so each is taken from
//! the file the proving binaries run with.
//!
//! By default, writes a human-readable report: each component's padded log size and usage, per
//! trace size for the leaf circuit and once (at the largest size) for the multiverifier.
//!
//! With `--registry`, instead writes the registry JSON: the shared target sizes and every
//! circuit's hash. Only this mode commits the real Cairo preprocessed root per trace size (it is
//! baked into the leaf circuit, so the hashes depend on it).
//!
//! The trace range is part of the family's identity: widening it grows the shared target, which
//! changes every hash.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use circuit_cairo_verifier::statement::MEMORY_VALUES_LIMBS;
use circuit_cairo_verifier::utils::load_program;
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
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use clap::Parser;
use leaf_prover::prove_leaf::leaf_verifier_config;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_prover::witness::prelude::QM31;
use stwo_cairo_prover::witness::preprocessed_trace::generate_preprocessed_commitment_root;
use stwo_cairo_utils::binary_utils::run_binary;

#[derive(Parser)]
struct Args {
    /// Path to write the output to. If omitted, prints to stdout.
    #[clap(long)]
    output_path: Option<PathBuf>,
    /// Smallest verified trace log size to measure (inclusive). Bounded below by the
    /// preprocessed-trace variant's sequence-column log height (20 for canonical_small, 25 for
    /// canonical).
    #[clap(long)]
    min_trace_log_size: u32,
    /// Largest verified trace log size to measure (inclusive).
    #[clap(long)]
    max_trace_log_size: u32,
    /// The verified Cairo proofs' prover params — the file the leaf prover runs with. Supplies
    /// their FRI config and preprocessed trace variant.
    #[clap(long)]
    cairo_prover_params_json: PathBuf,
    /// The circuit proofs' prover params (a `PcsConfig`) — the file the leaf prover and the
    /// recursive tree run with. Supplies the log blowup factor the circuit hashes commit to.
    #[clap(long)]
    circuit_prover_params_json: PathBuf,
    /// The compiled program the leaf circuit verifies (the leaf prover's `--program`).
    #[clap(long)]
    program: PathBuf,
    /// Write the registry JSON instead of the human-readable sizes report.
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

/// Reads a prover params JSON file.
fn read_params<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> T {
    let json = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Cannot read prover params from {}: {err}", path.display()));
    serde_json::from_str(&json)
        .unwrap_or_else(|err| panic!("Cannot parse prover params from {}: {err}", path.display()))
}

/// The slice of the Cairo prover params (a flat `ProverParameters` JSON) the leaf circuit shape
/// depends on.
#[derive(serde::Deserialize)]
struct CairoProverParams {
    /// FRI config of the verified Cairo proofs.
    fri_config: FriConfig,
    preprocessed_trace: PreProcessedTraceVariant,
}

/// Builds the leaf-prover verifier circuit topology (with the given preprocessed trace and its
/// component set) for a verified Cairo proof of `program` with the given PCS config. The result
/// is a `NoValue` context (topology only; no witness values).
fn build_leaf_verifier_context(
    preprocessed_trace: PreProcessedTraceVariant,
    program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    preprocessed_root: HashValue<QM31>,
    cairo_pcs_config: &PcsConfig,
) -> FinalizedContext<NoValue> {
    let verifier_config =
        leaf_verifier_config(preprocessed_trace, cairo_pcs_config, program, preprocessed_root);

    build_cairo_verifier_circuit(&verifier_config)
}

/// Builds the multiverifier circuit that verifies two proofs of the leaf verifier circuit for the
/// given verified Cairo PCS config.
fn build_multiverifier_context_for_trace(
    preprocessed_trace: PreProcessedTraceVariant,
    program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    preprocessed_root: HashValue<QM31>,
    cairo_pcs_config: &PcsConfig,
    circuit_pcs_config: &PcsConfig,
) -> FinalizedContext<NoValue> {
    let mut leaf_context = build_leaf_verifier_context(
        preprocessed_trace,
        program,
        preprocessed_root,
        cairo_pcs_config,
    );

    // The multiverifier verifies proofs of the (preprocessed) leaf circuit, proven at the leaf
    // circuit's own trace log size — mirroring `CanonicalCircuit::build`, which derives the
    // circuit proofs' lifting size the same way.
    let preprocessed_leaf = PreprocessedCircuit::preprocess_circuit(&mut leaf_context);
    let multiverifier_pcs_config = PcsConfig {
        lifting_log_size: preprocessed_leaf.trace_log_size
            + circuit_pcs_config.fri_config.log_blowup_factor,
        ..*circuit_pcs_config
    };
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

    let program = load_program(&args.program);

    // The three inputs the family's circuits are built from, each read from the file the proving
    // binaries run with.
    let cairo_params: CairoProverParams = read_params(&args.cairo_prover_params_json);
    let circuit_pcs_config: PcsConfig = read_params(&args.circuit_prover_params_json);
    let preprocessed_trace = cairo_params.preprocessed_trace;
    let cairo_log_blowup_factor = cairo_params.fri_config.log_blowup_factor;
    // The verified proofs' PCS config at a given trace size: the file's FRI shape, lifted to that
    // trace.
    let cairo_pcs_config = |trace_log_size: u32| PcsConfig {
        fri_config: cairo_params.fri_config,
        lifting_log_size: trace_log_size + cairo_log_blowup_factor,
    };

    // The Cairo preprocessed root baked into the leaf circuit at each trace size. The root only
    // affects constant VALUES (hence the circuit hash), not component sizes, so the expensive
    // commitment is computed only in `--registry` mode; the sizes report uses a dummy zero root.
    let preprocessed_roots: BTreeMap<u32, HashValue<QM31>> = (args.min_trace_log_size
        ..=args.max_trace_log_size)
        .map(|trace_log_size| {
            let root = if args.registry {
                generate_preprocessed_commitment_root::<Blake2sM31MerkleChannel>(
                    cairo_log_blowup_factor,
                    preprocessed_trace,
                    trace_log_size + cairo_log_blowup_factor,
                )
                .into()
            } else {
                [0u32; 8].into()
            };
            (trace_log_size, root)
        })
        .collect();

    // Build every leaf circuit in the range once.
    let leaf_contexts: Vec<(u32, FinalizedContext<NoValue>)> = (args.min_trace_log_size
        ..=args.max_trace_log_size)
        .map(|trace_log_size| {
            (
                trace_log_size,
                build_leaf_verifier_context(
                    preprocessed_trace,
                    program.clone(),
                    preprocessed_roots[&trace_log_size].clone(),
                    &cairo_pcs_config(trace_log_size),
                ),
            )
        })
        .collect();
    // Build the mv context for the largest trace size.
    let multiverifier_context = build_multiverifier_context_for_trace(
        preprocessed_trace,
        program,
        preprocessed_roots[&args.max_trace_log_size].clone(),
        &cairo_pcs_config(args.max_trace_log_size),
        &circuit_pcs_config,
    );

    let output = if args.registry {
        let circuit_log_blowup_factor = circuit_pcs_config.fri_config.log_blowup_factor;

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
                log_blowup_factor: cairo_log_blowup_factor,
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
