#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
use prover_types::cpu::*;
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

use super::component::{Claim, InteractionClaim, RelationElements};
use crate::components::range_check_7_2_5;

pub type InputType = [M31; 3];
pub type PackedInputType = [PackedM31; 3];

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn write_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
    ) -> InteractionClaimGenerator {
        todo!()
    }

    pub fn add_inputs(&mut self, inputs: &[InputType]) {
        self.inputs.extend(inputs);
    }
}

#[allow(non_snake_case)]
pub struct SubComponentInputs {}
impl SubComponentInputs {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {}
    }
}

pub fn write_trace_simd(
    inputs: Vec<InputType>,
) -> (
    Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>,
    SubComponentInputs,
    LookupData,
) {
    todo!()
}
#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
fn write_trace_row(
    dst: &mut [Col<SimdBackend, M31>],
    range_check_n_3_bits_7_2_5_input: InputType,
    row_index: usize,
    sub_component_inputs: &mut SubComponentInputs,
    lookup_data: &mut LookupData,
) {
}

#[allow(non_snake_case)]
pub struct LookupData {
    pub range_check_n_3_bits_7_2_5: [Vec<Vec<PackedM31>>; 1],
}
impl LookupData {
    #[allow(unused_variables)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            range_check_n_3_bits_7_2_5: [Vec::with_capacity(capacity)],
        }
    }
}

pub struct InteractionClaimGenerator {
    pub claim: Claim,
    pub lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        range_check_n_3_bits_7_2_5_lookup_elements: &range_check_7_2_5::RelationElements,
    ) -> InteractionClaim {
        todo!()
    }
}
