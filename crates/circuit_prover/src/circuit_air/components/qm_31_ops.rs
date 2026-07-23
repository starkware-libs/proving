// This file was created by the AIR team.

use crate::circuit_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 12;
pub const RELATION_USES_PER_ROW: [RelationUse; 1] = [RelationUse { relation_id: "Gate", uses: 2 }];

pub struct Eval {
    pub claim: Claim,
    pub common_lookup_elements: relations::CommonLookupElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub log_size: u32,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let trace_log_sizes = vec![self.log_size; N_TRACE_COLUMNS];
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 2];
        TreeVec::new(vec![trace_log_sizes, interaction_log_sizes])
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct InteractionClaim {
    pub claimed_sum: SecureField,
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.claim.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let M31_1 = E::F::from(M31::from(1));
        let M31_2 = E::F::from(M31::from(2));
        let M31_378353459 = E::F::from(M31::from(378353459));
        let qm31_ops_add_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() });
        let qm31_ops_mul_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() });
        let qm31_ops_pointwise_mul_flag = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_pointwise_mul_flag".to_owned(),
        });
        let qm31_ops_sub_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() });
        let qm31_ops_in0_address = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_in0_address".to_owned(),
        });
        let qm31_ops_in1_address = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_in1_address".to_owned(),
        });
        let qm31_ops_out_address = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_out_address".to_owned(),
        });
        let qm31_ops_mults =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_mults".to_owned() });
        let input_op0_limb0_col0 = eval.next_trace_mask();
        let input_op0_limb1_col1 = eval.next_trace_mask();
        let input_op0_limb2_col2 = eval.next_trace_mask();
        let input_op0_limb3_col3 = eval.next_trace_mask();
        let input_op1_limb0_col4 = eval.next_trace_mask();
        let input_op1_limb1_col5 = eval.next_trace_mask();
        let input_op1_limb2_col6 = eval.next_trace_mask();
        let input_op1_limb3_col7 = eval.next_trace_mask();
        let input_dst_limb0_col8 = eval.next_trace_mask();
        let input_dst_limb1_col9 = eval.next_trace_mask();
        let input_dst_limb2_col10 = eval.next_trace_mask();
        let input_dst_limb3_col11 = eval.next_trace_mask();

        // all flags sum to 1.
        eval.add_constraint(
            ((((qm31_ops_add_flag.clone() + qm31_ops_sub_flag.clone())
                + qm31_ops_mul_flag.clone())
                + qm31_ops_pointwise_mul_flag.clone())
                - M31_1.clone()),
        );
        // add_flag is a bit.
        eval.add_constraint(
            (qm31_ops_add_flag.clone() * (qm31_ops_add_flag.clone() - M31_1.clone())),
        );
        // sub_flag is a bit.
        eval.add_constraint(
            (qm31_ops_sub_flag.clone() * (qm31_ops_sub_flag.clone() - M31_1.clone())),
        );
        // mul_flag is a bit.
        eval.add_constraint(
            (qm31_ops_mul_flag.clone() * (qm31_ops_mul_flag.clone() - M31_1.clone())),
        );
        // pointwise_mul_flag is a bit.
        eval.add_constraint(
            (qm31_ops_pointwise_mul_flag.clone()
                * (qm31_ops_pointwise_mul_flag.clone() - M31_1.clone())),
        );
        eval.add_constraint(
            (input_dst_limb0_col8.clone()
                - (((((((((input_op0_limb0_col0.clone() * input_op1_limb0_col4.clone())
                    - (input_op0_limb1_col1.clone() * input_op1_limb1_col5.clone()))
                    + (M31_2.clone()
                        * ((input_op0_limb2_col2.clone() * input_op1_limb2_col6.clone())
                            - (input_op0_limb3_col3.clone()
                                * input_op1_limb3_col7.clone()))))
                    - (input_op0_limb2_col2.clone() * input_op1_limb3_col7.clone()))
                    - (input_op0_limb3_col3.clone() * input_op1_limb2_col6.clone()))
                    * qm31_ops_mul_flag.clone())
                    + ((input_op0_limb0_col0.clone() + input_op1_limb0_col4.clone())
                        * qm31_ops_add_flag.clone()))
                    + ((input_op0_limb0_col0.clone() - input_op1_limb0_col4.clone())
                        * qm31_ops_sub_flag.clone()))
                    + ((input_op0_limb0_col0.clone() * input_op1_limb0_col4.clone())
                        * qm31_ops_pointwise_mul_flag.clone()))),
        );
        eval.add_constraint(
            (input_dst_limb1_col9.clone()
                - (((((((((input_op0_limb0_col0.clone() * input_op1_limb1_col5.clone())
                    + (input_op0_limb1_col1.clone() * input_op1_limb0_col4.clone()))
                    + (M31_2.clone()
                        * ((input_op0_limb2_col2.clone() * input_op1_limb3_col7.clone())
                            + (input_op0_limb3_col3.clone()
                                * input_op1_limb2_col6.clone()))))
                    + (input_op0_limb2_col2.clone() * input_op1_limb2_col6.clone()))
                    - (input_op0_limb3_col3.clone() * input_op1_limb3_col7.clone()))
                    * qm31_ops_mul_flag.clone())
                    + ((input_op0_limb1_col1.clone() + input_op1_limb1_col5.clone())
                        * qm31_ops_add_flag.clone()))
                    + ((input_op0_limb1_col1.clone() - input_op1_limb1_col5.clone())
                        * qm31_ops_sub_flag.clone()))
                    + ((input_op0_limb1_col1.clone() * input_op1_limb1_col5.clone())
                        * qm31_ops_pointwise_mul_flag.clone()))),
        );
        eval.add_constraint(
            (input_dst_limb2_col10.clone()
                - ((((((((input_op0_limb0_col0.clone() * input_op1_limb2_col6.clone())
                    - (input_op0_limb1_col1.clone() * input_op1_limb3_col7.clone()))
                    + (input_op0_limb2_col2.clone() * input_op1_limb0_col4.clone()))
                    - (input_op0_limb3_col3.clone() * input_op1_limb1_col5.clone()))
                    * qm31_ops_mul_flag.clone())
                    + ((input_op0_limb2_col2.clone() + input_op1_limb2_col6.clone())
                        * qm31_ops_add_flag.clone()))
                    + ((input_op0_limb2_col2.clone() - input_op1_limb2_col6.clone())
                        * qm31_ops_sub_flag.clone()))
                    + ((input_op0_limb2_col2.clone() * input_op1_limb2_col6.clone())
                        * qm31_ops_pointwise_mul_flag.clone()))),
        );
        eval.add_constraint(
            (input_dst_limb3_col11.clone()
                - ((((((((input_op0_limb0_col0.clone() * input_op1_limb3_col7.clone())
                    + (input_op0_limb1_col1.clone() * input_op1_limb2_col6.clone()))
                    + (input_op0_limb2_col2.clone() * input_op1_limb1_col5.clone()))
                    + (input_op0_limb3_col3.clone() * input_op1_limb0_col4.clone()))
                    * qm31_ops_mul_flag.clone())
                    + ((input_op0_limb3_col3.clone() + input_op1_limb3_col7.clone())
                        * qm31_ops_add_flag.clone()))
                    + ((input_op0_limb3_col3.clone() - input_op1_limb3_col7.clone())
                        * qm31_ops_sub_flag.clone()))
                    + ((input_op0_limb3_col3.clone() * input_op1_limb3_col7.clone())
                        * qm31_ops_pointwise_mul_flag.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                qm31_ops_in0_address.clone(),
                input_op0_limb0_col0.clone(),
                input_op0_limb1_col1.clone(),
                input_op0_limb2_col2.clone(),
                input_op0_limb3_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                qm31_ops_in1_address.clone(),
                input_op1_limb0_col4.clone(),
                input_op1_limb1_col5.clone(),
                input_op1_limb2_col6.clone(),
                input_op1_limb3_col7.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(qm31_ops_mults.clone()),
            &[
                M31_378353459.clone(),
                qm31_ops_out_address.clone(),
                input_dst_limb0_col8.clone(),
                input_dst_limb1_col9.clone(),
                input_dst_limb2_col10.clone(),
                input_dst_limb3_col11.clone(),
            ],
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
    fn qm_31_ops_constraints_regression() {
        let mut rng = SmallRng::seed_from_u64(0);
        let eval = Eval {
            claim: Claim { log_size: 4 },
            common_lookup_elements: relations::CommonLookupElements::dummy(),
        };
        let expr_eval = eval.evaluate(ExprEvaluator::new());
        let assignment = expr_eval.random_assignment();

        let mut sum = QM31::zero();
        for c in expr_eval.constraints {
            sum += c.assign(&assignment) * rng.random::<QM31>();
        }

        constraints_regression_test_values::QM_31_OPS.assert_debug_eq(&sum);
    }
}
