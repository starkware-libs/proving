// This file was created by the AIR team.

use subroutines::split_16_low_part_size_7::Split16LowPartSize7;
use subroutines::split_16_low_part_size_9::Split16LowPartSize9;

use crate::circuit_air::components::prelude::*;

#[derive(Copy, Clone)]
pub struct VerifyXorRot32R7 {}

impl VerifyXorRot32R7 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [
            verify_xor_rot_32_r_7_input_limb_0,
            verify_xor_rot_32_r_7_input_limb_1,
            verify_xor_rot_32_r_7_input_limb_2,
            verify_xor_rot_32_r_7_input_limb_3,
            verify_xor_rot_32_r_7_input_limb_4,
            verify_xor_rot_32_r_7_input_limb_5,
        ]: [E::F; 6],
        enabler: E::F,
        ms_9_bits_col0: E::F,
        ms_9_bits_col1: E::F,
        ms_9_bits_col2: E::F,
        ms_9_bits_col3: E::F,
        ms_7_bits_col4: E::F,
        ms_7_bits_col5: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        eval: &mut E,
    ) -> [E::F; 0] {
        let M31_62225763 = E::F::from(M31::from(62225763));
        let M31_95781001 = E::F::from(M31::from(95781001));

        let [split_16_low_part_size_7_output_tmp_f88f9_1_limb_0] = Split16LowPartSize7::evaluate(
            [verify_xor_rot_32_r_7_input_limb_0.clone()],
            enabler.clone(),
            ms_9_bits_col0.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_7_output_tmp_f88f9_3_limb_0] = Split16LowPartSize7::evaluate(
            [verify_xor_rot_32_r_7_input_limb_1.clone()],
            enabler.clone(),
            ms_9_bits_col1.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_7_output_tmp_f88f9_5_limb_0] = Split16LowPartSize7::evaluate(
            [verify_xor_rot_32_r_7_input_limb_2.clone()],
            enabler.clone(),
            ms_9_bits_col2.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_7_output_tmp_f88f9_7_limb_0] = Split16LowPartSize7::evaluate(
            [verify_xor_rot_32_r_7_input_limb_3.clone()],
            enabler.clone(),
            ms_9_bits_col3.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_9_output_tmp_f88f9_9_limb_0] = Split16LowPartSize9::evaluate(
            [verify_xor_rot_32_r_7_input_limb_4.clone()],
            enabler.clone(),
            ms_7_bits_col4.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_9_output_tmp_f88f9_11_limb_0] = Split16LowPartSize9::evaluate(
            [verify_xor_rot_32_r_7_input_limb_5.clone()],
            enabler.clone(),
            ms_7_bits_col5.clone(),
            common_lookup_elements,
            eval,
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_95781001.clone(),
                ms_9_bits_col0.clone(),
                ms_9_bits_col2.clone(),
                split_16_low_part_size_9_output_tmp_f88f9_9_limb_0.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_62225763.clone(),
                split_16_low_part_size_7_output_tmp_f88f9_3_limb_0.clone(),
                split_16_low_part_size_7_output_tmp_f88f9_7_limb_0.clone(),
                ms_7_bits_col4.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_95781001.clone(),
                ms_9_bits_col1.clone(),
                ms_9_bits_col3.clone(),
                split_16_low_part_size_9_output_tmp_f88f9_11_limb_0.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_62225763.clone(),
                split_16_low_part_size_7_output_tmp_f88f9_1_limb_0.clone(),
                split_16_low_part_size_7_output_tmp_f88f9_5_limb_0.clone(),
                ms_7_bits_col5.clone(),
            ],
        ));

        []
    }
}
