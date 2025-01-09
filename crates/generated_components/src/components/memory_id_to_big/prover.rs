#![allow(unused_variables)]

use prover_types::simd::PackedFelt252;
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleChannel;

use super::component::N_M31_IN_FELT252;
use super::{Claim, InteractionClaim, RelationElements};

pub type InputType = M31;
pub type PackedInputType = PackedM31;
pub type LookupFelts = [PackedM31; N_M31_IN_FELT252 + 1];
pub struct ClaimGenerator {}
impl ClaimGenerator {
    pub fn deduce_output(&self, input: PackedM31) -> PackedFelt252 {
        todo!()
    }

    pub fn add_inputs(&self, addresses: &[InputType]) {
        todo!()
    }

    pub fn write_trace(
        &mut self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
    ) -> (Claim, InteractionClaimGenerator) {
        todo!()
    }
}

#[derive(Debug)]
pub struct InteractionClaimGenerator {
    pub adresses_and_values: [Vec<PackedM31>; N_M31_IN_FELT252 + 1],
    pub multiplicities: Vec<PackedM31>,
}
impl InteractionClaimGenerator {
    pub fn with_capacity(capacity: usize) -> Self {
        todo!()
    }

    pub fn write_interaction_trace(
        &self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        lookup_elements: &RelationElements,
    ) -> InteractionClaim {
        todo!()
    }
}
