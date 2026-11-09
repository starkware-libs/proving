// This file was created by the AIR team.

use crate::components::prelude::*;

#[derive(Copy, Clone)]
pub struct RangeCheckMemValueN28 {}

impl RangeCheckMemValueN28 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [
            range_check_mem_value_n_28_input_limb_0,
            range_check_mem_value_n_28_input_limb_1,
            range_check_mem_value_n_28_input_limb_2,
            range_check_mem_value_n_28_input_limb_3,
            range_check_mem_value_n_28_input_limb_4,
            range_check_mem_value_n_28_input_limb_5,
            range_check_mem_value_n_28_input_limb_6,
            range_check_mem_value_n_28_input_limb_7,
            range_check_mem_value_n_28_input_limb_8,
            range_check_mem_value_n_28_input_limb_9,
            range_check_mem_value_n_28_input_limb_10,
            range_check_mem_value_n_28_input_limb_11,
            range_check_mem_value_n_28_input_limb_12,
            range_check_mem_value_n_28_input_limb_13,
            range_check_mem_value_n_28_input_limb_14,
            range_check_mem_value_n_28_input_limb_15,
            range_check_mem_value_n_28_input_limb_16,
            range_check_mem_value_n_28_input_limb_17,
            range_check_mem_value_n_28_input_limb_18,
            range_check_mem_value_n_28_input_limb_19,
            range_check_mem_value_n_28_input_limb_20,
            range_check_mem_value_n_28_input_limb_21,
            range_check_mem_value_n_28_input_limb_22,
            range_check_mem_value_n_28_input_limb_23,
            range_check_mem_value_n_28_input_limb_24,
            range_check_mem_value_n_28_input_limb_25,
            range_check_mem_value_n_28_input_limb_26,
            range_check_mem_value_n_28_input_limb_27,
        ]: [E::F; 28],
        enabler: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        eval: &mut E,
    ) -> [E::F; 0] {
        let range_check_9_9_b_relation_id = E::F::from(relations::RANGE_CHECK_9_9_B_RELATION_ID);
        let range_check_9_9_c_relation_id = E::F::from(relations::RANGE_CHECK_9_9_C_RELATION_ID);
        let range_check_9_9_d_relation_id = E::F::from(relations::RANGE_CHECK_9_9_D_RELATION_ID);
        let range_check_9_9_e_relation_id = E::F::from(relations::RANGE_CHECK_9_9_E_RELATION_ID);
        let range_check_9_9_f_relation_id = E::F::from(relations::RANGE_CHECK_9_9_F_RELATION_ID);
        let range_check_9_9_g_relation_id = E::F::from(relations::RANGE_CHECK_9_9_G_RELATION_ID);
        let range_check_9_9_h_relation_id = E::F::from(relations::RANGE_CHECK_9_9_H_RELATION_ID);
        let range_check_9_9_relation_id = E::F::from(relations::RANGE_CHECK_9_9_RELATION_ID);

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_0.clone(),
                range_check_mem_value_n_28_input_limb_1.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_b_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_2.clone(),
                range_check_mem_value_n_28_input_limb_3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_c_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_4.clone(),
                range_check_mem_value_n_28_input_limb_5.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_d_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_6.clone(),
                range_check_mem_value_n_28_input_limb_7.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_e_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_8.clone(),
                range_check_mem_value_n_28_input_limb_9.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_f_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_10.clone(),
                range_check_mem_value_n_28_input_limb_11.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_g_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_12.clone(),
                range_check_mem_value_n_28_input_limb_13.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_h_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_14.clone(),
                range_check_mem_value_n_28_input_limb_15.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_16.clone(),
                range_check_mem_value_n_28_input_limb_17.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_b_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_18.clone(),
                range_check_mem_value_n_28_input_limb_19.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_c_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_20.clone(),
                range_check_mem_value_n_28_input_limb_21.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_d_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_22.clone(),
                range_check_mem_value_n_28_input_limb_23.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_e_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_24.clone(),
                range_check_mem_value_n_28_input_limb_25.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_9_9_f_relation_id.clone(),
                range_check_mem_value_n_28_input_limb_26.clone(),
                range_check_mem_value_n_28_input_limb_27.clone(),
            ],
        ));

        []
    }
}
