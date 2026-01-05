pub mod create_opcodes_json;
pub mod fn_sizes;
#[cfg(test)]
pub mod test_utils;

use convert_case::{Case, Casing};
use stwo_cairo_common::prover_types::cpu::{M31, PRIME};

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

// Convert a string to a random M31 using the FNV hash
pub fn random_m31(id: &str) -> M31 {
    let mut hash: u32 = 2166136261;

    for byte in id.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }

    M31::from_u32_unchecked(hash % PRIME)
}
