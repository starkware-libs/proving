use std::io::{self};
use std::path::Path;
use std::{fs, process};

use air_code_gen::code_gen::cairo_constraints::utils::get_git_rev;
use air_code_gen::code_gen::constraints::generate_constraints_code;
use air_code_gen::code_gen::supported_components::{
    get_supported_components, AutogenCodeFile, AutogenCodeType,
};
use air_code_gen::code_gen::trace_gen::RustProverGen;
use air_code_gen::code_gen::utils::{add_rust_file_to_module, reformat_rust_code};
use clap::Parser;
use compiled_casm_air::compiled_structs::{CompiledAirFn, TraceType};
use compiled_casm_air::utils::read_json;
use serde_json::from_value;

/// For each JSON in the given directory, create two AutogenCodeFile jobs:
/// One for its AIR and one for its WITNESS.
fn codegen_jobs_from_dir(dir: &Path) -> Vec<AutogenCodeFile> {
    let mut result = vec![];
    for entry in fs::read_dir(dir).unwrap() {
        let filename = entry.unwrap().file_name();
        let filename = filename.to_str().expect("Invalid filename");
        if filename.ends_with(".json") {
            result.push(AutogenCodeFile {
                source_rel_path: filename.to_string(),
                code_type: AutogenCodeType::AIR,
            });
            result.push(AutogenCodeFile {
                source_rel_path: filename.to_string(),
                code_type: AutogenCodeType::WITNESS,
            });
        }
    }
    result
}

/// Generates component code from JSON files in the source directory.
fn process_json_files(args: &Args) -> io::Result<()> {
    let src_dir = Path::new(&args.source);
    let constraints_dir = Path::new(&args.constraints_dest);
    let witness_dir = Path::new(&args.witness_dest);

    if !constraints_dir.exists() {
        panic!(
            "Destination directory does not exist: {:?}",
            constraints_dir
        );
    }
    if !witness_dir.exists() {
        panic!("Witness directory does not exist: {:?}", witness_dir);
    }

    let files_to_generate = if args.all {
        codegen_jobs_from_dir(src_dir)
    } else {
        get_supported_components()
    };

    let source_repo_rev = get_git_rev(src_dir);
    let source_rev_comment = format!("// AIR version {}\n", source_repo_rev);

    for job in files_to_generate {
        let json_path = src_dir.join(&job.source_rel_path);
        let serialized_air_fn = read_json(
            json_path
                .to_str()
                .ok_or_else(|| io::Error::other("Invalid file path"))?,
        );
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();

        if air_fn.r#type == TraceType::Inline && job.code_type == AutogenCodeType::WITNESS {
            // Inline functions don't have witness-generation code (it is inlined into the
            // witness-generation code of their callers)
            continue;
        }

        let (code, dest_dir) = match job.code_type {
            AutogenCodeType::WITNESS => (
                RustProverGen::new(air_fn.clone()).generate_witness_code(),
                witness_dir,
            ),
            AutogenCodeType::AIR => (generate_constraints_code(&air_fn), constraints_dir),
        };
        let code = source_rev_comment.clone() + &code.to_string().unwrap();

        let dest_dir = match air_fn.r#type {
            TraceType::Inline => dest_dir.join("subroutines/"),
            _ => dest_dir.to_path_buf(),
        };

        let filename = &format!("{}.rs", air_fn.name);

        let dest_path = dest_dir.join(filename);

        let formatted_code = reformat_rust_code(code);

        add_rust_file_to_module(dest_path.as_path(), formatted_code);
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct Args {
    /// Generate code from all JSONs in the source directory (default: only generate
    /// components known to be supported by stwo-cairo)
    #[clap(short, long)]
    all: bool,

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
    let args = Args::try_parse_from(std::env::args()).unwrap_or_else(|e| e.exit());

    // Process JSON files.
    match process_json_files(&args) {
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
