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

use super::component::NarrowFib__20;

#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct NarrowFib__20CpuTraceGenerator {
    pub inputs: Vec<[Felt; 2]>,
}
impl ComponentGen for NarrowFib__20CpuTraceGenerator {}

impl TraceGenerator<CpuBackend> for NarrowFib__20CpuTraceGenerator {
    type Component = NarrowFib__20;
    type Inputs = Vec<[Felt; 2]>;

    fn write_trace(
        component_id: &str,
        registry: &mut ComponentGenerationRegistry,
    ) -> Vec<CpuCircleEvaluation<Felt, BitReversedOrder>> {
        let generator = registry.get_generator::<NarrowFib__20CpuTraceGenerator>(component_id);
        write_trace_cpu(&generator.component(), &generator.inputs)
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
                .expect("Input not a power of 2!"),
        }
    }
}

#[allow(clippy::ptr_arg)]
pub fn write_trace_cpu(
    component: &NarrowFib__20,
    secrets: &Vec<[Felt; 2]>,
) -> Vec<CpuCircleEvaluation<BaseField, BitReversedOrder>> {
    let n_trace_columns = component.trace_log_degree_bounds()[0].len();
    let mut trace_values = vec![vec![BaseField::zero(); secrets.len()]; n_trace_columns];
    for (i, secret) in secrets.iter().enumerate() {
        write_trace_row(&mut trace_values, *secret, i);
    }

    let trace_domains = trace_values
        .iter()
        .map(|col| {
            CanonicCoset::new(col.len().checked_ilog2().expect("Input not a power of 2!"))
                .circle_domain()
        })
        .collect_vec();
    std::iter::zip(trace_values, trace_domains)
        .map(|(eval, trace_domain)| {
            CpuCircleEvaluation::<BaseField, BitReversedOrder>::new(trace_domain, eval)
        })
        .collect_vec()
}

#[allow(non_snake_case)]
#[allow(clippy::useless_conversion)]
fn write_trace_row(dst: &mut [Vec<BaseField>], NarrowFib__20_input: [Felt; 2], row_index: usize) {
    let deduction_tmp_0 = [NarrowFib__20_input[0], NarrowFib__20_input[1]];
    let col0 = deduction_tmp_0[0];
    dst[0][row_index] = col0.into();
    let col1 = deduction_tmp_0[1];
    dst[1][row_index] = col1.into();
    let col2 = ((col0) * (col0)) + ((col1) * (col1));
    dst[2][row_index] = col2.into();
    let col3 = ((col1) * (col1)) + ((col2) * (col2));
    dst[3][row_index] = col3.into();
    let col4 = ((col2) * (col2)) + ((col3) * (col3));
    dst[4][row_index] = col4.into();
    let col5 = ((col3) * (col3)) + ((col4) * (col4));
    dst[5][row_index] = col5.into();
    let col6 = ((col4) * (col4)) + ((col5) * (col5));
    dst[6][row_index] = col6.into();
    let col7 = ((col5) * (col5)) + ((col6) * (col6));
    dst[7][row_index] = col7.into();
    let col8 = ((col6) * (col6)) + ((col7) * (col7));
    dst[8][row_index] = col8.into();
    let col9 = ((col7) * (col7)) + ((col8) * (col8));
    dst[9][row_index] = col9.into();
    let col10 = ((col8) * (col8)) + ((col9) * (col9));
    dst[10][row_index] = col10.into();
    let col11 = ((col9) * (col9)) + ((col10) * (col10));
    dst[11][row_index] = col11.into();
    let col12 = ((col10) * (col10)) + ((col11) * (col11));
    dst[12][row_index] = col12.into();
    let col13 = ((col11) * (col11)) + ((col12) * (col12));
    dst[13][row_index] = col13.into();
    let col14 = ((col12) * (col12)) + ((col13) * (col13));
    dst[14][row_index] = col14.into();
    let col15 = ((col13) * (col13)) + ((col14) * (col14));
    dst[15][row_index] = col15.into();
    let col16 = ((col14) * (col14)) + ((col15) * (col15));
    dst[16][row_index] = col16.into();
    let col17 = ((col15) * (col15)) + ((col16) * (col16));
    dst[17][row_index] = col17.into();
    let col18 = ((col16) * (col16)) + ((col17) * (col17));
    dst[18][row_index] = col18.into();
    let col19 = ((col17) * (col17)) + ((col18) * (col18));
    dst[19][row_index] = col19.into();
    let col20 = ((col18) * (col18)) + ((col19) * (col19));
    dst[20][row_index] = col20.into();
    let col21 = ((col19) * (col19)) + ((col20) * (col20));
    dst[21][row_index] = col21.into();
}
