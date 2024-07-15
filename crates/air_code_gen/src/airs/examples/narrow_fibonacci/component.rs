#![allow(unused_imports)]
use stwo_prover::core::air::accumulation::PointEvaluationAccumulator;
use stwo_prover::core::air::mask::fixed_mask_points;
use stwo_prover::core::air::Component;
use stwo_prover::core::circle::CirclePoint;
use stwo_prover::core::constraints::coset_vanishing;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::pcs::TreeVec;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::{ColumnVec, InteractionElements};

#[allow(non_camel_case_types)]
pub struct NarrowFib__20 {
    pub log_n_instances: u32,
}

impl Component for NarrowFib__20 {
    fn n_constraints(&self) -> usize {
        20
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_n_instances + 1
    }

    fn trace_log_degree_bounds(&self) -> TreeVec<Vec<u32>> {
        TreeVec(vec![vec![self.log_n_instances; 22], vec![]])
    }

    fn mask_points(
        &self,
        point: CirclePoint<SecureField>,
    ) -> TreeVec<ColumnVec<Vec<CirclePoint<SecureField>>>> {
        TreeVec(vec![
            fixed_mask_points(&vec![vec![0_usize]; 22], point),
            vec![],
        ])
    }

    #[allow(unused_parens)]
    fn evaluate_constraint_quotients_at_point(
        &self,
        point: CirclePoint<SecureField>,
        mask: &ColumnVec<Vec<SecureField>>,
        evaluation_accumulator: &mut PointEvaluationAccumulator,
        _interaction_elements: &InteractionElements,
    ) {
        let constraint_zero_domain = CanonicCoset::new(self.log_n_instances).coset;
        let denominator_inv = coset_vanishing(constraint_zero_domain, point).inverse();
        let numerator = (mask[2][0] - ((mask[0][0] * mask[0][0]) + (mask[1][0] * mask[1][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[3][0] - ((mask[1][0] * mask[1][0]) + (mask[2][0] * mask[2][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[4][0] - ((mask[2][0] * mask[2][0]) + (mask[3][0] * mask[3][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[5][0] - ((mask[3][0] * mask[3][0]) + (mask[4][0] * mask[4][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[6][0] - ((mask[4][0] * mask[4][0]) + (mask[5][0] * mask[5][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[7][0] - ((mask[5][0] * mask[5][0]) + (mask[6][0] * mask[6][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[8][0] - ((mask[6][0] * mask[6][0]) + (mask[7][0] * mask[7][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[9][0] - ((mask[7][0] * mask[7][0]) + (mask[8][0] * mask[8][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[10][0] - ((mask[8][0] * mask[8][0]) + (mask[9][0] * mask[9][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[11][0] - ((mask[9][0] * mask[9][0]) + (mask[10][0] * mask[10][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[12][0] - ((mask[10][0] * mask[10][0]) + (mask[11][0] * mask[11][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[13][0] - ((mask[11][0] * mask[11][0]) + (mask[12][0] * mask[12][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[14][0] - ((mask[12][0] * mask[12][0]) + (mask[13][0] * mask[13][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[15][0] - ((mask[13][0] * mask[13][0]) + (mask[14][0] * mask[14][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[16][0] - ((mask[14][0] * mask[14][0]) + (mask[15][0] * mask[15][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[17][0] - ((mask[15][0] * mask[15][0]) + (mask[16][0] * mask[16][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[18][0] - ((mask[16][0] * mask[16][0]) + (mask[17][0] * mask[17][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[19][0] - ((mask[17][0] * mask[17][0]) + (mask[18][0] * mask[18][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[20][0] - ((mask[18][0] * mask[18][0]) + (mask[19][0] * mask[19][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[21][0] - ((mask[19][0] * mask[19][0]) + (mask[20][0] * mask[20][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
    }

    fn n_interaction_phases(&self) -> u32 {
        1
    }

    fn interaction_element_ids(&self) -> Vec<String> {
        vec![]
    }
}
