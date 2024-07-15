#![allow(unused_imports)]
use air_infra::core::prover_types::*;
use itertools::Itertools;
use num_traits::Zero;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
use stwo_prover::core::backend::CpuBackend;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
use stwo_prover::trace_generation::{ComponentGen, TraceGenerator};

use super::component::WideFib__8;
use crate::airs::examples::narrow_fibonacci::simd_trace::NarrowFib__20SimdTraceGenerator;
use crate::airs::examples::narrow_fibonacci::trace::NarrowFib__20CpuTraceGenerator;

#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct WideFib__8CpuTraceGenerator {
    pub inputs: Vec<Felt>,
}
impl ComponentGen for WideFib__8CpuTraceGenerator {}

impl TraceGenerator<CpuBackend> for WideFib__8CpuTraceGenerator {
    type Component = WideFib__8;
    type Inputs = Vec<Felt>;

    fn write_trace(
        component_id: &str,
        registry: &mut ComponentGenerationRegistry,
    ) -> Vec<CpuCircleEvaluation<Felt, BitReversedOrder>> {
        let generator = registry.get_generator::<Self>(component_id);
        #[allow(unused_variables)]
        let (trace, sub_component_inputs) =
            write_trace_cpu(&generator.component(), &generator.inputs);
        let sub_component_i =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("NarrowFib__20");
        sub_component_i.add_inputs(&sub_component_inputs.0);
        let sub_component_i =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("NarrowFib__20");
        sub_component_i.add_inputs(&sub_component_inputs.1);
        let sub_component_i =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("NarrowFib__20");
        sub_component_i.add_inputs(&sub_component_inputs.2);
        let sub_component_i =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("NarrowFib__20");
        sub_component_i.add_inputs(&sub_component_inputs.3);
        let sub_component_i =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("NarrowFib__20");
        sub_component_i.add_inputs(&sub_component_inputs.4);
        let sub_component_i =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("NarrowFib__20");
        sub_component_i.add_inputs(&sub_component_inputs.5);
        let sub_component_i =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("NarrowFib__20");
        sub_component_i.add_inputs(&sub_component_inputs.6);
        let sub_component_i =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("NarrowFib__20");
        sub_component_i.add_inputs(&sub_component_inputs.7);
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
                .expect("Input not a power of 2!"),
        }
    }
}

pub struct ReturnedInputs(
    pub Vec<[Felt; 2]>,
    pub Vec<[Felt; 2]>,
    pub Vec<[Felt; 2]>,
    pub Vec<[Felt; 2]>,
    pub Vec<[Felt; 2]>,
    pub Vec<[Felt; 2]>,
    pub Vec<[Felt; 2]>,
    pub Vec<[Felt; 2]>,
);

impl ReturnedInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self(
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
        )
    }
}

#[allow(clippy::ptr_arg)]
#[allow(clippy::type_complexity)]
#[allow(clippy::let_unit_value)]
pub fn write_trace_cpu(
    component: &WideFib__8,
    secrets: &Vec<Felt>,
) -> (
    Vec<CpuCircleEvaluation<BaseField, BitReversedOrder>>,
    ReturnedInputs,
) {
    let n_trace_columns = component.trace_log_degree_bounds()[0].len();
    let mut trace_values = vec![vec![BaseField::zero(); secrets.len()]; n_trace_columns];
    let mut sub_components_inputs = ReturnedInputs::with_capacity(secrets.len());
    secrets.iter().enumerate().for_each(|(i, secret)| {
        write_trace_row(&mut trace_values, *secret, i, &mut sub_components_inputs)
    });

    let trace = trace_values
        .into_iter()
        .map(|eval| {
            let domain =
                CanonicCoset::new(eval.len().checked_ilog2().expect("Input not a power of 2!"))
                    .circle_domain();
            CpuCircleEvaluation::<BaseField, BitReversedOrder>::new(domain, eval)
        })
        .collect_vec();

    (trace, sub_components_inputs)
}

#[allow(non_snake_case)]
#[allow(clippy::useless_conversion)]
#[allow(clippy::type_complexity)]
fn write_trace_row(
    dst: &mut [Vec<BaseField>],
    WideFib_input: Felt,
    row_index: usize,
    #[allow(unused_variables)] returned_inputs: &mut ReturnedInputs,
) {
    let col0 = WideFib_input;
    dst[0][row_index] = col0.into();
    returned_inputs.0.push([Felt::from(1), col0]);
    let deduction_tmp_1 = NarrowFib__20CpuTraceGenerator::deduce_output([Felt::from(1), col0]);
    let col1 = deduction_tmp_1[0];
    dst[1][row_index] = col1.into();
    let col2 = deduction_tmp_1[1];
    dst[2][row_index] = col2.into();
    returned_inputs.1.push([col1, col2]);
    let deduction_tmp_2 = NarrowFib__20CpuTraceGenerator::deduce_output([col1, col2]);
    let col3 = deduction_tmp_2[0];
    dst[3][row_index] = col3.into();
    let col4 = deduction_tmp_2[1];
    dst[4][row_index] = col4.into();
    returned_inputs.2.push([col3, col4]);
    let deduction_tmp_3 = NarrowFib__20CpuTraceGenerator::deduce_output([col3, col4]);
    let col5 = deduction_tmp_3[0];
    dst[5][row_index] = col5.into();
    let col6 = deduction_tmp_3[1];
    dst[6][row_index] = col6.into();
    returned_inputs.3.push([col5, col6]);
    let deduction_tmp_4 = NarrowFib__20CpuTraceGenerator::deduce_output([col5, col6]);
    let col7 = deduction_tmp_4[0];
    dst[7][row_index] = col7.into();
    let col8 = deduction_tmp_4[1];
    dst[8][row_index] = col8.into();
    returned_inputs.4.push([col7, col8]);
    let deduction_tmp_5 = NarrowFib__20CpuTraceGenerator::deduce_output([col7, col8]);
    let col9 = deduction_tmp_5[0];
    dst[9][row_index] = col9.into();
    let col10 = deduction_tmp_5[1];
    dst[10][row_index] = col10.into();
    returned_inputs.5.push([col9, col10]);
    let deduction_tmp_6 = NarrowFib__20CpuTraceGenerator::deduce_output([col9, col10]);
    let col11 = deduction_tmp_6[0];
    dst[11][row_index] = col11.into();
    let col12 = deduction_tmp_6[1];
    dst[12][row_index] = col12.into();
    returned_inputs.6.push([col11, col12]);
    let deduction_tmp_7 = NarrowFib__20CpuTraceGenerator::deduce_output([col11, col12]);
    let col13 = deduction_tmp_7[0];
    dst[13][row_index] = col13.into();
    let col14 = deduction_tmp_7[1];
    dst[14][row_index] = col14.into();
    returned_inputs.7.push([col13, col14]);
    let deduction_tmp_8 = NarrowFib__20CpuTraceGenerator::deduce_output([col13, col14]);
    let col15 = deduction_tmp_8[0];
    dst[15][row_index] = col15.into();
    let col16 = deduction_tmp_8[1];
    dst[16][row_index] = col16.into();
}
