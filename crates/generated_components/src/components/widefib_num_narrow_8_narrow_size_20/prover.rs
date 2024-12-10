#![allow(unused_parens)]
#![allow(unused_imports)]
use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
use prover_types::cpu::*;
use prover_types::simd::*;
use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
use stwo_prover::constraint_framework::Relation;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::column::BaseColumn;
use stwo_prover::core::backend::simd::conversion::Unpack;
use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
use stwo_prover::core::backend::simd::qm31::PackedQM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{Col, Column};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::utils::bit_reverse_coset_to_circle_domain_order;
use stwo_prover::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};

use super::component::{Claim, InteractionClaim};
use crate::components::{narrowfib_num_steps_20, pack_values};
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

    pub fn write_trace(
        mut self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        narrowfib_num_steps_20_state: &mut narrowfib_num_steps_20::ClaimGenerator,
    ) -> (Claim, InteractionClaimGenerator) {
        let n_calls = self.inputs.len();
        assert_ne!(n_calls, 0);
        let size = std::cmp::max(n_calls.next_power_of_two(), N_LANES);
        let need_padding = n_calls != size;

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
            .narrowfib_num_steps_20_inputs
            .iter()
            .for_each(|inputs| {
                narrowfib_num_steps_20_state.add_inputs(&inputs[..n_calls]);
            });

        tree_builder.extend_evals(
            trace
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
                .collect_vec(),
        );

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

    fn bit_reverse_coset_to_circle_domain_order(&mut self) {
        self.narrowfib_num_steps_20_inputs
            .iter_mut()
            .for_each(|vec| bit_reverse_coset_to_circle_domain_order(vec));
    }
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
pub fn write_trace_simd(
    inputs: Vec<PackedInputType>,
) -> (
    [BaseColumn; N_TRACE_COLUMNS],
    SubComponentInputs,
    LookupData,
) {
    const N_TRACE_COLUMNS: usize = 17;
    let mut trace: [_; N_TRACE_COLUMNS] =
        std::array::from_fn(|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES));

    let mut lookup_data = LookupData::with_capacity(inputs.len());
    #[allow(unused_mut)]
    let mut sub_components_inputs = SubComponentInputs::with_capacity(inputs.len());

    let M31_1 = PackedM31::broadcast(M31::from(1));

    inputs.into_iter().enumerate().for_each(
        |(row_index, wide_fib_num_narrow_8_narrow_size_20_input)| {
            let col0 = wide_fib_num_narrow_8_narrow_size_20_input;
            trace[0].data[row_index] = col0;
            let narrowfib_num_steps_20_output_tmp_d7cf_0 =
                narrowfib_num_steps_20::deduce_output([M31_1, col0]);
            let narrowfib_num_steps_20_output_col1 = narrowfib_num_steps_20_output_tmp_d7cf_0[0];
            trace[1].data[row_index] = narrowfib_num_steps_20_output_col1;
            let narrowfib_num_steps_20_output_col2 = narrowfib_num_steps_20_output_tmp_d7cf_0[1];
            trace[2].data[row_index] = narrowfib_num_steps_20_output_col2;
            sub_components_inputs.narrowfib_num_steps_20_inputs[0].extend([M31_1, col0].unpack());

            lookup_data.narrowfib_num_steps_20[0].push([
                M31_1,
                col0,
                narrowfib_num_steps_20_output_col1,
                narrowfib_num_steps_20_output_col2,
            ]);
            let narrowfib_num_steps_20_output_tmp_d7cf_1 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col1,
                narrowfib_num_steps_20_output_col2,
            ]);
            let narrowfib_num_steps_20_output_col3 = narrowfib_num_steps_20_output_tmp_d7cf_1[0];
            trace[3].data[row_index] = narrowfib_num_steps_20_output_col3;
            let narrowfib_num_steps_20_output_col4 = narrowfib_num_steps_20_output_tmp_d7cf_1[1];
            trace[4].data[row_index] = narrowfib_num_steps_20_output_col4;
            sub_components_inputs.narrowfib_num_steps_20_inputs[1].extend(
                [
                    narrowfib_num_steps_20_output_col1,
                    narrowfib_num_steps_20_output_col2,
                ]
                .unpack(),
            );

            lookup_data.narrowfib_num_steps_20[1].push([
                narrowfib_num_steps_20_output_col1,
                narrowfib_num_steps_20_output_col2,
                narrowfib_num_steps_20_output_col3,
                narrowfib_num_steps_20_output_col4,
            ]);
            let narrowfib_num_steps_20_output_tmp_d7cf_2 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col3,
                narrowfib_num_steps_20_output_col4,
            ]);
            let narrowfib_num_steps_20_output_col5 = narrowfib_num_steps_20_output_tmp_d7cf_2[0];
            trace[5].data[row_index] = narrowfib_num_steps_20_output_col5;
            let narrowfib_num_steps_20_output_col6 = narrowfib_num_steps_20_output_tmp_d7cf_2[1];
            trace[6].data[row_index] = narrowfib_num_steps_20_output_col6;
            sub_components_inputs.narrowfib_num_steps_20_inputs[2].extend(
                [
                    narrowfib_num_steps_20_output_col3,
                    narrowfib_num_steps_20_output_col4,
                ]
                .unpack(),
            );

            lookup_data.narrowfib_num_steps_20[2].push([
                narrowfib_num_steps_20_output_col3,
                narrowfib_num_steps_20_output_col4,
                narrowfib_num_steps_20_output_col5,
                narrowfib_num_steps_20_output_col6,
            ]);
            let narrowfib_num_steps_20_output_tmp_d7cf_3 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col5,
                narrowfib_num_steps_20_output_col6,
            ]);
            let narrowfib_num_steps_20_output_col7 = narrowfib_num_steps_20_output_tmp_d7cf_3[0];
            trace[7].data[row_index] = narrowfib_num_steps_20_output_col7;
            let narrowfib_num_steps_20_output_col8 = narrowfib_num_steps_20_output_tmp_d7cf_3[1];
            trace[8].data[row_index] = narrowfib_num_steps_20_output_col8;
            sub_components_inputs.narrowfib_num_steps_20_inputs[3].extend(
                [
                    narrowfib_num_steps_20_output_col5,
                    narrowfib_num_steps_20_output_col6,
                ]
                .unpack(),
            );

            lookup_data.narrowfib_num_steps_20[3].push([
                narrowfib_num_steps_20_output_col5,
                narrowfib_num_steps_20_output_col6,
                narrowfib_num_steps_20_output_col7,
                narrowfib_num_steps_20_output_col8,
            ]);
            let narrowfib_num_steps_20_output_tmp_d7cf_4 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col7,
                narrowfib_num_steps_20_output_col8,
            ]);
            let narrowfib_num_steps_20_output_col9 = narrowfib_num_steps_20_output_tmp_d7cf_4[0];
            trace[9].data[row_index] = narrowfib_num_steps_20_output_col9;
            let narrowfib_num_steps_20_output_col10 = narrowfib_num_steps_20_output_tmp_d7cf_4[1];
            trace[10].data[row_index] = narrowfib_num_steps_20_output_col10;
            sub_components_inputs.narrowfib_num_steps_20_inputs[4].extend(
                [
                    narrowfib_num_steps_20_output_col7,
                    narrowfib_num_steps_20_output_col8,
                ]
                .unpack(),
            );

            lookup_data.narrowfib_num_steps_20[4].push([
                narrowfib_num_steps_20_output_col7,
                narrowfib_num_steps_20_output_col8,
                narrowfib_num_steps_20_output_col9,
                narrowfib_num_steps_20_output_col10,
            ]);
            let narrowfib_num_steps_20_output_tmp_d7cf_5 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col9,
                narrowfib_num_steps_20_output_col10,
            ]);
            let narrowfib_num_steps_20_output_col11 = narrowfib_num_steps_20_output_tmp_d7cf_5[0];
            trace[11].data[row_index] = narrowfib_num_steps_20_output_col11;
            let narrowfib_num_steps_20_output_col12 = narrowfib_num_steps_20_output_tmp_d7cf_5[1];
            trace[12].data[row_index] = narrowfib_num_steps_20_output_col12;
            sub_components_inputs.narrowfib_num_steps_20_inputs[5].extend(
                [
                    narrowfib_num_steps_20_output_col9,
                    narrowfib_num_steps_20_output_col10,
                ]
                .unpack(),
            );

            lookup_data.narrowfib_num_steps_20[5].push([
                narrowfib_num_steps_20_output_col9,
                narrowfib_num_steps_20_output_col10,
                narrowfib_num_steps_20_output_col11,
                narrowfib_num_steps_20_output_col12,
            ]);
            let narrowfib_num_steps_20_output_tmp_d7cf_6 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col11,
                narrowfib_num_steps_20_output_col12,
            ]);
            let narrowfib_num_steps_20_output_col13 = narrowfib_num_steps_20_output_tmp_d7cf_6[0];
            trace[13].data[row_index] = narrowfib_num_steps_20_output_col13;
            let narrowfib_num_steps_20_output_col14 = narrowfib_num_steps_20_output_tmp_d7cf_6[1];
            trace[14].data[row_index] = narrowfib_num_steps_20_output_col14;
            sub_components_inputs.narrowfib_num_steps_20_inputs[6].extend(
                [
                    narrowfib_num_steps_20_output_col11,
                    narrowfib_num_steps_20_output_col12,
                ]
                .unpack(),
            );

            lookup_data.narrowfib_num_steps_20[6].push([
                narrowfib_num_steps_20_output_col11,
                narrowfib_num_steps_20_output_col12,
                narrowfib_num_steps_20_output_col13,
                narrowfib_num_steps_20_output_col14,
            ]);
            let narrowfib_num_steps_20_output_tmp_d7cf_7 = narrowfib_num_steps_20::deduce_output([
                narrowfib_num_steps_20_output_col13,
                narrowfib_num_steps_20_output_col14,
            ]);
            let narrowfib_num_steps_20_output_col15 = narrowfib_num_steps_20_output_tmp_d7cf_7[0];
            trace[15].data[row_index] = narrowfib_num_steps_20_output_col15;
            let narrowfib_num_steps_20_output_col16 = narrowfib_num_steps_20_output_tmp_d7cf_7[1];
            trace[16].data[row_index] = narrowfib_num_steps_20_output_col16;
            sub_components_inputs.narrowfib_num_steps_20_inputs[7].extend(
                [
                    narrowfib_num_steps_20_output_col13,
                    narrowfib_num_steps_20_output_col14,
                ]
                .unpack(),
            );

            lookup_data.narrowfib_num_steps_20[7].push([
                narrowfib_num_steps_20_output_col13,
                narrowfib_num_steps_20_output_col14,
                narrowfib_num_steps_20_output_col15,
                narrowfib_num_steps_20_output_col16,
            ]);
        },
    );

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
        narrowfib_num_steps_20_lookup_elements: &relations::NarrowFib_num_steps_20,
    ) -> InteractionClaim {
        let log_size = std::cmp::max(self.n_calls.next_power_of_two().ilog2(), LOG_N_LANES);
        let mut logup_gen = LogupTraceGenerator::new(log_size);

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

        let (trace, total_sum, claimed_sum) = if self.n_calls == 1 << log_size {
            let (trace, claimed_sum) = logup_gen.finalize_last();
            (trace, claimed_sum, None)
        } else {
            let (trace, [total_sum, claimed_sum]) =
                logup_gen.finalize_at([(1 << log_size) - 1, self.n_calls - 1]);
            (trace, total_sum, Some((claimed_sum, self.n_calls - 1)))
        };
        tree_builder.extend_evals(trace);

        InteractionClaim {
            logup_sums: (total_sum, claimed_sum),
        }
    }
}
