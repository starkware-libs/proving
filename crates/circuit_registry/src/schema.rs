//! The JSON schema for the circuit registry: the prover params its proofs are produced
//! with, a map of proof configs, the leaf verifiers (one per trace size), and the multiverifiers,
//! each identified by its circuit hash.

use std::collections::BTreeMap;

use circuit_common::finalize::ComponentSizes;
use leaf_proof_format::DigestHex;
use serde::{Deserialize, Serialize};
use stwo::core::fri::FriConfig;
use stwo_cairo_common::prover_params::ProverParameters;

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

impl From<&LogSizes> for ComponentSizes {
    fn from(log_sizes: &LogSizes) -> Self {
        ComponentSizes {
            eq: 1 << log_sizes.eq,
            qm31_ops: 1 << log_sizes.qm31_ops,
            m31_to_u32: 1 << log_sizes.m31_to_u32,
            triple_xor: 1 << log_sizes.triple_xor,
            blake_g_gate: 1 << log_sizes.blake_g_gate,
        }
    }
}

fn log_size(size: usize) -> u32 {
    size.next_power_of_two().ilog2()
}

/// A proof configuration: the (circuit-prover) FRI config and padded component log sizes a circuit
/// is proven with. Circuits proven using the same config can be verified using the same verifier
/// circuit.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CircuitProofConfig {
    pub fri_config: FriConfig,
    pub component_log_sizes: LogSizes,
}

impl CircuitProofConfig {
    /// The shared padding target every circuit proven under this config is padded to.
    pub fn target_sizes(&self) -> ComponentSizes {
        (&self.component_log_sizes).into()
    }
}

/// A leaf verifier circuit (verifying one Cairo proof of the given trace size), padded to its
/// config's component sizes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LeafVerifier {
    /// Key into `CircuitRegistry::circuit_proof_configs`.
    pub config: String,
    pub trace_log_size: u32,
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

/// The json output: the prover params of the verified Cairo proofs, a map of proof configs, the
/// leaf verifiers (one per trace size), and the multiverifiers. All circuits are padded to the
/// shared target sizes and proven with the same blowup, so they share a single config; the
/// multiverifier verifies proofs of the leaf circuit and is essentially the same across trace
/// sizes, so a single multiverifier is reported.
///
/// The params are part of the registry because the circuits — hence every hash here — are functions
/// of them, and so that a proving binary of these circuits needs no configuration beyond the
/// registry.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CircuitRegistry {
    /// The prover params the verified Cairo proofs are produced with.
    pub cairo_prover_params: ProverParameters,
    pub circuit_proof_configs: BTreeMap<String, CircuitProofConfig>,
    pub leaf_verifiers: Vec<LeafVerifier>,
    pub multiverifiers: Vec<Multiverifier>,
}
