//! Owned mirror structs for Cairo `CircuitClaim` and `CircuitInteractionClaim`.
//!
//! These mirror the structs in
//! `stwo-cairo/stwo_cairo_verifier/crates/circuit_air/src/claims.cairo`. The Cairo `Serde`
//! derive serializes a struct by emitting each field in declaration order, and a fixed-size
//! sequence as the bare concatenation of its elements (no length prefix); the layout here MUST
//! match the Cairo side exactly. For `CairoCircuitClaim`, components with empty `Claim {}` on the
//! Cairo side (fixed-size LOG_SIZE constants) contribute no fields. `CairoCircuitInteractionClaim`
//! carries the claimed sums in `ComponentList` order, so the Cairo side must consume them
//! in that same order.
//!
//! Both `CairoSerialize` and `CairoDeserialize` are derived, giving symmetric serde so
//! these types can round-trip in tests.

use circuit_verifier::circuit_claim::{CircuitClaim, CircuitInteractionClaim};
use circuit_verifier::circuit_components::N_COMPONENTS;
use stwo::core::fields::qm31::QM31;
use stwo_cairo_serialize::{CairoDeserialize, CairoSerialize};

/// Mirror of Cairo `CircuitClaim`.
///
/// Cairo layout:
/// - `public_data: CircuitPublicData { output_values: Array<QM31> }`
#[derive(Clone, Debug, PartialEq, Eq, CairoSerialize, CairoDeserialize)]
pub struct CairoCircuitClaim {
    pub output_values: Vec<QM31>,
}

impl CairoCircuitClaim {
    pub fn new(claim: &CircuitClaim) -> Self {
        let CircuitClaim { output_values } = claim;

        Self { output_values: output_values.clone() }
    }
}

/// Mirror of Cairo `CircuitInteractionClaim`.
///
/// Holds the per-component claimed sums in `ComponentList` order — the same order in
/// which `CircuitInteractionClaim` stores them. A `[QM31; N_COMPONENTS]` serializes via Cairo
/// `Serde` as the bare concatenation of its elements (no length prefix).
#[derive(Clone, Debug, PartialEq, Eq, CairoSerialize, CairoDeserialize)]
pub struct CairoCircuitInteractionClaim {
    pub claimed_sums: [QM31; N_COMPONENTS],
}

impl From<&CircuitInteractionClaim> for CairoCircuitInteractionClaim {
    fn from(c: &CircuitInteractionClaim) -> Self {
        let CircuitInteractionClaim { claimed_sums } = c;

        Self { claimed_sums: claimed_sums.into_array() }
    }
}
