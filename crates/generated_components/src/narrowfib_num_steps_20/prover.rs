#![allow(unused_parens)]
#![allow(unused_imports)]
use air_code_gen::code_gen::packed_types::*;
use air_infra::core::prover_types::*;
use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
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

use super::component::{Claim, ComponentLookupElements, InteractionClaim};
use crate::narrowfib_num_steps_20;

pub type InputType = [PackedM31; 2];

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn write_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
    ) -> ClaimProver {
        let len = self.inputs.len();
        #[allow(unused_variables)]
        let (trace, sub_components_inputs, lookup_data) = write_trace_simd(self.inputs);

        tree_builder.extend_evals(trace);
        let claim = Claim {
            log_size: len.ilog2() + LOG_N_LANES,
            n_calls: len * N_LANES,
        };

        ClaimProver { claim, lookup_data }
    }

    pub fn add_inputs(&mut self, inputs: &[InputType]) {
        self.inputs.extend(inputs);
    }
}

#[allow(non_snake_case)]
pub struct SubComponentInputs {}
impl SubComponentInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {}
    }
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
pub fn write_trace_simd(
    inputs: Vec<InputType>,
) -> (
    Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>,
    SubComponentInputs,
    LookupData,
) {
    let n_trace_columns = 22;
    let mut trace_values = (0..n_trace_columns)
        .map(|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES))
        .collect_vec();
    let mut lookup_data = LookupData::with_capacity(inputs.len());
    #[allow(unused_mut)]
    let mut sub_components_inputs = SubComponentInputs::with_capacity(inputs.len());
    inputs
        .into_iter()
        .enumerate()
        .for_each(|(row_index, narrowfib_num_steps_20_input)| {
            let tmp_0 = [
                narrowfib_num_steps_20_input[0],
                narrowfib_num_steps_20_input[1],
            ];
            let col0 = tmp_0[0];
            trace_values[0].data[row_index] = col0;
            let col1 = tmp_0[1];
            trace_values[1].data[row_index] = col1;

            // FibStep.
            let col2 = (((col0) * (col0)) + ((col1) * (col1)));
            trace_values[2].data[row_index] = col2;

            // FibStep.
            let col3 = (((col1) * (col1)) + ((col2) * (col2)));
            trace_values[3].data[row_index] = col3;

            // FibStep.
            let col4 = (((col2) * (col2)) + ((col3) * (col3)));
            trace_values[4].data[row_index] = col4;

            // FibStep.
            let col5 = (((col3) * (col3)) + ((col4) * (col4)));
            trace_values[5].data[row_index] = col5;

            // FibStep.
            let col6 = (((col4) * (col4)) + ((col5) * (col5)));
            trace_values[6].data[row_index] = col6;

            // FibStep.
            let col7 = (((col5) * (col5)) + ((col6) * (col6)));
            trace_values[7].data[row_index] = col7;

            // FibStep.
            let col8 = (((col6) * (col6)) + ((col7) * (col7)));
            trace_values[8].data[row_index] = col8;

            // FibStep.
            let col9 = (((col7) * (col7)) + ((col8) * (col8)));
            trace_values[9].data[row_index] = col9;

            // FibStep.
            let col10 = (((col8) * (col8)) + ((col9) * (col9)));
            trace_values[10].data[row_index] = col10;

            // FibStep.
            let col11 = (((col9) * (col9)) + ((col10) * (col10)));
            trace_values[11].data[row_index] = col11;

            // FibStep.
            let col12 = (((col10) * (col10)) + ((col11) * (col11)));
            trace_values[12].data[row_index] = col12;

            // FibStep.
            let col13 = (((col11) * (col11)) + ((col12) * (col12)));
            trace_values[13].data[row_index] = col13;

            // FibStep.
            let col14 = (((col12) * (col12)) + ((col13) * (col13)));
            trace_values[14].data[row_index] = col14;

            // FibStep.
            let col15 = (((col13) * (col13)) + ((col14) * (col14)));
            trace_values[15].data[row_index] = col15;

            // FibStep.
            let col16 = (((col14) * (col14)) + ((col15) * (col15)));
            trace_values[16].data[row_index] = col16;

            // FibStep.
            let col17 = (((col15) * (col15)) + ((col16) * (col16)));
            trace_values[17].data[row_index] = col17;

            // FibStep.
            let col18 = (((col16) * (col16)) + ((col17) * (col17)));
            trace_values[18].data[row_index] = col18;

            // FibStep.
            let col19 = (((col17) * (col17)) + ((col18) * (col18)));
            trace_values[19].data[row_index] = col19;

            // FibStep.
            let col20 = (((col18) * (col18)) + ((col19) * (col19)));
            trace_values[20].data[row_index] = col20;

            // FibStep.
            let col21 = (((col19) * (col19)) + ((col20) * (col20)));
            trace_values[21].data[row_index] = col21;

            lookup_data.narrowfib_num_steps_20[0].push([col0, col1, col20, col21]);
        });

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

#[allow(non_snake_case)]
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

pub struct ClaimProver {
    pub claim: Claim,
    pub lookup_data: LookupData,
}
impl ClaimProver {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        narrowfib_num_steps_20_lookup_elements: &narrowfib_num_steps_20::ComponentLookupElements,
    ) -> InteractionClaim {
        let log_size = self.claim.log_size;
        let mut logup_gen = LogupTraceGenerator::new(log_size);

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.narrowfib_num_steps_20[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = narrowfib_num_steps_20_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, -PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
