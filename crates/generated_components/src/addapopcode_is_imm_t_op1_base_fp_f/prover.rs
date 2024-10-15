#![allow(unused_parens)]
#![allow(unused_imports)]
use air_code_gen::code_gen::packed_types::*;
use air_infra::core::prover_types::*;
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
use crate::{memory_k_m31_v_felt252, memory_k_m31_v_m31, opcodes, verifyinstruction};

pub type InputType = PackedCasmState;

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn write_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        memory_k_m31_v_felt252_state: &mut memory_k_m31_v_felt252::ClaimGenerator,
        memory_k_m31_v_m31_state: &mut memory_k_m31_v_m31::ClaimGenerator,
        verifyinstruction_state: &mut verifyinstruction::ClaimGenerator,
    ) -> ClaimProver {
        let len = self.inputs.len();
        #[allow(unused_variables)]
        let (trace, sub_component_inputs, lookup_data) = write_trace_simd(
            self.inputs,
            memory_k_m31_v_felt252_state,
            memory_k_m31_v_m31_state,
        );
        sub_component_inputs
            .memory_k_m31_v_felt252_inputs
            .iter()
            .for_each(|inputs| {
                memory_k_m31_v_felt252_state.add_inputs(inputs);
            });
        sub_component_inputs
            .memory_k_m31_v_m31_inputs
            .iter()
            .for_each(|inputs| {
                memory_k_m31_v_m31_state.add_inputs(inputs);
            });
        sub_component_inputs
            .verifyinstruction_inputs
            .iter()
            .for_each(|inputs| {
                verifyinstruction_state.add_inputs(inputs);
            });

        tree_builder.extend_evals(trace);
        let claim = Claim {
            log_size: len.ilog2() + LOG_N_LANES,
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
    pub memory_k_m31_v_felt252_inputs: [Vec<memory_k_m31_v_felt252::InputType>; 2],
    pub memory_k_m31_v_m31_inputs: [Vec<memory_k_m31_v_m31::InputType>; 2],
    pub verifyinstruction_inputs: [Vec<verifyinstruction::InputType>; 1],
}
impl SubComponentInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            memory_k_m31_v_felt252_inputs: [
                Vec::with_capacity(capacity),
                Vec::with_capacity(capacity),
            ],
            memory_k_m31_v_m31_inputs: [Vec::with_capacity(capacity), Vec::with_capacity(capacity)],
            verifyinstruction_inputs: [Vec::with_capacity(capacity)],
        }
    }
}

