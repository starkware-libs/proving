#![allow(unused_parens)]
#![allow(unused_imports)]
use std::iter::zip;

use air_structs_derive::SubComponentInputs;
use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
use prover_types::cpu::*;
use prover_types::simd::*;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
use stwo_prover::constraint_framework::preprocessed_columns::PreprocessedColumn;
use stwo_prover::constraint_framework::Relation;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::column::BaseColumn;
use stwo_prover::core::backend::simd::conversion::Unpack;
use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
use stwo_prover::core::backend::simd::qm31::PackedQM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{BackendForChannel, Col, Column};
use stwo_prover::core::channel::{Channel, MerkleChannel};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::utils::bit_reverse_coset_to_circle_domain_order;

use super::component::{Claim, InteractionClaim};
use crate::components::{narrow_fib_num_steps_20, pack_values};
use crate::relations;

pub type InputType = M31;
pub type PackedInputType = PackedM31;
const N_TRACE_COLUMNS: usize = 17;

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn new(inputs: Vec<InputType>) -> Self {
        Self { inputs }
    }

    pub fn write_trace<MC: MerkleChannel>(
        mut self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
        narrow_fib_num_steps_20_state: &mut narrow_fib_num_steps_20::ClaimGenerator,
    ) -> (Claim, InteractionClaimGenerator)
    where
        SimdBackend: BackendForChannel<MC>,
    {
        let n_rows = self.inputs.len();
        assert_ne!(n_rows, 0);
        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let need_padding = n_rows != size;

        if need_padding {
            self.inputs.resize(size, *self.inputs.first().unwrap());
            bit_reverse_coset_to_circle_domain_order(&mut self.inputs);
        }

        let packed_inputs = pack_values(&self.inputs);
        let (trace, mut sub_components_inputs, lookup_data) = write_trace_simd(packed_inputs);

        if need_padding {
            sub_components_inputs.bit_reverse_coset_to_circle_domain_order();
        }
        sub_components_inputs
            .narrow_fib_num_steps_20_inputs
            .iter()
            .for_each(|inputs| {
                narrow_fib_num_steps_20_state.add_inputs(&inputs[..n_rows]);
            });

        tree_builder.extend_evals(trace.to_evals());

        (
            Claim { n_rows },
            InteractionClaimGenerator {
                n_rows,
                lookup_data,
            },
        )
    }

    pub fn add_inputs(&mut self, inputs: &[InputType]) {
        self.inputs.extend(inputs);
    }
}

