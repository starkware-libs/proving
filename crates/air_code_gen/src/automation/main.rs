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
fn process_json_files(src_dir: &Path, dest_dir: &Path) -> io::Result<()> {
    if !dest_dir.exists() {
        panic!("Destination directory does not exist: {:?}", dest_dir);
    }

    let mod_file_path = dest_dir.join("mod.rs");
    let mut mod_file_content = String::new();

    if mod_file_path.exists() {
        mod_file_content = fs::read_to_string(&mod_file_path)?;
    }

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let file_stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid file name"))?;

            // Generate code.
            let serialized_air_fn = read_json(
                path.to_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid file path"))?,
            );
            let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();

            let component_dir = dest_dir.join(file_stem);
            fs::create_dir_all(component_dir.clone())?;
            dump_component_code(air_fn, &component_dir);

            // Update the `mod.rs` file.
            if !mod_file_content.contains(&format!("mod {file_stem};")) {
                mod_file_content.push_str(&format!("pub mod {file_stem};\n"));
            }
        }
    }
    // Write the updated `mod.rs` file
    fs::write(mod_file_path, mod_file_content)?;

    Ok(())
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    source: String,

    #[clap(short, long)]
    destination: String,
}

/// Main CLI entry point
///
/// Example usage: `$ cargo run --bin cairo_code_gen -- --source
/// ./crates/compiled_casm_air/src/opcodes --destination
/// ~/stwo-cairo/stwo_cairo_prover/crates/prover/src/components`
fn main() {
    let args = Args::try_parse_from(std::env::args()).unwrap();

    // Parse CLI.
    let src_dir = Path::new(&args.source);
    let dest_dir = Path::new(&args.destination);

    // Process JSON files.
    match process_json_files(src_dir, dest_dir) {
        Ok(_) => {
            println!(
                "Successfully processed JSON files from {} to {}",
                args.source, args.destination
            );
            process::exit(0);
        }
        Err(err) => {
            eprintln!("Error processing JSON files: {}", err);
            process::exit(1);
        }
    }
}
