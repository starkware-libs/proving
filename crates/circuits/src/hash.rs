use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;

use crate::context::{Context, Var};
use crate::ivalue::{IValue, NoValue};
use crate::ops::{Constant, Guess};
use crate::utils::le_u32s_from_bytes;
use crate::wrappers::U32Wrapper;

#[cfg(test)]
#[path = "hash_test.rs"]
pub mod test;

/// The number of `u32` words of a 256-bit digest (e.g. Blake2s).
pub const DIGEST_N_WORDS: usize = 8;

/// A 256-bit digest (e.g. Blake2s, see [`blake2s_u32s`](crate::blake::blake2s_u32s)) kept as its
/// eight raw 32-bit words, each encoded as a QM31 `(low_u16, high_u16, 0, 0)`.
///
/// This is the primary in-circuit representation of a hash: the eight words are the faithful
/// digest and are *not* reduced mod `M31::P`.
///
/// `T` is the per-element representation: `QM31` for concrete witness values, or [`Var`] for
/// variables inside a [`Context`].
#[derive(Clone, Debug, PartialEq)]
pub struct HashValue<T>(pub [U32Wrapper<T>; DIGEST_N_WORDS]);

impl<Value: IValue> Guess<Value> for HashValue<Value> {
    type Target = HashValue<Var>;
    /// Guesses the eight words via [`U32Wrapper`]'s guess, so each is range-constrained to a valid
    /// `u32` (two 16-bit limbs each checked against `[0, 2^16)`) rather than an arbitrary field
    /// element.
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        HashValue(self.0.guess(context))
    }
}

impl<Value: IValue> Constant<Value> for HashValue<QM31> {
    type Target = HashValue<Var>;

    fn constant(&self, context: &mut Context<Value>) -> HashValue<Var> {
        HashValue(self.0.constant(context))
    }
}

impl HashValue<NoValue> {
    /// Creates a [`HashValue`] with [`NoValue`] elements, used when building the circuit topology
    /// without concrete witness values.
    pub fn no_value() -> Self {
        Self(std::array::from_fn(|_| U32Wrapper::no_value()))
    }
}

impl<Value: Copy> HashValue<Value> {
    /// Returns an iterator over the 8 u32 elements of the hash value.
    pub fn iter(&self) -> impl Iterator<Item = &U32Wrapper<Value>> {
        self.0.iter()
    }

    /// Returns the 8 u32 elements of the hash value as an array.
    pub fn as_array(&self) -> &[U32Wrapper<Value>; DIGEST_N_WORDS] {
        &self.0
    }

    /// Returns the 8 u32 elements of the hash value as a vector.
    pub fn to_vec(&self) -> Vec<U32Wrapper<Value>> {
        self.0.to_vec()
    }
}

impl<T> IntoIterator for HashValue<T> {
    type Item = U32Wrapper<T>;
    type IntoIter = std::array::IntoIter<U32Wrapper<T>, DIGEST_N_WORDS>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Value: Copy> std::ops::Index<usize> for HashValue<Value> {
    type Output = U32Wrapper<Value>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl From<Blake2sHash> for HashValue<QM31> {
    /// Encodes the [`Blake2sHash`] into eight [`U32Wrapper`]s, one per 32-bit (little-endian)
    /// hash word, each word is held as a QM31 `(low_u16, high_u16, 0, 0)`.
    fn from(value: Blake2sHash) -> Self {
        let words: [u32; DIGEST_N_WORDS] = le_u32s_from_bytes(value.0);
        HashValue::from(words)
    }
}

impl From<[u32; DIGEST_N_WORDS]> for HashValue<QM31> {
    /// Encodes the eight raw 32-bit hash words losslessly, each held as a QM31
    /// `(low_u16, high_u16, 0, 0)`. The words are not reduced mod `M31::P`.
    fn from(value: [u32; DIGEST_N_WORDS]) -> Self {
        HashValue(value.map(IValue::pack_u32))
    }
}
