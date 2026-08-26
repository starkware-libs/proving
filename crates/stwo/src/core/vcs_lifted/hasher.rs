use crate::core::vcs::hash::Hash;

#[cfg(test)]
#[path = "hasher_test.rs"]
mod test;

/// A hash function.
///
/// Keyed on the hasher type rather than on the digest type, since the digest does not determine
/// the hash function: `Blake2sHasherGeneric<false>` and `Blake2sHasherGeneric<true>` (whose output
/// is reduced modulo `M31::P`) are distinct hash functions, both producing a `Blake2sHash`.
pub trait Hasher {
    /// The digest this hash function produces.
    type Hash: Hash;

    /// Hashes a slice of `u32`s, each converted into the underlying hasher's data format.
    fn hash_u32s(words: &[u32]) -> Self::Hash;

    /// Hashes `H(words || digest)` in one pass — a hash of the concatenation, not `H(H(words),
    /// digest)`.
    fn hash_u32s_followed_by_digest(words: &[u32], digest: Self::Hash) -> Self::Hash;
}
