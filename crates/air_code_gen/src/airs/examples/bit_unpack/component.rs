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
pub struct BitUnpack__12 {
    pub log_n_instances: u32,
}

impl Component for BitUnpack__12 {
    fn n_constraints(&self) -> usize {
        13
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_n_instances + 1
    }

    fn trace_log_degree_bounds(&self) -> TreeVec<Vec<u32>> {
        TreeVec(vec![vec![self.log_n_instances; 13], vec![]])
    }

    fn mask_points(
        &self,
        point: CirclePoint<SecureField>,
    ) -> TreeVec<ColumnVec<Vec<CirclePoint<SecureField>>>> {
        TreeVec(vec![
            fixed_mask_points(&vec![vec![0_usize]; 13], point),
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
        let constraint_tmp_3 = (mask[0][0] - (mask[1][0] * M31::from(2)));
        let numerator = (constraint_tmp_3 * (constraint_tmp_3 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_5 = (mask[1][0] - (mask[2][0] * M31::from(2)));
        let numerator = (constraint_tmp_5 * (constraint_tmp_5 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_7 = (mask[2][0] - (mask[3][0] * M31::from(2)));
        let numerator = (constraint_tmp_7 * (constraint_tmp_7 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_9 = (mask[3][0] - (mask[4][0] * M31::from(2)));
        let numerator = (constraint_tmp_9 * (constraint_tmp_9 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_11 = (mask[4][0] - (mask[5][0] * M31::from(2)));
        let numerator = (constraint_tmp_11 * (constraint_tmp_11 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_13 = (mask[5][0] - (mask[6][0] * M31::from(2)));
        let numerator = (constraint_tmp_13 * (constraint_tmp_13 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_15 = (mask[6][0] - (mask[7][0] * M31::from(2)));
        let numerator = (constraint_tmp_15 * (constraint_tmp_15 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_17 = (mask[7][0] - (mask[8][0] * M31::from(2)));
        let numerator = (constraint_tmp_17 * (constraint_tmp_17 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_19 = (mask[8][0] - (mask[9][0] * M31::from(2)));
        let numerator = (constraint_tmp_19 * (constraint_tmp_19 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_21 = (mask[9][0] - (mask[10][0] * M31::from(2)));
        let numerator = (constraint_tmp_21 * (constraint_tmp_21 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_23 = (mask[10][0] - (mask[11][0] * M31::from(2)));
        let numerator = (constraint_tmp_23 * (constraint_tmp_23 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let constraint_tmp_25 = (mask[11][0] - (mask[12][0] * M31::from(2)));
        let numerator = (constraint_tmp_25 * (constraint_tmp_25 - M31::from(1)));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = mask[12][0];
        evaluation_accumulator.accumulate(numerator * denominator_inv);
    }

    fn n_interaction_phases(&self) -> u32 {
        1
    }

    fn interaction_element_ids(&self) -> Vec<String> {
        vec![]
    }
}
