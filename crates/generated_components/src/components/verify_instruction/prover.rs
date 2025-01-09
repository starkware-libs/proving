#![allow(unused_parens)]
#![allow(unused_imports)]
use std::iter::zip;

use air_structs_derive::SubComponentInputs;
use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
use prover_types::cpu::*;
use prover_types::simd::*;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
use stwo_prover::constraint_framework::preprocessed_columns::PreprocessedColumn;
use stwo_prover::constraint_framework::Relation;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::column::BaseColumn;
use stwo_prover::core::backend::simd::conversion::Unpack;
use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
use stwo_prover::core::backend::simd::qm31::PackedQM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{BackendForChannel, Col, Column};
use stwo_prover::core::channel::{Channel, MerkleChannel};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::utils::bit_reverse_coset_to_circle_domain_order;

use super::component::{Claim, InteractionClaim};
use crate::components::{
    memory_address_to_id, memory_id_to_big, pack_values, range_check_4_3, range_check_7_2_5,
};
use crate::relations;

pub type InputType = (M31, [M31; 3], [M31; 15]);
pub type PackedInputType = (PackedM31, [PackedM31; 3], [PackedM31; 15]);
const N_TRACE_COLUMNS: usize = 28;

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn new(inputs: Vec<InputType>) -> Self {
        Self { inputs }
    }

    pub fn write_trace<MC: MerkleChannel>(
        mut self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
        memory_address_to_id_state: &memory_address_to_id::ClaimGenerator,
        memory_id_to_big_state: &memory_id_to_big::ClaimGenerator,
        range_check_4_3_state: &range_check_4_3::ClaimGenerator,
        range_check_7_2_5_state: &range_check_7_2_5::ClaimGenerator,
    ) -> (Claim, InteractionClaimGenerator)
    where
        SimdBackend: BackendForChannel<MC>,
    {
        let n_rows = self.inputs.len();
        assert_ne!(n_rows, 0);
        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let need_padding = n_rows != size;

        if need_padding {
            self.inputs.resize(size, *self.inputs.first().unwrap());
            bit_reverse_coset_to_circle_domain_order(&mut self.inputs);
        }

        let packed_inputs = pack_values(&self.inputs);
        let (trace, mut sub_components_inputs, lookup_data) =
            write_trace_simd(n_rows, packed_inputs, memory_address_to_id_state);

        if need_padding {
            sub_components_inputs.bit_reverse_coset_to_circle_domain_order();
        }
        sub_components_inputs
            .memory_address_to_id_inputs
            .iter()
            .for_each(|inputs| {
                memory_address_to_id_state.add_inputs(&inputs[..n_rows]);
            });
        sub_components_inputs
            .memory_id_to_big_inputs
            .iter()
            .for_each(|inputs| {
                memory_id_to_big_state.add_inputs(&inputs[..n_rows]);
            });
        sub_components_inputs
            .range_check_4_3_inputs
            .iter()
            .for_each(|inputs| {
                range_check_4_3_state.add_inputs(&inputs[..n_rows]);
            });
        sub_components_inputs
            .range_check_7_2_5_inputs
            .iter()
            .for_each(|inputs| {
                range_check_7_2_5_state.add_inputs(&inputs[..n_rows]);
            });

        tree_builder.extend_evals(trace.to_evals());

        (
            Claim { n_rows },
            InteractionClaimGenerator {
                n_rows,
                lookup_data,
            },
        )
    }

    pub fn add_inputs(&self, _inputs: &[InputType]) {
        unimplemented!("Implement manually");
    }
}

