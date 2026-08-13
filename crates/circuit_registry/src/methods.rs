//! Registry queries.
//!
//! The proving binaries build their circuits from the registry: it holds the shared
//! padding target and the circuit hash each built circuit must come out with. A missing entry is
//! a hard error — the circuit is not one the registry lists.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::schema::{CircuitProofConfig, CircuitRegistry, LeafVerifier, Multiverifier};

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Cannot read the circuit registry from {1}: {0}")]
    Io(std::io::Error, PathBuf),
    #[error("Cannot parse the circuit registry from {1}: {0}")]
    Parse(serde_json::Error, PathBuf),
    #[error(
        "The circuit registry has no leaf verifier for a Cairo proof of trace log size \
         {trace_log_size}. Supported trace log sizes: {supported:?}."
    )]
    UnsupportedLeaf { trace_log_size: u32, supported: Vec<u32> },
    #[error("The circuit registry has no circuit proof config named {0:?}.")]
    UnknownConfig(String),
    #[error("The circuit registry describes {0} multiverifier circuits; expected exactly one.")]
    NotExactlyOneMultiverifier(usize),
    #[error("The circuit registry lists no leaf verifiers.")]
    NoLeafVerifiers,
}

impl CircuitRegistry {
    /// Reads a registry from its JSON file — the `circuit_params --registry` artifact published for
    /// the commit the binaries were built from.
    pub fn from_path(path: &Path) -> Result<Self, RegistryError> {
        let json = std::fs::read_to_string(path)
            .map_err(|err| RegistryError::Io(err, path.to_path_buf()))?;
        serde_json::from_str(&json).map_err(|err| RegistryError::Parse(err, path.to_path_buf()))
    }

    /// The proof config `id` names (a key of `circuit_proof_configs`, as referenced by the `config`
    /// field of a registry entry).
    pub fn config(&self, id: &str) -> Result<&CircuitProofConfig, RegistryError> {
        self.circuit_proof_configs
            .get(id)
            .ok_or_else(|| RegistryError::UnknownConfig(id.to_string()))
    }

    /// The leaf verifier circuit that verifies a Cairo proof of the given trace log size.
    pub fn leaf_verifier(&self, trace_log_size: u32) -> Result<&LeafVerifier, RegistryError> {
        self.leaf_verifiers.iter().find(|leaf| leaf.trace_log_size == trace_log_size).ok_or_else(
            || RegistryError::UnsupportedLeaf {
                trace_log_size,
                supported: self.leaf_verifiers.iter().map(|leaf| leaf.trace_log_size).collect(),
            },
        )
    }

    /// The largest verified Cairo trace log size the registry covers — the leaf circuit that bounds
    /// the sizes of every other circuit it lists.
    pub fn max_leaf_trace_log_size(&self) -> Result<u32, RegistryError> {
        self.leaf_verifiers
            .iter()
            .map(|leaf| leaf.trace_log_size)
            .max()
            .ok_or(RegistryError::NoLeafVerifiers)
    }

    /// The registry's single multiverifier circuit — the shape every layer of a recursive tree
    /// above the leaves is proven against.
    pub fn multiverifier(&self) -> Result<&Multiverifier, RegistryError> {
        match self.multiverifiers.as_slice() {
            [multiverifier] => Ok(multiverifier),
            others => Err(RegistryError::NotExactlyOneMultiverifier(others.len())),
        }
    }
}
