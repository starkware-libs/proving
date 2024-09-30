#![allow(unused_parens)]
#![allow(unused_imports)]
use air_code_gen::code_gen::packed_types::*;
use compiled_casm_air::prover_types::*;
use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
use stwo_prover::core::backend::simd::qm31::PackedQM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{Col, Column};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};

use super::component::{Claim, ComponentLookupElements, InteractionClaim};
use crate::{
    memory_k_m31_v_felt252, memory_k_m31_v_m31, rangecheck_n_2_bits_4_3, rangecheck_n_3_bits_7_2_5,
    verifyinstruction,
};

pub type InputType = (PackedM31, [PackedM31; 3], [PackedM31; 15]);

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn write_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        memory_k_m31_v_m31_state: &mut memory_k_m31_v_m31::ClaimGenerator,
        rangecheck_n_2_bits_4_3_state: &mut rangecheck_n_2_bits_4_3::ClaimGenerator,
        rangecheck_n_3_bits_7_2_5_state: &mut rangecheck_n_3_bits_7_2_5::ClaimGenerator,
    ) -> ClaimProver {
        let len = self.inputs.len();
        #[allow(unused_variables)]
        let (trace, sub_components_inputs, lookup_data) =
            write_trace_simd(self.inputs, memory_k_m31_v_m31_state);
        sub_components_inputs
            .memory_k_m31_v_m31_inputs
            .iter()
            .for_each(|inputs| {
                memory_k_m31_v_m31_state.add_inputs(inputs);
            });
        sub_components_inputs
            .rangecheck_n_2_bits_4_3_inputs
            .iter()
            .for_each(|inputs| {
                rangecheck_n_2_bits_4_3_state.add_inputs(inputs);
            });
        sub_components_inputs
            .rangecheck_n_3_bits_7_2_5_inputs
            .iter()
            .for_each(|inputs| {
                rangecheck_n_3_bits_7_2_5_state.add_inputs(inputs);
            });

        tree_builder.extend_evals(trace);
        let claim = Claim {
            n_calls: len * N_LANES,
        };

        ClaimProver { claim, lookup_data }
    }

    pub fn add_inputs(&mut self, inputs: &[InputType]) {
        self.inputs.extend(inputs);
    }
}