#[derive(SubComponentInputs, Uninitialized, IterMut, ParIterMut)]
pub struct SubComponentInputs {
    pub memory_address_to_id_inputs: [Vec<memory_address_to_id::InputType>; 1],
    pub memory_id_to_big_inputs: [Vec<memory_id_to_big::InputType>; 1],
    pub range_check_4_3_inputs: [Vec<range_check_4_3::InputType>; 1],
    pub range_check_7_2_5_inputs: [Vec<range_check_7_2_5::InputType>; 1],
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(
    n_rows: usize,
    inputs: Vec<PackedInputType>,
    memory_address_to_id_state: &memory_address_to_id::ClaimGenerator,
) -> (
    ComponentTrace<N_TRACE_COLUMNS>,
    SubComponentInputs,
    LookupData,
) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data, mut sub_components_inputs) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
            SubComponentInputs::uninitialized(log_size),
        )
    };

    let M31_0 = PackedM31::broadcast(M31::from(0));
    let M31_1 = PackedM31::broadcast(M31::from(1));
    let M31_128 = PackedM31::broadcast(M31::from(128));
    let M31_16 = PackedM31::broadcast(M31::from(16));
    let M31_2 = PackedM31::broadcast(M31::from(2));
    let M31_256 = PackedM31::broadcast(M31::from(256));
    let M31_32 = PackedM31::broadcast(M31::from(32));
    let M31_4 = PackedM31::broadcast(M31::from(4));
    let M31_64 = PackedM31::broadcast(M31::from(64));
    let M31_8 = PackedM31::broadcast(M31::from(8));
    let UInt16_11 = PackedUInt16::broadcast(UInt16::from(11));
    let UInt16_13 = PackedUInt16::broadcast(UInt16::from(13));
    let UInt16_15 = PackedUInt16::broadcast(UInt16::from(15));
    let UInt16_2 = PackedUInt16::broadcast(UInt16::from(2));
    let UInt16_3 = PackedUInt16::broadcast(UInt16::from(3));
    let UInt16_4 = PackedUInt16::broadcast(UInt16::from(4));
    let UInt16_511 = PackedUInt16::broadcast(UInt16::from(511));
    let UInt16_9 = PackedUInt16::broadcast(UInt16::from(9));

    trace
        .par_iter_mut()
        .enumerate()
        .zip(inputs.into_par_iter())
        .zip(lookup_data.par_iter_mut())
        .zip(sub_components_inputs.par_iter_mut().chunks(N_LANES))
        .for_each(
            |(
                (((row_index, row), verify_instruction_input), lookup_data),
                mut sub_components_inputs,
            )| {
                let input_tmp_16a4f_0 = (
                    verify_instruction_input.0,
                    [
                        verify_instruction_input.1[0],
                        verify_instruction_input.1[1],
                        verify_instruction_input.1[2],
                    ],
                    [
                        verify_instruction_input.2[0],
                        verify_instruction_input.2[1],
                        verify_instruction_input.2[2],
                        verify_instruction_input.2[3],
                        verify_instruction_input.2[4],
                        verify_instruction_input.2[5],
                        verify_instruction_input.2[6],
                        verify_instruction_input.2[7],
                        verify_instruction_input.2[8],
                        verify_instruction_input.2[9],
                        verify_instruction_input.2[10],
                        verify_instruction_input.2[11],
                        verify_instruction_input.2[12],
                        verify_instruction_input.2[13],
                        verify_instruction_input.2[14],
                    ],
                );
                let input_limb_0_col0 = input_tmp_16a4f_0.0;
                *row[0] = input_limb_0_col0;
                let input_limb_1_col1 = input_tmp_16a4f_0.1[0];
                *row[1] = input_limb_1_col1;
                let input_limb_2_col2 = input_tmp_16a4f_0.1[1];
                *row[2] = input_limb_2_col2;
                let input_limb_3_col3 = input_tmp_16a4f_0.1[2];
                *row[3] = input_limb_3_col3;
                let input_limb_4_col4 = input_tmp_16a4f_0.2[0];
                *row[4] = input_limb_4_col4;
                let input_limb_5_col5 = input_tmp_16a4f_0.2[1];
                *row[5] = input_limb_5_col5;
                let input_limb_6_col6 = input_tmp_16a4f_0.2[2];
                *row[6] = input_limb_6_col6;
                let input_limb_7_col7 = input_tmp_16a4f_0.2[3];
                *row[7] = input_limb_7_col7;
                let input_limb_8_col8 = input_tmp_16a4f_0.2[4];
                *row[8] = input_limb_8_col8;
                let input_limb_9_col9 = input_tmp_16a4f_0.2[5];
                *row[9] = input_limb_9_col9;
                let input_limb_10_col10 = input_tmp_16a4f_0.2[6];
                *row[10] = input_limb_10_col10;
                let input_limb_11_col11 = input_tmp_16a4f_0.2[7];
                *row[11] = input_limb_11_col11;
                let input_limb_12_col12 = input_tmp_16a4f_0.2[8];
                *row[12] = input_limb_12_col12;
                let input_limb_13_col13 = input_tmp_16a4f_0.2[9];
                *row[13] = input_limb_13_col13;
                let input_limb_14_col14 = input_tmp_16a4f_0.2[10];
                *row[14] = input_limb_14_col14;
                let input_limb_15_col15 = input_tmp_16a4f_0.2[11];
                *row[15] = input_limb_15_col15;
                let input_limb_16_col16 = input_tmp_16a4f_0.2[12];
                *row[16] = input_limb_16_col16;
                let input_limb_17_col17 = input_tmp_16a4f_0.2[13];
                *row[17] = input_limb_17_col17;
                let input_limb_18_col18 = input_tmp_16a4f_0.2[14];
                *row[18] = input_limb_18_col18;

                // Encode Offsets.

                let offset0_low_tmp_16a4f_1 =
                    ((PackedUInt16::from_m31(input_limb_1_col1)) & (UInt16_511));
                let offset0_low_col19 = offset0_low_tmp_16a4f_1.as_m31();
                *row[19] = offset0_low_col19;
                let offset0_mid_tmp_16a4f_2 =
                    ((PackedUInt16::from_m31(input_limb_1_col1)) >> (UInt16_9));
                let offset0_mid_col20 = offset0_mid_tmp_16a4f_2.as_m31();
                *row[20] = offset0_mid_col20;
                let offset1_low_tmp_16a4f_3 =
                    ((PackedUInt16::from_m31(input_limb_2_col2)) & (UInt16_3));
                let offset1_low_col21 = offset1_low_tmp_16a4f_3.as_m31();
                *row[21] = offset1_low_col21;
                let offset1_mid_tmp_16a4f_4 =
                    (((PackedUInt16::from_m31(input_limb_2_col2)) >> (UInt16_2)) & (UInt16_511));
                let offset1_mid_col22 = offset1_mid_tmp_16a4f_4.as_m31();
                *row[22] = offset1_mid_col22;
                let offset1_high_tmp_16a4f_5 =
                    ((PackedUInt16::from_m31(input_limb_2_col2)) >> (UInt16_11));
                let offset1_high_col23 = offset1_high_tmp_16a4f_5.as_m31();
                *row[23] = offset1_high_col23;
                let offset2_low_tmp_16a4f_6 =
                    ((PackedUInt16::from_m31(input_limb_3_col3)) & (UInt16_15));
                let offset2_low_col24 = offset2_low_tmp_16a4f_6.as_m31();
                *row[24] = offset2_low_col24;
                let offset2_mid_tmp_16a4f_7 =
                    (((PackedUInt16::from_m31(input_limb_3_col3)) >> (UInt16_4)) & (UInt16_511));
                let offset2_mid_col25 = offset2_mid_tmp_16a4f_7.as_m31();
                *row[25] = offset2_mid_col25;
                let offset2_high_tmp_16a4f_8 =
                    ((PackedUInt16::from_m31(input_limb_3_col3)) >> (UInt16_13));
                let offset2_high_col26 = offset2_high_tmp_16a4f_8.as_m31();
                *row[26] = offset2_high_col26;
                for (i, &input) in [offset0_mid_col20, offset1_low_col21, offset1_high_col23]
                    .unpack()
                    .iter()
                    .enumerate()
                {
                    *sub_components_inputs[i].range_check_7_2_5_inputs[0] = input;
                }
                *lookup_data.range_check_7_2_5_0 =
                    [offset0_mid_col20, offset1_low_col21, offset1_high_col23];
                for (i, &input) in [offset2_low_col24, offset2_high_col26]
                    .unpack()
                    .iter()
                    .enumerate()
                {
                    *sub_components_inputs[i].range_check_4_3_inputs[0] = input;
                }
                *lookup_data.range_check_4_3_0 = [offset2_low_col24, offset2_high_col26];

                // Mem Verify.

                let memory_address_to_id_value_tmp_16a4f_9 =
                    memory_address_to_id_state.deduce_output(input_limb_0_col0);
                let instruction_id_col27 = memory_address_to_id_value_tmp_16a4f_9;
                *row[27] = instruction_id_col27;
                for (i, &input) in input_limb_0_col0.unpack().iter().enumerate() {
                    *sub_components_inputs[i].memory_address_to_id_inputs[0] = input;
                }
                *lookup_data.memory_address_to_id_0 = [input_limb_0_col0, instruction_id_col27];
                for (i, &input) in instruction_id_col27.unpack().iter().enumerate() {
                    *sub_components_inputs[i].memory_id_to_big_inputs[0] = input;
                }
                *lookup_data.memory_id_to_big_0 = [
                    instruction_id_col27,
                    offset0_low_col19,
                    ((offset0_mid_col20) + ((offset1_low_col21) * (M31_128))),
                    offset1_mid_col22,
                    ((offset1_high_col23) + ((offset2_low_col24) * (M31_32))),
                    offset2_mid_col25,
                    ((offset2_high_col26)
                        + (((((((M31_0) + ((input_limb_4_col4) * (M31_8)))
                            + ((input_limb_5_col5) * (M31_16)))
                            + ((input_limb_6_col6) * (M31_32)))
                            + ((input_limb_7_col7) * (M31_64)))
                            + ((input_limb_8_col8) * (M31_128)))
                            + ((input_limb_9_col9) * (M31_256)))),
                    ((((((((((M31_0) + ((input_limb_10_col10) * (M31_1)))
                        + ((input_limb_11_col11) * (M31_2)))
                        + ((input_limb_12_col12) * (M31_4)))
                        + ((input_limb_13_col13) * (M31_8)))
                        + ((input_limb_14_col14) * (M31_16)))
                        + ((input_limb_15_col15) * (M31_32)))
                        + ((input_limb_16_col16) * (M31_64)))
                        + ((input_limb_17_col17) * (M31_128)))
                        + ((input_limb_18_col18) * (M31_256))),
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                    M31_0,
                ];

                *lookup_data.verify_instruction_0 = [
                    input_limb_0_col0,
                    input_limb_1_col1,
                    input_limb_2_col2,
                    input_limb_3_col3,
                    input_limb_4_col4,
                    input_limb_5_col5,
                    input_limb_6_col6,
                    input_limb_7_col7,
                    input_limb_8_col8,
                    input_limb_9_col9,
                    input_limb_10_col10,
                    input_limb_11_col11,
                    input_limb_12_col12,
                    input_limb_13_col13,
                    input_limb_14_col14,
                    input_limb_15_col15,
                    input_limb_16_col16,
                    input_limb_17_col17,
                    input_limb_18_col18,
                ];
            },
        );

    (trace, sub_components_inputs, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    memory_address_to_id_0: Vec<[PackedM31; 2]>,
    memory_id_to_big_0: Vec<[PackedM31; 29]>,
    range_check_4_3_0: Vec<[PackedM31; 2]>,
    range_check_7_2_5_0: Vec<[PackedM31; 3]>,
    verify_instruction_0: Vec<[PackedM31; 19]>,
}