#[derive(SubComponentInputs, Uninitialized, IterMut, ParIterMut)]
pub struct SubComponentInputs {
    pub narrow_fib_num_steps_20_inputs: [Vec<narrow_fib_num_steps_20::InputType>; 8],
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(
    inputs: Vec<PackedInputType>,
) -> (
    ComponentTrace<N_TRACE_COLUMNS>,
    SubComponentInputs,
    LookupData,
) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data, mut sub_components_inputs) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
            SubComponentInputs::uninitialized(log_size),
        )
    };

    let M31_1 = PackedM31::broadcast(M31::from(1));

    trace
        .par_iter_mut()
        .zip(inputs.into_par_iter())
        .zip(lookup_data.par_iter_mut())
        .zip(sub_components_inputs.par_iter_mut().chunks(N_LANES))
        .for_each(
            |(
                ((row, wide_fib_num_narrow_8_narrow_size_20_input), lookup_data),
                mut sub_components_inputs,
            )| {
                let col0 = wide_fib_num_narrow_8_narrow_size_20_input;
                *row[0] = col0;
                let narrow_fib_num_steps_20_output_tmp_d7cf2_0 =
                    narrow_fib_num_steps_20::deduce_output([M31_1, col0]);
                let narrow_fib_num_steps_20_output_col1 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_0[0];
                *row[1] = narrow_fib_num_steps_20_output_col1;
                let narrow_fib_num_steps_20_output_col2 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_0[1];
                *row[2] = narrow_fib_num_steps_20_output_col2;
                for (i, &input) in [M31_1, col0].unpack().iter().enumerate() {
                    *sub_components_inputs[i].narrow_fib_num_steps_20_inputs[0] = input;
                }
                *lookup_data.narrow_fib_num_steps_20_0 = [
                    M31_1,
                    col0,
                    narrow_fib_num_steps_20_output_col1,
                    narrow_fib_num_steps_20_output_col2,
                ];
                let narrow_fib_num_steps_20_output_tmp_d7cf2_1 =
                    narrow_fib_num_steps_20::deduce_output([
                        narrow_fib_num_steps_20_output_col1,
                        narrow_fib_num_steps_20_output_col2,
                    ]);
                let narrow_fib_num_steps_20_output_col3 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_1[0];
                *row[3] = narrow_fib_num_steps_20_output_col3;
                let narrow_fib_num_steps_20_output_col4 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_1[1];
                *row[4] = narrow_fib_num_steps_20_output_col4;
                for (i, &input) in [
                    narrow_fib_num_steps_20_output_col1,
                    narrow_fib_num_steps_20_output_col2,
                ]
                .unpack()
                .iter()
                .enumerate()
                {
                    *sub_components_inputs[i].narrow_fib_num_steps_20_inputs[1] = input;
                }
                *lookup_data.narrow_fib_num_steps_20_1 = [
                    narrow_fib_num_steps_20_output_col1,
                    narrow_fib_num_steps_20_output_col2,
                    narrow_fib_num_steps_20_output_col3,
                    narrow_fib_num_steps_20_output_col4,
                ];
                let narrow_fib_num_steps_20_output_tmp_d7cf2_2 =
                    narrow_fib_num_steps_20::deduce_output([
                        narrow_fib_num_steps_20_output_col3,
                        narrow_fib_num_steps_20_output_col4,
                    ]);
                let narrow_fib_num_steps_20_output_col5 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_2[0];
                *row[5] = narrow_fib_num_steps_20_output_col5;
                let narrow_fib_num_steps_20_output_col6 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_2[1];
                *row[6] = narrow_fib_num_steps_20_output_col6;
                for (i, &input) in [
                    narrow_fib_num_steps_20_output_col3,
                    narrow_fib_num_steps_20_output_col4,
                ]
                .unpack()
                .iter()
                .enumerate()
                {
                    *sub_components_inputs[i].narrow_fib_num_steps_20_inputs[2] = input;
                }
                *lookup_data.narrow_fib_num_steps_20_2 = [
                    narrow_fib_num_steps_20_output_col3,
                    narrow_fib_num_steps_20_output_col4,
                    narrow_fib_num_steps_20_output_col5,
                    narrow_fib_num_steps_20_output_col6,
                ];
                let narrow_fib_num_steps_20_output_tmp_d7cf2_3 =
                    narrow_fib_num_steps_20::deduce_output([
                        narrow_fib_num_steps_20_output_col5,
                        narrow_fib_num_steps_20_output_col6,
                    ]);
                let narrow_fib_num_steps_20_output_col7 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_3[0];
                *row[7] = narrow_fib_num_steps_20_output_col7;
                let narrow_fib_num_steps_20_output_col8 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_3[1];
                *row[8] = narrow_fib_num_steps_20_output_col8;
                for (i, &input) in [
                    narrow_fib_num_steps_20_output_col5,
                    narrow_fib_num_steps_20_output_col6,
                ]
                .unpack()
                .iter()
                .enumerate()
                {
                    *sub_components_inputs[i].narrow_fib_num_steps_20_inputs[3] = input;
                }
                *lookup_data.narrow_fib_num_steps_20_3 = [
                    narrow_fib_num_steps_20_output_col5,
                    narrow_fib_num_steps_20_output_col6,
                    narrow_fib_num_steps_20_output_col7,
                    narrow_fib_num_steps_20_output_col8,
                ];
                let narrow_fib_num_steps_20_output_tmp_d7cf2_4 =
                    narrow_fib_num_steps_20::deduce_output([
                        narrow_fib_num_steps_20_output_col7,
                        narrow_fib_num_steps_20_output_col8,
                    ]);
                let narrow_fib_num_steps_20_output_col9 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_4[0];
                *row[9] = narrow_fib_num_steps_20_output_col9;
                let narrow_fib_num_steps_20_output_col10 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_4[1];
                *row[10] = narrow_fib_num_steps_20_output_col10;
                for (i, &input) in [
                    narrow_fib_num_steps_20_output_col7,
                    narrow_fib_num_steps_20_output_col8,
                ]
                .unpack()
                .iter()
                .enumerate()
                {
                    *sub_components_inputs[i].narrow_fib_num_steps_20_inputs[4] = input;
                }
                *lookup_data.narrow_fib_num_steps_20_4 = [
                    narrow_fib_num_steps_20_output_col7,
                    narrow_fib_num_steps_20_output_col8,
                    narrow_fib_num_steps_20_output_col9,
                    narrow_fib_num_steps_20_output_col10,
                ];
                let narrow_fib_num_steps_20_output_tmp_d7cf2_5 =
                    narrow_fib_num_steps_20::deduce_output([
                        narrow_fib_num_steps_20_output_col9,
                        narrow_fib_num_steps_20_output_col10,
                    ]);
                let narrow_fib_num_steps_20_output_col11 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_5[0];
                *row[11] = narrow_fib_num_steps_20_output_col11;
                let narrow_fib_num_steps_20_output_col12 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_5[1];
                *row[12] = narrow_fib_num_steps_20_output_col12;
                for (i, &input) in [
                    narrow_fib_num_steps_20_output_col9,
                    narrow_fib_num_steps_20_output_col10,
                ]
                .unpack()
                .iter()
                .enumerate()
                {
                    *sub_components_inputs[i].narrow_fib_num_steps_20_inputs[5] = input;
                }
                *lookup_data.narrow_fib_num_steps_20_5 = [
                    narrow_fib_num_steps_20_output_col9,
                    narrow_fib_num_steps_20_output_col10,
                    narrow_fib_num_steps_20_output_col11,
                    narrow_fib_num_steps_20_output_col12,
                ];
                let narrow_fib_num_steps_20_output_tmp_d7cf2_6 =
                    narrow_fib_num_steps_20::deduce_output([
                        narrow_fib_num_steps_20_output_col11,
                        narrow_fib_num_steps_20_output_col12,
                    ]);
                let narrow_fib_num_steps_20_output_col13 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_6[0];
                *row[13] = narrow_fib_num_steps_20_output_col13;
                let narrow_fib_num_steps_20_output_col14 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_6[1];
                *row[14] = narrow_fib_num_steps_20_output_col14;
                for (i, &input) in [
                    narrow_fib_num_steps_20_output_col11,
                    narrow_fib_num_steps_20_output_col12,
                ]
                .unpack()
                .iter()
                .enumerate()
                {
                    *sub_components_inputs[i].narrow_fib_num_steps_20_inputs[6] = input;
                }
                *lookup_data.narrow_fib_num_steps_20_6 = [
                    narrow_fib_num_steps_20_output_col11,
                    narrow_fib_num_steps_20_output_col12,
                    narrow_fib_num_steps_20_output_col13,
                    narrow_fib_num_steps_20_output_col14,
                ];
                let narrow_fib_num_steps_20_output_tmp_d7cf2_7 =
                    narrow_fib_num_steps_20::deduce_output([
                        narrow_fib_num_steps_20_output_col13,
                        narrow_fib_num_steps_20_output_col14,
                    ]);
                let narrow_fib_num_steps_20_output_col15 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_7[0];
                *row[15] = narrow_fib_num_steps_20_output_col15;
                let narrow_fib_num_steps_20_output_col16 =
                    narrow_fib_num_steps_20_output_tmp_d7cf2_7[1];
                *row[16] = narrow_fib_num_steps_20_output_col16;
                for (i, &input) in [
                    narrow_fib_num_steps_20_output_col13,
                    narrow_fib_num_steps_20_output_col14,
                ]
                .unpack()
                .iter()
                .enumerate()
                {
                    *sub_components_inputs[i].narrow_fib_num_steps_20_inputs[7] = input;
                }
                *lookup_data.narrow_fib_num_steps_20_7 = [
                    narrow_fib_num_steps_20_output_col13,
                    narrow_fib_num_steps_20_output_col14,
                    narrow_fib_num_steps_20_output_col15,
                    narrow_fib_num_steps_20_output_col16,
                ];
            },
        );

    (trace, sub_components_inputs, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    narrow_fib_num_steps_20_0: Vec<[PackedM31; 4]>,
    narrow_fib_num_steps_20_1: Vec<[PackedM31; 4]>,
    narrow_fib_num_steps_20_2: Vec<[PackedM31; 4]>,
    narrow_fib_num_steps_20_3: Vec<[PackedM31; 4]>,
    narrow_fib_num_steps_20_4: Vec<[PackedM31; 4]>,
    narrow_fib_num_steps_20_5: Vec<[PackedM31; 4]>,
    narrow_fib_num_steps_20_6: Vec<[PackedM31; 4]>,
    narrow_fib_num_steps_20_7: Vec<[PackedM31; 4]>,
}