pub fn write_trace_simd(
    inputs: Vec<InputType>,
    memory_k_m31_v_felt252_state: &mut memory_k_m31_v_felt252::ClaimGenerator,
    memory_k_m31_v_m31_state: &mut memory_k_m31_v_m31::ClaimGenerator,
) -> (
    Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>,
    SubComponentInputs,
    LookupData,
) {
    let n_trace_columns = 9;
    let mut trace_values = (0..n_trace_columns)
        .map(|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES))
        .collect_vec();
    let mut lookup_data = LookupData::with_capacity(inputs.len());
    let mut sub_components_inputs = SubComponentInputs::with_capacity(inputs.len());
    inputs.into_iter().enumerate().for_each(|(i, input)| {
        write_trace_row(
            &mut trace_values,
            input,
            i,
            &mut sub_components_inputs,
            &mut lookup_data,
            memory_k_m31_v_felt252_state,
            memory_k_m31_v_m31_state,
        );
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
#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
fn write_trace_row(
    dst: &mut [Col<SimdBackend, M31>],
    addapopcode_is_imm_t_op1_base_fp_f_input: InputType,
    row_index: usize,
    sub_component_inputs: &mut SubComponentInputs,
    lookup_data: &mut LookupData,
    memory_k_m31_v_felt252_state: &mut memory_k_m31_v_felt252::ClaimGenerator,
    memory_k_m31_v_m31_state: &mut memory_k_m31_v_m31::ClaimGenerator,
) {
    let tmp_0 = addapopcode_is_imm_t_op1_base_fp_f_input;
    let col0 = tmp_0.pc;
    dst[0].data[row_index] = col0;
    let col1 = tmp_0.ap;
    dst[1].data[row_index] = col1;
    let col2 = tmp_0.fp;
    dst[2].data[row_index] = col2;

    // DecodeInstruction_a14b71db698d77c8.
    sub_component_inputs.memory_k_m31_v_m31_inputs[0].push(col0.into());
    let tmp_29 = memory_k_m31_v_m31_state.deduce_output(col0.into());
    sub_component_inputs.memory_k_m31_v_felt252_inputs[0].push(tmp_29.into());
    let tmp_30 = memory_k_m31_v_felt252_state.deduce_output(tmp_29.into());
    sub_component_inputs.verifyinstruction_inputs[0].push(
        (
            col0,
            [
                PackedM31::broadcast(M31::from(32767).into()),
                PackedM31::broadcast(M31::from(32767).into()),
                PackedM31::broadcast(M31::from(32769).into()),
            ],
            [
                PackedM31::broadcast(M31::from(1).into()),
                PackedM31::broadcast(M31::from(1).into()),
                PackedM31::broadcast(M31::from(1).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(1).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
                PackedM31::broadcast(M31::from(0).into()),
            ],
        )
            .into(),
    );
    lookup_data.verifyinstruction[0].push([
        col0,
        PackedM31::broadcast(M31::from(32767).into()),
        PackedM31::broadcast(M31::from(32767).into()),
        PackedM31::broadcast(M31::from(32769).into()),
        PackedM31::broadcast(M31::from(1).into()),
        PackedM31::broadcast(M31::from(1).into()),
        PackedM31::broadcast(M31::from(1).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(1).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
    ]);

    // ReadSmall.
    sub_component_inputs.memory_k_m31_v_m31_inputs[1]
        .push(((col0) + (PackedM31::broadcast(M31::from(1).into()))).into());
    let tmp_38 = memory_k_m31_v_m31_state
        .deduce_output(((col0) + (PackedM31::broadcast(M31::from(1).into()))).into());
    let col3 = tmp_38;
    dst[3].data[row_index] = col3; // id.
    lookup_data.memory_k_m31_v_m31[0]
        .push([((col0) + (PackedM31::broadcast(M31::from(1).into()))), col3]);
    sub_component_inputs.memory_k_m31_v_felt252_inputs[1].push(col3.into());
    let tmp_39 = memory_k_m31_v_felt252_state.deduce_output(col3.into());

    // CondDecodeSmallSign.
    let tmp_40 = tmp_39
        .get_m31(27)
        .eq(PackedM31::broadcast(M31::from(256).into()));
    let col4 = tmp_40.as_m31();
    dst[4].data[row_index] = col4; // msb.
    let tmp_41 = tmp_39
        .get_m31(20)
        .eq(PackedM31::broadcast(M31::from(511).into()));
    let col5 = tmp_41.as_m31();
    dst[5].data[row_index] = col5; // mid_limbs_set.

    let col6 = tmp_39.get_m31(0);
    dst[6].data[row_index] = col6; // limb_0.
    let col7 = tmp_39.get_m31(1);
    dst[7].data[row_index] = col7; // limb_1.
    let col8 = tmp_39.get_m31(2);
    dst[8].data[row_index] = col8; // limb_2.
    lookup_data.memory_k_m31_v_felt252[0].push([
        col3,
        col6,
        col7,
        col8,
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        ((col5) * (PackedM31::broadcast(M31::from(511).into()))),
        (((PackedM31::broadcast(M31::from(136).into())) * (col4)) - (col5)),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        PackedM31::broadcast(M31::from(0).into()),
        ((col4) * (PackedM31::broadcast(M31::from(256).into()))),
    ]);

    lookup_data.opcodes[0].push([col0, col1, col2]);
    lookup_data.opcodes[1].push([
        ((col0) + (PackedM31::broadcast(M31::from(2).into()))),
        ((col1)
            + (((((col8) * (PackedM31::broadcast(M31::from(262144).into())))
                + (((col7) * (PackedM31::broadcast(M31::from(512).into()))) + (col6)))
                - (col4))
                - ((PackedM31::broadcast(M31::from(134217728).into())) * (col5)))),
        col2,
    ]);
}

#[allow(non_snake_case)]
pub struct LookupData {
    pub memory_k_m31_v_felt252: [Vec<[PackedM31; 29]>; 1],
    pub memory_k_m31_v_m31: [Vec<[PackedM31; 2]>; 1],
    pub verifyinstruction: [Vec<[PackedM31; 19]>; 1],
    pub opcodes: [Vec<[PackedM31; 3]>; 2],
}
impl LookupData {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            memory_k_m31_v_felt252: [Vec::with_capacity(capacity)],
            memory_k_m31_v_m31: [Vec::with_capacity(capacity)],
            verifyinstruction: [Vec::with_capacity(capacity)],
            opcodes: [Vec::with_capacity(capacity), Vec::with_capacity(capacity)],
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
        verifyinstruction_lookup_elements: &verifyinstruction::ComponentLookupElements,
        opcodes_lookup_elements: &opcodes::ComponentLookupElements,
    ) -> InteractionClaim {
        let log_size = self.claim.log_size;
        let mut logup_gen = LogupTraceGenerator::new(log_size);

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.verifyinstruction[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = verifyinstruction_lookup_elements.combine(lookup_values);
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
        let lookup_row = &self.lookup_data.opcodes[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = opcodes_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.opcodes[1];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = opcodes_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, -PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
