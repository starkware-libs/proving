use std::path::Path;
use std::process::Command;

/// Runs the `circuit-params` binary for a single trace log size, asserting success. With
/// `registry`, passes `--registry` (JSON output); otherwise emits the human-readable report.
///
/// `output_path`, when given, is passed as `--output-path` and the file's contents are returned;
/// otherwise the binary's stdout is returned. Note that `run_binary` writes tracing output to
/// stdout, so anything that must be parsed has to go through `--output-path`.
fn run(registry: bool, output_path: Option<&Path>) -> String {
    let binary = env!("CARGO_BIN_EXE_circuit-params");
    let mut command = Command::new(binary);
    command.args(["--min-trace-log-size", "25", "--max-trace-log-size", "25"]);
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
    run(false, None);
}

// Slow: builds and Merkle-commits a ~2^24 preprocessed trace.
#[test]
#[cfg(feature = "slow-tests")]
fn run_circuit_params_binary_json() {
    let tmp_dir = tempfile::tempdir().expect("Cannot create temporary directory");
    let output_path = tmp_dir.path().join("registry.json");
    let json = run(true, Some(&output_path));

    let registry: circuit_registry::CircuitRegistry =
        serde_json::from_str(&json).unwrap_or_else(|err| panic!("invalid json: {err}\n{json}"));

    assert_eq!(registry.leaf_verifiers.len(), 1);
    assert_eq!(registry.leaf_verifiers[0].trace_log_size, 25);
    assert_eq!(registry.multiverifiers.len(), 1);
    assert!(registry.circuit_proof_configs.contains_key(&registry.leaf_verifiers[0].config));
    // The circuit hash commits to the config, so distinct circuits must not collide.
    assert_ne!(registry.leaf_verifiers[0].circuit_hash, registry.multiverifiers[0].circuit_hash);
}
