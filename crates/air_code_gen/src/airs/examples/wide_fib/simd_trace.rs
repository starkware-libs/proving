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

use super::component::WideFib__8;
use crate::airs::examples::narrow_fibonacci::simd_trace::NarrowFib__20SimdTraceGenerator;
use crate::airs::examples::narrow_fibonacci::trace::NarrowFib__20CpuTraceGenerator;
use crate::code_gen::packed_types::*;

#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct WideFib__8SimdTraceGenerator {
    pub inputs: Vec<PackedM31>,
}
impl ComponentGen for WideFib__8SimdTraceGenerator {}

impl TraceGenerator<SimdBackend> for WideFib__8SimdTraceGenerator {
    type Component = WideFib__8;
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
            .get_generator_mut::<NarrowFib__20SimdTraceGenerator>("NarrowFib__20")
            .add_inputs(&sub_component_inputs.NarrowFib__20_inputs);
        trace
    }

    fn add_inputs(&mut self, inputs: &Self::Inputs) {
        self.inputs.extend(inputs);
    }

    fn component(&self) -> WideFib__8 {
        WideFib__8 {
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
    pub NarrowFib__20_inputs: Vec<[PackedM31; 2]>,
}

impl ReturnedInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            NarrowFib__20_inputs: Vec::with_capacity(capacity * 8),
        }
    }
}

#[allow(clippy::ptr_arg)]
#[allow(clippy::type_complexity)]
#[allow(clippy::let_unit_value)]
pub fn write_trace_simd(
    component: &WideFib__8,
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
    WideFib_input: PackedM31,
    row_index: usize,
    #[allow(unused_variables)] returned_inputs: &mut ReturnedInputs,
) {
    let col0 = WideFib_input;
    dst[0][row_index] = col0;
    returned_inputs
        .NarrowFib__20_inputs
        .push([PackedM31::broadcast(M31::from(1).into()), col0]);
    let deduction_tmp_1 = NarrowFib__20SimdTraceGenerator::deduce_output([
        PackedM31::broadcast(M31::from(1).into()),
        col0,
    ]);
    let col1 = deduction_tmp_1[0];
    dst[1][row_index] = col1;
    let col2 = deduction_tmp_1[1];
    dst[2][row_index] = col2;
    returned_inputs.NarrowFib__20_inputs.push([col1, col2]);
    let deduction_tmp_2 = NarrowFib__20SimdTraceGenerator::deduce_output([col1, col2]);
    let col3 = deduction_tmp_2[0];
    dst[3][row_index] = col3;
    let col4 = deduction_tmp_2[1];
    dst[4][row_index] = col4;
    returned_inputs.NarrowFib__20_inputs.push([col3, col4]);
    let deduction_tmp_3 = NarrowFib__20SimdTraceGenerator::deduce_output([col3, col4]);
    let col5 = deduction_tmp_3[0];
    dst[5][row_index] = col5;
    let col6 = deduction_tmp_3[1];
    dst[6][row_index] = col6;
    returned_inputs.NarrowFib__20_inputs.push([col5, col6]);
    let deduction_tmp_4 = NarrowFib__20SimdTraceGenerator::deduce_output([col5, col6]);
    let col7 = deduction_tmp_4[0];
    dst[7][row_index] = col7;
    let col8 = deduction_tmp_4[1];
    dst[8][row_index] = col8;
    returned_inputs.NarrowFib__20_inputs.push([col7, col8]);
    let deduction_tmp_5 = NarrowFib__20SimdTraceGenerator::deduce_output([col7, col8]);
    let col9 = deduction_tmp_5[0];
    dst[9][row_index] = col9;
    let col10 = deduction_tmp_5[1];
    dst[10][row_index] = col10;
    returned_inputs.NarrowFib__20_inputs.push([col9, col10]);
    let deduction_tmp_6 = NarrowFib__20SimdTraceGenerator::deduce_output([col9, col10]);
    let col11 = deduction_tmp_6[0];
    dst[11][row_index] = col11;
    let col12 = deduction_tmp_6[1];
    dst[12][row_index] = col12;
    returned_inputs.NarrowFib__20_inputs.push([col11, col12]);
    let deduction_tmp_7 = NarrowFib__20SimdTraceGenerator::deduce_output([col11, col12]);
    let col13 = deduction_tmp_7[0];
    dst[13][row_index] = col13;
    let col14 = deduction_tmp_7[1];
    dst[14][row_index] = col14;
    returned_inputs.NarrowFib__20_inputs.push([col13, col14]);
    let deduction_tmp_8 = NarrowFib__20SimdTraceGenerator::deduce_output([col13, col14]);
    let col15 = deduction_tmp_8[0];
    dst[15][row_index] = col15;
    let col16 = deduction_tmp_8[1];
    dst[16][row_index] = col16;
}
