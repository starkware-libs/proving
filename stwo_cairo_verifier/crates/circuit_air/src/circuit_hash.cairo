//! Circuit hash: `blake2s(log_blowup_factor || component_log_sizes || preprocessed_root)`.
//!
//! Cairo port of `compute_circuit_hash_host` in the `circuit-verifier` crate (stwo-circuits repo).
//! The component log sizes are taken in `ComponentList` (canonical) order, so this must match the
//! packing there byte-for-byte for the Fiat-Shamir transcript to agree.
use stwo_verifier_core::Hash;
#[cfg(not(feature: "poseidon252_verifier"))]
use stwo_verifier_utils::blake2s::hash_u32s_followed_by_digest;
#[cfg(feature: "poseidon252_verifier")]
use stwo_verifier_utils::poseidon252::hash_u32s_followed_by_digest;
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
#[cfg(not(feature: "poseidon252_verifier"))]
pub fn compute_circuit_hash(log_blowup_factor: u32, preprocessed_root: Hash) -> Hash {
    Hash {
        hash: hash_u32s_followed_by_digest(
            config_words(log_blowup_factor).span(), preprocessed_root.hash,
        ),
    }
}

/// Computes the circuit hash as `poseidon(config_words || preprocessed_root)`: the config words
/// packed seven to a felt252, then the root absorbed whole as the final element. The root is a
/// felt252 here, so it is never decomposed into words the way the Blake2s digest is.
#[cfg(feature: "poseidon252_verifier")]
pub fn compute_circuit_hash(log_blowup_factor: u32, preprocessed_root: Hash) -> Hash {
    hash_u32s_followed_by_digest(config_words(log_blowup_factor).span(), preprocessed_root)
}

/// Known-answer test mirroring `compute_circuit_hash_matches_reference` for the Poseidon252
/// build: the production `log_blowup_factor` (1) and component log sizes, cross-checked against
/// `circuit_prover::circuit_hash::compute_circuit_hash::<Poseidon252MerkleHasher>`. Pins the felt
/// packing — words seven to a felt, the root absorbed last — against the host the prover mixes
/// in.
#[cfg(test)]
#[cfg(feature: "poseidon252_verifier")]
mod poseidon_tests {
    use super::compute_circuit_hash;

    #[test]
    fn compute_circuit_hash_matches_reference() {
        assert!(
            compute_circuit_hash(
                1, 0x0172b6763ef45133e1d1d1a507f14fe24702c221cdbbc7ca7c0e5d654b008d27,
            ) == 0x034adf23e8131cdb790fdb1412b8147f9c7d8904db61e9990f886c4d835ffdfc,
        );
    }
}
