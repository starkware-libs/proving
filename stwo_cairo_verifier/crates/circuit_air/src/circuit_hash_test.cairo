#[cfg(not(feature: "poseidon252_verifier"))]
mod blake2s_tests {
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
        assert!(
            compute_circuit_hash(log_blowup_factor, preprocessed_root).hash.unbox() == expected,
        );
    }
}

#[cfg(feature: "poseidon252_verifier")]
mod poseidon_tests {
    use crate::circuit_hash::compute_circuit_hash;

    /// Known-answer test mirroring `compute_circuit_hash_matches_reference` for the Poseidon252
    /// build: the production `log_blowup_factor` (1) and component log sizes, cross-checked
    /// against `circuit_prover::circuit_hash::compute_circuit_hash::<Poseidon252MerkleHasher>`.
    /// Pins the felt packing (words seven to a felt, the root absorbed last) against the host
    /// the prover mixes in.
    #[test]
    fn compute_circuit_hash_matches_reference() {
        assert!(
            compute_circuit_hash(
                1, 0x0172b6763ef45133e1d1d1a507f14fe24702c221cdbbc7ca7c0e5d654b008d27,
            ) == 0x034adf23e8131cdb790fdb1412b8147f9c7d8904db61e9990f886c4d835ffdfc,
        );
    }
}
