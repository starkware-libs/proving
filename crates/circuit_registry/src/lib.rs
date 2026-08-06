//! The circuit registry: the set of verifier circuits the system supports, identified by their
//! circuit hashes. This crate defines the registry's JSON schema ([`CircuitRegistry`] and friends),
//! emitted by the `circuit-params` tool, and the queries the proving binaries build their circuits
//! from (see [`methods`]).

mod methods;
mod schema;

pub use leaf_proof_format::DigestHex;
pub use methods::RegistryError;
pub use schema::{CircuitProofConfig, CircuitRegistry, LeafVerifier, LogSizes, Multiverifier};
