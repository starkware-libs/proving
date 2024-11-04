#![allow(unused_parens)]
#![allow(unused_imports)]
use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
use prover_types::cpu::*;
use prover_types::simd::*;
use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
use stwo_prover::core::backend::simd::qm31::PackedQM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{Col, Column};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};

use super::component::{Claim, InteractionClaim, RelationElements};
use crate::narrowfib_num_steps_20;

pub type InputType = PackedM31;

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn write_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        narrowfib_num_steps_20_state: &mut narrowfib_num_steps_20::ClaimGenerator,
    ) -> (Claim, InteractionClaimGenerator) {
        let len = self.inputs.len();
        #[allow(unused_variables)]
        let (trace, sub_components_inputs, lookup_data) = write_trace_simd(self.inputs);
        sub_components_inputs
            .narrowfib_num_steps_20_inputs
            .iter()
            .for_each(|inputs| {
                narrowfib_num_steps_20_state.add_inputs(inputs);
            });

        tree_builder.extend_evals(trace);

        let n_calls = len * N_LANES;
        (
            Claim { n_calls },
            InteractionClaimGenerator {
                n_calls,
                lookup_data,
            },
        )
    }

    pub fn add_inputs(&mut self, inputs: &[InputType]) {
        self.inputs.extend(inputs);
    }
}