pub struct InteractionClaimGenerator {
    n_rows: usize,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace<MC: MerkleChannel>(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
        memory_address_to_id: &relations::MemoryAddressToId,
        memory_id_to_big: &relations::MemoryIdToBig,
        range_check_4_3: &relations::RangeCheck_4_3,
        range_check_7_2_5: &relations::RangeCheck_7_2_5,
        verify_instruction: &relations::VerifyInstruction,
    ) -> InteractionClaim
    where
        SimdBackend: BackendForChannel<MC>,
    {
        let log_size = std::cmp::max(self.n_rows.next_power_of_two().ilog2(), LOG_N_LANES);
        let mut logup_gen = LogupTraceGenerator::new(log_size);

        // Sum logup terms in pairs.
        let mut col_gen = logup_gen.new_col();
        for (i, (values0, values1)) in zip(
            &self.lookup_data.range_check_7_2_5_0,
            &self.lookup_data.range_check_4_3_0,
        )
        .enumerate()
        {
            let denom0: PackedQM31 = range_check_7_2_5.combine(values0);
            let denom1: PackedQM31 = range_check_4_3.combine(values1);
            col_gen.write_frac(i, denom0 + denom1, denom0 * denom1);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        for (i, (values0, values1)) in zip(
            &self.lookup_data.memory_address_to_id_0,
            &self.lookup_data.memory_id_to_big_0,
        )
        .enumerate()
        {
            let denom0: PackedQM31 = memory_address_to_id.combine(values0);
            let denom1: PackedQM31 = memory_id_to_big.combine(values1);
            col_gen.write_frac(i, denom0 + denom1, denom0 * denom1);
        }
        col_gen.finalize_col();

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        for (i, values) in self.lookup_data.verify_instruction_0.iter().enumerate() {
            let denom = verify_instruction.combine(values);
            col_gen.write_frac(i, -PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let (trace, total_sum, claimed_sum) = if self.n_rows == 1 << log_size {
            let (trace, claimed_sum) = logup_gen.finalize_last();
            (trace, claimed_sum, None)
        } else {
            let (trace, [total_sum, claimed_sum]) =
                logup_gen.finalize_at([(1 << log_size) - 1, self.n_rows - 1]);
            (trace, total_sum, Some((claimed_sum, self.n_rows - 1)))
        };
        tree_builder.extend_evals(trace);

        InteractionClaim {
            logup_sums: (total_sum, claimed_sum),
        }
    }
}
