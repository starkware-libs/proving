// This file was created by the AIR team.

use subroutines::triple_sum_32::TripleSum32;
use subroutines::verify_triple_sum_32::VerifyTripleSum32;
use subroutines::verify_xor_rot_32_r_7::VerifyXorRot32R7;
use subroutines::verify_xor_rot_32_r_8::VerifyXorRot32R8;
use subroutines::xor_rot_32_r_12::XorRot32R12;
use subroutines::xor_rot_32_r_16::XorRot32R16;

use crate::circuit_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 52;
pub const RELATION_USES_PER_ROW: [RelationUse; 7] = [
    RelationUse { relation_id: "Gate", uses: 6 },
    RelationUse { relation_id: "VerifyBitwiseXor_12", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_4", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_7", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 6 },
    RelationUse { relation_id: "VerifyBitwiseXor_8_B", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_9", uses: 2 },
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
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 13];
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
        let M31_0 = E::F::from(M31::from(0));
        let M31_1 = E::F::from(M31::from(1));
        let M31_378353459 = E::F::from(M31::from(378353459));
        let blake_g_gate_input_addr_a = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_input_addr_a".to_owned(),
        });
        let blake_g_gate_input_addr_b = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_input_addr_b".to_owned(),
        });
        let blake_g_gate_input_addr_c = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_input_addr_c".to_owned(),
        });
        let blake_g_gate_input_addr_d = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_input_addr_d".to_owned(),
        });
        let blake_g_gate_input_addr_f0 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_input_addr_f0".to_owned(),
        });
        let blake_g_gate_input_addr_f1 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_input_addr_f1".to_owned(),
        });
        let blake_g_gate_output_addr_a = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_output_addr_a".to_owned(),
        });
        let blake_g_gate_multiplicity = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_multiplicity".to_owned(),
        });
        let blake_g_gate_output_addr_b = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_output_addr_b".to_owned(),
        });
        let blake_g_gate_output_addr_c = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_output_addr_c".to_owned(),
        });
        let blake_g_gate_output_addr_d = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "blake_g_gate_output_addr_d".to_owned(),
        });
        let input_a_limb_0_col0 = eval.next_trace_mask();
        let input_a_limb_1_col1 = eval.next_trace_mask();
        let input_b_limb_0_col2 = eval.next_trace_mask();
        let input_b_limb_1_col3 = eval.next_trace_mask();
        let input_c_limb_0_col4 = eval.next_trace_mask();
        let input_c_limb_1_col5 = eval.next_trace_mask();
        let input_d_limb_0_col6 = eval.next_trace_mask();
        let input_d_limb_1_col7 = eval.next_trace_mask();
        let input_f0_limb_0_col8 = eval.next_trace_mask();
        let input_f0_limb_1_col9 = eval.next_trace_mask();
        let input_f1_limb_0_col10 = eval.next_trace_mask();
        let input_f1_limb_1_col11 = eval.next_trace_mask();
        let input_a_out_limb_0_col12 = eval.next_trace_mask();
        let input_a_out_limb_1_col13 = eval.next_trace_mask();
        let input_b_out_limb_0_col14 = eval.next_trace_mask();
        let input_b_out_limb_1_col15 = eval.next_trace_mask();
        let input_c_out_limb_0_col16 = eval.next_trace_mask();
        let input_c_out_limb_1_col17 = eval.next_trace_mask();
        let input_d_out_limb_0_col18 = eval.next_trace_mask();
        let input_d_out_limb_1_col19 = eval.next_trace_mask();
        let triple_sum32_res_limb_0_col20 = eval.next_trace_mask();
        let triple_sum32_res_limb_1_col21 = eval.next_trace_mask();
        let ms_8_bits_col22 = eval.next_trace_mask();
        let ms_8_bits_col23 = eval.next_trace_mask();
        let ms_8_bits_col24 = eval.next_trace_mask();
        let ms_8_bits_col25 = eval.next_trace_mask();
        let xor_col26 = eval.next_trace_mask();
        let xor_col27 = eval.next_trace_mask();
        let xor_col28 = eval.next_trace_mask();
        let xor_col29 = eval.next_trace_mask();
        let triple_sum32_res_limb_0_col30 = eval.next_trace_mask();
        let triple_sum32_res_limb_1_col31 = eval.next_trace_mask();
        let ms_4_bits_col32 = eval.next_trace_mask();
        let ms_4_bits_col33 = eval.next_trace_mask();
        let ms_4_bits_col34 = eval.next_trace_mask();
        let ms_4_bits_col35 = eval.next_trace_mask();
        let xor_col36 = eval.next_trace_mask();
        let xor_col37 = eval.next_trace_mask();
        let xor_col38 = eval.next_trace_mask();
        let xor_col39 = eval.next_trace_mask();
        let ms_8_bits_col40 = eval.next_trace_mask();
        let ms_8_bits_col41 = eval.next_trace_mask();
        let ms_8_bits_col42 = eval.next_trace_mask();
        let ms_8_bits_col43 = eval.next_trace_mask();
        let ms_8_bits_col44 = eval.next_trace_mask();
        let ms_8_bits_col45 = eval.next_trace_mask();
        let ms_9_bits_col46 = eval.next_trace_mask();
        let ms_9_bits_col47 = eval.next_trace_mask();
        let ms_9_bits_col48 = eval.next_trace_mask();
        let ms_9_bits_col49 = eval.next_trace_mask();
        let ms_7_bits_col50 = eval.next_trace_mask();
        let ms_7_bits_col51 = eval.next_trace_mask();

        TripleSum32::evaluate(
            [
                input_a_limb_0_col0.clone(),
                input_a_limb_1_col1.clone(),
                input_b_limb_0_col2.clone(),
                input_b_limb_1_col3.clone(),
                input_f0_limb_0_col8.clone(),
                input_f0_limb_1_col9.clone(),
            ],
            M31_1.clone(),
            triple_sum32_res_limb_0_col20.clone(),
            triple_sum32_res_limb_1_col21.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [
            xor_rot_32_r_16_output_tmp_86b8b_21_limb_0,
            xor_rot_32_r_16_output_tmp_86b8b_21_limb_1,
        ] = XorRot32R16::evaluate(
            [
                triple_sum32_res_limb_0_col20.clone(),
                triple_sum32_res_limb_1_col21.clone(),
                input_d_limb_0_col6.clone(),
                input_d_limb_1_col7.clone(),
            ],
            M31_1.clone(),
            ms_8_bits_col22.clone(),
            ms_8_bits_col23.clone(),
            ms_8_bits_col24.clone(),
            ms_8_bits_col25.clone(),
            xor_col26.clone(),
            xor_col27.clone(),
            xor_col28.clone(),
            xor_col29.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        TripleSum32::evaluate(
            [
                input_c_limb_0_col4.clone(),
                input_c_limb_1_col5.clone(),
                xor_rot_32_r_16_output_tmp_86b8b_21_limb_0.clone(),
                xor_rot_32_r_16_output_tmp_86b8b_21_limb_1.clone(),
                M31_0.clone(),
                M31_0.clone(),
            ],
            M31_1.clone(),
            triple_sum32_res_limb_0_col30.clone(),
            triple_sum32_res_limb_1_col31.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [
            xor_rot_32_r_12_output_tmp_86b8b_43_limb_0,
            xor_rot_32_r_12_output_tmp_86b8b_43_limb_1,
        ] = XorRot32R12::evaluate(
            [
                input_b_limb_0_col2.clone(),
                input_b_limb_1_col3.clone(),
                triple_sum32_res_limb_0_col30.clone(),
                triple_sum32_res_limb_1_col31.clone(),
            ],
            M31_1.clone(),
            ms_4_bits_col32.clone(),
            ms_4_bits_col33.clone(),
            ms_4_bits_col34.clone(),
            ms_4_bits_col35.clone(),
            xor_col36.clone(),
            xor_col37.clone(),
            xor_col38.clone(),
            xor_col39.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        VerifyTripleSum32::evaluate(
            [
                triple_sum32_res_limb_0_col20.clone(),
                triple_sum32_res_limb_1_col21.clone(),
                xor_rot_32_r_12_output_tmp_86b8b_43_limb_0.clone(),
                xor_rot_32_r_12_output_tmp_86b8b_43_limb_1.clone(),
                input_f1_limb_0_col10.clone(),
                input_f1_limb_1_col11.clone(),
                input_a_out_limb_0_col12.clone(),
                input_a_out_limb_1_col13.clone(),
            ],
            M31_1.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        VerifyXorRot32R8::evaluate(
            [
                input_a_out_limb_0_col12.clone(),
                input_a_out_limb_1_col13.clone(),
                xor_rot_32_r_16_output_tmp_86b8b_21_limb_0.clone(),
                xor_rot_32_r_16_output_tmp_86b8b_21_limb_1.clone(),
                input_d_out_limb_0_col18.clone(),
                input_d_out_limb_1_col19.clone(),
            ],
            M31_1.clone(),
            ms_8_bits_col40.clone(),
            ms_8_bits_col41.clone(),
            ms_8_bits_col42.clone(),
            ms_8_bits_col43.clone(),
            ms_8_bits_col44.clone(),
            ms_8_bits_col45.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        VerifyTripleSum32::evaluate(
            [
                triple_sum32_res_limb_0_col30.clone(),
                triple_sum32_res_limb_1_col31.clone(),
                input_d_out_limb_0_col18.clone(),
                input_d_out_limb_1_col19.clone(),
                M31_0.clone(),
                M31_0.clone(),
                input_c_out_limb_0_col16.clone(),
                input_c_out_limb_1_col17.clone(),
            ],
            M31_1.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        VerifyXorRot32R7::evaluate(
            [
                xor_rot_32_r_12_output_tmp_86b8b_43_limb_0.clone(),
                xor_rot_32_r_12_output_tmp_86b8b_43_limb_1.clone(),
                input_c_out_limb_0_col16.clone(),
                input_c_out_limb_1_col17.clone(),
                input_b_out_limb_0_col14.clone(),
                input_b_out_limb_1_col15.clone(),
            ],
            M31_1.clone(),
            ms_9_bits_col46.clone(),
            ms_9_bits_col47.clone(),
            ms_9_bits_col48.clone(),
            ms_9_bits_col49.clone(),
            ms_7_bits_col50.clone(),
            ms_7_bits_col51.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_input_addr_a.clone(),
                input_a_limb_0_col0.clone(),
                input_a_limb_1_col1.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_input_addr_b.clone(),
                input_b_limb_0_col2.clone(),
                input_b_limb_1_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_input_addr_c.clone(),
                input_c_limb_0_col4.clone(),
                input_c_limb_1_col5.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_input_addr_d.clone(),
                input_d_limb_0_col6.clone(),
                input_d_limb_1_col7.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_input_addr_f0.clone(),
                input_f0_limb_0_col8.clone(),
                input_f0_limb_1_col9.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_input_addr_f1.clone(),
                input_f1_limb_0_col10.clone(),
                input_f1_limb_1_col11.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(blake_g_gate_multiplicity.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_output_addr_a.clone(),
                input_a_out_limb_0_col12.clone(),
                input_a_out_limb_1_col13.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(blake_g_gate_multiplicity.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_output_addr_b.clone(),
                input_b_out_limb_0_col14.clone(),
                input_b_out_limb_1_col15.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(blake_g_gate_multiplicity.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_output_addr_c.clone(),
                input_c_out_limb_0_col16.clone(),
                input_c_out_limb_1_col17.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(blake_g_gate_multiplicity.clone()),
            &[
                M31_378353459.clone(),
                blake_g_gate_output_addr_d.clone(),
                input_d_out_limb_0_col18.clone(),
                input_d_out_limb_1_col19.clone(),
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
    fn blake_g_gate_constraints_regression() {
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

        constraints_regression_test_values::BLAKE_G_GATE.assert_debug_eq(&sum);
    }
}
