#![allow(unused_imports)]
use stwo_prover::core::air::accumulation::DomainEvaluationAccumulator;
use stwo_prover::core::air::{Component, ComponentProver, ComponentTrace};
use stwo_prover::core::backend::simd::column::{BaseFieldVec, SecureFieldVec};
use stwo_prover::core::backend::simd::m31::PackedBaseField;
use stwo_prover::core::backend::simd::qm31::PackedSecureField;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{Column, ColumnOps};
use stwo_prover::core::constraints::coset_vanishing;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::fields::FieldOps;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::InteractionElements;

use super::component::WideFib__8;

impl ComponentProver<SimdBackend> for WideFib__8 {
    #[allow(unused_parens)]
    fn evaluate_constraint_quotients_on_domain(
        &self,
        _trace: &ComponentTrace<'_, SimdBackend>,
        _evaluation_accumulator: &mut DomainEvaluationAccumulator<SimdBackend>,
        _interaction_elements: &InteractionElements,
    ) {
        todo!()
    }
}
