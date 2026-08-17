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
use circuit_prover::circuit_hash::preprocessed_circuit_hash;
use circuit_registry::{
    CircuitProofConfig, CircuitRegistry, DigestHex, LeafVerifier, Multiverifier,
};
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use clap::Parser;
use leaf_prover::prove_leaf::leaf_verifier_config;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_prover::prover::ProverParameters;
use stwo_cairo_prover::witness::prelude::QM31;
use stwo_cairo_prover::witness::preprocessed_trace::generate_preprocessed_commitment_root;
use stwo_cairo_utils::binary_utils::run_binary;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

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
    /// The verified Cairo proofs' prover params. Shape the leaf circuits (via the FRI config and
    /// preprocessed trace variant) and are recorded in the registry for the leaf prover to run
    /// with.
    #[clap(long)]
    cairo_prover_params_json: PathBuf,
    /// The circuit proofs' FRI config, recorded in the registry for the leaf prover and the
    /// recursive tree to run with. Their lifting size is not configurable: each circuit is built
    /// as if lifted to `trace_size + blowup`.
    #[clap(long)]
    circuit_fri_config_json: PathBuf,
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

/// A stand-in for the Cairo preprocessed root, for the passes that only read component sizes: the
/// root ends up in the circuit's constant VALUES, never in its topology, and committing the real
/// one is the most expensive thing here.
const DUMMY_PREPROCESSED_ROOT: [u32; 8] = [0; 8];

/// Builds the circuits of one registry definition: holds the definition's inputs (program, prover
/// params), so that a leaf verifier circuit can be specified and built from just the trace log
/// size of the Cairo proof it verifies.
struct CircuitBuilder {
    preprocessed_trace: PreProcessedTraceVariant,
    /// The program every leaf proof of these circuits attests to.
    program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    /// FRI config of the verified Cairo proofs.
    cairo_fri_config: FriConfig,
}

impl CircuitBuilder {
    /// The verified proofs' PCS config at `trace_log_size`: `cairo_fri_config`, lifted to that
    /// trace.
    fn cairo_pcs_config(&self, trace_log_size: u32) -> PcsConfig {
        PcsConfig::from_fri_and_trace_size(self.cairo_fri_config, trace_log_size)
    }

    /// Commits the verified Cairo proofs' preprocessed trace at `trace_log_size` to get the root
    /// baked into the leaf circuit.
    fn cairo_preprocessed_root(&self, trace_log_size: u32) -> HashValue<QM31> {
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
    fn build_context(
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
    fn build_multiverifier_context_for_trace(
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

/// A circuit's preprocessed-trace layout: its `trace_log_size` and every preprocessed column's log
/// size. Circuits verified by one verifier circuit must share it.
fn preprocessed_layout(
    circuit: &PreprocessedCircuit,
) -> (u32, OrderedHashMap<PreProcessedColumnId, u32>) {
    (circuit.trace_log_size, circuit.preprocessed_trace.log_sizes())
}

/// Pads `context` to the shared `target_sizes` and preprocesses it.
fn padded_preprocessed_circuit(
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
fn shared_target_fixpoint(
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

    // The three inputs the definition names, each read from the file the proving binaries run
    // with.
    let cairo_params: ProverParameters = read_params(&args.cairo_prover_params_json);
    let circuit_fri_config: FriConfig = read_params(&args.circuit_fri_config_json);

    let circuit_builder = CircuitBuilder {
        preprocessed_trace: cairo_params.preprocessed_trace,
        program,
        cairo_fri_config: cairo_params.fri_config,
    };
    let trace_log_sizes = args.min_trace_log_size..=args.max_trace_log_size;

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
        let (target_sizes, preprocessed_multiverifier) =
            shared_target_fixpoint(leaves_max_sizes, circuit_fri_config);

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

        let multiverifier_context = circuit_builder
            .build_multiverifier_context_for_trace(args.max_trace_log_size, circuit_fri_config);
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
