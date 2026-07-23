#![allow(unused_parens)]
#![allow(clippy::too_many_arguments)]
use crate::circuit_air::components::triple_xor::{Claim, InteractionClaim, N_TRACE_COLUMNS};
use crate::witness::components::prelude::*;
use crate::witness::components::verify_bitwise_xor_8;

pub type InputType = [UInt32; 4];
pub type PackedInputType = [PackedUInt32; 4];

/// Retrieves the component's inputs from the context values, using the addresses provided in the
/// preprocessed trace.
#[allow(clippy::uninit_vec)]
pub fn extract_component_inputs(
    input_addr_col_0: &[usize],
    input_addr_col_1: &[usize],
    input_addr_col_2: &[usize],
    output_addr_col: &[usize],
    context_values: &[QM31],
) -> Vec<InputType> {
    let n_rows = input_addr_col_0.len();
    assert_eq!(n_rows, input_addr_col_1.len());
    assert_eq!(n_rows, input_addr_col_2.len());
    assert_eq!(n_rows, output_addr_col.len());

    let mut inputs = Vec::with_capacity(n_rows);
    unsafe {
        inputs.set_len(n_rows);
    }

    (
        inputs.par_iter_mut(),
        input_addr_col_0.par_iter(),
        input_addr_col_1.par_iter(),
        input_addr_col_2.par_iter(),
        output_addr_col.par_iter(),
    )
        .into_par_iter()
        .for_each(|(input, &addr_0, &addr_1, &addr_2, &out_addr)| {
            let qm31_in_addr_0 = context_values[addr_0].to_m31_array();
            let qm31_in_addr_1 = context_values[addr_1].to_m31_array();
            let qm31_in_addr_2 = context_values[addr_2].to_m31_array();
            let qm31_out_addr = context_values[out_addr].to_m31_array();
            *input = [
                UInt32::from(qm31_in_addr_0[0].0 | (qm31_in_addr_0[1].0 << 16)),
                UInt32::from(qm31_in_addr_1[0].0 | (qm31_in_addr_1[1].0 << 16)),
                UInt32::from(qm31_in_addr_2[0].0 | (qm31_in_addr_2[1].0 << 16)),
                UInt32::from(qm31_out_addr[0].0 | (qm31_out_addr[1].0 << 16)),
            ];
        });

    inputs
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct SubComponentInputs {
    verify_bitwise_xor_8: [Vec<verify_bitwise_xor_8::PackedInputType>; 8],
}

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: &PreProcessedTrace,
    verify_bitwise_xor_8_state: &verify_bitwise_xor_8::ClaimGenerator,
) -> (ComponentTrace<N_TRACE_COLUMNS>, Claim, InteractionClaimGenerator) {
    let input_addr_col_0 = preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "triple_xor_input_addr_0".to_owned() });
    let input_addr_col_1 = preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "triple_xor_input_addr_1".to_owned() });
    let input_addr_col_2 = preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "triple_xor_input_addr_2".to_owned() });
    let output_addr_col = preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "triple_xor_output_addr".to_owned() });
    let multiplicity_col = preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "triple_xor_multiplicity".to_owned() });

    let inputs = extract_component_inputs(
        input_addr_col_0,
        input_addr_col_1,
        input_addr_col_2,
        output_addr_col,
        context_values,
    );

    let n_rows = inputs.len();
    assert_ne!(n_rows, 0);
    let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
    let log_size = size.ilog2();

    let packed_inputs = pack_values(&inputs);

    let preprocessed_columns =
        [input_addr_col_0, input_addr_col_1, input_addr_col_2, output_addr_col, multiplicity_col]
            .into_iter()
            .map(|col| Col::<SimdBackend, M31>::from_iter(col.iter().map(|&x| M31::from(x))).data)
            .collect_vec();

    let (trace, lookup_data, sub_component_inputs) =
        write_trace_simd(packed_inputs, preprocessed_columns);
    for inputs in sub_component_inputs.verify_bitwise_xor_8 {
        verify_bitwise_xor_8_state.add_packed_inputs(&inputs, 0);
    }

    (trace, Claim { log_size }, InteractionClaimGenerator { log_size, lookup_data })
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

    let M31_112558620 = PackedM31::broadcast(M31::from(112558620));
    let M31_256 = PackedM31::broadcast(M31::from(256));
    let M31_378353459 = PackedM31::broadcast(M31::from(378353459));
    let UInt16_8 = PackedUInt16::broadcast(UInt16::from(8));

    let [
        triple_xor_input_addr_0,
        triple_xor_input_addr_1,
        triple_xor_input_addr_2,
        triple_xor_output_addr,
        mults,
    ]: [Vec<PackedM31>; 5] = preprocessed_columns.try_into().unwrap();

    (
        trace.par_iter_mut(),
        lookup_data.par_iter_mut(),
        sub_component_inputs.par_iter_mut(),
        inputs.into_par_iter(),
    )
        .into_par_iter()
        .enumerate()
        .for_each(|(row_index, (row, lookup_data, sub_component_inputs, triple_xor_input))| {
            let triple_xor_input_addr_0 = triple_xor_input_addr_0[row_index];
            let triple_xor_input_addr_1 = triple_xor_input_addr_1[row_index];
            let triple_xor_input_addr_2 = triple_xor_input_addr_2[row_index];
            let triple_xor_output_addr = triple_xor_output_addr[row_index];

            let input_a_limb_0_col0 = triple_xor_input[0].low().as_m31();
            *row[0] = input_a_limb_0_col0;
            let input_a_limb_1_col1 = triple_xor_input[0].high().as_m31();
            *row[1] = input_a_limb_1_col1;
            let input_b_limb_0_col2 = triple_xor_input[1].low().as_m31();
            *row[2] = input_b_limb_0_col2;
            let input_b_limb_1_col3 = triple_xor_input[1].high().as_m31();
            *row[3] = input_b_limb_1_col3;
            let input_c_limb_0_col4 = triple_xor_input[2].low().as_m31();
            *row[4] = input_c_limb_0_col4;
            let input_c_limb_1_col5 = triple_xor_input[2].high().as_m31();
            *row[5] = input_c_limb_1_col5;
            let input_a_xor_b_xor_c_limb_0_col6 = triple_xor_input[3].low().as_m31();
            *row[6] = input_a_xor_b_xor_c_limb_0_col6;
            let input_a_xor_b_xor_c_limb_1_col7 = triple_xor_input[3].high().as_m31();
            *row[7] = input_a_xor_b_xor_c_limb_1_col7;

            // Split 16 Low Part Size 8.

            let ms_8_bits_tmp_b1429_0 = (triple_xor_input[0].low() >> (UInt16_8));
            let ms_8_bits_col8 = ms_8_bits_tmp_b1429_0.as_m31();
            *row[8] = ms_8_bits_col8;
            let split_16_low_part_size_8_output_tmp_b1429_1 =
                [((input_a_limb_0_col0) - ((ms_8_bits_col8) * (M31_256))), ms_8_bits_col8];

            // Split 16 Low Part Size 8.

            let ms_8_bits_tmp_b1429_2 = (triple_xor_input[0].high() >> (UInt16_8));
            let ms_8_bits_col9 = ms_8_bits_tmp_b1429_2.as_m31();
            *row[9] = ms_8_bits_col9;
            let split_16_low_part_size_8_output_tmp_b1429_3 =
                [((input_a_limb_1_col1) - ((ms_8_bits_col9) * (M31_256))), ms_8_bits_col9];

            // Split 16 Low Part Size 8.

            let ms_8_bits_tmp_b1429_4 = (triple_xor_input[1].low() >> (UInt16_8));
            let ms_8_bits_col10 = ms_8_bits_tmp_b1429_4.as_m31();
            *row[10] = ms_8_bits_col10;
            let split_16_low_part_size_8_output_tmp_b1429_5 =
                [((input_b_limb_0_col2) - ((ms_8_bits_col10) * (M31_256))), ms_8_bits_col10];

            // Split 16 Low Part Size 8.

            let ms_8_bits_tmp_b1429_6 = (triple_xor_input[1].high() >> (UInt16_8));
            let ms_8_bits_col11 = ms_8_bits_tmp_b1429_6.as_m31();
            *row[11] = ms_8_bits_col11;
            let split_16_low_part_size_8_output_tmp_b1429_7 =
                [((input_b_limb_1_col3) - ((ms_8_bits_col11) * (M31_256))), ms_8_bits_col11];

            // Split 16 Low Part Size 8.

            let ms_8_bits_tmp_b1429_8 = (triple_xor_input[2].low() >> (UInt16_8));
            let ms_8_bits_col12 = ms_8_bits_tmp_b1429_8.as_m31();
            *row[12] = ms_8_bits_col12;
            let split_16_low_part_size_8_output_tmp_b1429_9 =
                [((input_c_limb_0_col4) - ((ms_8_bits_col12) * (M31_256))), ms_8_bits_col12];

            // Split 16 Low Part Size 8.

            let ms_8_bits_tmp_b1429_10 = (triple_xor_input[2].high() >> (UInt16_8));
            let ms_8_bits_col13 = ms_8_bits_tmp_b1429_10.as_m31();
            *row[13] = ms_8_bits_col13;
            let split_16_low_part_size_8_output_tmp_b1429_11 =
                [((input_c_limb_1_col5) - ((ms_8_bits_col13) * (M31_256))), ms_8_bits_col13];

            // Split 16 Low Part Size 8.

            let ms_8_bits_tmp_b1429_12 = (triple_xor_input[3].low() >> (UInt16_8));
            let ms_8_bits_col14 = ms_8_bits_tmp_b1429_12.as_m31();
            *row[14] = ms_8_bits_col14;
            let split_16_low_part_size_8_output_tmp_b1429_13 = [
                ((input_a_xor_b_xor_c_limb_0_col6) - ((ms_8_bits_col14) * (M31_256))),
                ms_8_bits_col14,
            ];

            // Split 16 Low Part Size 8.

            let ms_8_bits_tmp_b1429_14 = (triple_xor_input[3].high() >> (UInt16_8));
            let ms_8_bits_col15 = ms_8_bits_tmp_b1429_14.as_m31();
            *row[15] = ms_8_bits_col15;
            let split_16_low_part_size_8_output_tmp_b1429_15 = [
                ((input_a_xor_b_xor_c_limb_1_col7) - ((ms_8_bits_col15) * (M31_256))),
                ms_8_bits_col15,
            ];

            // Bitwise Xor Num Bits 8.

            let xor_tmp_b1429_16 =
                ((PackedUInt16::from_m31(split_16_low_part_size_8_output_tmp_b1429_1[0]))
                    ^ (PackedUInt16::from_m31(split_16_low_part_size_8_output_tmp_b1429_5[0])));
            let xor_col16 = xor_tmp_b1429_16.as_m31();
            *row[16] = xor_col16;
            *sub_component_inputs.verify_bitwise_xor_8[0] = [
                split_16_low_part_size_8_output_tmp_b1429_1[0],
                split_16_low_part_size_8_output_tmp_b1429_5[0],
                xor_col16,
            ];
            *lookup_data.verify_bitwise_xor_8_0 = [
                M31_112558620,
                split_16_low_part_size_8_output_tmp_b1429_1[0],
                split_16_low_part_size_8_output_tmp_b1429_5[0],
                xor_col16,
            ];

            // Bitwise Xor Num Bits 8.

            let xor_tmp_b1429_18 = ((PackedUInt16::from_m31(ms_8_bits_col8))
                ^ (PackedUInt16::from_m31(ms_8_bits_col10)));
            let xor_col17 = xor_tmp_b1429_18.as_m31();
            *row[17] = xor_col17;
            *sub_component_inputs.verify_bitwise_xor_8[1] =
                [ms_8_bits_col8, ms_8_bits_col10, xor_col17];
            *lookup_data.verify_bitwise_xor_8_1 =
                [M31_112558620, ms_8_bits_col8, ms_8_bits_col10, xor_col17];

            // Bitwise Xor Num Bits 8.

            let xor_tmp_b1429_20 =
                ((PackedUInt16::from_m31(split_16_low_part_size_8_output_tmp_b1429_3[0]))
                    ^ (PackedUInt16::from_m31(split_16_low_part_size_8_output_tmp_b1429_7[0])));
            let xor_col18 = xor_tmp_b1429_20.as_m31();
            *row[18] = xor_col18;
            *sub_component_inputs.verify_bitwise_xor_8[2] = [
                split_16_low_part_size_8_output_tmp_b1429_3[0],
                split_16_low_part_size_8_output_tmp_b1429_7[0],
                xor_col18,
            ];
            *lookup_data.verify_bitwise_xor_8_2 = [
                M31_112558620,
                split_16_low_part_size_8_output_tmp_b1429_3[0],
                split_16_low_part_size_8_output_tmp_b1429_7[0],
                xor_col18,
            ];

            // Bitwise Xor Num Bits 8.

            let xor_tmp_b1429_22 = ((PackedUInt16::from_m31(ms_8_bits_col9))
                ^ (PackedUInt16::from_m31(ms_8_bits_col11)));
            let xor_col19 = xor_tmp_b1429_22.as_m31();
            *row[19] = xor_col19;
            *sub_component_inputs.verify_bitwise_xor_8[3] =
                [ms_8_bits_col9, ms_8_bits_col11, xor_col19];
            *lookup_data.verify_bitwise_xor_8_3 =
                [M31_112558620, ms_8_bits_col9, ms_8_bits_col11, xor_col19];

            *sub_component_inputs.verify_bitwise_xor_8[4] = [
                xor_col16,
                split_16_low_part_size_8_output_tmp_b1429_9[0],
                split_16_low_part_size_8_output_tmp_b1429_13[0],
            ];
            *lookup_data.verify_bitwise_xor_8_4 = [
                M31_112558620,
                xor_col16,
                split_16_low_part_size_8_output_tmp_b1429_9[0],
                split_16_low_part_size_8_output_tmp_b1429_13[0],
            ];
            *sub_component_inputs.verify_bitwise_xor_8[5] =
                [xor_col17, ms_8_bits_col12, ms_8_bits_col14];
            *lookup_data.verify_bitwise_xor_8_5 =
                [M31_112558620, xor_col17, ms_8_bits_col12, ms_8_bits_col14];
            *sub_component_inputs.verify_bitwise_xor_8[6] = [
                xor_col18,
                split_16_low_part_size_8_output_tmp_b1429_11[0],
                split_16_low_part_size_8_output_tmp_b1429_15[0],
            ];
            *lookup_data.verify_bitwise_xor_8_6 = [
                M31_112558620,
                xor_col18,
                split_16_low_part_size_8_output_tmp_b1429_11[0],
                split_16_low_part_size_8_output_tmp_b1429_15[0],
            ];
            *sub_component_inputs.verify_bitwise_xor_8[7] =
                [xor_col19, ms_8_bits_col13, ms_8_bits_col15];
            *lookup_data.verify_bitwise_xor_8_7 =
                [M31_112558620, xor_col19, ms_8_bits_col13, ms_8_bits_col15];

            *lookup_data.gate_0 =
                [M31_378353459, triple_xor_input_addr_0, input_a_limb_0_col0, input_a_limb_1_col1];
            *lookup_data.gate_1 =
                [M31_378353459, triple_xor_input_addr_1, input_b_limb_0_col2, input_b_limb_1_col3];
            *lookup_data.gate_2 =
                [M31_378353459, triple_xor_input_addr_2, input_c_limb_0_col4, input_c_limb_1_col5];
            *lookup_data.gate_3 = [
                M31_378353459,
                triple_xor_output_addr,
                input_a_xor_b_xor_c_limb_0_col6,
                input_a_xor_b_xor_c_limb_1_col7,
            ];
            *lookup_data.mults_0 = mults[row_index];
        });
    (trace, lookup_data, sub_component_inputs)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    gate_0: Vec<[PackedM31; 4]>,
    gate_1: Vec<[PackedM31; 4]>,
    gate_2: Vec<[PackedM31; 4]>,
    gate_3: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_0: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_1: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_2: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_3: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_4: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_5: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_6: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_7: Vec<[PackedM31; 4]>,
    mults_0: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        common_lookup_elements: &relations::CommonLookupElements,
    ) -> (Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>, InteractionClaim) {
        let mut logup_gen = unsafe { LogupTraceGenerator::uninitialized(self.log_size) };

        // Sum logup terms in pairs.
        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.verify_bitwise_xor_8_0,
            &self.lookup_data.verify_bitwise_xor_8_1,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.verify_bitwise_xor_8_2,
            &self.lookup_data.verify_bitwise_xor_8_3,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.verify_bitwise_xor_8_4,
            &self.lookup_data.verify_bitwise_xor_8_5,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.verify_bitwise_xor_8_6,
            &self.lookup_data.verify_bitwise_xor_8_7,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.gate_0, &self.lookup_data.gate_1)
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.gate_2,
            &self.lookup_data.gate_3,
            &self.lookup_data.mults_0,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1, &mults_0)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0 * mults_0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();

        (trace, InteractionClaim { claimed_sum })
    }
}
