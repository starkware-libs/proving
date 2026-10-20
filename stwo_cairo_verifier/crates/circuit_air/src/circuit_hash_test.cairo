use core::box::BoxImpl;
use stwo_verifier_core::vcs::blake2s_hasher::Blake2sHash;
use crate::circuit_hash::{BLAKE2S_DIGEST_N_WORDS, compute_circuit_hash};

/// Known-answer test: `blake2s` over the production `log_blowup_factor` (1) and component log
/// sizes, and `preprocessed_root = [0, 1, .., 7]`, cross-checked against
/// `circuit_prover::circuit_hash::compute_circuit_hash`. This pins the byte packing against
/// the host-side implementation the prover mixes into the channel.
#[test]
fn compute_circuit_hash_matches_reference() {
    let log_blowup_factor = 1;
    let preprocessed_root = Blake2sHash { hash: BoxImpl::new([0, 1, 2, 3, 4, 5, 6, 7]) };
    let expected: [u32; BLAKE2S_DIGEST_N_WORDS] = [
        0x5b6cadf2, 0x78860d2c, 0xde9b6924, 0xf656020c, 0xc965e2b2, 0x0bb57f82, 0x9236ceb4,
        0x388feeb7,
    ];
    assert!(compute_circuit_hash(log_blowup_factor, preprocessed_root).hash.unbox() == expected);
}
