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

use super::component::BitUnpack__12;
use crate::code_gen::packed_types::*;

#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct BitUnpack__12SimdTraceGenerator {
    pub inputs: Vec<PackedUInt16>,
}
impl ComponentGen for BitUnpack__12SimdTraceGenerator {}

impl TraceGenerator<SimdBackend> for BitUnpack__12SimdTraceGenerator {
    type Component = BitUnpack__12;
    type Inputs = Vec<PackedUInt16>;

    fn write_trace(
        component_id: &str,
        registry: &mut ComponentGenerationRegistry,
    ) -> Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>> {
        let generator = registry.get_generator::<Self>(component_id);
        #[allow(unused_variables)]
        let (trace, sub_component_inputs) =
            write_trace_simd(&generator.component(), &generator.inputs);
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
                .expect("Input not a power of 2!")
                + LOG_N_LANES,
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
pub fn write_trace_simd(
    component: &BitUnpack__12,
    secrets: &Vec<PackedUInt16>,
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
    BitUnpack__12_input: PackedUInt16,
    row_index: usize,
    #[allow(unused_variables)] returned_inputs: &mut ReturnedInputs,
) {
    let col0 = BitUnpack__12_input.as_m31();
    dst[0][row_index] = col0;
    let deduction_tmp_2 =
        (BitUnpack__12_input) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col1 = deduction_tmp_2.as_m31();
    dst[1][row_index] = col1;
    let deduction_tmp_4 = (deduction_tmp_2) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col2 = deduction_tmp_4.as_m31();
    dst[2][row_index] = col2;
    let deduction_tmp_6 = (deduction_tmp_4) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col3 = deduction_tmp_6.as_m31();
    dst[3][row_index] = col3;
    let deduction_tmp_8 = (deduction_tmp_6) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col4 = deduction_tmp_8.as_m31();
    dst[4][row_index] = col4;
    let deduction_tmp_10 = (deduction_tmp_8) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col5 = deduction_tmp_10.as_m31();
    dst[5][row_index] = col5;
    let deduction_tmp_12 = (deduction_tmp_10) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col6 = deduction_tmp_12.as_m31();
    dst[6][row_index] = col6;
    let deduction_tmp_14 = (deduction_tmp_12) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col7 = deduction_tmp_14.as_m31();
    dst[7][row_index] = col7;
    let deduction_tmp_16 = (deduction_tmp_14) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col8 = deduction_tmp_16.as_m31();
    dst[8][row_index] = col8;
    let deduction_tmp_18 = (deduction_tmp_16) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col9 = deduction_tmp_18.as_m31();
    dst[9][row_index] = col9;
    let deduction_tmp_20 = (deduction_tmp_18) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col10 = deduction_tmp_20.as_m31();
    dst[10][row_index] = col10;
    let deduction_tmp_22 = (deduction_tmp_20) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col11 = deduction_tmp_22.as_m31();
    dst[11][row_index] = col11;
    let deduction_tmp_24 = (deduction_tmp_22) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col12 = deduction_tmp_24.as_m31();
    dst[12][row_index] = col12;
}
