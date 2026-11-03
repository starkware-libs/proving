//! Generates the leaf the e2e tests fold, for the lib tests and the CLI test alike. Compiled under
//! the `slow-tests` feature only.

use std::path::{Path, PathBuf};

use leaf_prover::prove_leaf::prove_leaf_from_files;
use starknet_types_core::felt::Felt;
use stwo_cairo_utils::file_utils::repo_root;

use crate::test_data::canonical_small_for_tests_circuit_registry_path;
use crate::{LeafInput, LeafOutputHashType, LeafOutputPreimage};

/// The simple-output task's input for the leaf most tests fold; the program copies it to its
/// output.
pub const LEAF_TASK_OUTPUT: [u32; 3] = [11, 13, 17];

/// The leaf bootloader's input: one simple-output task over `task_output` with a blake program
/// hash, the path to dump the output preimage to, and blake as the output hash.
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
        "output_hash_func": "blake",
    }))
    .unwrap()
}

/// Proves the leaf simple bootloader running the simple-output task over `task_output` and builds
/// the `LeafInput` the backend would: the proof plus the dumped output preimage.
pub fn generate_leaf(dir: &Path, task_output: &[u32]) -> LeafInput {
    let dump_path = dir.join("leaf_preimage.json");
    let input_path = dir.join("leaf_bl_input.json");
    std::fs::write(&input_path, leaf_bl_input_json(&dump_path, task_output)).unwrap();

    let leaf = prove_leaf_from_files(
        &repo_root().join("compiled_cairo_programs/leaf_simple_bootloader_compiled.json"),
        &Some(input_path),
        &canonical_small_for_tests_circuit_registry_path(),
    );
    let output_preimage: Vec<Felt> =
        serde_json::from_str(&std::fs::read_to_string(&dump_path).unwrap()).unwrap();
    LeafInput {
        proof: leaf,
        output: LeafOutputPreimage {
            hash_type: LeafOutputHashType::Blake2s,
            preimage: output_preimage,
        },
    }
}
