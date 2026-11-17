use itertools::Itertools;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

use super::SimdBackend;
use crate::core::fields::m31::BaseField;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::vcs::MerkleHasher;
use crate::core::vcs::poseidon252_hash::Poseidon252Hash;
use crate::core::vcs::poseidon252_merkle::Poseidon252MerkleHasher;
use crate::parallel_iter;
use crate::prover::backend::{Col, Column, ColumnOps};
use crate::prover::vcs::ops::MerkleOps;

impl ColumnOps<Poseidon252Hash> for SimdBackend {
    type Column = Vec<Poseidon252Hash>;

    fn bit_reverse_column(_column: &mut Self::Column) {
        unimplemented!()
    }
}

impl MerkleOps<Poseidon252MerkleHasher> for SimdBackend {
    // TODO(ShaharS): replace with SIMD implementation.
    fn commit_on_layer(
        log_size: u32,
        prev_layer: Option<&Vec<Poseidon252Hash>>,
        columns: &[&Col<Self, BaseField>],
    ) -> Vec<Poseidon252Hash> {
        let iter = parallel_iter!(0..(1 << log_size));
        iter.map(|i| {
            Poseidon252MerkleHasher::hash_node(
                prev_layer.map(|prev_layer| (prev_layer[2 * i], prev_layer[2 * i + 1])),
                &columns.iter().map(|column| column.at(i)).collect_vec(),
            )
        })
        .collect()
    }
}
