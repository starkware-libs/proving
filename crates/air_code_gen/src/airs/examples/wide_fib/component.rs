#![allow(unused_imports)]
use stwo_prover::core::air::accumulation::PointEvaluationAccumulator;
use stwo_prover::core::air::mask::fixed_mask_points;
use stwo_prover::core::air::Component;
use stwo_prover::core::circle::CirclePoint;
use stwo_prover::core::constraints::coset_vanishing;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::pcs::TreeVec;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::{ColumnVec, InteractionElements};

#[allow(non_camel_case_types)]
pub struct WideFib_d7cf24d545e710f9 {
    pub log_n_instances: u32,
}

impl Component for WideFib_d7cf24d545e710f9 {
    fn n_constraints(&self) -> usize {
        8 * 3
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_n_instances + 1
    }

    fn trace_log_degree_bounds(&self) -> TreeVec<Vec<u32>> {
        TreeVec(vec![vec![self.log_n_instances; 18], vec![]])
    }

    fn mask_points(
        &self,
        point: CirclePoint<SecureField>,
    ) -> TreeVec<ColumnVec<Vec<CirclePoint<SecureField>>>> {
        TreeVec(vec![
            fixed_mask_points(&vec![vec![0_usize]; 18], point),
            vec![],
        ])
    }

    #[allow(unused_parens)]
    fn evaluate_constraint_quotients_at_point(
        &self,
        _point: CirclePoint<SecureField>,
        _mask: &ColumnVec<Vec<SecureField>>,
        _evaluation_accumulator: &mut PointEvaluationAccumulator,
        _interaction_elements: &InteractionElements,
    ) {
        todo!()
    }

    fn n_interaction_phases(&self) -> u32 {
        2
    }

    fn interaction_element_ids(&self) -> Vec<String> {
        vec![]
    }
}