#[allow(non_snake_case)]
pub struct SubComponentInputs {
    pub memory_k_m31_v_m31_inputs: [Vec<memory_k_m31_v_m31::InputType>; 1],
    pub rangecheck_n_2_bits_4_3_inputs: [Vec<rangecheck_n_2_bits_4_3::InputType>; 1],
    pub rangecheck_n_3_bits_7_2_5_inputs: [Vec<rangecheck_n_3_bits_7_2_5::InputType>; 1],
}
impl SubComponentInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            memory_k_m31_v_m31_inputs: [Vec::with_capacity(capacity)],
            rangecheck_n_2_bits_4_3_inputs: [Vec::with_capacity(capacity)],
            rangecheck_n_3_bits_7_2_5_inputs: [Vec::with_capacity(capacity)],
        }
    }
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
pub fn write_trace_simd(
    inputs: Vec<InputType>,
    memory_k_m31_v_m31_state: &mut memory_k_m31_v_m31::ClaimGenerator,
) -> (
    Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>,
    SubComponentInputs,
    LookupData,
) {
    let n_trace_columns = 28;
    let mut trace_values = (0..n_trace_columns)
        .map(|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES))
        .collect_vec();
    let mut lookup_data = LookupData::with_capacity(inputs.len());
    #[allow(unused_mut)]
    let mut sub_components_inputs = SubComponentInputs::with_capacity(inputs.len());

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

    inputs
        .into_iter()
        .enumerate()
        .for_each(|(row_index, verifyinstruction_input)| {
            let tmp_0 = (
                verifyinstruction_input.0,
                [
                    verifyinstruction_input.1[0],
                    verifyinstruction_input.1[1],
                    verifyinstruction_input.1[2],
                ],
                [
                    verifyinstruction_input.2[0],
                    verifyinstruction_input.2[1],
                    verifyinstruction_input.2[2],
                    verifyinstruction_input.2[3],
                    verifyinstruction_input.2[4],
                    verifyinstruction_input.2[5],
                    verifyinstruction_input.2[6],
                    verifyinstruction_input.2[7],
                    verifyinstruction_input.2[8],
                    verifyinstruction_input.2[9],
                    verifyinstruction_input.2[10],
                    verifyinstruction_input.2[11],
                    verifyinstruction_input.2[12],
                    verifyinstruction_input.2[13],
                    verifyinstruction_input.2[14],
                ],
            );
            let col0 = tmp_0.0;
            trace_values[0].data[row_index] = col0;
            let col1 = tmp_0.1[0];
            trace_values[1].data[row_index] = col1;
            let col2 = tmp_0.1[1];
            trace_values[2].data[row_index] = col2;
            let col3 = tmp_0.1[2];
            trace_values[3].data[row_index] = col3;
            let col4 = tmp_0.2[0];
            trace_values[4].data[row_index] = col4;
            let col5 = tmp_0.2[1];
            trace_values[5].data[row_index] = col5;
            let col6 = tmp_0.2[2];
            trace_values[6].data[row_index] = col6;
            let col7 = tmp_0.2[3];
            trace_values[7].data[row_index] = col7;
            let col8 = tmp_0.2[4];
            trace_values[8].data[row_index] = col8;
            let col9 = tmp_0.2[5];
            trace_values[9].data[row_index] = col9;
            let col10 = tmp_0.2[6];
            trace_values[10].data[row_index] = col10;
            let col11 = tmp_0.2[7];
            trace_values[11].data[row_index] = col11;
            let col12 = tmp_0.2[8];
            trace_values[12].data[row_index] = col12;
            let col13 = tmp_0.2[9];
            trace_values[13].data[row_index] = col13;
            let col14 = tmp_0.2[10];
            trace_values[14].data[row_index] = col14;
            let col15 = tmp_0.2[11];
            trace_values[15].data[row_index] = col15;
            let col16 = tmp_0.2[12];
            trace_values[16].data[row_index] = col16;
            let col17 = tmp_0.2[13];
            trace_values[17].data[row_index] = col17;
            let col18 = tmp_0.2[14];
            trace_values[18].data[row_index] = col18;
            let tmp_11 = ((PackedUInt16::from_m31(col1)) & (UInt16_511));
            let col19 = tmp_11.as_m31();
            trace_values[19].data[row_index] = col19;
            let tmp_12 = ((PackedUInt16::from_m31(col1)) >> (UInt16_9));
            let col20 = tmp_12.as_m31();
            trace_values[20].data[row_index] = col20;
            let tmp_13 = ((PackedUInt16::from_m31(col2)) & (UInt16_3));
            let col21 = tmp_13.as_m31();
            trace_values[21].data[row_index] = col21;
            let tmp_14 = (((PackedUInt16::from_m31(col2)) >> (UInt16_2)) & (UInt16_511));
            let col22 = tmp_14.as_m31();
            trace_values[22].data[row_index] = col22;
            let tmp_15 = ((PackedUInt16::from_m31(col2)) >> (UInt16_11));
            let col23 = tmp_15.as_m31();
            trace_values[23].data[row_index] = col23;
            let tmp_16 = ((PackedUInt16::from_m31(col3)) & (UInt16_15));
            let col24 = tmp_16.as_m31();
            trace_values[24].data[row_index] = col24;
            let tmp_17 = (((PackedUInt16::from_m31(col3)) >> (UInt16_4)) & (UInt16_511));
            let col25 = tmp_17.as_m31();
            trace_values[25].data[row_index] = col25;
            let tmp_18 = ((PackedUInt16::from_m31(col3)) >> (UInt16_13));
            let col26 = tmp_18.as_m31();
            trace_values[26].data[row_index] = col26;
            sub_components_inputs.rangecheck_n_3_bits_7_2_5_inputs[0]
                .push([col20, col21, col23].into());
            lookup_data.rangecheck_n_3_bits_7_2_5[0].push([col20, col21, col23]);
            sub_components_inputs.rangecheck_n_2_bits_4_3_inputs[0].push([col24, col26].into());
            lookup_data.rangecheck_n_2_bits_4_3[0].push([col24, col26]);
            sub_components_inputs.memory_k_m31_v_m31_inputs[0].push(col0.into());
            let tmp_24 = memory_k_m31_v_m31_state.deduce_output(col0.into());
            let col27 = tmp_24;
            trace_values[27].data[row_index] = col27;
            lookup_data.memory_k_m31_v_m31[0].push([col0, col27]);
            lookup_data.memory_k_m31_v_felt252[0].push([
                col27,
                col19,
                ((col20) + ((col21) * (M31_128))),
                col22,
                ((col23) + ((col24) * (M31_32))),
                col25,
                ((col26)
                    + (((((((M31_0) + ((col4) * (M31_8))) + ((col5) * (M31_16)))
                        + ((col6) * (M31_32)))
                        + ((col7) * (M31_64)))
                        + ((col8) * (M31_128)))
                        + ((col9) * (M31_256)))),
                ((((((((((M31_0) + ((col10) * (M31_1))) + ((col11) * (M31_2)))
                    + ((col12) * (M31_4)))
                    + ((col13) * (M31_8)))
                    + ((col14) * (M31_16)))
                    + ((col15) * (M31_32)))
                    + ((col16) * (M31_64)))
                    + ((col17) * (M31_128)))
                    + ((col18) * (M31_256))),
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
            ]);
            lookup_data.verifyinstruction[0].push([
                col0, col1, col2, col3, col4, col5, col6, col7, col8, col9, col10, col11, col12,
                col13, col14, col15, col16, col17, col18,
            ]);
        });

    let trace = trace_values
        .into_iter()
        .map(|eval| {
            let domain = CanonicCoset::new(
                eval.len()
                    .checked_ilog2()
                    .expect("Input is not a power of 2!"),
            )
            .circle_domain();
            CircleEvaluation::<SimdBackend, M31, BitReversedOrder>::new(domain, eval)
        })
        .collect_vec();
    (trace, sub_components_inputs, lookup_data)
}

