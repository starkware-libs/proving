use std::fs;
use std::path::{Path, PathBuf};

use air_code_gen::cairo::claims::generate_claims_cairo_file;
use air_code_gen::cairo::sample_evaluations as cairo_sample_evaluations;
use air_code_gen::circuit::all_components::generate_all_components_file;
use air_code_gen::circuit::sample_evaluations as circuit_sample_evaluations;
use air_code_gen::rust::claims::generate_claims_rust_file;
use air_code_gen::rust::claims_generator::generate_claim_generator_file;
use air_code_gen::rust::components::generate_components_rust_file;
use air_code_gen::rust::provers::generate_provers_rust_file;
use air_code_gen::supported_components::{AutogenCodeFile, AutogenCodeType, is_supported};
use air_code_gen::utils::{
    STWO_CAIRO_AIR_CONFIG, STWO_CIRCUITS_AIR_CONFIG, add_file_to_module, format_air_fn_code,
    generate_air_fn_code, generated_code_path, get_git_rev, load_air_fns,
};
use air_common::REGISTRY_PROPERTIES_FILE_NAME;
use air_compile::compiled_structs::CompiledAirFn;
use air_infra::core::air_fn_registry::AirFnStat;
use airs::casm::casm_registry::create_casm_registry_ordered_by_stwo_cairo;
use clap::Parser;
use eval_air_fn_constraints::SampleEvaluation;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Serialize;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use xshell::{Shell, cmd};

const DEFAULT_SOURCE_DIR: &str = ".";
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
    pub canonical_small_ppt_n_trace_cells: u32,
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
fn get_jobs(
    source_dir: &Path,
    dest_dir: &Path,
    code_type: &AutogenCodeType,
) -> Vec<AutogenCodeFile> {
    let mut result = vec![];
    let mut skipped_files = 0;

    for json_path in jsons_in_dir(source_dir) {
        let air_fn_name = json_path
            .file_stem()
            .expect("Invalid path")
            .to_str()
            .expect("Invalid filename")
            .to_string();
        let is_subroutine = json_path.parent().expect("Invalid path").ends_with("subroutines");
        if *code_type == AutogenCodeType::WITNESS && is_subroutine {
            // Skip witness generation for subroutines (in witness code, the subroutines are
            // inlined into their caller files).
            continue;
        }

        let job = AutogenCodeFile {
            air_fn_name: air_fn_name.clone(),
            source_path: json_path.clone(),
            dest_dir: dest_dir.to_path_buf(),
            code_type: *code_type,
        };

        if is_supported(&job) {
            result.push(job);
        } else {
            skipped_files += 1;
        }
    }

    println!(
        "Will generate {} files of type {:?}. Skipped {skipped_files} manually-implemented files.",
        result.len(),
        code_type,
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
        let compiled_air_fn = compiled_air_fns
            .get(&job.air_fn_name)
            .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
        let sample_evaluation = sample_evaluations.get(&job.air_fn_name);

        // Here we write un-formatted code, which should be formatted by the caller at the end. This
        // is more efficient than formatting each file when generating it due to the startup
        // time of rustfmt and scarb.
        let code = generate_air_fn_code(compiled_air_fn, sample_evaluation, job.code_type);
        let dest_path = generated_code_path(
            compiled_air_fn,
            &target_repo_path.join(&job.dest_dir),
            &job.code_type,
        );
        add_file_to_module(dest_path.as_path(), code, &job.code_type);
    }
}

fn format_rust(path: &Path) {
    let shell = Shell::new().unwrap();
    println!("Formatting Rust code...");
    shell.change_dir(path);
    cmd!(shell, "env -u RUSTUP_TOOLCHAIN cargo fmt --all").quiet().run().unwrap();
}

fn format_stwo_cairo(stwo_cairo_path: &Path) {
    format_rust(&stwo_cairo_path.join("stwo_cairo_prover"));

    let shell = Shell::new().unwrap();
    println!("Formatting Cairo code...");
    shell.change_dir(stwo_cairo_path.join("stwo_cairo_verifier"));
    cmd!(shell, "scarb fmt").quiet().run().unwrap();
}

