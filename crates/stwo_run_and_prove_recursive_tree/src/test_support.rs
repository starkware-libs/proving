//! Generates the leaf the e2e tests fold, for the lib tests and the CLI test alike. Compiled under
//! the `slow-tests` feature only.

use std::path::{Path, PathBuf};

use leaf_prover::prove_leaf::prove_leaf_from_files;
use num_bigint::BigUint;

use crate::{LeafInput, LeafOutputHashType, LeafOutputPreimage};

/// The committed canonical_small registry the e2e proves against.
pub fn circuit_registry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/circuit_registry.json")
}

/// The simple-output task's input for the leaf most tests fold; the program copies it to its
/// output.
pub const LEAF_TASK_OUTPUT: [u32; 3] = [11, 13, 17];

/// The leaf bootloader's input: one simple-output task over `task_output` with a blake program
/// hash, and the path to dump the output preimage to.
fn leaf_bl_input_json(dump_path: &Path, task_output: &[u32]) -> String {
    let task_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/simple_output_compiled.json");
    serde_json::to_string_pretty(&serde_json::json!({
        "tasks": [{
            "type": "RunProgramTask",
            "path": task_path.to_str().unwrap(),
            "program_input": {"output": task_output},
            "program_hash_function": "blake",
        }],
        "fact_topologies_path": null,
        "single_page": true,
        "output_preimage_dump_path": dump_path.to_str().unwrap(),
    }))
    .unwrap()
}

/// Proves the leaf simple bootloader running the simple-output task over `task_output` and builds
/// the `LeafInput` the backend would: the proof plus the dumped output preimage, re-encoded from
/// hex to decimal.
pub fn generate_leaf(dir: &Path, task_output: &[u32]) -> LeafInput {
    let dump_path = dir.join("leaf_preimage.json");
    let input_path = dir.join("leaf_bl_input.json");
    std::fs::write(&input_path, leaf_bl_input_json(&dump_path, task_output)).unwrap();

    let leaf = prove_leaf_from_files(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../cairo-program-runner-lib/resources/compiled_programs/bootloaders/\
             leaf_simple_bootloader_compiled.json",
        ),
        &Some(input_path),
        &circuit_registry_path(),
    );
    let dumped: Vec<String> =
        serde_json::from_str(&std::fs::read_to_string(&dump_path).unwrap()).unwrap();
    let output_preimage = dumped
        .iter()
        .map(|hex| {
            BigUint::parse_bytes(hex.trim_start_matches("0x").as_bytes(), 16)
                .expect("dump entries are hex felts")
                .to_string()
        })
        .collect();
    LeafInput {
        proof: leaf,
        output: LeafOutputPreimage {
            hash_type: LeafOutputHashType::Blake2s,
            preimage: output_preimage,
        },
    }
}
