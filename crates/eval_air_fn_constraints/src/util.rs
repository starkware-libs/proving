use std::collections::BTreeMap;

use compiled_casm_air::compiled_structs::ExternalState;
use indexmap::IndexMap;
use num_traits::One;
use serde::{Deserialize, Serialize};
use stwo_cairo_common::prover_types::cpu::{M31, PRIME, QM31};

/// The values of "globals" that are available in all scopes.
#[derive(Serialize, Deserialize)]
pub struct Environment {
    // We use a BTreeMap to have a stable order when serializing this to JSON (makes
    // it easier to do regression tests)
    pub public_params: BTreeMap<String, M31>,

    // IndexMap is only used here because of its ability to serialize maps with non-string keys
    // (`ExternalState` in our case). We do not care about the order of the external states.
    #[serde(with = "indexmap::map::serde_seq")]
    pub external_states: IndexMap<ExternalState, QM31>,
}

/// The interaction elements for a specific relation.
#[derive(Debug, Serialize, Deserialize)]
pub struct LookupElements {
    pub z: QM31,
    pub alpha: QM31,
}

impl LookupElements {
    /// Compute the logup denominator for a given relation tuple
    /// The result is -z + alpha * tuple[0] + alpha^2 * tuple[1] + alpha^3 * tuple[2] ...
    pub fn compute_logup_denominator(&self, tuple: &Vec<QM31>) -> QM31 {
        let mut result = -self.z;
        let mut coeff = QM31::one();

        for elem in tuple {
            result += coeff * *elem;
            coeff *= self.alpha;
        }

        result
    }
}

// Convert a string to a random QM31 using the FNV hash
// (see https://en.wikipedia.org/wiki/Fowler-Noll-Vo_hash_function)
pub fn random_qm31(id: &String) -> QM31 {
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
pub fn random_m31(id: &String) -> M31 {
    let mut hash: u32 = 2166136261;

    for byte in id.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }

    M31::from_u32_unchecked(hash % PRIME)
}

/// Build a circle point from its `t` parametrization
/// Returns (x,y).
pub fn circle_point_from_t(t: QM31) -> (QM31, QM31) {
    (
        (QM31::one() - t * t) / (t * t + QM31::one()),
        (t + t) / (t * t + QM31::one()),
    )
}
