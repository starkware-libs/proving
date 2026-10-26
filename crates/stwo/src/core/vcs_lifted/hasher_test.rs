use std_shims::{ToString, Vec};

use super::Hasher;
use crate::core::channel::{Blake2sChannelGeneric, Channel, Keccak256Channel};
use crate::core::vcs::blake2_hash::{Blake2sHash, Blake2sM31Hasher};
use crate::core::vcs::keccak256_hash::Keccak256Hash;
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

/// A non-trivial digest to absorb after the words.
const DIGEST: &str = "efee1538e0216d3a09ce742ce768c44a7db004d801c40f66040994353f55fe85";

fn hex_bytes(hex: &str) -> [u8; 32] {
    core::array::from_fn(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap())
}

#[test]
fn blake2s_hash_u32s_followed_by_digest() {
    let digest = Blake2sHash(hex_bytes(DIGEST));

    // `blake2s(little endian words || the digest's bytes)`.
    assert_eq!(
        Blake2sMerkleHasher::hash_u32s_followed_by_digest(&WORDS, digest).to_string(),
        "18a60268d189dccf5065ddd361d280474fbc2b08d75a7e4e1c20e57a67f2f7fa"
    );
}

#[test]
fn keccak256_hash_u32s_followed_by_digest() {
    let digest = Keccak256Hash(hex_bytes(DIGEST));

    // `keccak256(big endian words || the digest's bytes)`.
    assert_eq!(
        Keccak256MerkleHasher::hash_u32s_followed_by_digest(&WORDS, digest).to_string(),
        "f7aa8c249a7fc55c3911a96e56f13528eef537fc71c9a23eeb3923e2375a952f"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn poseidon252_hash_u32s_followed_by_digest() {
    use starknet_ff::FieldElement as FieldElement252;

    use crate::core::vcs_lifted::poseidon252_merkle::Poseidon252MerkleHasher;

    let digest = FieldElement252::from_hex_be(
        "0x0172b6763ef45133e1d1d1a507f14fe24702c221cdbbc7ca7c0e5d654b008d27",
    )
    .unwrap();

    // The felt252 digest is absorbed as one more element after the packed words, never
    // decomposed into `u32`s.
    assert_eq!(
        Poseidon252MerkleHasher::hash_u32s_followed_by_digest(&WORDS, digest),
        FieldElement252::from_hex_be(
            "0x037ca7a4626aa8c89673f4d524492432a2234d024746a28e861328a79b9301a0"
        )
        .unwrap()
    );
}

/// For the byte-digest hashers, absorbing the digest is absorbing its eight words — the
/// decomposition callers used to do themselves. Pins that they can stop without changing a
/// single transcript.
#[test]
fn hash_u32s_followed_by_digest_absorbs_the_digest_words() {
    let digest = Blake2sHash(hex_bytes(DIGEST));
    let words: [u32; 8] = core::array::from_fn(|i| {
        u32::from_le_bytes(digest.0[i * 4..(i + 1) * 4].try_into().unwrap())
    });

    assert_eq!(
        Blake2sMerkleHasher::hash_u32s_followed_by_digest(&WORDS, digest),
        Blake2sMerkleHasher::hash_u32s(&[WORDS.as_slice(), words.as_slice()].concat())
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
