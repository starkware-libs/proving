//! A binary used to convert Cairo programs to circuit proofs
//!
//! Operates in two steps:
//!     1. Runs the given Cairo program and proves it (like stwo_run_and_prove)
//!     2. Uses a circuit to verify the proof from (1)
//!     3. Proves the execution of the circuit from (2)
//!
//! Outputs a file with the final proof, the preprocessed root of the verifier circuit,
//! and the circuit hash identifying that circuit together with the config its root is
//! interpreted under. It is assumed that the user knows the output of the program
//! (required to verify the proof) by some other means.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use leaf_prover::prove_leaf::prove_leaf_from_files;
use stwo_cairo_utils::binary_utils::run_binary;

#[derive(Parser)]
struct Args {
    #[clap(long = "program", help = "Absolute path to the compiled program.")]
    program: PathBuf,
    #[clap(long = "program_input", help = "Absolute path to the program input file.")]
    program_input: Option<PathBuf>,
    #[clap(
        long = "circuit_registry_json",
        help = "JSON file containing the circuit registry this leaf's circuit belongs to. \
                Supplies the prover params of both proofs, the padding target for the verifier \
                circuit, and the hash that circuit must come out with."
    )]
    circuit_registry_json: PathBuf,
    #[clap(long = "output_path", help = "Path to write the output file")]
    output_path: PathBuf,
}

fn main() -> ExitCode {
    run_binary(run, "leaf_prover")
}

fn run() -> Result<(), String> {
    let args = Args::parse();
    let output =
        prove_leaf_from_files(&args.program, &args.program_input, &args.circuit_registry_json);

    fs::write(&args.output_path, serde_json::to_string_pretty(&output).unwrap()).unwrap_or_else(
        |err| panic!("Cannot write output to {}: {err}", args.output_path.display()),
    );

    Ok(())
}
