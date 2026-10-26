pub use std::array::from_fn;
pub use std::collections::HashMap;
pub use std::simd::num::{SimdInt, SimdUint};
pub use std::simd::{Simd, u32x16};
pub use std::sync::Arc;
pub use std::sync::atomic::{AtomicU32, Ordering};

pub use circuit_common::Qm31OpsTraceGenerator;
pub use circuit_common::preprocessed::PreProcessedTrace;
pub use circuit_verifier::circuit_claim::{ClaimedSum, ComponentLogSize};
pub use circuit_verifier::relations;
pub use itertools::{Itertools, multizip};
pub use num_traits::{One, Zero};
pub use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
    IntoParallelRefMutIterator, ParallelIterator,
};
pub use stwo::core::fields::m31::M31;
pub use stwo::core::fields::qm31::{QM31, SecureField};
pub use stwo::core::poly::circle::CanonicCoset;
pub use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
pub use stwo::prover::TreeBuilder;
pub use stwo::prover::backend::Col;
pub use stwo::prover::backend::simd::SimdBackend;
pub use stwo::prover::backend::simd::column::BaseColumn;
pub use stwo::prover::backend::simd::conversion::{Pack, Unpack};
pub use stwo::prover::backend::simd::m31::{LOG_N_LANES, N_LANES, PackedM31};
pub use stwo::prover::backend::simd::qm31::PackedQM31;
pub use stwo::prover::poly::BitReversedOrder;
pub use stwo::prover::poly::circle::CircleEvaluation;
pub use stwo_air_utils::trace::component_trace::ComponentTrace;
pub use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
pub use stwo_cairo_common::preprocessed_columns::blake::{
    BLAKE_SIGMA, BLAKE_SIGMA_TABLE, N_BLAKE_ROUNDS, N_BLAKE_SIGMA_COLS, sigma, sigma_m31,
};
pub use stwo_cairo_common::preprocessed_columns::preprocessed_trace::{PreProcessedColumn, Seq};
pub use stwo_cairo_common::prover_types::cpu::{UInt16, UInt32};
pub use stwo_cairo_common::prover_types::simd::{
    PackedBool, PackedM31Type, PackedUInt16, PackedUInt32, SIMD_ENUMERATION_0,
};
pub use stwo_cairo_prover::witness::fast_deduction::blake::{
    G_STATE_INDICES, PackedBlakeRoundSigma, PackedTripleXor32,
};
pub use stwo_cairo_prover::witness::utils::{AtomicMultiplicityColumn, Enabler};
pub use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;
pub use stwo_constraint_framework::{LogupTraceGenerator, Relation};

pub use crate::witness::utils::pack_values;

const NUM_INPUT_WORDS_G: usize = 6;
const NUM_OUTPUT_WORDS_G: usize = 4;

/// Local shim for stwo-cairo's `PackedBlakeG` with a public `blake_g` method
/// (upstream keeps it private).
#[derive(Debug)]
pub struct PackedBlakeG {}

impl PackedBlakeG {
    pub fn deduce_output(
        input: [PackedUInt32; NUM_INPUT_WORDS_G],
    ) -> [PackedUInt32; NUM_OUTPUT_WORDS_G] {
        PackedBlakeG::blake_g(input.map(|x| x.simd)).map(|simd| PackedUInt32 { simd })
    }

    pub fn blake_g(input: [u32x16; NUM_INPUT_WORDS_G]) -> [u32x16; NUM_OUTPUT_WORDS_G] {
        let [mut a, mut b, mut c, mut d, m0, m1] = input;

        a = a + b + m0;
        d ^= a;
        d = (d >> 16) | (d << (u32::BITS - 16));

        c += d;
        b ^= c;
        b = (b >> 12) | (b << (u32::BITS - 12));

        a = a + b + m1;
        d ^= a;
        d = (d >> 8) | (d << (u32::BITS - 8));

        c += d;
        b ^= c;
        b = (b >> 7) | (b << (u32::BITS - 7));

        [a, b, c, d]
    }
}

/// Create the input_to_row map used in const-size components.
///
/// `preprocessed_trace` - The preprocessed trace.
/// `column_ids` - PreProcessedColumnId for each input column of the component.
///
/// Returns a mapping from input tuple to its row number. Used to find
/// out which multiplicity value to update for a given input.
pub fn make_input_to_row<const N: usize>(
    preprocessed_trace: &PreProcessedTrace,
    column_ids: [PreProcessedColumnId; N],
) -> HashMap<[M31; N], usize> {
    let mut result: HashMap<[M31; N], usize> = HashMap::new();

    let columns = column_ids.iter().map(|id| preprocessed_trace.get_column(id)).collect_vec();
    let log_size = columns[0].len().ilog2();
    assert!(
        columns.iter().all(|c| c.len().ilog2() == log_size),
        "input_to_row columns of different sizes"
    );

    for packed_row in 0..(1 << (log_size - LOG_N_LANES)) {
        let row_offset = packed_row * N_LANES;
        for i in 0..N_LANES {
            let key: [M31; N] = columns
                .iter()
                .map(|column| M31::from(column[row_offset + i]))
                .collect_vec()
                .try_into()
                .expect("Unexpected number of column values");
            result.insert(key, row_offset + i);
        }
    }

    result
}

pub fn pack_preprocessed_column(column: &[usize]) -> Vec<PackedM31> {
    let values: Vec<M31> = column.par_iter().map(|&v| M31::from(v)).collect();
    pack_values(&values)
}
