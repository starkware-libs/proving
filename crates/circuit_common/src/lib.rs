use circuits::blake::BLAKE2S_DIGEST_N_WORDS;

pub mod component_utils;
pub mod finalize;
pub mod preprocessed;

// TODO(Anat): Take from somewhere stable.
pub const N_LANES: usize = 16;
/// The number of reserved variables in a verifier circuit.
///
/// Equal to [`BLAKE2S_DIGEST_N_WORDS`]: the reserved wires hold the circuit's output, which is the
/// unreduced Blake2s digest (one wire per 32-bit word).
pub const N_RESERVED: usize = BLAKE2S_DIGEST_N_WORDS;

pub struct Qm31OpsTraceGenerator {
    pub first_permutation_row: usize,
}