fn generate_registry_properties_file(src: &Path, dst: &Path) {
    let registry_path = src.join(REGISTRY_PROPERTIES_FILE_NAME);
    let registry_file = fs::read_to_string(&registry_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", registry_path.display()));
    let registry_src = serde_json::from_str(&registry_file).expect("Invalid registry.json file");

    let registry_out = VersionedCasmRegistry {
        air_version: get_git_rev(src),
        canonical_ppt_n_trace_cells: PreProcessedTraceVariant::Canonical.n_trace_cells(),
        canonical_without_pedersen_ppt_n_trace_cells:
            PreProcessedTraceVariant::CanonicalWithoutPedersen.n_trace_cells(),
        canonical_small_ppt_n_trace_cells: PreProcessedTraceVariant::CanonicalSmall.n_trace_cells(),
        air_fns: registry_src,
    };

    fs::write(dst, serde_json::to_string_pretty(&registry_out).expect("Cannot serialize registry"))
        .unwrap_or_else(|e| panic!("Cannot write to {}: {e}", dst.display()))
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
    /// Source directory of stwo_air_infra
    #[clap(long, default_value = DEFAULT_SOURCE_DIR)]
    source: PathBuf,

    #[clap(long, default_value = DEFAULT_STWO_CIRCUITS_PATH)]
    stwo_circuits_path: PathBuf,

    /// Skip formatting the generated code
    #[clap(long, default_value_t = false)]
    skip_format: bool,
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
/// `$ cargo run --bin air_code_gen -- generate-stwo-cairo --source
///     . --stwo-cairo-path ~/stwo-cairo/`
///
/// Generate a single file (output to stdout):
/// `$ cargo run --bin air_code_gen -- single --source
///      . --rust-constraints /path/to/biwise_builtin.json`
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
        AutogenCodeType::AIR(STWO_CAIRO_AIR_CONFIG)
    } else if args.witness {
        AutogenCodeType::WITNESS
    } else {
        panic!(
            "Code type not specified. Use --cairo-constraints, --circuit-constraints, \
             --rust-constraints or --witness"
        )
    };

    let job = AutogenCodeFile {
        air_fn_name: air_fn_name.clone(),
        source_path: args.file,
        dest_dir: "<stdout>".into(), // We output to stdout and don't use this value
        code_type,
    };
    let (compiled_air_fns, sample_evaluations) = load_air_fns(&args.source, &[job]);

    let raw_code = generate_air_fn_code(
        compiled_air_fns.get(&air_fn_name).expect("AirFn missing"),
        sample_evaluations.get(&air_fn_name),
        code_type,
    );
    let code = format_air_fn_code(raw_code, &code_type);

    print!("{code}");
}

