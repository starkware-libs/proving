use std::fs;
use std::path::{absolute, Path, PathBuf};

use air_code_gen::code_gen::cairo_constraints::sample_evaluations::generate_sample_evaluations_file;
use air_code_gen::code_gen::supported_components::{
    is_supported, AutogenCodeFile, AutogenCodeType,
};
use air_code_gen::code_gen::utils::{
    add_file_to_module, format_air_fn_code, generate_air_fn_code, generated_code_path, get_git_rev,
    load_air_fns,
};
use clap::Parser;
use compiled_casm_air::compiled_structs::{CompiledAirFn, CompiledAirFnStat};
use compiled_casm_air::utils::REGISTRY_PROPERTIES_FILE_NAME;
use eval_air_fn_constraints::SampleEvaluation;
use indexmap::IndexMap;
use serde::Serialize;
use xshell::{cmd, Shell};

const DEFAULT_SOURCE_DIR: &str = "./crates/compiled_casm_air/src";
const DEFAULT_STWO_CAIRO_PATH: &str = "../stwo-cairo/";

#[derive(Serialize)]
struct VersionedCasmRegistry {
    /// The Git commit hash of the repository we took the statistics from
    pub air_version: String,
    pub air_fns: IndexMap<String, CompiledAirFnStat>,
}

