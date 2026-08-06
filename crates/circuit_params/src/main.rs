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
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
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

/// Builds one family's circuits: everything their construction needs, so that a leaf verifier
/// circuit is built from its verified Cairo proof's trace log size alone.
struct CircuitBuilder {
    preprocessed_trace: PreProcessedTraceVariant,
    /// The program every leaf proof of the family attests to.
    program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    /// FRI config of the verified Cairo proofs.
    cairo_fri_config: FriConfig,
    /// The Cairo preprocessed root baked into the leaf circuit at each trace size.
    preprocessed_roots: BTreeMap<u32, HashValue<QM31>>,
}

impl CircuitBuilder {
    /// The verified proofs' PCS config at `trace_log_size`: `cairo_fri_config`, lifted to that
    /// trace.
    fn cairo_pcs_config(&self, trace_log_size: u32) -> PcsConfig {
        PcsConfig::from_fri_and_trace_size(self.cairo_fri_config, trace_log_size)
    }

    /// Builds the leaf verifier circuit topology for a verified Cairo proof of `trace_log_size`.
    fn build_context(&self, trace_log_size: u32) -> FinalizedContext<NoValue> {
        let verifier_config = leaf_verifier_config(
            self.preprocessed_trace,
            &self.cairo_pcs_config(trace_log_size),
            self.program.clone(),
            self.preprocessed_roots[&trace_log_size].clone(),
        );

        build_cairo_verifier_circuit(&verifier_config)
    }

    /// Builds the multiverifier over the *default-padded* leaf circuit for `trace_log_size` —
    /// only for the sizes report; the registry pads the leaf to the family's target first (see
    /// [`shared_target_fixpoint`]).
    fn build_multiverifier_context_for_trace(
        &self,
        trace_log_size: u32,
        circuit_pcs_config: &PcsConfig,
    ) -> FinalizedContext<NoValue> {
        let mut leaf_context = self.build_context(trace_log_size);

        let preprocessed_leaf = PreprocessedCircuit::preprocess_circuit(&mut leaf_context);
        build_multiverifier_context(
            &preprocessed_leaf,
            multiverifier_pcs_config(&preprocessed_leaf, circuit_pcs_config),
        )
    }
}

/// The PCS config of the circuit proofs the multiverifier verifies: the circuit prover's config,
/// lifted to the verified circuit's own trace log size.
fn multiverifier_pcs_config(
    preprocessed_leaf: &PreprocessedCircuit,
    circuit_pcs_config: &PcsConfig,
) -> PcsConfig {
    PcsConfig::from_fri_and_trace_size(
        circuit_pcs_config.fri_config,
        preprocessed_leaf.trace_log_size,
    )
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

/// The family's shared target sizes, with the multiverifier padded to them.
///
/// The target and the multiverifier's sizes are a fixpoint of each other: the multiverifier
/// verifies proofs of the TARGET-PADDED leaf circuit, and the target is the elementwise max over
/// every leaf circuit and that very multiverifier. Starts from the leaves' max (`target_sizes`)
/// and iterates — a larger target can grow the multiverifier, growing the target again.
///
/// The multiverifier is built over the largest leaf (arbitrarily — it depends only on the
/// target-padded shape, identical for all leaves), rebuilt each iteration: padding is destructive
/// (it allocates variable indices), so a padded context cannot be padded further.
fn shared_target_fixpoint(
    circuit_builder: &CircuitBuilder,
    max_trace_log_size: u32,
    mut target_sizes: ComponentSizes,
    circuit_pcs_config: &PcsConfig,
) -> (ComponentSizes, PreprocessedCircuit) {
    loop {
        let leaf_context = circuit_builder.build_context(max_trace_log_size);
        let preprocessed_leaf = padded_preprocessed_circuit(leaf_context, &target_sizes);
        let multiverifier_context = build_multiverifier_context(
            &preprocessed_leaf,
            multiverifier_pcs_config(&preprocessed_leaf, circuit_pcs_config),
        );
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

    // The three inputs the family's circuits are built from, each read from the file the proving
    // binaries run with.
    let cairo_params: CairoProverParams = read_params(&args.cairo_prover_params_json);
    let circuit_pcs_config: PcsConfig = read_params(&args.circuit_prover_params_json);
    let cairo_log_blowup_factor = cairo_params.fri_config.log_blowup_factor;

    // The Cairo preprocessed root baked into the leaf circuit at each trace size. The root only
    // affects constant VALUES (hence the circuit hash), not component sizes, so the expensive
    // commitment is computed only in `--registry` mode; the sizes report uses a dummy zero root.
    let preprocessed_roots: BTreeMap<u32, HashValue<QM31>> = (args.min_trace_log_size
        ..=args.max_trace_log_size)
        .map(|trace_log_size| {
            let root = if args.registry {
                generate_preprocessed_commitment_root::<Blake2sM31MerkleChannel>(
                    cairo_log_blowup_factor,
                    cairo_params.preprocessed_trace,
                    trace_log_size + cairo_log_blowup_factor,
                )
                .into()
            } else {
                [0u32; 8].into()
            };
            (trace_log_size, root)
        })
        .collect();
    let circuit_builder = CircuitBuilder {
        preprocessed_trace: cairo_params.preprocessed_trace,
        program,
        cairo_fri_config: cairo_params.fri_config,
        preprocessed_roots,
    };

    // Build every leaf circuit in the range once.
    let leaf_contexts: BTreeMap<u32, FinalizedContext<NoValue>> = (args.min_trace_log_size
        ..=args.max_trace_log_size)
        .map(|trace_log_size| (trace_log_size, circuit_builder.build_context(trace_log_size)))
        .collect();
    let output = if args.registry {
        let circuit_log_blowup_factor = circuit_pcs_config.fri_config.log_blowup_factor;

        let leaves_max_sizes = leaf_contexts
            .values()
            .map(compute_padded_sizes)
            .reduce(|max_sizes, sizes| max_sizes.elementwise_max(&sizes))
            .expect("the trace range is non-empty");
        let (target_sizes, preprocessed_multiverifier) = shared_target_fixpoint(
            &circuit_builder,
            args.max_trace_log_size,
            leaves_max_sizes,
            &circuit_pcs_config,
        );

        // Homogeneity: padded to the shared target, every circuit of the family must have the
        // multiverifier's preprocessed-trace layout — the layout it verifies (each leaf is
        // asserted below).
        let family_layout = preprocessed_layout(&preprocessed_multiverifier);

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
        let leaf_verifier = |trace_log_size: u32, padded_leaf: &PreprocessedCircuit| {
            assert_eq!(
                preprocessed_layout(padded_leaf),
                family_layout,
                "the leaf circuit for trace log size {trace_log_size} does not have the family's \
                 preprocessed layout"
            );
            LeafVerifier {
                config: CONFIG_ID.to_string(),
                trace_log_size,
                log_blowup_factor: cairo_log_blowup_factor,
                circuit_hash: circuit_hash_hex(padded_leaf, circuit_log_blowup_factor),
            }
        };
        let leaf_verifiers = leaf_contexts
            .into_iter()
            .map(|(trace_log_size, context)| {
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

        let multiverifier_context = circuit_builder
            .build_multiverifier_context_for_trace(args.max_trace_log_size, &circuit_pcs_config);
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
