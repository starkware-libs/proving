#![allow(unused_imports)]
use std::iter::zip;

use air_infra::core::prover_types::*;
use itertools::Itertools;
use num_traits::Zero;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{Col, Column};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};
use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;

use super::Claim;
use crate::airs::examples::NarrowFib_1ddf31c88316e62f;
use crate::code_gen::packed_types::*;

pub type InputType = PackedM31;
pub type OutputType = PackedM31;

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn write_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        narrowfib_1ddf31c88316e62f_state: &mut NarrowFib_1ddf31c88316e62f::ClaimGenerator,
    ) -> ClaimProver {
        let len = self.inputs.len();
        let (trace, lookup_data) = write_trace_simd(self.inputs);
        lookup_data
            .narrowfib_1ddf31c88316e62f_inputs
            .iter()
            .for_each(|inputs| {
                narrowfib_1ddf31c88316e62f_state.add_inputs(inputs);
            });

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

pub struct ClaimProver {
    pub claim: Claim,
    pub lookup_data: LookupData,
}

#[allow(non_snake_case)]
pub struct LookupData {
    pub self_inputs: Vec<InputType>,
    pub self_outputs: Vec<OutputType>,
    pub narrowfib_1ddf31c88316e62f_inputs: [Vec<NarrowFib_1ddf31c88316e62f::InputType>; 8],
    pub narrowfib_1ddf31c88316e62f_outputs: [Vec<NarrowFib_1ddf31c88316e62f::OutputType>; 8],
}
impl LookupData {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            self_inputs: Vec::with_capacity(capacity),
            self_outputs: Vec::with_capacity(capacity),
            narrowfib_1ddf31c88316e62f_inputs: [
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
            ],
            narrowfib_1ddf31c88316e62f_outputs: [
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

pub fn write_trace_simd(
    inputs: Vec<InputType>,
) -> (
    Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>,
    LookupData,
) {
    let n_trace_columns = 17;
    let mut trace_values = (0..n_trace_columns)
        .map(|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES))
        .collect_vec();
    let mut sub_components_inputs = LookupData::with_capacity(inputs.len());
    inputs.into_iter().enumerate().for_each(|(i, input)| {
        write_trace_row(&mut trace_values, input, i, &mut sub_components_inputs);
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
    (trace, sub_components_inputs)
}
#[allow(clippy::useless_conversion)]
fn write_trace_row(
    #[allow(unused_variables)] dst: &mut [Col<SimdBackend, M31>],
    widefib_d7cf24d545e710f9_input: InputType,
    row_index: usize,
    lookup_data: &mut LookupData,
) {
    let col0 = widefib_d7cf24d545e710f9_input;
    dst[0].data[row_index] = col0;
    lookup_data.narrowfib_1ddf31c88316e62f_inputs[0]
        .push([PackedM31::broadcast(M31::from(1).into()), col0]);
    let tmp_1 = NarrowFib_1ddf31c88316e62f::deduce_output([
        PackedM31::broadcast(M31::from(1).into()),
        col0,
    ]);
    lookup_data.narrowfib_1ddf31c88316e62f_outputs[0].push(tmp_1);
    let col1 = tmp_1[0];
    dst[1].data[row_index] = col1;
    let col2 = tmp_1[1];
    dst[2].data[row_index] = col2;
    lookup_data.narrowfib_1ddf31c88316e62f_inputs[1].push([col1, col2]);
    let tmp_2 = NarrowFib_1ddf31c88316e62f::deduce_output([col1, col2]);
    lookup_data.narrowfib_1ddf31c88316e62f_outputs[1].push(tmp_2);
    let col3 = tmp_2[0];
    dst[3].data[row_index] = col3;
    let col4 = tmp_2[1];
    dst[4].data[row_index] = col4;
    lookup_data.narrowfib_1ddf31c88316e62f_inputs[2].push([col3, col4]);
    let tmp_3 = NarrowFib_1ddf31c88316e62f::deduce_output([col3, col4]);
    lookup_data.narrowfib_1ddf31c88316e62f_outputs[2].push(tmp_3);
    let col5 = tmp_3[0];
    dst[5].data[row_index] = col5;
    let col6 = tmp_3[1];
    dst[6].data[row_index] = col6;
    lookup_data.narrowfib_1ddf31c88316e62f_inputs[3].push([col5, col6]);
    let tmp_4 = NarrowFib_1ddf31c88316e62f::deduce_output([col5, col6]);
    lookup_data.narrowfib_1ddf31c88316e62f_outputs[3].push(tmp_4);
    let col7 = tmp_4[0];
    dst[7].data[row_index] = col7;
    let col8 = tmp_4[1];
    dst[8].data[row_index] = col8;
    lookup_data.narrowfib_1ddf31c88316e62f_inputs[4].push([col7, col8]);
    let tmp_5 = NarrowFib_1ddf31c88316e62f::deduce_output([col7, col8]);
    lookup_data.narrowfib_1ddf31c88316e62f_outputs[4].push(tmp_5);
    let col9 = tmp_5[0];
    dst[9].data[row_index] = col9;
    let col10 = tmp_5[1];
    dst[10].data[row_index] = col10;
    lookup_data.narrowfib_1ddf31c88316e62f_inputs[5].push([col9, col10]);
    let tmp_6 = NarrowFib_1ddf31c88316e62f::deduce_output([col9, col10]);
    lookup_data.narrowfib_1ddf31c88316e62f_outputs[5].push(tmp_6);
    let col11 = tmp_6[0];
    dst[11].data[row_index] = col11;
    let col12 = tmp_6[1];
    dst[12].data[row_index] = col12;
    lookup_data.narrowfib_1ddf31c88316e62f_inputs[6].push([col11, col12]);
    let tmp_7 = NarrowFib_1ddf31c88316e62f::deduce_output([col11, col12]);
    lookup_data.narrowfib_1ddf31c88316e62f_outputs[6].push(tmp_7);
    let col13 = tmp_7[0];
    dst[13].data[row_index] = col13;
    let col14 = tmp_7[1];
    dst[14].data[row_index] = col14;
    lookup_data.narrowfib_1ddf31c88316e62f_inputs[7].push([col13, col14]);
    let tmp_8 = NarrowFib_1ddf31c88316e62f::deduce_output([col13, col14]);
    lookup_data.narrowfib_1ddf31c88316e62f_outputs[7].push(tmp_8);
    let col15 = tmp_8[0];
    dst[15].data[row_index] = col15;
    let col16 = tmp_8[1];
    dst[16].data[row_index] = col16;

    lookup_data.self_inputs.push(widefib_d7cf24d545e710f9_input);
    lookup_data.self_outputs.push(col16);
}
