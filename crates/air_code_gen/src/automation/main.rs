use std::fs;
use std::path::{Path, PathBuf};

use air_code_gen::code_gen::supported_components::{
    is_supported, AutogenCodeFile, AutogenCodeType,
};
use air_code_gen::code_gen::utils::{generate_air_fn_code, get_git_rev, write_air_fn_code};
use clap::Parser;
use compiled_casm_air::compiled_structs::{CompiledAirFn, CompiledAirFnStat, TraceType};
use compiled_casm_air::utils::{read_json, REGISTRY_PROPERTIES_FILE_NAME};
use indexmap::IndexMap;
use serde::Serialize;

#[derive(Serialize)]
struct VersionedCasmRegistry {
    /// The Git commit hash of the repository we took the statistics from
    pub air_version: String,
    pub air_fns: IndexMap<String, CompiledAirFnStat>,
}

impl VersionedCasmRegistry {
    pub fn add_stats_from(
        &mut self,
        source_registry: &IndexMap<String, CompiledAirFnStat>,
        air_fn: &CompiledAirFn,
    ) {
        if air_fn.r#type == TraceType::Inline {
            // We don't export statistics for inline AirFns
            return;
        }
        self.air_fns.insert(
            air_fn.name.clone(),
            source_registry
                .get(&air_fn.name)
                .unwrap_or_else(|| panic!("Missing casm_registry entry for {}", &air_fn.name))
                .clone(),
        );
    }
}

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

/// For each JSON in the given directory, create AutogenCodeFile jobs
/// for each code type.
fn get_jobs(args: &Args) -> Vec<AutogenCodeFile> {
    let mut result = vec![];

    let mut skipped_files = 0;
    for json_path in jsons_in_dir(&args.source) {
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
            if is_supported(&job) || args.all {
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

fn load_air_fns(jobs: &[AutogenCodeFile]) -> IndexMap<String, CompiledAirFn> {
    let mut result = IndexMap::new();

    for job in jobs {
        if result.contains_key(&job.air_fn_name) {
            continue;
        }
        let compiled_air_fn: CompiledAirFn =
            serde_json::from_value(read_json(job.source_path.to_str().expect("Invalid path")))
                .unwrap_or_else(|err| {
                    panic!("Invalid AirFn in {}: {err}", job.source_path.display())
                });
        result.insert(job.air_fn_name.clone(), compiled_air_fn);
    }

    result
}

/// Generates component code from JSON files in the source directory.
fn generate_files(
    args: &Args,
    compiled_air_fns: &IndexMap<String, CompiledAirFn>,
    jobs: &[AutogenCodeFile],
) {
    let source_repo_rev = get_git_rev(&args.source);
    let source_rev_comment = format!("// AIR version {}\n", source_repo_rev);

    for job in jobs.iter() {
        let dest_dir = dest_dir_for_job(job, args);
        let compiled_air_fn = compiled_air_fns
            .get(&job.air_fn_name)
            .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
        let code = generate_air_fn_code(compiled_air_fn, job.code_type);
        let code = source_rev_comment.clone() + &code;

        write_air_fn_code(compiled_air_fn, code, dest_dir, job.code_type);
    }
}

fn generate_registry_properties_file(
    args: &Args,
    compiled_air_fns: &IndexMap<String, CompiledAirFn>,
    jobs: &[AutogenCodeFile],
) {
    let source_repo_rev = get_git_rev(&args.source);
    let casm_registry_src = read_casm_registry(&args.source);
    let mut casm_registry_out = VersionedCasmRegistry {
        air_version: source_repo_rev,
        air_fns: Default::default(),
    };

    for job in jobs.iter() {
        let compiled_air_fn = compiled_air_fns
            .get(&job.air_fn_name)
            .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));

        // Only include in `casm_registry.json` the components that are supported by the verifier
        if job.code_type == AutogenCodeType::CAIRO {
            casm_registry_out.add_stats_from(&casm_registry_src, compiled_air_fn);
        }
    }

    let dest_path = args
        .casm_registry_dest
        .as_deref()
        .expect("Need path to store casm_registry.json")
        .join(REGISTRY_PROPERTIES_FILE_NAME);
    fs::write(
        &dest_path,
        serde_json::to_string_pretty(&casm_registry_out).expect("Cannot serialize casm_registry"),
    )
    .unwrap_or_else(|e| panic!("Cannot write to {}: {e}", dest_path.display()))
}

fn dest_dir_for_job<'a>(job: &AutogenCodeFile, args: &'a Args) -> &'a PathBuf {
    match job.code_type {
        AutogenCodeType::WITNESS => &args.witness_dest,
        AutogenCodeType::AIR => &args.rust_constraints_dest,
        AutogenCodeType::CAIRO => &args.cairo_constraints_dest,
    }
}

fn read_casm_registry(compiled_jsons_dir: &Path) -> IndexMap<String, CompiledAirFnStat> {
    let casm_registry_path = compiled_jsons_dir
        .parent()
        .unwrap_or_else(|| {
            panic!(
                "Cannot find casm_registry - no parent for {}",
                compiled_jsons_dir.display()
            )
        })
        .join(REGISTRY_PROPERTIES_FILE_NAME);
    let casm_registry_file = fs::read_to_string(&casm_registry_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", casm_registry_path.display()));
    serde_json::from_str(&casm_registry_file).expect("Invalid casm_registry.json file")
}

#[derive(Debug, Parser)]
struct Args {
    /// Generate code from all JSONs in the source directory (default: only generate
    /// components known to be supported by stwo-cairo)
    #[clap(short, long)]
    all: bool,

    #[clap(long)]
    source: PathBuf,

    // TODO: Make the `*_dest` arguments optional
    #[clap(long)]
    rust_constraints_dest: PathBuf,

    #[clap(long)]
    witness_dest: PathBuf,

    #[clap(long)]
    cairo_constraints_dest: PathBuf,

    #[clap(long)]
    casm_registry_dest: Option<PathBuf>,
}

/// Main CLI entry point
///
/// Example usage: `$ cargo run --bin cairo_code_gen -- --source
/// ./crates/compiled_casm_air/src/opcodes --rust-constraints-dest
/// ~/stwo-cairo/stwo_cairo_prover/crates/cairo_air/src/components --witness-dest
/// ~/stwo-cairo/stwo_cairo_prover/crates/prover/src/witness/components --cairo-constraints-dest
/// ~/stwo-cairo/stwo_cairo_verifier/crates/cairo_air/src/components`
fn main() {
    let args = Args::try_parse_from(std::env::args()).unwrap_or_else(|e| e.exit());

    check_args(&args);

    let jobs = get_jobs(&args);
    let compiled_air_fns = load_air_fns(&jobs);
    generate_files(&args, &compiled_air_fns, &jobs);

    if args.casm_registry_dest.is_some() {
        generate_registry_properties_file(&args, &compiled_air_fns, &jobs);
    }

    println!(
        "Successfully processed JSON files from {} to {}, {} and {}",
        args.source.display(),
        args.rust_constraints_dest.display(),
        args.witness_dest.display(),
        args.cairo_constraints_dest.display()
    );
}

fn check_args(args: &Args) {
    for dir in [
        &args.rust_constraints_dest,
        &args.witness_dest,
        &args.cairo_constraints_dest,
    ] {
        if !dir.exists() {
            panic!("Destination directory does not exist: {:?}", dir);
        }
    }
}
