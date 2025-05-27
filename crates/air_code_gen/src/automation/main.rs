use std::io::{self};
use std::path::{Path, PathBuf};
use std::{fs, process};

use air_code_gen::code_gen::supported_components::{
    is_supported, AutogenCodeFile, AutogenCodeType,
};
use air_code_gen::code_gen::utils::{generate_air_fn_code, get_git_rev, write_air_fn_code};
use clap::Parser;
use compiled_casm_air::compiled_structs::CompiledAirFn;
use compiled_casm_air::utils::read_json;
use serde_json::from_value;

fn jsons_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut result = vec![];
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_str().unwrap();
            if filename.ends_with(".json") {
                result.push(path);
            }
        } else {
            result.append(&mut jsons_in_dir(&path));
        }
    }
    result
}

/// For each JSON in the given directory, create two AutogenCodeFile jobs:
/// One for its AIR and one for its WITNESS.
fn codegen_jobs_from_dir(dir: &Path) -> Vec<AutogenCodeFile> {
    let mut result = vec![];
    for json_path in jsons_in_dir(dir) {
        let rel_path: String = json_path
            .strip_prefix(dir)
            .unwrap()
            .to_str()
            .unwrap()
            .into();
        result.push(AutogenCodeFile {
            source_rel_path: rel_path.clone(),
            code_type: AutogenCodeType::AIR,
        });
        result.push(AutogenCodeFile {
            source_rel_path: rel_path,
            code_type: AutogenCodeType::WITNESS,
        });
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

    let files_to_generate = codegen_jobs_from_dir(src_dir);

    let source_repo_rev = get_git_rev(src_dir);
    let source_rev_comment = format!("// AIR version {}\n", source_repo_rev);

    let mut skipped_files = 0;

    for job in files_to_generate.iter() {
        let json_path = src_dir.join(&job.source_rel_path);
        let serialized_air_fn = read_json(
            json_path
                .to_str()
                .ok_or_else(|| io::Error::other("Invalid file path"))?,
        );
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();

        if !is_supported(job, &air_fn) && !args.all {
            // The autogeneration logic doesn't support this.
            skipped_files += 1;
            continue;
        }

        let dest_dir = match job.code_type {
            AutogenCodeType::WITNESS => witness_dir,
            AutogenCodeType::AIR => constraints_dir,
        };
        let code = generate_air_fn_code(&air_fn, job.code_type);
        let code = source_rev_comment.clone() + &code;

        write_air_fn_code(&air_fn, code, dest_dir);
    }

    let generated_files = files_to_generate.len() - skipped_files;
    println!(
        "Generated {generated_files} files. Skipped {skipped_files} manually-implemented files."
    );

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
