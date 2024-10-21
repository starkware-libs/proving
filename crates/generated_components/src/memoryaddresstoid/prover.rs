#![allow(unused_variables)]

use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleChannel;

use super::{Claim, ComponentLookupElements, InteractionClaim};

pub type InputType = PackedM31;
pub type LookupFelts = [PackedM31; 2];
pub struct ClaimGenerator {}
impl ClaimGenerator {
    pub fn deduce_output(&self, input: PackedM31) -> PackedM31 {
        todo!()
    }

    pub fn add_inputs(&mut self, addresses: &[InputType]) {
        todo!()
    }

    pub fn write_trace(
        &mut self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
    ) -> (Claim, ClaimProver) {
        todo!()
    }
}

#[derive(Debug)]
pub struct ClaimProver {}
impl ClaimProver {
    pub fn with_capacity(capacity: usize) -> Self {
        todo!()
    }

    pub fn write_interaction_trace(
        &self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
        lookup_elements: &ComponentLookupElements,
    ) -> InteractionClaim {
        todo!()
    }
}
