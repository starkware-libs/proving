use serde::{Deserialize, Serialize};

use super::hasher::Hasher;
use super::merkle_hasher::MerkleHasherLifted;
use crate::core::channel::{Blake2sChannel, MerkleChannel};
use crate::core::fields::m31::BaseField;
use crate::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

pub type Blake2sMerkleHasher = Blake2sHasher;

impl Hasher for Blake2sHasher {
    type Hash = Blake2sHash;

    fn hash_u32s(words: &[u32]) -> Self::Hash {
        let mut hasher = Self::default();
        words.iter().for_each(|word| hasher.update(&word.to_le_bytes()));
        hasher.finalize()
    }

    fn hash_u32s_followed_by_digest(words: &[u32], digest: Self::Hash) -> Self::Hash {
        let mut hasher = Self::default();
        words.iter().for_each(|word| hasher.update(&word.to_le_bytes()));
        hasher.update(&digest.0);
        hasher.finalize()
    }
}

impl MerkleHasherLifted for Blake2sMerkleHasher {
    fn hash_children(children_hashes: (Self::Hash, Self::Hash)) -> Self::Hash {
        let mut hasher = Self::default();
        let (left_child, right_child) = children_hashes;
        hasher.update(&left_child.0);
        hasher.update(&right_child.0);

        hasher.finalize()
    }

    fn update_leaf(&mut self, column_values: &[BaseField]) {
        column_values.iter().for_each(|x| self.update(&x.0.to_le_bytes()));
    }

    fn finalize(self) -> Self::Hash {
        self.finalize()
    }
}

/// A [`MerkleChannel`] that commits with the **standard** Blake2s Merkle hasher and runs
/// Fiat-Shamir through the **standard** Blake2s channel.
#[derive(Default)]
pub struct Blake2sMerkleChannel;

impl MerkleChannel for Blake2sMerkleChannel {
    type C = Blake2sChannel;
    type H = Blake2sMerkleHasher;

    fn mix_hash(channel: &mut Self::C, hash: Blake2sHash) {
        channel.update_digest(Blake2sHasher::concat_and_hash(&channel.digest(), &hash));
    }
}

/// Dummy implementations of `Serialize` and `Deserialize` for `Blake2sMerkleHasher` (we
/// cannot simply derive them because its inner field doesn't implement these traits and is from an
/// external crate).
/// Note: remove this code when possible.
impl Serialize for Blake2sMerkleHasher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ser = serializer.serialize_struct("Blake2sMerkleHasher", 1)?;
        serde::ser::SerializeStruct::end(ser)
    }
}

impl<'de> Deserialize<'de> for Blake2sMerkleHasher {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::default())
    }
}
