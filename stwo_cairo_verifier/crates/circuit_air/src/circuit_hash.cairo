//! Circuit hash: `blake2s(log_blowup_factor || component_log_sizes || preprocessed_root)`.
//!
//! Cairo port of `compute_circuit_hash_host` in the `circuit-verifier` crate (stwo-circuits repo).
//! The component log sizes are taken in `ComponentList` (canonical) order, so this must match the
//! packing there byte-for-byte for the Fiat-Shamir transcript to agree.
use stwo_verifier_core::Hash;
use stwo_verifier_utils::blake2s::hash_u32s;
use crate::multiverifier_consts::COMPONENT_LOG_SIZES;
use crate::per_component::PerComponentTrait;

/// Number of 32-bit words in a Blake2s-256 digest.
pub const BLAKE2S_DIGEST_N_WORDS: usize = 8;

/// Packs `log_blowup_factor` (byte 0) followed by each component's preprocessed log size (one byte
/// each, in canonical order) into little-endian u32 words. The total byte count `1 + N_COMPONENTS`
/// must be a multiple of 4.
fn config_words(log_blowup_factor: u32) -> Array<u32> {
    let mut config_bytes = array![log_blowup_factor];
    config_bytes.append_span(COMPONENT_LOG_SIZES.to_fixed_array().span());
    let mut config_bytes = config_bytes.span();

    let mut words = array![];
    while let Some(boxed) = config_bytes.multi_pop_front::<4>() {
        let [b0, b1, b2, b3] = boxed.unbox();
        words.append(b0 + b1 * 0x100 + b2 * 0x10000 + b3 * 0x1000000);
    }
    assert!(config_bytes.is_empty());
    words
}

/// Computes the circuit hash: `blake2s(log_blowup_factor || component_log_sizes ||
/// preprocessed_root)`, packing each value as little-endian bytes. The log blowup factor and
/// component log sizes are the circuit's hardcoded constants; only the preprocessed root varies.
pub fn compute_circuit_hash(log_blowup_factor: u32, preprocessed_root: Hash) -> Hash {
    let mut words = config_words(log_blowup_factor);
    words.append_span(preprocessed_root.hash.unbox().span());
    Hash { hash: hash_u32s(words.span()) }
}
