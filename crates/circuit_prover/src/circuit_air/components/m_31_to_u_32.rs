// This file was created by the AIR team.

use crate::circuit_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 4;
pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "Gate", uses: 1 },
    RelationUse { relation_id: "RangeCheck_16", uses: 3 },
];

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
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 3];
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
        let M31_1008385708 = E::F::from(M31::from(1008385708));
        let M31_32767 = E::F::from(M31::from(32767));
        let M31_378353459 = E::F::from(M31::from(378353459));
        let M31_65536 = E::F::from(M31::from(65536));
        let m31_to_u32_input_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "m31_to_u32_input_addr".to_owned(),
        });
        let m31_to_u32_output_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "m31_to_u32_output_addr".to_owned(),
        });
        let m31_to_u32_multiplicity = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "m31_to_u32_multiplicity".to_owned(),
        });
        let input_m31_col0 = eval.next_trace_mask();
        let input_u32_limb_0_col1 = eval.next_trace_mask();
        let input_u32_limb_1_col2 = eval.next_trace_mask();
        let inv_or_one_col3 = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[M31_1008385708.clone(), input_u32_limb_0_col1.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[M31_1008385708.clone(), input_u32_limb_1_col2.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[M31_1008385708.clone(), (M31_32767.clone() - input_u32_limb_1_col2.clone())],
        ));

        // input is zero then limb_low is zero.
        eval.add_constraint(
            (((input_m31_col0.clone() * inv_or_one_col3.clone()) - M31_1.clone())
                * input_u32_limb_0_col1.clone()),
        );
        // input reconstruction.
        eval.add_constraint(
            (input_m31_col0.clone()
                - (input_u32_limb_0_col1.clone()
                    + (input_u32_limb_1_col2.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[M31_378353459.clone(), m31_to_u32_input_addr.clone(), input_m31_col0.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(m31_to_u32_multiplicity.clone()),
            &[
                M31_378353459.clone(),
                m31_to_u32_output_addr.clone(),
                input_u32_limb_0_col1.clone(),
                input_u32_limb_1_col2.clone(),
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
    fn m_31_to_u_32_constraints_regression() {
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

        constraints_regression_test_values::M_31_TO_U_32.assert_debug_eq(&sum);
    }
}
