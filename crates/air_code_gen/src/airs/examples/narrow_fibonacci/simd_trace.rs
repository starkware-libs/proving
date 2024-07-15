#![allow(unused_imports)]
use std::iter::zip;

use air_infra::core::prover_types::*;
use itertools::Itertools;
use num_traits::Zero;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::column::BaseFieldVec;
use stwo_prover::core::backend::simd::m31::PackedBaseField;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
use stwo_prover::trace_generation::{ComponentGen, TraceGenerator};

use super::component::NarrowFib__20;
use crate::code_gen::packed_types::*;

#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct NarrowFib__20SimdTraceGenerator {
    pub inputs: Vec<[PackedFelt; 2]>,
}
impl ComponentGen for NarrowFib__20SimdTraceGenerator {}

impl TraceGenerator<SimdBackend> for NarrowFib__20SimdTraceGenerator {
    type Component = NarrowFib__20;
    type Inputs = Vec<[PackedFelt; 2]>;

    fn write_trace(
        component_id: &str,
        registry: &mut ComponentGenerationRegistry,
    ) -> Vec<CircleEvaluation<SimdBackend, Felt, BitReversedOrder>> {
        let generator = registry.get_generator::<Self>(component_id);
        #[allow(unused_variables)]
        let (trace, sub_component_inputs) =
            write_trace_simd(&generator.component(), &generator.inputs);
        trace
    }

    fn add_inputs(&mut self, inputs: &Self::Inputs) {
        self.inputs.extend(inputs);
    }

    fn component(&self) -> NarrowFib__20 {
        NarrowFib__20 {
            log_n_instances: self
                .inputs
                .len()
                .checked_ilog2()
                .expect("Input not a power of 2!")
                + LOG_N_LANES,
        }
    }
}

pub struct ReturnedInputs();

impl ReturnedInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self()
    }
}

#[allow(clippy::ptr_arg)]
#[allow(clippy::type_complexity)]
#[allow(clippy::let_unit_value)]
pub fn write_trace_simd(
    component: &NarrowFib__20,
    secrets: &Vec<[PackedFelt; 2]>,
) -> (
    Vec<CircleEvaluation<SimdBackend, Felt, BitReversedOrder>>,
    ReturnedInputs,
) {
    let n_trace_columns = component.trace_log_degree_bounds()[0].len();
    let mut trace_values = vec![vec![PackedBaseField::zero(); secrets.len()]; n_trace_columns];
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
            CircleEvaluation::<SimdBackend, Felt, BitReversedOrder>::new(trace_domain, eval)
        })
        .collect_vec();

    (trace, sub_components_inputs)
}

#[allow(non_snake_case)]
#[allow(clippy::useless_conversion)]
#[allow(clippy::type_complexity)]
fn write_trace_row(
    dst: &mut [Vec<PackedBaseField>],
    NarrowFib__20_input: [PackedFelt; 2],
    row_index: usize,
    #[allow(unused_variables)] returned_inputs: &mut ReturnedInputs,
) {
    let deduction_tmp_0 = [NarrowFib__20_input[0], NarrowFib__20_input[1]];
    let col0 = deduction_tmp_0[0];
    dst[0][row_index] = col0;
    let col1 = deduction_tmp_0[1];
    dst[1][row_index] = col1;
    let col2 = ((col0) * (col0)) + ((col1) * (col1));
    dst[2][row_index] = col2;
    let col3 = ((col1) * (col1)) + ((col2) * (col2));
    dst[3][row_index] = col3;
    let col4 = ((col2) * (col2)) + ((col3) * (col3));
    dst[4][row_index] = col4;
    let col5 = ((col3) * (col3)) + ((col4) * (col4));
    dst[5][row_index] = col5;
    let col6 = ((col4) * (col4)) + ((col5) * (col5));
    dst[6][row_index] = col6;
    let col7 = ((col5) * (col5)) + ((col6) * (col6));
    dst[7][row_index] = col7;
    let col8 = ((col6) * (col6)) + ((col7) * (col7));
    dst[8][row_index] = col8;
    let col9 = ((col7) * (col7)) + ((col8) * (col8));
    dst[9][row_index] = col9;
    let col10 = ((col8) * (col8)) + ((col9) * (col9));
    dst[10][row_index] = col10;
    let col11 = ((col9) * (col9)) + ((col10) * (col10));
    dst[11][row_index] = col11;
    let col12 = ((col10) * (col10)) + ((col11) * (col11));
    dst[12][row_index] = col12;
    let col13 = ((col11) * (col11)) + ((col12) * (col12));
    dst[13][row_index] = col13;
    let col14 = ((col12) * (col12)) + ((col13) * (col13));
    dst[14][row_index] = col14;
    let col15 = ((col13) * (col13)) + ((col14) * (col14));
    dst[15][row_index] = col15;
    let col16 = ((col14) * (col14)) + ((col15) * (col15));
    dst[16][row_index] = col16;
    let col17 = ((col15) * (col15)) + ((col16) * (col16));
    dst[17][row_index] = col17;
    let col18 = ((col16) * (col16)) + ((col17) * (col17));
    dst[18][row_index] = col18;
    let col19 = ((col17) * (col17)) + ((col18) * (col18));
    dst[19][row_index] = col19;
    let col20 = ((col18) * (col18)) + ((col19) * (col19));
    dst[20][row_index] = col20;
    let col21 = ((col19) * (col19)) + ((col20) * (col20));
    dst[21][row_index] = col21;
}
