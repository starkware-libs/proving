use std_shims::ToString;

use super::MerkleHasherLifted;
use crate::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;
use crate::core::vcs_lifted::keccak256_merkle::Keccak256MerkleHasher;

const WORDS: [u32; 9] = [0, 1, 2, 3, 0x1234_5678, u32::MAX, 7, 8, 9];

#[test]
fn blake2s_hash_u32s() {
    // Little endian words, as `Blake2sChannelGeneric::mix_u32s` encodes them.
    assert_eq!(
        Blake2sMerkleHasher::hash_u32s(&WORDS).to_string(),
        "efee1538e0216d3a09ce742ce768c44a7db004d801c40f66040994353f55fe85"
    );
}

#[test]
fn keccak256_hash_u32s() {
    // Big endian words, as `Keccak256Channel::mix_u32s` encodes them.
    assert_eq!(
        Keccak256MerkleHasher::hash_u32s(&WORDS).to_string(),
        "599c3b5811a74c3a7142e18b89352bd53285dedaa60d180cc9aa3e7c8aad6593"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn poseidon252_hash_u32s() {
    use starknet_ff::FieldElement as FieldElement252;

    use crate::core::vcs_lifted::poseidon252_merkle::Poseidon252MerkleHasher;

    // Seven words per felt252, as `Poseidon252Channel::mix_u32s` packs them.
    assert_eq!(
        Poseidon252MerkleHasher::hash_u32s(&WORDS),
        FieldElement252::from_hex_be(
            "0x0172b6763ef45133e1d1d1a507f14fe24702c221cdbbc7ca7c0e5d654b008d27"
        )
        .unwrap()
    );
}

/// The `u32` encoding must not collide across input lengths: for Poseidon252 the last felt252
/// is zero-padded, so its number of packed words has to be part of the preimage.
#[test]
fn hash_u32s_distinguishes_lengths() {
    assert_ne!(Blake2sMerkleHasher::hash_u32s(&[1]), Blake2sMerkleHasher::hash_u32s(&[1, 0]));
    assert_ne!(Keccak256MerkleHasher::hash_u32s(&[1]), Keccak256MerkleHasher::hash_u32s(&[1, 0]));
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::core::vcs_lifted::poseidon252_merkle::Poseidon252MerkleHasher;
        assert_ne!(
            Poseidon252MerkleHasher::hash_u32s(&[1]),
            Poseidon252MerkleHasher::hash_u32s(&[1, 0])
        );
    }
}
