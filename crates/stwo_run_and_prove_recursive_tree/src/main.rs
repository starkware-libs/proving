//! CLI entry point for the recursive circuit-proof tree binary.
//!
//! See `lib.rs` for the algorithm. The CLI takes the leaves list, the circuit prover params, and
//! the three root output paths; the rest of the proving configuration is fixed by the canonical
//! circuit shape (see `canonical`).

use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;

use circuit_registry::CircuitRegistry;
use clap::Parser;
use stwo_cairo_utils::binary_utils::run_binary;
use stwo_run_and_prove_recursive_tree::{
    ProofFormat, RecursiveTreeError, load_leaves, stwo_run_and_prove_recursive_tree,
};
use tracing::{Level, info, span};

/// CLI arguments. Names use `snake_case` to match the existing pipeline convention of
/// `--<file_name>=<path>`.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// JSON manifest `{"leaves": ["<path>", ...]}` listing one leaf input file per leaf, in fold
    /// order: a `leaf_prover` output with the backend-injected fields flattened in (see
    /// `LeafInput`). Named `--program_input` to match the sibling binaries' CLI convention.
    #[clap(long = "program_input")]
    program_input: PathBuf,

    /// Output path for the root circuit proof, in the format `--proof_format` selects.
    #[clap(long = "proof_path")]
    proof_path: PathBuf,

    /// Output path for the root node's output values (JSON array of raw `u32` digest words).
    #[clap(long = "program_output")]
    program_output: PathBuf,

    /// Output path for the nested `PackedNode` JSON tree consumed by a future per-leaf unpacker.
    #[clap(long = "packed_node_tree_path")]
    packed_node_tree_path: PathBuf,

    /// The circuit registry of the circuits being folded — the same file the leaf prover ran with.
    /// Supplies the circuit proofs' prover params, the registry's padding target and the
    /// multiverifier hash the canonical circuit is checked against.
    #[clap(long = "circuit_registry_json")]
    circuit_registry_json: PathBuf,

    /// Maximum number of pair reductions proven in parallel within a layer.
    #[clap(long = "max_concurrency", default_value_t = NonZeroUsize::new(1).unwrap())]
    max_concurrency: NonZeroUsize,

    /// Who consumes the root proof: `cairo_verifier` (the default) emits the Cairo circuit
    /// verifier's felt252 arguments stream; `circuit` emits CircuitSerialize bytes for the
    /// applicative unpacking verifier.
    #[clap(long = "proof_format", value_enum, default_value_t = ProofFormat::CairoVerifier)]
    proof_format: ProofFormat,
}

fn main() -> ExitCode {
    run_binary(run, "stwo_run_and_prove_recursive_tree")
}

fn run() -> Result<(), RecursiveTreeError> {
    let _span = span!(Level::INFO, "stwo_run_and_prove_recursive_tree::run").entered();
    let args = Args::parse();
    let leaves = load_leaves(&args.program_input)?;
    let registry = CircuitRegistry::from_path(&args.circuit_registry_json)?;
    info!(n_leaves = leaves.len(), "Starting in-binary recursive circuit-proof tree reduction.");
    let stats = stwo_run_and_prove_recursive_tree(
        leaves,
        &registry,
        &args.proof_path,
        &args.program_output,
        &args.packed_node_tree_path,
        args.max_concurrency,
        args.proof_format,
    )?;
    info!(
        n_layers = stats.n_layers,
        n_pair_reductions = stats.n_pair_reductions,
        "Recursive tree reduction complete."
    );
    Ok(())
}
