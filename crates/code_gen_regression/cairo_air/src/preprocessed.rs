use itertools::Itertools;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::poly::circle::CircleEvaluation;
use stwo_prover::core::poly::BitReversedOrder;

pub trait PreProcessedColumn {
    fn log_size(&self) -> u32;
    fn id(&self) -> PreProcessedColumnId;
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>;
}

/// A column with the numbers [0..(2^log_size)-1].
#[derive(Debug, Clone)]
pub struct Seq {
    pub log_size: u32,
}
impl Seq {
    pub const fn new(log_size: u32) -> Self {
        Self { log_size }
    }

    pub fn packed_at(&self, vec_row: usize) -> PackedM31 {
        unimplemented!()
    }
}
impl PreProcessedColumn for Seq {
    fn log_size(&self) -> u32 {
        self.log_size
    }
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        unimplemented!()
    }
    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("Seq_{}", self.log_size),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitwiseXor {
    n_bits: u32,
    col_index: usize,
}
impl BitwiseXor {
    pub const fn new(n_bits: u32, col_index: usize) -> Self {
        Self { n_bits, col_index }
    }

    pub fn packed_at(&self, vec_row: usize) -> PackedM31 {
        unimplemented!()
    }
}
impl PreProcessedColumn for BitwiseXor {
    fn log_size(&self) -> u32 {
        2 * self.n_bits
    }
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        unimplemented!()
    }
    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("bitwise_xor_{}_{}", self.n_bits, self.col_index),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PedersenPoints {
    index: usize,
}
impl PedersenPoints {
    pub fn new(col: usize) -> Self {
        Self { index: col }
    }

    pub fn packed_at(&self, vec_row: usize) -> PackedM31 {
        unimplemented!()
    }
}
impl PreProcessedColumn for PedersenPoints {
    fn log_size(&self) -> u32 {
        unimplemented!()
    }
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        unimplemented!()
    }
    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("pedersen_points_{}", self.index),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RangeCheck<const N: usize> {
    ranges: [u32; N],
    column_idx: usize,
}
impl<const N: usize> RangeCheck<N> {
    pub fn new(ranges: [u32; N], column_idx: usize) -> Self {
        Self { ranges, column_idx }
    }

    pub fn packed_at(&self, vec_row: usize) -> PackedM31 {
        unimplemented!()
    }
}
impl<const N: usize> PreProcessedColumn for RangeCheck<N> {
    fn log_size(&self) -> u32 {
        self.ranges.iter().sum()
    }
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        unimplemented!()
    }
    fn id(&self) -> PreProcessedColumnId {
        let ranges = self.ranges.iter().join("_");
        PreProcessedColumnId {
            id: format!("range_check_{}_column_{}", ranges, self.column_idx).to_string(),
        }
    }
}
