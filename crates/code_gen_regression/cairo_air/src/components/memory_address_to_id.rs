#![allow(unused_variables)]
use stwo_constraint_framework::{relation, EvalAtRow, FrameworkEval};
use stwo_prover::core::channel::Channel;
use stwo_prover::core::pcs::TreeVec;

pub const N_M31_IN_FELT252: usize = 28;
pub const MULTIPLICITY_COLUMN_OFFSET: usize = N_M31_IN_FELT252 + 1;
// TODO(AlonH): Make memory size configurable.
pub const N_MEMORY_COLUMNS: usize = N_M31_IN_FELT252 + 2;
pub const LOG_MEMORY_ADDRESS_BOUND: u32 = 7;
pub const MEMORY_ADDRESS_BOUND: usize = 1 << LOG_MEMORY_ADDRESS_BOUND;
pub const N_LOGUP_POWERS: usize = N_MEMORY_COLUMNS + 1;
pub const N_BITS_PER_FELT: usize = 9;

relation!(RelationElements, N_MEMORY_COLUMNS);

/// Addresses are continuous and start from 0.
/// Values are Felt252 stored as `N_M31_IN_FELT252` M31 values (each value containing 9 bits).
#[derive(Clone)]
pub struct MemoryComponent {}
impl MemoryComponent {
    pub const fn n_columns(&self) -> usize {
        todo!()
    }
    pub fn new(
        claim: Claim,
        lookup_elements: RelationElements,
        interaction_claim: InteractionClaim,
    ) -> Self {
        todo!()
    }
}

impl FrameworkEval for MemoryComponent {
    fn log_size(&self) -> u32 {
        todo!()
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        todo!()
    }

    fn evaluate<E: EvalAtRow>(&self, eval: E) -> E {
        todo!()
    }
}

#[derive(Clone)]
pub struct Claim {
    pub log_address_bound: u32,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        todo!()
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        todo!()
    }
}

#[derive(Clone)]
pub struct InteractionClaim {}
impl InteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        todo!()
    }
}
