//! Serializes circuit proof into the format expected by the Cairo verifier
//! (`stwo-cairo/stwo_cairo_verifier/crates/circuit_verifier`).
//!
//! The Cairo verifier's executable signature is:
//!
//! ```cairo
//! fn main(proof: CircuitProof) -> VerificationOutput
//! ```
//!
//! and uses the standard `#[derive(Serde)]` to deserialize the proof from the felt252
//! input stream produced by `scarb execute --arguments-file`. The verifier-config
//! constants (output addresses, preprocessed root, lifting log size,
//! preprocessed column log sizes) are hardcoded inside the Cairo verifier binary for a
//! specific circuit topology and are NOT carried on the wire.

pub mod claim;
pub mod proof;

#[cfg(test)]
mod test;

pub use claim::{CairoCircuitClaim, CairoCircuitInteractionClaim};
pub use proof::{CairoCircuitProof, CairoStarkProof, prepare_circuit_proof_for_cairo_verifier};
