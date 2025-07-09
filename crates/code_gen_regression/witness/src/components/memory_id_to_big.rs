#![allow(unused_variables)]

use cairo_air::components::memory_id_to_big::{Claim, InteractionClaim, RelationElements};
use stwo::core::fields::m31::M31;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleChannel;
use stwo::prover::backend::simd::m31::PackedM31;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::TreeBuilder;
use stwo_cairo_common::memory::N_M31_IN_FELT252;
use stwo_cairo_common::prover_types::simd::PackedFelt252;

pub type InputType = M31;
pub type PackedInputType = PackedM31;
pub type LookupFelts = [PackedM31; N_M31_IN_FELT252 + 1];
pub struct ClaimGenerator {}
impl ClaimGenerator {
    pub fn deduce_output(&self, input: PackedM31) -> PackedFelt252 {
        todo!()
    }

    pub fn add_packed_inputs(&self, addresses: &[PackedInputType]) {
        todo!()
    }

    pub fn add_packed_input(&self, input: &PackedInputType) {
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
