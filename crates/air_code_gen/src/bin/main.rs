use std::fs;
use std::path::{Path, PathBuf};

use air_code_gen::claims_cairo::generate_claims_cairo_file;
use air_code_gen::claims_generator::generate_claim_generator_file;
use air_code_gen::claims_rust::generate_claims_rust_file;
use air_code_gen::components_code_gen::cairo_constraints::sample_evaluations as cairo_sample_evaluations;
use air_code_gen::components_code_gen::circuit_constraints::all_components::generate_all_components_file;
use air_code_gen::components_code_gen::circuit_constraints::sample_evaluations as circuit_sample_evaluations;
use air_code_gen::components_code_gen::supported_components::{
    is_supported, AutogenCodeFile, AutogenCodeType,
};
use air_code_gen::components_rust::generate_components_rust_file;
use air_code_gen::provers_rust::generate_provers_rust_file;
use air_code_gen::utils::{
    add_file_to_module, format_air_fn_code, generate_air_fn_code, generated_code_path, get_git_rev,
    load_air_fns,
};
use air_common::REGISTRY_PROPERTIES_FILE_NAME;
use air_compile::compiled_structs::CompiledAirFn;
use air_infra::airs::casm::casm_registry::create_casm_registry_ordered_by_stwo_cairo;
use air_infra::core::air_fn_registry::AirFnStat;
use clap::Parser;
use eval_air_fn_constraints::SampleEvaluation;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Serialize;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::{
    CANONICAL_SIZE, CANONICAL_WITHOUT_PEDERSEN_SIZE,
};
use xshell::{cmd, Shell};

const DEFAULT_SOURCE_DIR: &str = "./crates/compiled_casm_air";
const DEFAULT_STWO_CAIRO_PATH: &str = "../stwo-cairo/";
const DEFAULT_STWO_CIRCUITS_PATH: &str = "../stwo-circuits/";
pub const CLAIM_GENERATOR_FILE_PATH: &str =
    "stwo_cairo_prover/crates/prover/src/witness/cairo_claim_generator.rs";
pub const CLAIMS_RUST_FILE_PATH: &str = "stwo_cairo_prover/crates/cairo-air/src/claims.rs";
pub const CLAIMS_CAIRO_FILE_PATH: &str = "stwo_cairo_verifier/crates/cairo_air/src/claims.cairo";
pub const COMPONENTS_RUST_FILE_PATH: &str =
    "stwo_cairo_prover/crates/cairo-air/src/cairo_components.rs";
pub const PROVERS_UTILS_FILE_PATH: &str = "stwo_cairo_prover/crates/prover/src/utils.rs";

