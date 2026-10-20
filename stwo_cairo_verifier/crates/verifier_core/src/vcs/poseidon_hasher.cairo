use core::array::ArrayTrait;
#[cfg(not(feature: "poseidon252_verifier"))]
use core::num::traits::Zero;
use core::poseidon::{hades_permutation, poseidon_hash_span};
use stwo_verifier_utils::add_length_padding;
use crate::BaseField;
use crate::fields::m31::M31_SHIFT;
use crate::fields::qm31::QM31_EXTENSION_DEGREE;
use super::hasher::MerkleHasher;
/// 8 M31 elements are packed into a hash, since 252 // 31 = 8.
const M31_ELEMENTS_IN_HASH: usize = 8;

pub impl PoseidonMerkleHasher of MerkleHasher {
    type Hash = felt252;

    fn hash_children(left: Self::Hash, right: Self::Hash) -> Self::Hash {
        let (s0, _s1, _s2) = hades_permutation(left, right, 2);
        s0
    }

    fn hash_leaf(mut column_values: Span<BaseField>) -> Self::Hash {
        // Most often a single QM31 column commitment due to FRI.
        if let Some(values) = column_values.try_into() {
            // Inlined and simplified `poseidon_hash_span(...)` for better performance.
            let [v0, v1, v2, v3]: [BaseField; QM31_EXTENSION_DEGREE] = (*values).unbox();
            let mut word: felt252 = v0.inner.into();
            word = word * M31_SHIFT + v1.inner.into();
            word = word * M31_SHIFT + v2.inner.into();
            word = word * M31_SHIFT + v3.inner.into();

            // Add the length padding to the word. Note that `word` < 2^124
            // and `QM31_EXTENSION_DEGREE` = 4, so the invocation of
            // `add_length_padding` is sound.
            // See also the docstring of [`crate::utils::add_length_padding`].
            let padded_word = add_length_padding(word, QM31_EXTENSION_DEGREE);
            let (hash, _, _) = hades_permutation(padded_word, 1, 0);
            return hash;
        }

        let mut hash_array = array![];
        // TODO(andrew): Measure performance diff and consider inlining `poseidon_hash_span(..)`
        // functionality here to do all packing and hashing in a single pass.
        while let Some(values) = column_values.multi_pop_front::<M31_ELEMENTS_IN_HASH>() {
            let [v0, v1, v2, v3, v4, v5, v6, v7] = (*values).unbox();
            let mut word = v0.into();
            word = word * M31_SHIFT + v1.into();
            word = word * M31_SHIFT + v2.into();
            word = word * M31_SHIFT + v3.into();
            word = word * M31_SHIFT + v4.into();
            word = word * M31_SHIFT + v5.into();
            word = word * M31_SHIFT + v6.into();
            word = word * M31_SHIFT + v7.into();
            hash_array.append(word);
        }

        if let Some(first_word) = column_values.pop_front() {
            let remainder_length = column_values.len() + 1;
            let mut word: felt252 = (*first_word).into();
            for v in column_values {
                word = word * M31_SHIFT + (*v).into();
            }
            // Add the length padding to the word. Note that `word` < 2^{7*31}
            // and `remainder_length` < 8, so the invocation of
            // `add_length_padding` is sound.
            // See also the docstring of [`crate::utils::add_length_padding`].
            let padded_word = add_length_padding(word, remainder_length);
            hash_array.append(padded_word);
        }

        poseidon_hash_span(hash_array.span())
    }
}
