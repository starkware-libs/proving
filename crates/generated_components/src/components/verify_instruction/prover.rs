#![allow(unused_parens)]
#![allow(unused_imports)]
use std::iter::zip;

use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
use prover_types::cpu::*;
use prover_types::simd::*;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo_cairo_prover::cairo_air::preprocessed::{PreProcessedColumn, Seq};
use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
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
use stwo_prover::core::utils::{
    bit_reverse_coset_to_circle_domain_order, bit_reverse_index, coset_index_to_circle_domain_index,
};

use super::component::{Claim, InteractionClaim};
use crate::components::{
    memory_address_to_id, memory_id_to_big, pack_values, range_check_4_3, range_check_7_2_5,
};
use crate::relations;

pub type InputType = (M31, [M31; 3], [M31; 2], M31);
pub type PackedInputType = (PackedM31, [PackedM31; 3], [PackedM31; 2], PackedM31);
const N_TRACE_COLUMNS: usize = 16;

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
        let log_size = size.ilog2();

        if need_padding {
            self.inputs.resize(size, *self.inputs.first().unwrap());
            bit_reverse_coset_to_circle_domain_order(&mut self.inputs);
        }

        let packed_inputs = pack_values(&self.inputs);

        let (trace, lookup_data) = write_trace_simd(
            n_rows,
            packed_inputs,
            memory_address_to_id_state,
            memory_id_to_big_state,
            range_check_4_3_state,
            range_check_7_2_5_state,
        );

        tree_builder.extend_evals(trace.to_evals());

        (
            Claim { log_size },
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        )
    }

    pub fn add_input(&self, input: &InputType) {
        unimplemented!("Implement manually");
    }

    pub fn add_inputs(&self, inputs: &[InputType]) {
        for input in inputs {
            self.add_input(input);
        }
    }
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(
    n_rows: usize,
    inputs: Vec<PackedInputType>,
    memory_address_to_id_state: &memory_address_to_id::ClaimGenerator,
    memory_id_to_big_state: &memory_id_to_big::ClaimGenerator,
    range_check_4_3_state: &range_check_4_3::ClaimGenerator,
    range_check_7_2_5_state: &range_check_7_2_5::ClaimGenerator,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
        )
    };

    let M31_0 = PackedM31::broadcast(M31::from(0));
    let M31_128 = PackedM31::broadcast(M31::from(128));
    let M31_32 = PackedM31::broadcast(M31::from(32));
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
        .for_each(
            |(((row_index, row), verify_instruction_input), lookup_data)| {
                let input_tmp_16a4f_0 = (
                    verify_instruction_input.0,
                    [
                        verify_instruction_input.1[0],
                        verify_instruction_input.1[1],
                        verify_instruction_input.1[2],
                    ],
                    [verify_instruction_input.2[0], verify_instruction_input.2[1]],
                    verify_instruction_input.3,
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
                let input_limb_6_col6 = input_tmp_16a4f_0.3;
                *row[6] = input_limb_6_col6;

                // Encode Offsets.

                let offset0_low_tmp_16a4f_1 =
                    ((PackedUInt16::from_m31(input_limb_1_col1)) & (UInt16_511));
                let offset0_low_col7 = offset0_low_tmp_16a4f_1.as_m31();
                *row[7] = offset0_low_col7;
                let offset0_mid_tmp_16a4f_2 =
                    ((PackedUInt16::from_m31(input_limb_1_col1)) >> (UInt16_9));
                let offset0_mid_col8 = offset0_mid_tmp_16a4f_2.as_m31();
                *row[8] = offset0_mid_col8;
                let offset1_low_tmp_16a4f_3 =
                    ((PackedUInt16::from_m31(input_limb_2_col2)) & (UInt16_3));
                let offset1_low_col9 = offset1_low_tmp_16a4f_3.as_m31();
                *row[9] = offset1_low_col9;
                let offset1_mid_tmp_16a4f_4 =
                    (((PackedUInt16::from_m31(input_limb_2_col2)) >> (UInt16_2)) & (UInt16_511));
                let offset1_mid_col10 = offset1_mid_tmp_16a4f_4.as_m31();
                *row[10] = offset1_mid_col10;
                let offset1_high_tmp_16a4f_5 =
                    ((PackedUInt16::from_m31(input_limb_2_col2)) >> (UInt16_11));
                let offset1_high_col11 = offset1_high_tmp_16a4f_5.as_m31();
                *row[11] = offset1_high_col11;
                let offset2_low_tmp_16a4f_6 =
                    ((PackedUInt16::from_m31(input_limb_3_col3)) & (UInt16_15));
                let offset2_low_col12 = offset2_low_tmp_16a4f_6.as_m31();
                *row[12] = offset2_low_col12;
                let offset2_mid_tmp_16a4f_7 =
                    (((PackedUInt16::from_m31(input_limb_3_col3)) >> (UInt16_4)) & (UInt16_511));
                let offset2_mid_col13 = offset2_mid_tmp_16a4f_7.as_m31();
                *row[13] = offset2_mid_col13;
                let offset2_high_tmp_16a4f_8 =
                    ((PackedUInt16::from_m31(input_limb_3_col3)) >> (UInt16_13));
                let offset2_high_col14 = offset2_high_tmp_16a4f_8.as_m31();
                *row[14] = offset2_high_col14;
                let range_check_7_2_5_inputs_0 =
                    [offset0_mid_col8, offset1_low_col9, offset1_high_col11].unpack();
                *lookup_data.range_check_7_2_5_0 =
                    [offset0_mid_col8, offset1_low_col9, offset1_high_col11];
                let range_check_4_3_inputs_0 = [offset2_low_col12, offset2_high_col14].unpack();
                *lookup_data.range_check_4_3_0 = [offset2_low_col12, offset2_high_col14];

                // Mem Verify.

                let memory_address_to_id_value_tmp_16a4f_9 =
                    memory_address_to_id_state.deduce_output(input_limb_0_col0);
                let instruction_id_col15 = memory_address_to_id_value_tmp_16a4f_9;
                *row[15] = instruction_id_col15;
                let memory_address_to_id_inputs_0 = input_limb_0_col0.unpack();
                *lookup_data.memory_address_to_id_0 = [input_limb_0_col0, instruction_id_col15];
                let memory_id_to_big_inputs_0 = instruction_id_col15.unpack();
                *lookup_data.memory_id_to_big_0 = [
                    instruction_id_col15,
                    offset0_low_col7,
                    ((offset0_mid_col8) + ((offset1_low_col9) * (M31_128))),
                    offset1_mid_col10,
                    ((offset1_high_col11) + ((offset2_low_col12) * (M31_32))),
                    offset2_mid_col13,
                    ((offset2_high_col14) + (input_limb_4_col4)),
                    input_limb_5_col5,
                    input_limb_6_col6,
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
                ];

                // Add sub-components inputs.
                #[allow(clippy::needless_range_loop)]
                for i in 0..N_LANES {
                    if bit_reverse_index(
                        coset_index_to_circle_domain_index(row_index * N_LANES + i, log_size),
                        log_size,
                    ) < n_rows
                    {
                        range_check_7_2_5_state.add_input(&range_check_7_2_5_inputs_0[i]);
                        range_check_4_3_state.add_input(&range_check_4_3_inputs_0[i]);
                        memory_address_to_id_state.add_input(&memory_address_to_id_inputs_0[i]);
                        memory_id_to_big_state.add_input(&memory_id_to_big_inputs_0[i]);
                    }
                }
            },
        );

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    memory_address_to_id_0: Vec<[PackedM31; 2]>,
    memory_id_to_big_0: Vec<[PackedM31; 29]>,
    range_check_4_3_0: Vec<[PackedM31; 2]>,
    range_check_7_2_5_0: Vec<[PackedM31; 3]>,
    verify_instruction_0: Vec<[PackedM31; 7]>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
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
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

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

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