#[allow(non_snake_case)]
pub struct LookupData {
    pub memory_k_m31_v_felt252: [Vec<[PackedM31; 29]>; 1],
    pub memory_k_m31_v_m31: [Vec<[PackedM31; 2]>; 1],
    pub rangecheck_n_2_bits_4_3: [Vec<[PackedM31; 2]>; 1],
    pub rangecheck_n_3_bits_7_2_5: [Vec<[PackedM31; 3]>; 1],
    pub verifyinstruction: [Vec<[PackedM31; 19]>; 1],
}
impl LookupData {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            memory_k_m31_v_felt252: [Vec::with_capacity(capacity)],
            memory_k_m31_v_m31: [Vec::with_capacity(capacity)],
            rangecheck_n_2_bits_4_3: [Vec::with_capacity(capacity)],
            rangecheck_n_3_bits_7_2_5: [Vec::with_capacity(capacity)],
            verifyinstruction: [Vec::with_capacity(capacity)],
        }
    }
}

pub struct ClaimProver {
    pub claim: Claim,
    pub lookup_data: LookupData,
}
impl ClaimProver {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        memory_k_m31_v_felt252_lookup_elements: &memory_k_m31_v_felt252::ComponentLookupElements,
        memory_k_m31_v_m31_lookup_elements: &memory_k_m31_v_m31::ComponentLookupElements,
        rangecheck_n_2_bits_4_3_lookup_elements: &rangecheck_n_2_bits_4_3::ComponentLookupElements,
        rangecheck_n_3_bits_7_2_5_lookup_elements: &rangecheck_n_3_bits_7_2_5::ComponentLookupElements,
        verifyinstruction_lookup_elements: &verifyinstruction::ComponentLookupElements,
    ) -> InteractionClaim {
        let log_size = self.claim.n_calls.next_power_of_two().ilog2();
        let mut logup_gen = LogupTraceGenerator::new(log_size);

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.rangecheck_n_3_bits_7_2_5[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = rangecheck_n_3_bits_7_2_5_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.rangecheck_n_2_bits_4_3[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = rangecheck_n_2_bits_4_3_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.memory_k_m31_v_m31[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = memory_k_m31_v_m31_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.memory_k_m31_v_felt252[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = memory_k_m31_v_felt252_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.verifyinstruction[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = verifyinstruction_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, -PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
