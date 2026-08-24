use std_shims::{ToString, Vec};

use super::Hasher;
use crate::core::channel::{Blake2sChannelGeneric, Channel, Keccak256Channel};
use crate::core::vcs::blake2_hash::Blake2sM31Hasher;
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

/// [`Hasher::hash_u32s`] must encode `u32`s exactly like [`Channel::mix_u32s`] of the same hash
/// function. The two are tied by an identity: mixing words into a channel hashes the digest
/// followed by the encoded words, so as long as the digest is itself the encoding of some words,
/// `mix_u32s(words)` equals `hash_u32s(digest_words ++ words)`.
#[test]
fn blake2s_hash_u32s_matches_mix_u32s() {
    let mut channel = Blake2sChannelGeneric::<false>::default();
    // Advance the channel, so that the digest prefix is not the default one.
    channel.mix_u64(1);
    let prefix: Vec<u32> = channel.digest().0.chunks_exact(4).map(le_word).collect();
    channel.mix_u32s(&WORDS);

    assert_eq!(
        channel.digest(),
        Blake2sMerkleHasher::hash_u32s(&[prefix, WORDS.to_vec()].concat())
    );
}

/// The `M31`-reduced Blake2s is a different hash function with the same digest type; keying
/// [`Hasher`] on the hasher type, rather than on the digest, is what keeps the two apart.
#[test]
fn blake2s_m31_hash_u32s_matches_mix_u32s() {
    let mut channel = Blake2sChannelGeneric::<true>::default();
    channel.mix_u64(1);
    let prefix: Vec<u32> = channel.digest().0.chunks_exact(4).map(le_word).collect();
    channel.mix_u32s(&WORDS);

    assert_eq!(channel.digest(), Blake2sM31Hasher::hash_u32s(&[prefix, WORDS.to_vec()].concat()));
    assert_ne!(Blake2sM31Hasher::hash_u32s(&WORDS), Blake2sMerkleHasher::hash_u32s(&WORDS));
}

#[test]
fn keccak256_hash_u32s_matches_mix_u32s() {
    let mut channel = Keccak256Channel::default();
    channel.mix_u64(1);
    let prefix: Vec<u32> = channel.digest().0.chunks_exact(4).map(be_word).collect();
    channel.mix_u32s(&WORDS);

    assert_eq!(
        channel.digest(),
        Keccak256MerkleHasher::hash_u32s(&[prefix, WORDS.to_vec()].concat())
    );
}

/// Poseidon252 absorbs the digest as a felt252 rather than as words, so the prefix has to be a
/// digest that words can encode: exactly `U32S_IN_BLOCK` words pack into one felt252 with no
/// length padding, and being a whole block they leave the packing of the mixed words unchanged.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn poseidon252_hash_u32s_matches_mix_u32s() {
    use crate::core::channel::Poseidon252Channel;
    use crate::core::vcs::poseidon252_merkle::{U32S_IN_BLOCK, construct_felt252s_from_u32s};
    use crate::core::vcs_lifted::poseidon252_merkle::Poseidon252MerkleHasher;

    let prefix: [u32; U32S_IN_BLOCK] = [1, 2, 3, 4, 5, 6, 7];
    let mut channel = Poseidon252Channel::default();
    channel.update_digest(construct_felt252s_from_u32s(&prefix)[0]);
    channel.mix_u32s(&WORDS);

    assert_eq!(
        channel.digest(),
        Poseidon252MerkleHasher::hash_u32s(&[prefix.as_slice(), &WORDS].concat())
    );
}

fn le_word(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().unwrap())
}

fn be_word(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes.try_into().unwrap())
}
