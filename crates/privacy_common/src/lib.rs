pub mod consts;
#[cfg(test)]
mod tests;
pub mod utils;

use std::error::Error;

use cairo_vm::types::program::Program;
use circuit_registry::CircuitRegistry;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::Blake2Felt252;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

use crate::consts::{LARGE_PROOFS_CIRCUIT_REGISTRY_JSON, PRIVACY_BOOTLOADER_JSON};

pub struct PrivacyProofOutput {
    /// Proof bytes, laid out as the serialized [`utils::ProofHeader`] of the `privacy-prove` crate
    /// that generated the proof, followed by the compressed proof. The format must be
    /// consistent between the prover (`privacy_recursive_prove`) and the verifier
    /// (`verify_recursive_circuit`).
    pub proof: Vec<u8>,
    pub output_preimage: Vec<Felt>,
}

pub fn get_privacy_bootloader_program() -> Result<Program, Box<dyn Error>> {
    let bootloader_program = Program::from_bytes(PRIVACY_BOOTLOADER_JSON, Some("main"))?;
    Ok(bootloader_program)
}

/// Computes the Blake2s digest that the privacy bootloader emits as its output memory cells.
///
/// The bootloader hashes the felt-encoded `output_preimage` with Blake2s (the same encoding as
/// [`Blake2Felt252::encode_felt252_data_and_calc_blake_hash`]) and stores the raw 256-bit digest
/// across its output cells. This returns that digest directly (before any packing/reduction into
/// a felt), which is the `output_hash` the circuit cairo verifier expects.
pub fn compute_privacy_bootloader_output_hash(output_preimage: &[Felt]) -> Blake2sHash {
    let u32_words = Blake2Felt252::encode_felts_to_u32s(output_preimage);
    let byte_stream: Vec<u8> = u32_words.iter().flat_map(|word| word.to_le_bytes()).collect();
    Blake2sHasher::hash(&byte_stream)
}

pub fn get_large_proofs_circuit_registry() -> CircuitRegistry {
    serde_json::from_str(LARGE_PROOFS_CIRCUIT_REGISTRY_JSON).unwrap()
}
