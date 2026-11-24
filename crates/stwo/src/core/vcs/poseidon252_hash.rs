use core::fmt;

use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt as FieldElement252;

/// The hash the Poseidon252 Merkle hashers produce.
///
/// Serializes and prints as the felt252 it wraps.
#[derive(Copy, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Poseidon252Hash(pub FieldElement252);

impl Poseidon252Hash {
    /// The felt252's big-endian bytes.
    pub fn to_bytes_be(&self) -> [u8; 32] {
        self.0.to_bytes_be()
    }
}

impl fmt::Display for Poseidon252Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for Poseidon252Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for Poseidon252Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl super::hash::Hash for Poseidon252Hash {}
