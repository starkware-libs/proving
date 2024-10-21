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
use crate::{memoryaddresstoid, memoryidtobig, opcodes, verifyinstruction};

pub type InputType = PackedCasmState;

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn write_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        memoryaddresstoid_state: &mut memoryaddresstoid::ClaimGenerator,
        memoryidtobig_state: &mut memoryidtobig::ClaimGenerator,
        verifyinstruction_state: &mut verifyinstruction::ClaimGenerator,
    ) -> ClaimProver {
        let len = self.inputs.len();
        #[allow(unused_variables)]
        let (trace, sub_components_inputs, lookup_data) =
            write_trace_simd(self.inputs, memoryaddresstoid_state, memoryidtobig_state);
        sub_components_inputs
            .memoryaddresstoid_inputs
            .iter()
            .for_each(|inputs| {
                memoryaddresstoid_state.add_inputs(inputs);
            });
        sub_components_inputs
            .memoryidtobig_inputs
            .iter()
            .for_each(|inputs| {
                memoryidtobig_state.add_inputs(inputs);
            });
        sub_components_inputs
            .verifyinstruction_inputs
            .iter()
            .for_each(|inputs| {
                verifyinstruction_state.add_inputs(inputs);
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
    pub memoryaddresstoid_inputs: [Vec<memoryaddresstoid::InputType>; 2],
    pub memoryidtobig_inputs: [Vec<memoryidtobig::InputType>; 2],
    pub verifyinstruction_inputs: [Vec<verifyinstruction::InputType>; 1],
}
impl SubComponentInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            memoryaddresstoid_inputs: [Vec::with_capacity(capacity), Vec::with_capacity(capacity)],
            memoryidtobig_inputs: [Vec::with_capacity(capacity), Vec::with_capacity(capacity)],
            verifyinstruction_inputs: [Vec::with_capacity(capacity)],
        }
    }
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
pub fn write_trace_simd(
    inputs: Vec<InputType>,
    memoryaddresstoid_state: &mut memoryaddresstoid::ClaimGenerator,
    memoryidtobig_state: &mut memoryidtobig::ClaimGenerator,
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
    #[allow(unused_mut)]
    let mut sub_components_inputs = SubComponentInputs::with_capacity(inputs.len());

    let M31_0 = PackedM31::broadcast(M31::from(0));
    let M31_1 = PackedM31::broadcast(M31::from(1));
    let M31_134217728 = PackedM31::broadcast(M31::from(134217728));
    let M31_136 = PackedM31::broadcast(M31::from(136));
    let M31_2 = PackedM31::broadcast(M31::from(2));
    let M31_256 = PackedM31::broadcast(M31::from(256));
    let M31_262144 = PackedM31::broadcast(M31::from(262144));
    let M31_32767 = PackedM31::broadcast(M31::from(32767));
    let M31_32769 = PackedM31::broadcast(M31::from(32769));
    let M31_511 = PackedM31::broadcast(M31::from(511));
    let M31_512 = PackedM31::broadcast(M31::from(512));

    inputs.into_iter().enumerate().for_each(
        |(row_index, addapopcode_is_imm_t_op1_base_fp_f_input)| {
            let tmp_0 = addapopcode_is_imm_t_op1_base_fp_f_input;
            let col0 = tmp_0.pc;
            trace_values[0].data[row_index] = col0;
            let col1 = tmp_0.ap;
            trace_values[1].data[row_index] = col1;
            let col2 = tmp_0.fp;
            trace_values[2].data[row_index] = col2;
            sub_components_inputs.memoryaddresstoid_inputs[0].push(col0.into());
            let tmp_41 = memoryaddresstoid_state.deduce_output(col0.into());
            sub_components_inputs.memoryidtobig_inputs[0].push(tmp_41.into());
            let tmp_42 = memoryidtobig_state.deduce_output(tmp_41.into());
            sub_components_inputs.verifyinstruction_inputs[0].push(
                (
                    col0,
                    [M31_32767, M31_32767, M31_32769],
                    [
                        M31_1, M31_1, M31_1, M31_0, M31_0, M31_0, M31_0, M31_0, M31_0, M31_0,
                        M31_1, M31_0, M31_0, M31_0, M31_0,
                    ],
                )
                    .into(),
            );
            lookup_data.verifyinstruction[0].push([
                col0, M31_32767, M31_32767, M31_32769, M31_1, M31_1, M31_1, M31_0, M31_0, M31_0,
                M31_0, M31_0, M31_0, M31_0, M31_1, M31_0, M31_0, M31_0, M31_0,
            ]);
            sub_components_inputs.memoryaddresstoid_inputs[1].push(((col0) + (M31_1)).into());
            let tmp_50 = memoryaddresstoid_state.deduce_output(((col0) + (M31_1)).into());
            let col3 = tmp_50;
            trace_values[3].data[row_index] = col3;
            lookup_data.memoryaddresstoid[0].push([((col0) + (M31_1)), col3]);
            sub_components_inputs.memoryidtobig_inputs[1].push(col3.into());
            let tmp_51 = memoryidtobig_state.deduce_output(col3.into());
            let tmp_52 = tmp_51.get_m31(27).eq(M31_256);
            let col4 = tmp_52.as_m31();
            trace_values[4].data[row_index] = col4;
            let tmp_53 = tmp_51.get_m31(20).eq(M31_511);
            let col5 = tmp_53.as_m31();
            trace_values[5].data[row_index] = col5;
            let col6 = tmp_51.get_m31(0);
            trace_values[6].data[row_index] = col6;
            let col7 = tmp_51.get_m31(1);
            trace_values[7].data[row_index] = col7;
            let col8 = tmp_51.get_m31(2);
            trace_values[8].data[row_index] = col8;
            lookup_data.memoryidtobig[0].push([
                col3,
                col6,
                col7,
                col8,
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                ((col5) * (M31_511)),
                (((M31_136) * (col4)) - (col5)),
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                ((col4) * (M31_256)),
            ]);
            lookup_data.opcodes[0].push([col0, col1, col2]);
            lookup_data.opcodes[1].push([
                ((col0) + (M31_2)),
                ((col1)
                    + (((((col8) * (M31_262144)) + (((col7) * (M31_512)) + (col6))) - (col4))
                        - ((M31_134217728) * (col5)))),
                col2,
            ]);
        },
    );

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
    pub memoryaddresstoid: [Vec<[PackedM31; 2]>; 1],
    pub memoryidtobig: [Vec<[PackedM31; 29]>; 1],
    pub verifyinstruction: [Vec<[PackedM31; 19]>; 1],
    pub opcodes: [Vec<[PackedM31; 3]>; 2],
}
impl LookupData {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            memoryaddresstoid: [Vec::with_capacity(capacity)],
            memoryidtobig: [Vec::with_capacity(capacity)],
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
        memoryaddresstoid_lookup_elements: &memoryaddresstoid::ComponentLookupElements,
        memoryidtobig_lookup_elements: &memoryidtobig::ComponentLookupElements,
        verifyinstruction_lookup_elements: &verifyinstruction::ComponentLookupElements,
        opcodes_lookup_elements: &opcodes::ComponentLookupElements,
    ) -> InteractionClaim {
        let log_size = self.claim.n_calls.next_power_of_two().ilog2();
        let mut logup_gen = LogupTraceGenerator::new(log_size);

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.verifyinstruction[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = verifyinstruction_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.memoryaddresstoid[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = memoryaddresstoid_lookup_elements.combine(lookup_values);
            col_gen.write_frac(i, PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        let lookup_row = &self.lookup_data.memoryidtobig[0];
        for (i, lookup_values) in lookup_row.iter().enumerate() {
            let denom = memoryidtobig_lookup_elements.combine(lookup_values);
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