fn generate_stwo_cairo(args: GenerateStwoCairoArgs) {
    if !args.source.exists() {
        panic!("Source directory does not exist: {}", args.source.display());
    }

    let compiled_casm_crate = args.source.join("crates/compiled_casm_air");
    let compiled_circuit_crate = args.source.join("crates/compiled_circuit_air");
    let jobs_desc = [
        (
            &compiled_casm_crate,
            Path::new("stwo_cairo_prover/crates/cairo-air/src/components"),
            AutogenCodeType::AIR(STWO_CAIRO_AIR_CONFIG),
        ),
        (
            &compiled_casm_crate,
            Path::new("stwo_cairo_verifier/crates/cairo_air/src/components"),
            AutogenCodeType::CAIRO,
        ),
        (
            &compiled_casm_crate,
            Path::new("stwo_cairo_prover/crates/prover/src/witness/components"),
            AutogenCodeType::WITNESS,
        ),
        (
            &compiled_circuit_crate,
            Path::new("stwo_cairo_verifier/crates/circuit_air/src/components"),
            AutogenCodeType::CAIRO,
        ),
    ];

    for (src, dst, code) in jobs_desc {
        let jobs = get_jobs(&src.join("compiled_jsons"), dst, &code);
        let (compiled_air_fns, sample_evaluations) = load_air_fns(src, &jobs);
        generate_files(&args.stwo_cairo_path, &compiled_air_fns, &sample_evaluations, &jobs);

        if code == AutogenCodeType::CAIRO {
            cairo_sample_evaluations::generate_sample_evaluations_file(
                &args.stwo_cairo_path.join(dst),
                &get_git_rev(&args.source),
                &sample_evaluations,
            );
        }
    }

    generate_registry_properties_file(
        &compiled_casm_crate,
        &args.stwo_cairo_path.join("stwo_cairo_prover/crates/common/casm_registry.json"),
    );

    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();

    let claim_generator_code = generate_claim_generator_file(&compiled_registry);
    fs::write(
        args.stwo_cairo_path.join(CLAIM_GENERATOR_FILE_PATH),
        claim_generator_code.to_string().unwrap(),
    )
    .expect("Failed to write claim generator code");

    let claims_rust_code = generate_claims_rust_file(&compiled_registry);
    fs::write(
        args.stwo_cairo_path.join(CLAIMS_RUST_FILE_PATH),
        claims_rust_code.to_string().unwrap(),
    )
    .expect("Failed to write claims rust code");

    let components_rust_code = generate_components_rust_file(&compiled_registry);
    fs::write(
        args.stwo_cairo_path.join(COMPONENTS_RUST_FILE_PATH),
        components_rust_code.to_string().unwrap(),
    )
    .expect("Failed to write components rust code");

    let provers_utils_code = generate_provers_rust_file(&compiled_registry.keys().collect_vec());
    fs::write(
        args.stwo_cairo_path.join(PROVERS_UTILS_FILE_PATH),
        provers_utils_code.to_string().unwrap(),
    )
    .expect("Failed to write provers utils code");

    let claims_cairo_code = generate_claims_cairo_file(&compiled_registry);
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
        compiled_casm_crate.display(),
        args.stwo_cairo_path.display(),
    );
}

fn generate_stwo_circuits(args: GenerateStwoCircuitsArgs) {
    if !args.source.exists() {
        panic!("Source directory does not exist: {}", args.source.display());
    }

    let compiled_casm_crate = args.source.join("crates/compiled_casm_air");
    let compiled_circuit_crate = args.source.join("crates/compiled_circuit_air");
    let jobs_desc = [
        (
            &compiled_casm_crate,
            Path::new("crates/cairo_verifier/src/components"),
            AutogenCodeType::CIRCUIT,
        ),
        (
            &compiled_circuit_crate,
            Path::new("crates/circuit_verifier/src/components"),
            AutogenCodeType::CIRCUIT,
        ),
        (
            &compiled_circuit_crate,
            Path::new("crates/circuit_prover/src/circuit_air/components"),
            AutogenCodeType::AIR(STWO_CIRCUITS_AIR_CONFIG),
        ),
    ];

    let git_rev = get_git_rev(&args.source);

    for (src, dst, code) in jobs_desc {
        let jobs = get_jobs(&src.join("compiled_jsons"), dst, &code);
        let (compiled_air_fns, sample_evaluations) = load_air_fns(src, &jobs);
        generate_files(&args.stwo_circuits_path, &compiled_air_fns, &sample_evaluations, &jobs);

        if code == AutogenCodeType::CIRCUIT {
            circuit_sample_evaluations::generate_sample_evaluations_file(
                &args.stwo_circuits_path.join(dst).join(".."),
                &git_rev,
                &sample_evaluations,
            );
        }
    }

    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();
    generate_all_components_file(
        &args.stwo_circuits_path.join("crates/cairo_verifier/src"),
        &compiled_registry,
    );

    if !args.skip_format {
        format_rust(&args.stwo_circuits_path);
    }
}
