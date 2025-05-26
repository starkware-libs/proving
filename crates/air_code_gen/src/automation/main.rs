use std::fs::{self};
use std::io::{self};
use std::path::Path;
use std::process;

use air_code_gen::code_gen::utils::dump_component_code;
use clap::Parser;
use compiled_casm_air::compiled_structs::CompiledAirFn;
use compiled_casm_air::utils::read_json;
use serde_json::from_value;

/// Generates component code for every `.json` file in the source directory.
// TODO(Ohad): separate constraints and witness code generation.
fn process_json_files(
    src_dir: &Path,
    constraints_dir: &Path,
    witness_dir: &Path,
) -> io::Result<()> {
    if !constraints_dir.exists() {
        panic!(
            "Destination directory does not exist: {:?}",
            constraints_dir
        );
    }
    if !witness_dir.exists() {
        panic!("Witness directory does not exist: {:?}", witness_dir);
    }

    let constraints_mod_file_path = constraints_dir.join("mod.rs");
    let mut constraints_mod_file_content = String::new();
    let witness_mod_file_path = witness_dir.join("mod.rs");
    let mut witness_mod_file_content = String::new();

    if constraints_mod_file_path.exists() {
        constraints_mod_file_content = fs::read_to_string(&constraints_mod_file_path)?;
    }
    if witness_mod_file_path.exists() {
        witness_mod_file_content = fs::read_to_string(&witness_mod_file_path)?;
    }

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            // Generate code.
            let serialized_air_fn = read_json(
                path.to_str()
                    .ok_or_else(|| io::Error::other("Invalid file path"))?,
            );
            let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
            dump_component_code(&air_fn, constraints_dir, witness_dir);

            // Update the `mod.rs` files.
            let file_stem = air_fn.name.clone();
            if !constraints_mod_file_content.contains(&format!("mod {file_stem};")) {
                constraints_mod_file_content.push_str(&format!("pub mod {file_stem};\n"));
            }
            if !witness_mod_file_content.contains(&format!("mod {file_stem};")) {
                witness_mod_file_content.push_str(&format!("pub mod {file_stem};\n"));
            }
        }
    }
    // Write the updated `mod.rs` files.
    fs::write(constraints_mod_file_path, constraints_mod_file_content)?;
    fs::write(witness_mod_file_path, witness_mod_file_content)?;

    Ok(())
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    source: String,

    #[clap(short, long)]
    constraints_dest: String,

    #[clap(short, long)]
    witness_dest: String,
}

/// Main CLI entry point
///
/// Example usage: `$ cargo run --bin cairo_code_gen -- --source
/// ./crates/compiled_casm_air/src/opcodes --constraints-dest
/// ~/stwo-cairo/stwo_cairo_prover/crates/cairo_air/src/components --witness-dest
/// ~/stwo-cairo/stwo_cairo_prover/crates/prover/src/witness/components`
fn main() {
    let args = Args::try_parse_from(std::env::args()).expect("Could not parse CLI arguments.");

    // Parse CLI.
    let src_dir = Path::new(&args.source);
    let constraints_dest = Path::new(&args.constraints_dest);
    let witness_dest = Path::new(&args.witness_dest);

    // Process JSON files.
    match process_json_files(src_dir, constraints_dest, witness_dest) {
        Ok(_) => {
            println!(
                "Successfully processed JSON files from {} to {} and {}",
                args.source, args.constraints_dest, args.witness_dest
            );
            process::exit(0);
        }
        Err(err) => {
            eprintln!("Error processing JSON files: {}", err);
            process::exit(1);
        }
    }
}
