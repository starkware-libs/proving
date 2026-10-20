use crate::BaseField;

/// A Merkle hasher for a standard Merkle tree.
///
/// All committed columns have the same length. Leaves hold the hashes of the rows and the inner
/// nodes hold the hash of their two children.
pub trait MerkleHasher {
    type Hash;

    /// Hashes an inner Merkle node from the hashes of its two children.
    fn hash_children(left: Self::Hash, right: Self::Hash) -> Self::Hash;

    /// Hashes a Merkle leaf from its column values.
    fn hash_leaf(column_values: Span<BaseField>) -> Self::Hash;
}
