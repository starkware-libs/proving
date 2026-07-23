#![allow(unused_parens)]
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::qm31::QM31;
use stwo_cairo_common::prover_types::simd::EqExtend;

use crate::circuit_air::components::m_31_to_u_32::{Claim, InteractionClaim, N_TRACE_COLUMNS};
use crate::witness::components::prelude::*;
use crate::witness::components::range_check_16;

pub type InputType = (M31, UInt32);
pub type PackedInputType = (PackedM31, PackedUInt32);

/// Retrieves the component's inputs from the context values, using the addresses provided in the
/// preprocessed trace.
#[allow(clippy::uninit_vec)]
pub fn extract_component_inputs(
    input_addr_col: &[usize],
    context_values: &[QM31],
) -> Vec<InputType> {
    let n_rows = input_addr_col.len();
    let mut inputs = Vec::with_capacity(n_rows);
    unsafe {
        inputs.set_len(n_rows);
    }

    (inputs.par_iter_mut(), input_addr_col).into_par_iter().for_each(|(input, &input_addr)| {
        let m31_input = context_values[input_addr].to_m31_array()[0];
        *input = (m31_input, UInt32::from(m31_input.0));
    });

    inputs
}

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: &PreProcessedTrace,
    range_check_16_state: &range_check_16::ClaimGenerator,
) -> (ComponentTrace<N_TRACE_COLUMNS>, Claim, InteractionClaimGenerator) {
    let input_addr_col = preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "m31_to_u32_input_addr".to_owned() });
    let output_addr_col = preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "m31_to_u32_output_addr".to_owned() });
    let multiplicity_col = preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "m31_to_u32_multiplicity".to_owned() });

    let inputs = extract_component_inputs(input_addr_col, context_values);

    let n_rows = inputs.len();
    assert_ne!(n_rows, 0);
    let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
    let log_size = size.ilog2();

    let packed_inputs = pack_values(&inputs);

    let preprocessed_columns = [input_addr_col, output_addr_col, multiplicity_col]
        .into_iter()
        .map(|col| Col::<SimdBackend, M31>::from_iter(col.iter().map(|&x| M31::from(x))).data)
        .collect_vec();

    let (trace, lookup_data, sub_component_inputs) =
        write_trace_simd(packed_inputs, preprocessed_columns);
    for inputs in sub_component_inputs.range_check_16 {
        range_check_16_state.add_packed_inputs(&inputs, 0);
    }

    (trace, Claim { log_size }, InteractionClaimGenerator { log_size, lookup_data })
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct SubComponentInputs {
    range_check_16: [Vec<range_check_16::PackedInputType>; 3],
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(
    inputs: Vec<PackedInputType>,
    preprocessed_columns: Vec<Vec<PackedM31>>,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData, SubComponentInputs) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data, mut sub_component_inputs) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
            SubComponentInputs::uninitialized(log_n_packed_rows),
        )
    };

    let M31_0 = PackedM31::broadcast(M31::from(0));
    let M31_1 = PackedM31::broadcast(M31::from(1));
    let M31_1008385708 = PackedM31::broadcast(M31::from(1008385708));
    let M31_32767 = PackedM31::broadcast(M31::from(32767));
    let M31_378353459 = PackedM31::broadcast(M31::from(378353459));
    let [m31_to_u32_input_addr, m31_to_u32_output_addr, m31_to_u32_multiplicity]: [Vec<PackedM31>;
        3] = preprocessed_columns.try_into().unwrap();

    (
        trace.par_iter_mut(),
        lookup_data.par_iter_mut(),
        sub_component_inputs.par_iter_mut(),
        inputs.into_par_iter(),
    )
        .into_par_iter()
        .enumerate()
        .for_each(
            |(row_index, (row, lookup_data, sub_component_inputs, m_31_to_u_32_input))| {
                let m31_to_u32_input_addr = m31_to_u32_input_addr[row_index];
                let m31_to_u32_output_addr = m31_to_u32_output_addr[row_index];
                let m31_to_u32_multiplicity = m31_to_u32_multiplicity[row_index];
                let input_m31_col0 = m_31_to_u_32_input.0;
                *row[0] = input_m31_col0;
                let input_u32_limb_0_col1 = m_31_to_u_32_input.1.low().as_m31();
                *row[1] = input_u32_limb_0_col1;
                let input_u32_limb_1_col2 = m_31_to_u_32_input.1.high().as_m31();
                *row[2] = input_u32_limb_1_col2;
                *sub_component_inputs.range_check_16[0] = [input_u32_limb_0_col1];
                *lookup_data.range_check_16_0 = [M31_1008385708, input_u32_limb_0_col1];
                *sub_component_inputs.range_check_16[1] = [input_u32_limb_1_col2];
                *lookup_data.range_check_16_1 = [M31_1008385708, input_u32_limb_1_col2];
                *sub_component_inputs.range_check_16[2] = [((M31_32767) - (input_u32_limb_1_col2))];
                *lookup_data.range_check_16_2 =
                    [M31_1008385708, ((M31_32767) - (input_u32_limb_1_col2))];
                let input_is_zero_tmp_c1f7a_0 = M31_0.eq(input_m31_col0);
                let inv_or_one_col3 =
                    ((input_is_zero_tmp_c1f7a_0.as_m31()) + (input_m31_col0)).inverse();
                *row[3] = inv_or_one_col3;
                *lookup_data.gate_0 = [M31_378353459, m31_to_u32_input_addr, input_m31_col0];
                *lookup_data.gate_1 = [
                    M31_378353459,
                    m31_to_u32_output_addr,
                    input_u32_limb_0_col1,
                    input_u32_limb_1_col2,
                ];
                *lookup_data.mults_0 = m31_to_u32_multiplicity;
            },
        );

    (trace, lookup_data, sub_component_inputs)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    gate_0: Vec<[PackedM31; 3]>,
    gate_1: Vec<[PackedM31; 4]>,
    range_check_16_0: Vec<[PackedM31; 2]>,
    range_check_16_1: Vec<[PackedM31; 2]>,
    range_check_16_2: Vec<[PackedM31; 2]>,
    mults_0: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        _common_lookup_elements: &relations::CommonLookupElements,
    ) -> (Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>, InteractionClaim) {
        if self.log_size == 0 {
            return (vec![], InteractionClaim { claimed_sum: QM31::zero() });
        }
        let common_lookup_elements = _common_lookup_elements;
        let mut logup_gen = unsafe { LogupTraceGenerator::uninitialized(self.log_size) };

        // Sum logup terms in pairs.
        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_16_0,
            &self.lookup_data.range_check_16_1,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.range_check_16_2, &self.lookup_data.gate_0)
            .into_par_iter()
            .for_each(|(writer, values0, gate_values)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(gate_values);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.gate_1, &self.lookup_data.mults_0)
            .into_par_iter()
            .for_each(|(writer, values, &mults_0)| {
                let denom = common_lookup_elements.combine(values);
                writer.write_frac(-PackedQM31::one() * mults_0, denom);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();

        (trace, InteractionClaim { claimed_sum })
    }
}
