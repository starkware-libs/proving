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
use stwo_prover::core::backend::{BackendForChannel, Col, Column};
use stwo_prover::core::channel::{Channel, MerkleChannel};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::utils::bit_reverse_coset_to_circle_domain_order;

use super::component::{Claim, InteractionClaim};
use crate::components::pack_values;
use crate::relations;

pub type InputType = [M31; 2];
pub type PackedInputType = [PackedM31; 2];
const N_TRACE_COLUMNS: usize = 22;

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
    ) -> (Claim, InteractionClaimGenerator)
    where
        SimdBackend: BackendForChannel<MC>,
    {
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

pub struct SubComponentInputs {}
impl SubComponentInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {}
    }

    fn bit_reverse_coset_to_circle_domain_order(&mut self) {}
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
    const N_TRACE_COLUMNS: usize = 22;
    let mut trace: [_; N_TRACE_COLUMNS] =
        std::array::from_fn(|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES));

    let mut lookup_data = LookupData::with_capacity(inputs.len());
    #[allow(unused_mut)]
    let mut sub_components_inputs = SubComponentInputs::with_capacity(inputs.len());

    inputs
        .into_iter()
        .enumerate()
        .for_each(|(row_index, narrow_fib_num_steps_20_input)| {
            let input_tmp_1ddf_0 = [
                narrow_fib_num_steps_20_input[0],
                narrow_fib_num_steps_20_input[1],
            ];
            let input_col0 = input_tmp_1ddf_0[0];
            trace[0].data[row_index] = input_col0;
            let input_col1 = input_tmp_1ddf_0[1];
            trace[1].data[row_index] = input_col1;

            // FibStep.

            let col2 = (((input_col0) * (input_col0)) + ((input_col1) * (input_col1)));
            trace[2].data[row_index] = col2;

            // FibStep.

            let col3 = (((input_col1) * (input_col1)) + ((col2) * (col2)));
            trace[3].data[row_index] = col3;

            // FibStep.

            let col4 = (((col2) * (col2)) + ((col3) * (col3)));
            trace[4].data[row_index] = col4;

            // FibStep.

            let col5 = (((col3) * (col3)) + ((col4) * (col4)));
            trace[5].data[row_index] = col5;

            // FibStep.

            let col6 = (((col4) * (col4)) + ((col5) * (col5)));
            trace[6].data[row_index] = col6;

            // FibStep.

            let col7 = (((col5) * (col5)) + ((col6) * (col6)));
            trace[7].data[row_index] = col7;

            // FibStep.

            let col8 = (((col6) * (col6)) + ((col7) * (col7)));
            trace[8].data[row_index] = col8;

            // FibStep.

            let col9 = (((col7) * (col7)) + ((col8) * (col8)));
            trace[9].data[row_index] = col9;

            // FibStep.

            let col10 = (((col8) * (col8)) + ((col9) * (col9)));
            trace[10].data[row_index] = col10;

            // FibStep.

            let col11 = (((col9) * (col9)) + ((col10) * (col10)));
            trace[11].data[row_index] = col11;

            // FibStep.

            let col12 = (((col10) * (col10)) + ((col11) * (col11)));
            trace[12].data[row_index] = col12;

            // FibStep.

            let col13 = (((col11) * (col11)) + ((col12) * (col12)));
            trace[13].data[row_index] = col13;

            // FibStep.

            let col14 = (((col12) * (col12)) + ((col13) * (col13)));
            trace[14].data[row_index] = col14;

            // FibStep.

            let col15 = (((col13) * (col13)) + ((col14) * (col14)));
            trace[15].data[row_index] = col15;

            // FibStep.

            let col16 = (((col14) * (col14)) + ((col15) * (col15)));
            trace[16].data[row_index] = col16;

            // FibStep.

            let col17 = (((col15) * (col15)) + ((col16) * (col16)));
            trace[17].data[row_index] = col17;

            // FibStep.

            let col18 = (((col16) * (col16)) + ((col17) * (col17)));
            trace[18].data[row_index] = col18;

            // FibStep.

            let col19 = (((col17) * (col17)) + ((col18) * (col18)));
            trace[19].data[row_index] = col19;

            // FibStep.

            let col20 = (((col18) * (col18)) + ((col19) * (col19)));
            trace[20].data[row_index] = col20;

            // FibStep.

            let col21 = (((col19) * (col19)) + ((col20) * (col20)));
            trace[21].data[row_index] = col21;

            lookup_data.narrowfib_num_steps_20[0].push([input_col0, input_col1, col20, col21]);
        });

    (trace, sub_components_inputs, lookup_data)
}

pub struct LookupData {
    pub narrowfib_num_steps_20: [Vec<[PackedM31; 4]>; 1],
}
impl LookupData {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            narrowfib_num_steps_20: [Vec::with_capacity(capacity)],
        }
    }
}

pub struct InteractionClaimGenerator {
    pub n_calls: usize,
    pub lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace<MC: MerkleChannel>(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
        narrowfib_num_steps_20_lookup_elements: &relations::NarrowFib_num_steps_20,
    ) -> InteractionClaim
    where
        SimdBackend: BackendForChannel<MC>,
    {
        let log_size = std::cmp::max(self.n_calls.next_power_of_two().ilog2(), LOG_N_LANES);
        let mut logup_gen = LogupTraceGenerator::new(log_size);

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, -PackedQM31::one(), denom);
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
