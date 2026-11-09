// This file was created by the AIR team.

use crate::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 8;
pub const LOG_SIZE: u32 = 20;
pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

pub struct Eval {
    pub claim: Claim,
    pub common_lookup_elements: relations::CommonLookupElements,
}

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize, CairoDeserialize)]
pub struct Claim {}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let trace_log_sizes = vec![LOG_SIZE; N_TRACE_COLUMNS];
        let interaction_log_sizes = vec![LOG_SIZE; SECURE_EXTENSION_DEGREE * 4];
        TreeVec::new(vec![trace_log_sizes, interaction_log_sizes])
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize, CairoDeserialize)]
pub struct InteractionClaim {
    pub claimed_sum: SecureField,
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        LOG_SIZE
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let range_check_20_b_relation_id = E::F::from(relations::RANGE_CHECK_20_B_RELATION_ID);
        let range_check_20_c_relation_id = E::F::from(relations::RANGE_CHECK_20_C_RELATION_ID);
        let range_check_20_d_relation_id = E::F::from(relations::RANGE_CHECK_20_D_RELATION_ID);
        let range_check_20_e_relation_id = E::F::from(relations::RANGE_CHECK_20_E_RELATION_ID);
        let range_check_20_f_relation_id = E::F::from(relations::RANGE_CHECK_20_F_RELATION_ID);
        let range_check_20_g_relation_id = E::F::from(relations::RANGE_CHECK_20_G_RELATION_ID);
        let range_check_20_h_relation_id = E::F::from(relations::RANGE_CHECK_20_H_RELATION_ID);
        let range_check_20_relation_id = E::F::from(relations::RANGE_CHECK_20_RELATION_ID);
        let seq_20 = eval.get_preprocessed_column(PreProcessedColumnId { id: "seq_20".to_owned() });
        let multiplicity_0_col0 = eval.next_trace_mask();
        let multiplicity_1_col1 = eval.next_trace_mask();
        let multiplicity_2_col2 = eval.next_trace_mask();
        let multiplicity_3_col3 = eval.next_trace_mask();
        let multiplicity_4_col4 = eval.next_trace_mask();
        let multiplicity_5_col5 = eval.next_trace_mask();
        let multiplicity_6_col6 = eval.next_trace_mask();
        let multiplicity_7_col7 = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_0_col0.clone()),
            &[range_check_20_relation_id.clone(), seq_20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_1_col1.clone()),
            &[range_check_20_b_relation_id.clone(), seq_20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_2_col2.clone()),
            &[range_check_20_c_relation_id.clone(), seq_20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_3_col3.clone()),
            &[range_check_20_d_relation_id.clone(), seq_20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_4_col4.clone()),
            &[range_check_20_e_relation_id.clone(), seq_20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_5_col5.clone()),
            &[range_check_20_f_relation_id.clone(), seq_20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_6_col6.clone()),
            &[range_check_20_g_relation_id.clone(), seq_20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_7_col7.clone()),
            &[range_check_20_h_relation_id.clone(), seq_20.clone()],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};
    use stwo::core::fields::qm31::QM31;
    use stwo_constraint_framework::expr::ExprEvaluator;

    use super::*;

    #[test]
    fn range_check_20_constraints_regression() {
        let mut rng = SmallRng::seed_from_u64(0);
        let eval = Eval {
            claim: Claim {},
            common_lookup_elements: relations::CommonLookupElements::dummy(),
        };
        let expr_eval = eval.evaluate(ExprEvaluator::new());
        let assignment = expr_eval.random_assignment();

        let mut sum = QM31::zero();
        for c in expr_eval.constraints {
            sum += c.assign(&assignment) * rng.random::<QM31>();
        }

        constraints_regression_test_values::RANGE_CHECK_20.assert_debug_eq(&sum);
    }
}
