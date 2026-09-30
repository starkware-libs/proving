//! `stwo_circuit_air`: AIR-specific verifier-side logic written in Cairo for the stwo-circuits
//! circuit.

pub mod circuit_air;

pub mod circuit_hash;
pub use circuit_hash::compute_circuit_hash;
pub mod claims;
pub mod components;
pub mod multiverifier_consts;
pub mod per_component;
pub mod prelude;
pub mod preprocessed_columns;
pub mod relations;
pub mod verifier;
pub use verifier::{CircuitProof, INTERACTION_POW_BITS, get_verification_output, verify_circuit};
