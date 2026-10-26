use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use stwo::core::fields::m31::M31;
use stwo_cairo_common::prover_types::felt::split_f252;

use crate::statement::MEMORY_VALUES_LIMBS;

/// Divides `a` by `b`, panicking if `b` does not divide `a` evenly.
pub fn safe_div(a: usize, b: usize) -> usize {
    assert_eq!(a % b, 0, "{b} does not divide {a}");
    a / b
}

pub fn get_test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test_data/")
}

pub fn get_proof_file_path(test_name: &str) -> PathBuf {
    get_test_data_dir().join(test_name).join("proof.bin")
}

/// Loads a compiled Cairo program from a JSON file and converts each felt252
/// into 28 9-bit M31 limbs using [split_f252].
pub fn load_program(json_path: &std::path::Path) -> Arc<[[M31; MEMORY_VALUES_LIMBS]]> {
    let json: serde_json::Value = serde_json::from_reader(File::open(json_path).unwrap()).unwrap();
    json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hex_str| {
            let s = hex_str.as_str().unwrap().strip_prefix("0x").unwrap();
            let padded = format!("{s:0>64}");
            let hi = u128::from_str_radix(&padded[..32], 16).unwrap();
            let lo = u128::from_str_radix(&padded[32..], 16).unwrap();
            let mut words = [0u32; 8];
            for i in 0..4 {
                words[i] = (lo >> (32 * i)) as u32;
                words[i + 4] = (hi >> (32 * i)) as u32;
            }
            split_f252(words)
        })
        .collect()
}