fn jsons_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut result = vec![];
    let directory_entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("Cannot read {}: {err}", dir.display()));
    for entry in directory_entries {
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

/// For each JSON in the given directory, create AutogenCodeFile jobs
/// for each code type.
fn get_stwo_cairo_jobs(args: &GenerateStwoCairoArgs) -> Vec<AutogenCodeFile> {
    let mut result = vec![];

    let mut skipped_files = 0;
    for json_path in jsons_in_dir(&args.source.join("compiled_jsons")) {
        let air_fn_name = json_path
            .file_stem()
            .expect("Invalid path")
            .to_str()
            .expect("Invalid filename")
            .to_string();
        let is_subroutine = json_path
            .parent()
            .expect("Invalid path")
            .ends_with("subroutines");
        for code_type in [
            AutogenCodeType::AIR,
            AutogenCodeType::WITNESS,
            AutogenCodeType::CAIRO,
        ] {
            if code_type == AutogenCodeType::WITNESS && is_subroutine {
                // Skip witness generation for subroutines (in witness code, the subroutines are
                // inlined into their caller files).
                continue;
            }

            let job = AutogenCodeFile {
                air_fn_name: air_fn_name.clone(),
                source_path: json_path.clone(),
                code_type,
            };
            if is_supported(&job) {
                result.push(job);
            } else {
                skipped_files += 1;
            }
        }
    }
    println!(
        "Will generate {} files. Skipped {skipped_files} manually-implemented files.",
        result.len()
    );
    result
}

/// Generates component code from JSON files in the source directory.
fn generate_files(
    stwo_cairo_path: &Path,
    compiled_air_fns: &IndexMap<String, CompiledAirFn>,
    sample_evaluations: &IndexMap<String, SampleEvaluation>,
    jobs: &[AutogenCodeFile],
) {
    for job in jobs.iter() {
        let dest_dir = dest_dir_for_job(job, stwo_cairo_path);
        let compiled_air_fn = compiled_air_fns
            .get(&job.air_fn_name)
            .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
        let sample_evaluation = sample_evaluations.get(&job.air_fn_name);

        // Here we write un-formatted code, which should be formatted by the caller at the end. This
        // is more efficient than formatting each file when generating it due to the startup
        // time of rustfmt and scarb.
        let code = generate_air_fn_code(compiled_air_fn, sample_evaluation, job.code_type);
        let dest_path = generated_code_path(compiled_air_fn, &dest_dir, job.code_type);
        add_file_to_module(dest_path.as_path(), code, job.code_type);
    }
}

fn format_stwo_cairo(stwo_cairo_path: &Path) {
    // Convert the path to absolute, as change_dir works relative to the shell
    // current directory, and this changes after the first call to change_dir.
    let stwo_cairo_path = absolute(stwo_cairo_path).expect("Invalid path to stwo-cairo");

    let shell = Shell::new().unwrap();
    println!("Formatting Rust code...");
    shell.change_dir(stwo_cairo_path.join("stwo_cairo_prover"));
    cmd!(shell, "cargo fmt").quiet().run().unwrap();

    println!("Formatting Cairo code...");
    shell.change_dir(stwo_cairo_path.join("stwo_cairo_verifier"));
    cmd!(shell, "scarb fmt").quiet().run().unwrap();
}

fn generate_registry_properties_file(args: &GenerateStwoCairoArgs) {
    let source_repo_rev = get_git_rev(&args.source);
    let casm_registry_src = read_casm_registry(&args.source);
    let casm_registry_out = VersionedCasmRegistry {
        air_version: source_repo_rev,
        air_fns: casm_registry_src,
    };

    let dest_path = args
        .stwo_cairo_path
        .join("stwo_cairo_prover/crates/common/casm_registry.json");
    fs::write(
        &dest_path,
        serde_json::to_string_pretty(&casm_registry_out).expect("Cannot serialize casm_registry"),
    )
    .unwrap_or_else(|e| panic!("Cannot write to {}: {e}", dest_path.display()))
}

fn dest_dir_for_job(job: &AutogenCodeFile, stwo_cairo_path: &Path) -> PathBuf {
    let path_in_stwo_cairo = match job.code_type {
        AutogenCodeType::WITNESS => "stwo_cairo_prover/crates/prover/src/witness/components",
        AutogenCodeType::AIR => "stwo_cairo_prover/crates/cairo-air/src/components",
        AutogenCodeType::CAIRO => "stwo_cairo_verifier/crates/cairo_air/src/components",
    };

    stwo_cairo_path.join(path_in_stwo_cairo)
}

fn read_casm_registry(compiled_crate_src: &Path) -> IndexMap<String, CompiledAirFnStat> {
    let casm_registry_path = compiled_crate_src.join(REGISTRY_PROPERTIES_FILE_NAME);
    let casm_registry_file = fs::read_to_string(&casm_registry_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", casm_registry_path.display()));
    serde_json::from_str(&casm_registry_file).expect("Invalid casm_registry.json file")
}

#[derive(Debug, Parser)]
struct GenerateStwoCairoArgs {
    /// Source directory of the compiled_casm_air crate
    #[clap(long, default_value = DEFAULT_SOURCE_DIR)]
    source: PathBuf,

    #[clap(long, default_value = DEFAULT_STWO_CAIRO_PATH)]
    stwo_cairo_path: PathBuf,
}

#[derive(Debug, Parser)]
struct SingleArgs {
    #[clap(long, conflicts_with = "cairo_constraints")]
    rust_constraints: bool,
    #[clap(long, conflicts_with = "witness")]
    cairo_constraints: bool,
    #[clap(long, conflicts_with = "rust_constraints")]
    witness: bool,

    /// Source directory of the compiled_casm_air crate
    #[clap(long, default_value = DEFAULT_SOURCE_DIR)]
    source: PathBuf,

    /// JSON file to generate code for
    file: PathBuf,
}

#[derive(Debug, Parser)]
enum Subcommand {
    GenerateStwoCairo(GenerateStwoCairoArgs),
    Single(SingleArgs),
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

/// Main CLI entry point
///
/// # Example usage:
///
/// Generate code to stwo-cairo:
/// `$ cargo run --bin cairo_code_gen -- generate-stwo-cairo --source
///     ./crates/compiled_casm_air/src/ --stwo-cairo-path ~/stwo-cairo/`
///
/// Generate a single file (output to stdout):
/// `$ cargo run --bin cairo_code_gen -- single --source
///      ./crates/compiled_casm_air/src/ --rust-constraints /path/to/biwise_builtin.json`
fn main() {
    let args = Args::try_parse_from(std::env::args()).unwrap_or_else(|e| e.exit());

    match args.subcommand {
        Subcommand::GenerateStwoCairo(cmd_args) => generate_stwo_cairo(cmd_args),
        Subcommand::Single(cmd_args) => generate_single(cmd_args),
    }
}

fn generate_single(args: SingleArgs) {
    if !args.source.exists() {
        panic!("Source directory does not exist: {}", args.source.display());
    }

    let air_fn_name = args
        .file
        .file_stem()
        .expect("Invalid path")
        .to_str()
        .expect("Invalid filename")
        .to_string();

    let code_type = if args.cairo_constraints {
        AutogenCodeType::CAIRO
    } else if args.rust_constraints {
        AutogenCodeType::AIR
    } else if args.witness {
        AutogenCodeType::WITNESS
    } else {
        panic!("Code type not specified. Use --cairo-constraints, --rust-constraints or --witness")
    };

    let job = AutogenCodeFile {
        air_fn_name: air_fn_name.clone(),
        source_path: args.file,
        code_type,
    };
    let (compiled_air_fns, sample_evaluations) = load_air_fns(&args.source, &[job]);

    let raw_code = generate_air_fn_code(
        compiled_air_fns.get(&air_fn_name).expect("AirFn missing"),
        sample_evaluations.get(&air_fn_name),
        code_type,
    );
    let code = format_air_fn_code(raw_code, code_type);

    print!("{}", code);
}

fn generate_stwo_cairo(args: GenerateStwoCairoArgs) {
    if !args.source.exists() {
        panic!("Source directory does not exist: {}", args.source.display());
    }

    let jobs = get_stwo_cairo_jobs(&args);
    let (compiled_air_fns, sample_evaluations) = load_air_fns(&args.source, &jobs);

    generate_files(
        &args.stwo_cairo_path,
        &compiled_air_fns,
        &sample_evaluations,
        &jobs,
    );
    generate_sample_evaluations_file(
        &args
            .stwo_cairo_path
            .join("stwo_cairo_verifier/crates/cairo_air/src/components"),
        &get_git_rev(&args.source),
        &sample_evaluations,
    );

    generate_registry_properties_file(&args);

    format_stwo_cairo(&args.stwo_cairo_path);

    println!(
        "Successfully processed JSON files from {} to {}",
        args.source.display(),
        args.stwo_cairo_path.display(),
    );
}
