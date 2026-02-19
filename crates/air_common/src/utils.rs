use std::fs;
use std::path::PathBuf;

use convert_case::{Case, Casing};
use serde_json::Value;
use stwo_cairo_common::prover_types::cpu::{M31, PRIME, QM31};

pub fn project_root() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
}

pub fn read_json(file_path: &str) -> Value {
    let json_file =
        fs::read_to_string(file_path).unwrap_or_else(|e| panic!("Failed to read {file_path}: {e}"));
    serde_json::from_str(&json_file)
        .unwrap_or_else(|e| panic!("Invalid JSON file {file_path}: {e}"))
}

pub fn is_false(b: &bool) -> bool {
    !b
}

pub fn fix_str(mut name: String) -> String {
    while name.contains("__") {
        name = name.replace("__", "_");
    }
    if name.ends_with('_') {
        name.pop();
    }
    if name.starts_with('_') {
        name = name[1..].to_string();
    }
    name.to_case(Case::Snake)
}

// Convert a string to a random QM31 using the FNV hash
// (see https://en.wikipedia.org/wiki/Fowler-Noll-Vo_hash_function)
pub fn random_qm31(id: &str) -> QM31 {
    let mut hash: u128 = 144066263297769815596495629667062367629;

    for byte in id.as_bytes() {
        hash ^= *byte as u128;
        hash = hash.wrapping_mul(309485009821345068724781371);
    }

    let (a, hash) = (hash % (PRIME as u128), hash / (PRIME as u128));
    let (b, hash) = (hash % (PRIME as u128), hash / (PRIME as u128));
    let (c, hash) = (hash % (PRIME as u128), hash / (PRIME as u128));
    let d = hash % (PRIME as u128);
    QM31::from_u32_unchecked(
        a.try_into().unwrap(),
        b.try_into().unwrap(),
        c.try_into().unwrap(),
        d.try_into().unwrap(),
    )
}

// Convert a string to a random M31 using the FNV hash
pub fn random_m31(id: &str) -> M31 {
    let mut hash: u32 = 2166136261;

    for byte in id.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }

    M31::from_u32_unchecked(hash % PRIME)
}
