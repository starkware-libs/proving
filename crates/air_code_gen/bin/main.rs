use std::fs;
use std::path::{Path, PathBuf};

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
    generate_air_fn_code, generated_code_path, load_air_fns,
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

const DEFAULT_ROOT_DIR: &str = ".";
pub const CLAIM_GENERATOR_FILE_PATH: &str = "crates/prover/src/witness/cairo_claim_generator.rs";
pub const CLAIMS_RUST_FILE_PATH: &str = "crates/cairo-air/src/claims.rs";
pub const COMPONENTS_RUST_FILE_PATH: &str = "crates/cairo-air/src/cairo_components.rs";
pub const PROVERS_UTILS_FILE_PATH: &str = "crates/prover/src/utils.rs";

#[derive(Serialize)]
struct CasmRegistry {
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

fn format_cairo(path: &Path) {
    let shell = Shell::new().unwrap();
    println!("Formatting Cairo code...");
    shell.change_dir(path.join("stwo_cairo_verifier"));
    cmd!(shell, "scarb fmt").quiet().run().unwrap();
}

fn generate_registry_properties_file(src: &Path, dst: &Path) {
    let registry_path = src.join(REGISTRY_PROPERTIES_FILE_NAME);
    let registry_file = fs::read_to_string(&registry_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", registry_path.display()));
    let registry_src = serde_json::from_str(&registry_file).expect("Invalid registry.json file");

    let registry_out = CasmRegistry {
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
struct GenerateArgs {
    /// Repository root: holds the compiled AIR dirs under `outputs/` and is the destination
    /// of the generated code.
    #[clap(long, default_value = DEFAULT_ROOT_DIR)]
    root_dir: PathBuf,

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

    /// The directory that contains the compiled AIR and sample evaluations: either
    /// `outputs/compiled_casm_air` or `outputs/compiled_circuit_air`.
    #[clap(long)]
    source: PathBuf,

    /// JSON file to generate code for
    file: PathBuf,
}

#[derive(Debug, Parser)]
enum Subcommand {
    GenerateStwoCairo(GenerateArgs),
    GenerateStwoCircuits(GenerateArgs),
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
/// Generate the stwo-cairo code:
/// `$ cargo run --bin air_code_gen -- generate-stwo-cairo --root-dir ~/proving-dev`
///
/// Generate the stwo-circuits code:
/// `$ cargo run --bin air_code_gen -- generate-stwo-circuits`
///
/// Generate a single file (output to stdout):
/// `$ cargo run --bin air_code_gen -- single --source outputs/compiled_circuit_air
///     --rust-constraints /path/to/biwise_builtin.json`
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

fn generate_stwo_cairo(args: GenerateArgs) {
    if !args.root_dir.exists() {
        panic!("Source directory does not exist: {}", args.root_dir.display());
    }

    let compiled_casm_dir = args.root_dir.join("outputs/compiled_casm_air");
    let compiled_circuit_dir = args.root_dir.join("outputs/compiled_circuit_air");
    let jobs_desc = [
        (
            &compiled_casm_dir,
            Path::new("crates/cairo-air/src/components"),
            AutogenCodeType::AIR(STWO_CAIRO_AIR_CONFIG),
        ),
        (
            &compiled_casm_dir,
            Path::new("crates/prover/src/witness/components"),
            AutogenCodeType::WITNESS,
        ),
        (
            &compiled_circuit_dir,
            Path::new("stwo_cairo_verifier/crates/circuit_air/src/components"),
            AutogenCodeType::CAIRO,
        ),
    ];

    for (src, dst, code) in jobs_desc {
        let jobs = get_jobs(&src.join("compiled_jsons"), dst, &code);
        let (compiled_air_fns, sample_evaluations) = load_air_fns(src, &jobs);
        generate_files(&args.root_dir, &compiled_air_fns, &sample_evaluations, &jobs);

        if code == AutogenCodeType::CAIRO {
            cairo_sample_evaluations::generate_sample_evaluations_file(
                &args.root_dir.join(dst),
                &sample_evaluations,
            );
        }
    }

    generate_registry_properties_file(
        &compiled_casm_dir,
        &args.root_dir.join("crates/common/casm_registry.json"),
    );

    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();

    let claim_generator_code = generate_claim_generator_file(&compiled_registry);
    fs::write(
        args.root_dir.join(CLAIM_GENERATOR_FILE_PATH),
        claim_generator_code.to_string().unwrap(),
    )
    .expect("Failed to write claim generator code");

    let claims_rust_code = generate_claims_rust_file(&compiled_registry);
    fs::write(args.root_dir.join(CLAIMS_RUST_FILE_PATH), claims_rust_code.to_string().unwrap())
        .expect("Failed to write claims rust code");

    let components_rust_code = generate_components_rust_file(&compiled_registry);
    fs::write(
        args.root_dir.join(COMPONENTS_RUST_FILE_PATH),
        components_rust_code.to_string().unwrap(),
    )
    .expect("Failed to write components rust code");

    let provers_utils_code = generate_provers_rust_file(&compiled_registry.keys().collect_vec());
    fs::write(args.root_dir.join(PROVERS_UTILS_FILE_PATH), provers_utils_code.to_string().unwrap())
        .expect("Failed to write provers utils code");

    if !args.skip_format {
        format_rust(&args.root_dir);
        format_cairo(&args.root_dir);
    }

    println!(
        "Successfully processed JSON files from {} to {}",
        compiled_casm_dir.display(),
        args.root_dir.display(),
    );
}

fn generate_stwo_circuits(args: GenerateArgs) {
    if !args.root_dir.exists() {
        panic!("Source directory does not exist: {}", args.root_dir.display());
    }

    let compiled_casm_dir = args.root_dir.join("outputs/compiled_casm_air");
    let compiled_circuit_dir = args.root_dir.join("outputs/compiled_circuit_air");
    let jobs_desc = [
        (
            &compiled_casm_dir,
            Path::new("crates/cairo_verifier/src/components"),
            AutogenCodeType::CIRCUIT,
        ),
        (
            &compiled_circuit_dir,
            Path::new("crates/circuit_verifier/src/components"),
            AutogenCodeType::CIRCUIT,
        ),
        (
            &compiled_circuit_dir,
            Path::new("crates/circuit_prover/src/circuit_air/components"),
            AutogenCodeType::AIR(STWO_CIRCUITS_AIR_CONFIG),
        ),
    ];

    for (src, dst, code) in jobs_desc {
        let jobs = get_jobs(&src.join("compiled_jsons"), dst, &code);
        let (compiled_air_fns, sample_evaluations) = load_air_fns(src, &jobs);
        generate_files(&args.root_dir, &compiled_air_fns, &sample_evaluations, &jobs);

        if code == AutogenCodeType::CIRCUIT {
            circuit_sample_evaluations::generate_sample_evaluations_file(
                &args.root_dir.join(dst).join(".."),
                &sample_evaluations,
            );
        }
    }

    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();
    generate_all_components_file(
        &args.root_dir.join("crates/cairo_verifier/src"),
        &compiled_registry,
    );

    if !args.skip_format {
        format_rust(&args.root_dir);
    }
}
