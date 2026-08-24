use core::fmt::Debug;

use crate::core::fields::m31::BaseField;
use crate::core::vcs::hash::Hash;

#[cfg(test)]
#[path = "merkle_hasher_test.rs"]
mod test;

/// An interface for a hasher that only operates on types `Self::Hash` or
/// `BaseField`, as opposed to the underlying hasher's data format (e.g. bytes in the case of
/// Blake2s or elements of other fields in the case of Poseidon252).
pub trait MerkleHasherLifted: Debug + Default + Clone {
    type Hash: Hash;

    /// Hashes an inner Merkle node.
    fn hash_children(children_hashes: (Self::Hash, Self::Hash)) -> Self::Hash;

    // TODO(ilya): Move to a `Hasher` trait; hashing `u32`s is not a Merkle operation.
    /// Hashes a slice of `u32`s, each converted into the underlying hasher's data format.
    ///
    /// Unrelated to the Merkle tree: a one-shot hash of the words alone, with no children hashes
    /// and no domain separation. The `u32` encoding matches the one
    /// [`mix_u32s`](crate::core::channel::Channel::mix_u32s) uses for the same hash function.
    fn hash_u32s(words: &[u32]) -> Self::Hash;

    /// Converts each `BaseField` elements into the underlying hasher's data format,
    /// and updates the hasher's state.
    fn update_leaf(&mut self, column_values: &[BaseField]);

    /// Finalizes the underlying hasher.
    fn finalize(self) -> Self::Hash;
}
