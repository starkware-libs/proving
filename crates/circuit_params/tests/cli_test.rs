use std::path::{Path, PathBuf};
use std::process::Command;

/// The trace log size both committed families' leaves are proven at.
const TRACE_LOG_SIZE: &str = "20";

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// A circuit family committed in this repo: the files `circuit-params` builds its circuits from,
/// and the registry the resulting hashes are committed in — which the binaries of that family then
/// run with, and which the tests below are the fix target of.
struct Family {
    cairo_prover_params_json: PathBuf,
    circuit_fri_config_json: PathBuf,
    program: PathBuf,
    /// Read only by the fix-target tests, which are the slow ones.
    #[cfg(feature = "slow-tests")]
    committed_registry: PathBuf,
}

impl Family {
    /// The family the recursive tree's golden e2e (`test_golden_four_leaves_e2e`) proves with; its
    /// leaves attest to the leaf simple bootloader.
    fn recursive_tree() -> Self {
        let tree = crates_dir().join("stwo_run_and_prove_recursive_tree/test_data");
        Self {
            cairo_prover_params_json: tree.join("cairo_prover_params.json"),
            circuit_fri_config_json: tree.join("circuit_fri_config.json"),
            program: tree.join("leaf_simple_bootloader_compiled.json"),
            #[cfg(feature = "slow-tests")]
            committed_registry: tree.join("circuit_registry.json"),
        }
    }

    /// The family the leaf prover's own CLI test proves against; its leaves attest to
    /// `use_all_opcodes_and_builtins`. Only its registry is checked here, by a slow test.
    #[cfg(feature = "slow-tests")]
    fn leaf_prover() -> Self {
        let data = crates_dir().join("leaf_prover/tests/data");
        Self {
            cairo_prover_params_json: data.join("cairo_prover_params_canonical_small.json"),
            circuit_fri_config_json: data.join("circuit_fri_config_canonical_small.json"),
            program: data.join("use_all_opcodes_and_builtins_compiled.json"),
            committed_registry: data.join("circuit_registry_canonical_small.json"),
        }
    }
}

/// Runs the `circuit-params` binary over `family`, asserting success; `registry` passes
/// `--registry`.
///
/// Returns the `--output-path` file's contents when `output_path` is given, the binary's stdout
/// otherwise (`run_binary` mixes tracing into stdout, so parsed output must go through a file).
fn run(family: &Family, registry: bool, output_path: Option<&Path>) -> String {
    let binary = env!("CARGO_BIN_EXE_circuit-params");
    let mut command = Command::new(binary);
    command
        .arg("--min-trace-log-size")
        .arg(TRACE_LOG_SIZE)
        .arg("--max-trace-log-size")
        .arg(TRACE_LOG_SIZE)
        .arg("--cairo-prover-params-json")
        .arg(&family.cairo_prover_params_json)
        .arg("--circuit-fri-config-json")
        .arg(&family.circuit_fri_config_json)
        .arg("--program")
        .arg(&family.program);
    if registry {
        command.arg("--registry");
    }
    if let Some(path) = output_path {
        command.arg("--output-path").arg(path);
    }
    let output = command.output().expect("Cannot run circuit-params");

    assert!(
        output.status.success(),
        "circuit-params failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    match output_path {
        Some(path) => std::fs::read_to_string(path).expect("Cannot read the output file"),
        None => String::from_utf8(output.stdout).expect("stdout is not valid UTF-8"),
    }
}

/// Run the human-readable report path; only a successful run is checked.
#[test]
fn run_circuit_params_binary_info() {
    run(&Family::recursive_tree(), false, None);
}

/// Regenerates `family`'s registry and asserts it matches the committed one, so a change to the
/// circuits shows up here rather than as a hash mismatch inside a proving binary. Run with `FIX=1`
/// to regenerate.
#[cfg(feature = "slow-tests")]
fn assert_committed_registry_is_up_to_date(family: &Family) {
    let tmp_dir = tempfile::tempdir().expect("Cannot create temporary directory");
    let output_path = tmp_dir.path().join("registry.json");
    let json = run(family, true, Some(&output_path));

    let registry: circuit_registry::CircuitRegistry =
        serde_json::from_str(&json).unwrap_or_else(|err| panic!("invalid json: {err}\n{json}"));

    assert_eq!(registry.leaf_verifiers.len(), 1);
    assert_eq!(registry.leaf_verifiers[0].trace_log_size, 20);
    assert_eq!(registry.multiverifiers.len(), 1);
    assert!(registry.circuit_proof_configs.contains_key(&registry.leaf_verifiers[0].config));
    // The circuit hash commits to the config, so distinct circuits must not collide.
    assert_ne!(registry.leaf_verifiers[0].circuit_hash, registry.multiverifiers[0].circuit_hash);

    let committed_path = &family.committed_registry;
    if std::env::var("FIX").is_ok() {
        std::fs::write(committed_path, &json)
            .unwrap_or_else(|err| panic!("Cannot write to {}: {err}", committed_path.display()));
        return;
    }
    // Compared as JSON: `CircuitRegistry` holds the prover params, whose type is not `PartialEq`.
    let committed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(committed_path).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap(),
        committed,
        "{} does not match this run's output; run with FIX=1 to regenerate",
        committed_path.display()
    );
}

// Slow: commits the verified Cairo proofs' preprocessed trace (lifted to trace + blowup).
#[test]
#[cfg(feature = "slow-tests")]
fn test_recursive_tree_registry_is_up_to_date() {
    assert_committed_registry_is_up_to_date(&Family::recursive_tree());
}

// Slow: as above. This family's registry is used by `leaf_prover`'s CLI test.
#[test]
#[cfg(feature = "slow-tests")]
fn test_leaf_prover_registry_is_up_to_date() {
    assert_committed_registry_is_up_to_date(&Family::leaf_prover());
}
