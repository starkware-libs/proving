//! The JSON schema for the circuit registry: a map of proof configs, the leaf verifiers (one per
//! trace size), and the multiverifiers, each identified by its circuit hash.

use std::collections::BTreeMap;

use circuit_common::finalize::ComponentSizes;
use leaf_proof_format::DigestHex;
use serde::{Deserialize, Serialize};

/// The padded log sizes of the components that circuits are padded to a shared target on.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LogSizes {
    pub eq: u32,
    pub qm31_ops: u32,
    pub m31_to_u32: u32,
    pub triple_xor: u32,
    pub blake_g_gate: u32,
}

impl From<&ComponentSizes> for LogSizes {
    fn from(padded: &ComponentSizes) -> Self {
        LogSizes {
            eq: log_size(padded.eq),
            qm31_ops: log_size(padded.qm31_ops),
            m31_to_u32: log_size(padded.m31_to_u32),
            triple_xor: log_size(padded.triple_xor),
            blake_g_gate: log_size(padded.blake_g_gate),
        }
    }
}

fn log_size(size: usize) -> u32 {
    size.next_power_of_two().ilog2()
}

/// A proof configuration: the (circuit-prover) log blowup factor and padded component log sizes a
/// circuit is proven with. Circuits proven using the same config can be verified using the same
/// verifier circuit.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CircuitProofConfig {
    pub log_blowup_factor: u32,
    pub component_log_sizes: LogSizes,
}

/// A leaf verifier circuit (verifying one Cairo proof of the given trace size and log blowup
/// factor), padded to its config's component sizes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LeafVerifier {
    /// Key into `CircuitRegistry::circuit_proof_configs`.
    pub config: String,
    pub trace_log_size: u32,
    /// Log blowup factor of the Cairo proof this leaf verifies.
    pub log_blowup_factor: u32,
    /// `blake2s(log_blowup_factor || component_log_sizes || preprocessed_root)` — the value that
    /// identifies this circuit in a verifier's public output.
    pub circuit_hash: DigestHex,
}

/// The multiverifier circuit, padded to its config's component sizes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Multiverifier {
    /// Key into `CircuitRegistry::circuit_proof_configs`: the multiverifier's own config.
    pub config: String,
    /// Configs of the two circuits whose proofs the multiverifier verifies.
    pub input_configs: [String; 2],
    /// `blake2s(log_blowup_factor || component_log_sizes || preprocessed_root)` — the value that
    /// identifies this circuit in a verifier's public output.
    pub circuit_hash: DigestHex,
}

/// The json output: a map of proof configs, the leaf verifiers (one per trace size), and the
/// multiverifiers. All circuits are padded to the shared target sizes and proven with the same
/// blowup, so they share a single config; the multiverifier verifies proofs of the leaf circuit and
/// is essentially the same across trace sizes, so a single multiverifier is reported.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CircuitRegistry {
    pub circuit_proof_configs: BTreeMap<String, CircuitProofConfig>,
    pub leaf_verifiers: Vec<LeafVerifier>,
    pub multiverifiers: Vec<Multiverifier>,
}
