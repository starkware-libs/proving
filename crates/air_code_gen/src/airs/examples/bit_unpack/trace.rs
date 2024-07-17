#![allow(unused_imports)]
use air_infra::core::prover_types::*;
use itertools::Itertools;
use num_traits::Zero;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
use stwo_prover::core::backend::CpuBackend;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
use stwo_prover::trace_generation::{ComponentGen, TraceGenerator};

use super::component::BitUnpack__12;

#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct BitUnpack__12CpuTraceGenerator {
    pub inputs: Vec<UInt16>,
}
impl ComponentGen for BitUnpack__12CpuTraceGenerator {}

impl TraceGenerator<CpuBackend> for BitUnpack__12CpuTraceGenerator {
    type Component = BitUnpack__12;
    type Inputs = Vec<UInt16>;

    fn write_trace(
        component_id: &str,
        registry: &mut ComponentGenerationRegistry,
    ) -> Vec<CpuCircleEvaluation<M31, BitReversedOrder>> {
        let generator = registry.get_generator::<Self>(component_id);
        #[allow(unused_variables)]
        let (trace, sub_component_inputs) =
            write_trace_cpu(&generator.component(), &generator.inputs);
        trace
    }

    fn add_inputs(&mut self, inputs: &Self::Inputs) {
        self.inputs.extend(inputs);
    }

    fn component(&self) -> BitUnpack__12 {
        BitUnpack__12 {
            log_n_instances: self
                .inputs
                .len()
                .checked_ilog2()
                .expect("Input not a power of 2!"),
        }
    }
}

#[allow(non_snake_case)]
pub struct ReturnedInputs {}

impl ReturnedInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {}
    }
}

#[allow(clippy::ptr_arg)]
#[allow(clippy::type_complexity)]
#[allow(clippy::let_unit_value)]
pub fn write_trace_cpu(
    component: &BitUnpack__12,
    secrets: &Vec<UInt16>,
) -> (
    Vec<CpuCircleEvaluation<M31, BitReversedOrder>>,
    ReturnedInputs,
) {
    let n_trace_columns = component.trace_log_degree_bounds()[0].len();
    let mut trace_values = vec![vec![M31::zero(); secrets.len()]; n_trace_columns];
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
            CpuCircleEvaluation::<M31, BitReversedOrder>::new(domain, eval)
        })
        .collect_vec();

    (trace, sub_components_inputs)
}

#[allow(non_snake_case)]
#[allow(clippy::useless_conversion)]
#[allow(clippy::type_complexity)]
fn write_trace_row(
    dst: &mut [Vec<M31>],
    BitUnpack__12_input: UInt16,
    row_index: usize,
    #[allow(unused_variables)] returned_inputs: &mut ReturnedInputs,
) {
    let col0 = BitUnpack__12_input.as_m31();
    dst[0][row_index] = col0.into();
    let deduction_tmp_2 = (BitUnpack__12_input) >> (UInt16::from(1));
    let col1 = deduction_tmp_2.as_m31();
    dst[1][row_index] = col1.into();
    let deduction_tmp_4 = (deduction_tmp_2) >> (UInt16::from(1));
    let col2 = deduction_tmp_4.as_m31();
    dst[2][row_index] = col2.into();
    let deduction_tmp_6 = (deduction_tmp_4) >> (UInt16::from(1));
    let col3 = deduction_tmp_6.as_m31();
    dst[3][row_index] = col3.into();
    let deduction_tmp_8 = (deduction_tmp_6) >> (UInt16::from(1));
    let col4 = deduction_tmp_8.as_m31();
    dst[4][row_index] = col4.into();
    let deduction_tmp_10 = (deduction_tmp_8) >> (UInt16::from(1));
    let col5 = deduction_tmp_10.as_m31();
    dst[5][row_index] = col5.into();
    let deduction_tmp_12 = (deduction_tmp_10) >> (UInt16::from(1));
    let col6 = deduction_tmp_12.as_m31();
    dst[6][row_index] = col6.into();
    let deduction_tmp_14 = (deduction_tmp_12) >> (UInt16::from(1));
    let col7 = deduction_tmp_14.as_m31();
    dst[7][row_index] = col7.into();
    let deduction_tmp_16 = (deduction_tmp_14) >> (UInt16::from(1));
    let col8 = deduction_tmp_16.as_m31();
    dst[8][row_index] = col8.into();
    let deduction_tmp_18 = (deduction_tmp_16) >> (UInt16::from(1));
    let col9 = deduction_tmp_18.as_m31();
    dst[9][row_index] = col9.into();
    let deduction_tmp_20 = (deduction_tmp_18) >> (UInt16::from(1));
    let col10 = deduction_tmp_20.as_m31();
    dst[10][row_index] = col10.into();
    let deduction_tmp_22 = (deduction_tmp_20) >> (UInt16::from(1));
    let col11 = deduction_tmp_22.as_m31();
    dst[11][row_index] = col11.into();
    let deduction_tmp_24 = (deduction_tmp_22) >> (UInt16::from(1));
    let col12 = deduction_tmp_24.as_m31();
    dst[12][row_index] = col12.into();
}