pub struct InteractionClaimGenerator {
    n_rows: usize,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace<MC: MerkleChannel>(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
        narrow_fib_num_steps_20: &relations::NarrowFibNumSteps20,
    ) -> InteractionClaim
    where
        SimdBackend: BackendForChannel<MC>,
    {
        let log_size = std::cmp::max(self.n_rows.next_power_of_two().ilog2(), LOG_N_LANES);
        let mut logup_gen = LogupTraceGenerator::new(log_size);

        // Sum logup terms in pairs.
        let mut col_gen = logup_gen.new_col();
        for (i, (values0, values1)) in zip(
            &self.lookup_data.narrow_fib_num_steps_20_0,
            &self.lookup_data.narrow_fib_num_steps_20_1,
        )
        .enumerate()
        {
            let denom0: PackedQM31 = narrow_fib_num_steps_20.combine(values0);
            let denom1: PackedQM31 = narrow_fib_num_steps_20.combine(values1);
            col_gen.write_frac(i, denom0 + denom1, denom0 * denom1);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        for (i, (values0, values1)) in zip(
            &self.lookup_data.narrow_fib_num_steps_20_2,
            &self.lookup_data.narrow_fib_num_steps_20_3,
        )
        .enumerate()
        {
            let denom0: PackedQM31 = narrow_fib_num_steps_20.combine(values0);
            let denom1: PackedQM31 = narrow_fib_num_steps_20.combine(values1);
            col_gen.write_frac(i, denom0 + denom1, denom0 * denom1);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        for (i, (values0, values1)) in zip(
            &self.lookup_data.narrow_fib_num_steps_20_4,
            &self.lookup_data.narrow_fib_num_steps_20_5,
        )
        .enumerate()
        {
            let denom0: PackedQM31 = narrow_fib_num_steps_20.combine(values0);
            let denom1: PackedQM31 = narrow_fib_num_steps_20.combine(values1);
            col_gen.write_frac(i, denom0 + denom1, denom0 * denom1);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        for (i, (values0, values1)) in zip(
            &self.lookup_data.narrow_fib_num_steps_20_6,
            &self.lookup_data.narrow_fib_num_steps_20_7,
        )
        .enumerate()
        {
            let denom0: PackedQM31 = narrow_fib_num_steps_20.combine(values0);
            let denom1: PackedQM31 = narrow_fib_num_steps_20.combine(values1);
            col_gen.write_frac(i, denom0 + denom1, denom0 * denom1);
        }
        col_gen.finalize_col();

        let (trace, total_sum, claimed_sum) = if self.n_rows == 1 << log_size {
            let (trace, claimed_sum) = logup_gen.finalize_last();
            (trace, claimed_sum, None)
        } else {
            let (trace, [total_sum, claimed_sum]) =
                logup_gen.finalize_at([(1 << log_size) - 1, self.n_rows - 1]);
            (trace, total_sum, Some((claimed_sum, self.n_rows - 1)))
        };
        tree_builder.extend_evals(trace);

        InteractionClaim {
            logup_sums: (total_sum, claimed_sum),
        }
    }
}
