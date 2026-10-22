use std::path::PathBuf;
use std::process::Command;

/// The committed canonical_small circuit registry.
fn circuit_registry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/circuit_registry.json")
}

/// A malformed `--program_input` makes `load_leaves` fail (before any expensive setup); the binary
/// must exit non-zero (the `run_binary` error arm) rather than panicking or silently succeeding.
#[test]
fn test_invalid_program_input_exits_nonzero() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    let leaves_path = dir.join("leaves.json");
    std::fs::write(&leaves_path, b"not valid json").expect("write bad leaves.json");
    let status = Command::new(env!("CARGO_BIN_EXE_stwo_run_and_prove_recursive_tree"))
        .arg("--program_input")
        .arg(&leaves_path)
        .arg("--proof_path")
        .arg(dir.join("p"))
        .arg("--program_output")
        .arg(dir.join("po"))
        .arg("--packed_node_tree_path")
        .arg(dir.join("pout"))
        .arg("--circuit_registry_json")
        .arg(circuit_registry_path())
        .status()
        .expect("spawn recursive-tree binary");
    assert!(!status.success(), "binary should exit non-zero on malformed input, got: {status:?}",);
}

/// The success path: the binary self-folds a generated leaf into a root proof. The proof itself is
/// checked in `test_fold_one_leaf`; this test covers argument parsing and the output files.
#[cfg(feature = "slow-tests")]
#[test]
fn test_single_leaf_cli_succeeds() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    use stwo_run_and_prove_recursive_tree::test_support::{LEAF_TASK_OUTPUT, generate_leaf};
    let leaf = generate_leaf(dir, &LEAF_TASK_OUTPUT);
    let leaf_path = dir.join("leaf.json");
    std::fs::write(&leaf_path, serde_json::to_string(&leaf).expect("serialize leaf"))
        .expect("write leaf.json");
    let manifest_path = dir.join("leaves.json");
    std::fs::write(&manifest_path, format!(r#"{{"leaves":[{:?}]}}"#, leaf_path.to_str().unwrap()))
        .expect("write leaves.json");

    let root_proof = dir.join("root.proof");
    let status = Command::new(env!("CARGO_BIN_EXE_stwo_run_and_prove_recursive_tree"))
        .arg("--program_input")
        .arg(&manifest_path)
        .arg("--proof_path")
        .arg(&root_proof)
        .arg("--program_output")
        .arg(dir.join("root_outputs.json"))
        .arg("--packed_node_tree_path")
        .arg(dir.join("root_packed.json"))
        .arg("--circuit_registry_json")
        .arg(circuit_registry_path())
        .status()
        .expect("spawn recursive-tree binary");

    assert!(status.success(), "single-leaf run should succeed, got: {status:?}");
    // The root is the Cairo verifier's arguments stream, and the packed root is a single-child
    // `Composite` (the self-fold).
    let root_felts: Vec<String> =
        serde_json::from_str(&std::fs::read_to_string(&root_proof).expect("read root proof"))
            .expect("root proof parses as a JSON array of hex felts");
    assert!(!root_felts.is_empty() && root_felts.iter().all(|f| f.starts_with("0x")));
    let packed: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("root_packed.json")).expect("read packed tree"),
    )
    .expect("packed tree parses");
    assert_eq!(packed["Composite"]["subtasks"].as_array().expect("subtasks array").len(), 1);
    assert!(dir.join("root_outputs.json").exists(), "root outputs file written");
}