pub struct SubComponentInputs {
    pub narrowfib_num_steps_20_inputs: [Vec<narrowfib_num_steps_20::InputType>; 8],
}
impl SubComponentInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            narrowfib_num_steps_20_inputs: [
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
            ],
        }
    }
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
pub fn write_trace_simd(
    inputs: Vec<InputType>,
) -> (
    Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>,
    SubComponentInputs,
    LookupData,
) {
    const N_TRACE_COLUMNS: usize = 17;
    let mut trace_values: [_; N_TRACE_COLUMNS] =
        std::array::from_fn(|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES));

    let mut lookup_data = LookupData::with_capacity(inputs.len());
    #[allow(unused_mut)]
    let mut sub_components_inputs = SubComponentInputs::with_capacity(inputs.len());

    let M31_1 = PackedM31::broadcast(M31::from(1));

    inputs.into_iter().enumerate().for_each(
        |(row_index, widefib_num_narrow_8_narrow_size_20_input)| {
            let col0 = widefib_num_narrow_8_narrow_size_20_input;
            trace_values[0].data[row_index] = col0;
            sub_components_inputs.narrowfib_num_steps_20_inputs[0].push([M31_1, col0]);
            let narrowfib_num_steps_20_output_tmp_1 =
                narrowfib_num_steps_20::deduce_output([M31_1, col0]);
            let narrowfib_num_steps_20_output_col1 = narrowfib_num_steps_20_output_tmp_1[0];
            trace_values[1].data[row_index] = narrowfib_num_steps_20_output_col1;
            let narrowfib_num_steps_20_output_col2 = narrowfib_num_steps_20_output_tmp_1[1];
            trace_values[2].data[row_index] = narrowfib_num_steps_20_output_col2;
            lookup_data.narrowfib_num_steps_20[0].push([
                M31_1,
                col0,
                narrowfib_num_steps_20_output_col1,
                narrowfib_num_steps_20_output_col2,
            ]);
            sub_components_inputs.narrowfib_num_steps_20_inputs[1].push([
                narrowfib_num_steps_20_output_col1,
                narrowfib_num_steps_20_output_col2,
            ]);
            let narrowfib_num_steps_20_output_tmp_2 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col1,
                narrowfib_num_steps_20_output_col2,
            ]);
            let narrowfib_num_steps_20_output_col3 = narrowfib_num_steps_20_output_tmp_2[0];
            trace_values[3].data[row_index] = narrowfib_num_steps_20_output_col3;
            let narrowfib_num_steps_20_output_col4 = narrowfib_num_steps_20_output_tmp_2[1];
            trace_values[4].data[row_index] = narrowfib_num_steps_20_output_col4;
            lookup_data.narrowfib_num_steps_20[1].push([
                narrowfib_num_steps_20_output_col1,
                narrowfib_num_steps_20_output_col2,
                narrowfib_num_steps_20_output_col3,
                narrowfib_num_steps_20_output_col4,
            ]);
            sub_components_inputs.narrowfib_num_steps_20_inputs[2].push([
                narrowfib_num_steps_20_output_col3,
                narrowfib_num_steps_20_output_col4,
            ]);
            let narrowfib_num_steps_20_output_tmp_3 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col3,
                narrowfib_num_steps_20_output_col4,
            ]);
            let narrowfib_num_steps_20_output_col5 = narrowfib_num_steps_20_output_tmp_3[0];
            trace_values[5].data[row_index] = narrowfib_num_steps_20_output_col5;
            let narrowfib_num_steps_20_output_col6 = narrowfib_num_steps_20_output_tmp_3[1];
            trace_values[6].data[row_index] = narrowfib_num_steps_20_output_col6;
            lookup_data.narrowfib_num_steps_20[2].push([
                narrowfib_num_steps_20_output_col3,
                narrowfib_num_steps_20_output_col4,
                narrowfib_num_steps_20_output_col5,
                narrowfib_num_steps_20_output_col6,
            ]);
            sub_components_inputs.narrowfib_num_steps_20_inputs[3].push([
                narrowfib_num_steps_20_output_col5,
                narrowfib_num_steps_20_output_col6,
            ]);
            let narrowfib_num_steps_20_output_tmp_4 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col5,
                narrowfib_num_steps_20_output_col6,
            ]);
            let narrowfib_num_steps_20_output_col7 = narrowfib_num_steps_20_output_tmp_4[0];
            trace_values[7].data[row_index] = narrowfib_num_steps_20_output_col7;
            let narrowfib_num_steps_20_output_col8 = narrowfib_num_steps_20_output_tmp_4[1];
            trace_values[8].data[row_index] = narrowfib_num_steps_20_output_col8;
            lookup_data.narrowfib_num_steps_20[3].push([
                narrowfib_num_steps_20_output_col5,
                narrowfib_num_steps_20_output_col6,
                narrowfib_num_steps_20_output_col7,
                narrowfib_num_steps_20_output_col8,
            ]);
            sub_components_inputs.narrowfib_num_steps_20_inputs[4].push([
                narrowfib_num_steps_20_output_col7,
                narrowfib_num_steps_20_output_col8,
            ]);
            let narrowfib_num_steps_20_output_tmp_5 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col7,
                narrowfib_num_steps_20_output_col8,
            ]);
            let narrowfib_num_steps_20_output_col9 = narrowfib_num_steps_20_output_tmp_5[0];
            trace_values[9].data[row_index] = narrowfib_num_steps_20_output_col9;
            let narrowfib_num_steps_20_output_col10 = narrowfib_num_steps_20_output_tmp_5[1];
            trace_values[10].data[row_index] = narrowfib_num_steps_20_output_col10;
            lookup_data.narrowfib_num_steps_20[4].push([
                narrowfib_num_steps_20_output_col7,
                narrowfib_num_steps_20_output_col8,
                narrowfib_num_steps_20_output_col9,
                narrowfib_num_steps_20_output_col10,
            ]);
            sub_components_inputs.narrowfib_num_steps_20_inputs[5].push([
                narrowfib_num_steps_20_output_col9,
                narrowfib_num_steps_20_output_col10,
            ]);
            let narrowfib_num_steps_20_output_tmp_6 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col9,
                narrowfib_num_steps_20_output_col10,
            ]);
            let narrowfib_num_steps_20_output_col11 = narrowfib_num_steps_20_output_tmp_6[0];
            trace_values[11].data[row_index] = narrowfib_num_steps_20_output_col11;
            let narrowfib_num_steps_20_output_col12 = narrowfib_num_steps_20_output_tmp_6[1];
            trace_values[12].data[row_index] = narrowfib_num_steps_20_output_col12;
            lookup_data.narrowfib_num_steps_20[5].push([
                narrowfib_num_steps_20_output_col9,
                narrowfib_num_steps_20_output_col10,
                narrowfib_num_steps_20_output_col11,
                narrowfib_num_steps_20_output_col12,
            ]);
            sub_components_inputs.narrowfib_num_steps_20_inputs[6].push([
                narrowfib_num_steps_20_output_col11,
                narrowfib_num_steps_20_output_col12,
            ]);
            let narrowfib_num_steps_20_output_tmp_7 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col11,
                narrowfib_num_steps_20_output_col12,
            ]);
            let narrowfib_num_steps_20_output_col13 = narrowfib_num_steps_20_output_tmp_7[0];
            trace_values[13].data[row_index] = narrowfib_num_steps_20_output_col13;
            let narrowfib_num_steps_20_output_col14 = narrowfib_num_steps_20_output_tmp_7[1];
            trace_values[14].data[row_index] = narrowfib_num_steps_20_output_col14;
            lookup_data.narrowfib_num_steps_20[6].push([
                narrowfib_num_steps_20_output_col11,
                narrowfib_num_steps_20_output_col12,
                narrowfib_num_steps_20_output_col13,
                narrowfib_num_steps_20_output_col14,
            ]);
            sub_components_inputs.narrowfib_num_steps_20_inputs[7].push([
                narrowfib_num_steps_20_output_col13,
                narrowfib_num_steps_20_output_col14,
            ]);
            let narrowfib_num_steps_20_output_tmp_8 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col13,
                narrowfib_num_steps_20_output_col14,
            ]);
            let narrowfib_num_steps_20_output_col15 = narrowfib_num_steps_20_output_tmp_8[0];
            trace_values[15].data[row_index] = narrowfib_num_steps_20_output_col15;
            let narrowfib_num_steps_20_output_col16 = narrowfib_num_steps_20_output_tmp_8[1];
            trace_values[16].data[row_index] = narrowfib_num_steps_20_output_col16;
            lookup_data.narrowfib_num_steps_20[7].push([
                narrowfib_num_steps_20_output_col13,
                narrowfib_num_steps_20_output_col14,
                narrowfib_num_steps_20_output_col15,
                narrowfib_num_steps_20_output_col16,
            ]);
        },
    );

    let trace = trace_values
        .into_iter()
        .map(|eval| {
            let domain = CanonicCoset::new(
                eval.len()
                    .checked_ilog2()
                    .expect("Input is not a power of 2!"),
            )
            .circle_domain();
            CircleEvaluation::<SimdBackend, M31, BitReversedOrder>::new(domain, eval)
        })
        .collect_vec();
    (trace, sub_components_inputs, lookup_data)
}

pub struct LookupData {
    pub narrowfib_num_steps_20: [Vec<[PackedM31; 4]>; 8],
}
impl LookupData {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            narrowfib_num_steps_20: [
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
            ],
        }
    }
}

pub struct InteractionClaimGenerator {
    pub n_calls: usize,
    pub lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        narrowfib_num_steps_20_lookup_elements: &narrowfib_num_steps_20::RelationElements,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.n_calls.next_power_of_two().ilog2());

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[1];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[2];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[3];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[4];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[5];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[6];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[7];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