#[derive(Serialize)]
struct VersionedCasmRegistry {
    /// The Git commit hash of the repository we took the statistics from
    pub air_version: String,
    /// The total number of trace cells in the preprocessed tables, taken from stwo-cairo
    pub canonical_ppt_n_trace_cells: u32,
    pub canonical_without_pedersen_ppt_n_trace_cells: u32,
    pub air_fns: IndexMap<String, AirFnStat>,
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
fn get_jobs(source_dir: &Path, code_types: &[AutogenCodeType]) -> Vec<AutogenCodeFile> {
    let mut result = vec![];

    let mut skipped_files = 0;
    for json_path in jsons_in_dir(&source_dir.join("compiled_jsons")) {
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
        for code_type in code_types {
            if *code_type == AutogenCodeType::WITNESS && is_subroutine {
                // Skip witness generation for subroutines (in witness code, the subroutines are
                // inlined into their caller files).
                continue;
            }

            let job = AutogenCodeFile {
                air_fn_name: air_fn_name.clone(),
                source_path: json_path.clone(),
                code_type: *code_type,
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
    target_repo_path: &Path,
    compiled_air_fns: &IndexMap<String, CompiledAirFn>,
    sample_evaluations: &IndexMap<String, SampleEvaluation>,
    jobs: &[AutogenCodeFile],
) {
    for job in jobs.iter() {
        let dest_dir = dest_dir_for_job(job, target_repo_path);
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

fn format_rust(path: &Path) {
    let shell = Shell::new().unwrap();
    println!("Formatting Rust code...");
    shell.change_dir(path);
    cmd!(shell, "env -u RUSTUP_TOOLCHAIN cargo fmt --all")
        .quiet()
        .run()
        .unwrap();
}

fn format_stwo_cairo(stwo_cairo_path: &Path) {
    format_rust(&stwo_cairo_path.join("stwo_cairo_prover"));

    let shell = Shell::new().unwrap();
    println!("Formatting Cairo code...");
    shell.change_dir(stwo_cairo_path.join("stwo_cairo_verifier"));
    cmd!(shell, "scarb fmt").quiet().run().unwrap();
}

fn generate_registry_properties_file(args: &GenerateStwoCairoArgs) {
    let source_repo_rev = get_git_rev(&args.source);
    let casm_registry_src = read_casm_registry(&args.source);
    let casm_registry_out = VersionedCasmRegistry {
        air_version: source_repo_rev,
        canonical_ppt_n_trace_cells: CANONICAL_SIZE,
        canonical_without_pedersen_ppt_n_trace_cells: CANONICAL_WITHOUT_PEDERSEN_SIZE,
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

fn dest_dir_for_job(job: &AutogenCodeFile, target_repo_path: &Path) -> PathBuf {
    let path_in_target_repo = match job.code_type {
        AutogenCodeType::WITNESS => "stwo_cairo_prover/crates/prover/src/witness/components",
        AutogenCodeType::AIR => "stwo_cairo_prover/crates/cairo-air/src/components",
        AutogenCodeType::CAIRO => "stwo_cairo_verifier/crates/cairo_air/src/components",
        AutogenCodeType::CIRCUIT => "crates/stwo-circuits/src/cairo_air/components",
    };

    target_repo_path.join(path_in_target_repo)
}

fn read_casm_registry(compiled_crate_src: &Path) -> IndexMap<String, AirFnStat> {
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

    /// Skip formatting the generated code
    #[clap(long, default_value_t = false)]
    skip_format: bool,
}

#[derive(Debug, Parser)]
struct GenerateStwoCircuitsArgs {
    /// Source directory of the compiled_casm_air crate
    #[clap(long, default_value = DEFAULT_SOURCE_DIR)]
    source: PathBuf,

    #[clap(long, default_value = DEFAULT_STWO_CIRCUITS_PATH)]
    stwo_circuits_path: PathBuf,
}

#[derive(Debug, Parser)]
struct SingleArgs {
    #[clap(long, group = "code_type")]
    rust_constraints: bool,
    #[clap(long, group = "code_type")]
    circuit_constraints: bool,
    #[clap(long, group = "code_type")]
    cairo_constraints: bool,
    #[clap(long, group = "code_type")]
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
    GenerateStwoCircuits(GenerateStwoCircuitsArgs),
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
///     ./crates/compiled_casm_air/ --stwo-cairo-path ~/stwo-cairo/`
///
/// Generate a single file (output to stdout):
/// `$ cargo run --bin cairo_code_gen -- single --source
///      ./crates/compiled_casm_air/ --rust-constraints /path/to/biwise_builtin.json`
fn main() {
    let args = Args::try_parse_from(std::env::args()).unwrap_or_else(|e| e.exit());

    match args.subcommand {
        Subcommand::GenerateStwoCairo(cmd_args) => generate_stwo_cairo(cmd_args),
        Subcommand::GenerateStwoCircuits(cmd_args) => generate_stwo_circuits(cmd_args),
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
    } else if args.circuit_constraints {
        AutogenCodeType::CIRCUIT
    } else if args.rust_constraints {
        AutogenCodeType::AIR
    } else if args.witness {
        AutogenCodeType::WITNESS
    } else {
        panic!("Code type not specified. Use --cairo-constraints, --circuit-constraints, --rust-constraints or --witness")
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

    print!("{code}");
}

fn generate_stwo_cairo(args: GenerateStwoCairoArgs) {
    if !args.source.exists() {
        panic!("Source directory does not exist: {}", args.source.display());
    }

    let jobs = get_jobs(
        &args.source,
        &[
            AutogenCodeType::AIR,
            AutogenCodeType::CAIRO,
            AutogenCodeType::WITNESS,
        ],
    );
    let (compiled_air_fns, sample_evaluations) = load_air_fns(&args.source, &jobs);

    generate_files(
        &args.stwo_cairo_path,
        &compiled_air_fns,
        &sample_evaluations,
        &jobs,
    );
    cairo_sample_evaluations::generate_sample_evaluations_file(
        &args
            .stwo_cairo_path
            .join("stwo_cairo_verifier/crates/cairo_air/src/components"),
        &get_git_rev(&args.source),
        &sample_evaluations,
    );

    generate_registry_properties_file(&args);

    let compiled_regisry = create_casm_registry_ordered_by_stwo_cairo();

    let claim_generator_code = generate_claim_generator_file(&compiled_regisry);
    fs::write(
        args.stwo_cairo_path.join(CLAIM_GENERATOR_FILE_PATH),
        claim_generator_code.to_string().unwrap(),
    )
    .expect("Failed to write claim generator code");

    let claims_rust_code = generate_claims_rust_file(&compiled_regisry);
    fs::write(
        args.stwo_cairo_path.join(CLAIMS_RUST_FILE_PATH),
        claims_rust_code.to_string().unwrap(),
    )
    .expect("Failed to write claims rust code");

    let components_rust_code =
        generate_components_rust_file(&compiled_regisry.keys().collect_vec());
    fs::write(
        args.stwo_cairo_path.join(COMPONENTS_RUST_FILE_PATH),
        components_rust_code.to_string().unwrap(),
    )
    .expect("Failed to write components rust code");

    let provers_utils_code = generate_provers_rust_file(&compiled_regisry.keys().collect_vec());
    fs::write(
        args.stwo_cairo_path.join(PROVERS_UTILS_FILE_PATH),
        provers_utils_code.to_string().unwrap(),
    )
    .expect("Failed to write provers utils code");

    let claims_cairo_code = generate_claims_cairo_file(&compiled_regisry);
    fs::write(
        args.stwo_cairo_path.join(CLAIMS_CAIRO_FILE_PATH),
        claims_cairo_code.to_string().unwrap(),
    )
    .expect("Failed to write claims cairo code");

    if !args.skip_format {
        format_stwo_cairo(&args.stwo_cairo_path);
    }

    println!(
        "Successfully processed JSON files from {} to {}",
        args.source.display(),
        args.stwo_cairo_path.display(),
    );
}

fn generate_stwo_circuits(args: GenerateStwoCircuitsArgs) {
    if !args.source.exists() {
        panic!("Source directory does not exist: {}", args.source.display());
    }

    let jobs = get_jobs(&args.source, &[AutogenCodeType::CIRCUIT]);
    let (compiled_air_fns, sample_evaluations) = load_air_fns(&args.source, &jobs);

    generate_files(
        &args.stwo_circuits_path,
        &compiled_air_fns,
        &sample_evaluations,
        &jobs,
    );

    circuit_sample_evaluations::generate_sample_evaluations_file(
        &args
            .stwo_circuits_path
            .join("crates/stwo-circuits/src/cairo_air"),
        &get_git_rev(&args.source),
        &sample_evaluations,
    );

    let compiled_regisry = create_casm_registry_ordered_by_stwo_cairo();
    generate_all_components_file(
        &args
            .stwo_circuits_path
            .join("crates/stwo-circuits/src/cairo_air"),
        &compiled_regisry,
    );

    format_rust(&args.stwo_circuits_path);
}
