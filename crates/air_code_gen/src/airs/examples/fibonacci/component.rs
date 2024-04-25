use stwo_prover::core::air::accumulation::PointEvaluationAccumulator;
use stwo_prover::core::air::mask::fixed_mask_points;
use stwo_prover::core::air::Component;
use stwo_prover::core::circle::CirclePoint;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::ColumnVec;

#[allow(non_camel_case_types)]
pub struct Fib__1000 {
    pub log_n_instances: u32,
}

impl Component for Fib__1000 {
    fn n_constraints(&self) -> usize {
        998
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_n_instances + 1
    }

    fn trace_log_degree_bounds(&self) -> Vec<u32> {
        vec![self.log_n_instances; 999]
    }

    fn mask_points(
        &self,
        point: CirclePoint<SecureField>,
    ) -> ColumnVec<Vec<CirclePoint<SecureField>>> {
        fixed_mask_points(&vec![vec![0_usize]; 999], point)
    }

    fn evaluate_constraint_quotients_at_point(
        &self,
        _point: CirclePoint<SecureField>,
        _mask: &ColumnVec<Vec<SecureField>>,
        _evaluation_accumulator: &mut PointEvaluationAccumulator,
    ) {
        unimplemented!()
    }
}
