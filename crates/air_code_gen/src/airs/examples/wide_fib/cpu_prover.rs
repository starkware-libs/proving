#![allow(unused_imports)]
use num_traits::identities::Zero;
use stwo_prover::core::air::accumulation::DomainEvaluationAccumulator;
use stwo_prover::core::air::{Component, ComponentProver, ComponentTrace};
use stwo_prover::core::backend::{Column, CpuBackend};
use stwo_prover::core::constraints::coset_vanishing;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::utils::bit_reverse;
use stwo_prover::core::InteractionElements;

use super::component::WideFib__8;

impl ComponentProver<CpuBackend> for WideFib__8 {
    #[allow(unused_parens)]
    fn evaluate_constraint_quotients_on_domain(
        &self,
        _trace: &ComponentTrace<'_, CpuBackend>,
        _evaluation_accumulator: &mut DomainEvaluationAccumulator<CpuBackend>,
        _interaction_elements: &InteractionElements,
    ) {
        todo!()
    }
}
