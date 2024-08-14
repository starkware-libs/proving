#![allow(unused_imports)]
use std::iter::zip;

use air_infra::core::prover_types::*;
use itertools::Itertools;
use num_traits::Zero;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::column::BaseFieldVec;
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
use stwo_prover::trace_generation::{ComponentGen, TraceGenerator};

use super::component::WideFib_d7cf24d545e710f9;
use crate::airs::examples::NarrowFib_1ddf31c88316e62fSimdTraceGenerator;
use crate::code_gen::packed_types::*;

#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct WideFib_d7cf24d545e710f9SimdTraceGenerator {
    pub inputs: Vec<PackedM31>,
}
impl ComponentGen for WideFib_d7cf24d545e710f9SimdTraceGenerator {}

impl TraceGenerator<SimdBackend> for WideFib_d7cf24d545e710f9SimdTraceGenerator {
    type Component = WideFib_d7cf24d545e710f9;
    type Inputs = Vec<PackedM31>;

    fn write_trace(
        component_id: &str,
        registry: &mut ComponentGenerationRegistry,
    ) -> Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>> {
        let generator = registry.get_generator::<Self>(component_id);
        #[allow(unused_variables)]
        let (trace, sub_component_inputs) =
            write_trace_simd(&generator.component(), &generator.inputs);
        registry
            .get_generator_mut::<NarrowFib_1ddf31c88316e62fSimdTraceGenerator>(
                "NarrowFib_1ddf31c88316e62f",
            )
            .add_inputs(&sub_component_inputs.NarrowFib_1ddf31c88316e62f_inputs);
        trace
    }

    fn add_inputs(&mut self, inputs: &Self::Inputs) {
        self.inputs.extend(inputs);
    }

    fn component(&self) -> WideFib_d7cf24d545e710f9 {
        WideFib_d7cf24d545e710f9 {
            log_n_instances: self
                .inputs
                .len()
                .checked_ilog2()
                .expect("Input not a power of 2!")
                + LOG_N_LANES,
        }
    }
}

#[allow(non_snake_case)]
pub struct ReturnedInputs {
    pub NarrowFib_1ddf31c88316e62f_inputs: Vec<[PackedM31; 2]>,
}

impl ReturnedInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            NarrowFib_1ddf31c88316e62f_inputs: Vec::with_capacity(capacity * 8),
        }
    }
}

#[allow(clippy::ptr_arg)]
#[allow(clippy::type_complexity)]
#[allow(clippy::let_unit_value)]
pub fn write_trace_simd(
    component: &WideFib_d7cf24d545e710f9,
    secrets: &Vec<PackedM31>,
) -> (
    Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>,
    ReturnedInputs,
) {
    let n_trace_columns = component.trace_log_degree_bounds()[0].len();
    let mut trace_values = vec![vec![PackedM31::zero(); secrets.len()]; n_trace_columns];
    let mut sub_components_inputs = ReturnedInputs::with_capacity(secrets.len());
    secrets.iter().enumerate().for_each(|(i, secret)| {
        write_trace_row(&mut trace_values, *secret, i, &mut sub_components_inputs)
    });

    let trace = trace_values
        .into_iter()
        .map(|eval| {
            let length = eval.len() * N_LANES;
            let eval = BaseFieldVec { data: eval, length };

            let trace_domain =
                CanonicCoset::new(length.checked_ilog2().expect("Input not a power of 2!"))
                    .circle_domain();
            CircleEvaluation::<SimdBackend, M31, BitReversedOrder>::new(trace_domain, eval)
        })
        .collect_vec();

    (trace, sub_components_inputs)
}

#[allow(non_snake_case)]
#[allow(clippy::useless_conversion)]
#[allow(clippy::type_complexity)]
fn write_trace_row(
    dst: &mut [Vec<PackedM31>],
    WideFib_d7cf24d545e710f9_input: PackedM31,
    row_index: usize,
    #[allow(unused_variables)] returned_inputs: &mut ReturnedInputs,
) {
    let col0 = WideFib_d7cf24d545e710f9_input;
    dst[0][row_index] = col0;
    returned_inputs
        .NarrowFib_1ddf31c88316e62f_inputs
        .push([PackedM31::broadcast(M31::from(1).into()), col0]);
    let tmp_1 = NarrowFib_1ddf31c88316e62fSimdTraceGenerator::deduce_output([
        PackedM31::broadcast(M31::from(1).into()),
        col0,
    ]);
    let col1 = tmp_1[0];
    dst[1][row_index] = col1;
    let col2 = tmp_1[1];
    dst[2][row_index] = col2;
    returned_inputs
        .NarrowFib_1ddf31c88316e62f_inputs
        .push([col1, col2]);
    let tmp_2 = NarrowFib_1ddf31c88316e62fSimdTraceGenerator::deduce_output([col1, col2]);
    let col3 = tmp_2[0];
    dst[3][row_index] = col3;
    let col4 = tmp_2[1];
    dst[4][row_index] = col4;
    returned_inputs
        .NarrowFib_1ddf31c88316e62f_inputs
        .push([col3, col4]);
    let tmp_3 = NarrowFib_1ddf31c88316e62fSimdTraceGenerator::deduce_output([col3, col4]);
    let col5 = tmp_3[0];
    dst[5][row_index] = col5;
    let col6 = tmp_3[1];
    dst[6][row_index] = col6;
    returned_inputs
        .NarrowFib_1ddf31c88316e62f_inputs
        .push([col5, col6]);
    let tmp_4 = NarrowFib_1ddf31c88316e62fSimdTraceGenerator::deduce_output([col5, col6]);
    let col7 = tmp_4[0];
    dst[7][row_index] = col7;
    let col8 = tmp_4[1];
    dst[8][row_index] = col8;
    returned_inputs
        .NarrowFib_1ddf31c88316e62f_inputs
        .push([col7, col8]);
    let tmp_5 = NarrowFib_1ddf31c88316e62fSimdTraceGenerator::deduce_output([col7, col8]);
    let col9 = tmp_5[0];
    dst[9][row_index] = col9;
    let col10 = tmp_5[1];
    dst[10][row_index] = col10;
    returned_inputs
        .NarrowFib_1ddf31c88316e62f_inputs
        .push([col9, col10]);
    let tmp_6 = NarrowFib_1ddf31c88316e62fSimdTraceGenerator::deduce_output([col9, col10]);
    let col11 = tmp_6[0];
    dst[11][row_index] = col11;
    let col12 = tmp_6[1];
    dst[12][row_index] = col12;
    returned_inputs
        .NarrowFib_1ddf31c88316e62f_inputs
        .push([col11, col12]);
    let tmp_7 = NarrowFib_1ddf31c88316e62fSimdTraceGenerator::deduce_output([col11, col12]);
    let col13 = tmp_7[0];
    dst[13][row_index] = col13;
    let col14 = tmp_7[1];
    dst[14][row_index] = col14;
    returned_inputs
        .NarrowFib_1ddf31c88316e62f_inputs
        .push([col13, col14]);
    let tmp_8 = NarrowFib_1ddf31c88316e62fSimdTraceGenerator::deduce_output([col13, col14]);
    let col15 = tmp_8[0];
    dst[15][row_index] = col15;
    let col16 = tmp_8[1];
    dst[16][row_index] = col16;
}
