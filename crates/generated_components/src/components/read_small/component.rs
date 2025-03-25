use crate::components::prelude::constraint_eval::*;

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
pub struct ReadSmall {}

impl ReadSmall {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    pub fn evaluate<E: EvalAtRow>(
        read_small_input: E::F,
        eval: &mut E,
        memory_address_to_id_lookup_elements: &relations::MemoryAddressToId,
        memory_id_to_big_lookup_elements: &relations::MemoryIdToBig,
    ) -> [E::F; 2] {
        let M31_0 = E::F::from(M31::from(0));
        let M31_1 = E::F::from(M31::from(1));
        let M31_134217728 = E::F::from(M31::from(134217728));
        let M31_136 = E::F::from(M31::from(136));
        let M31_256 = E::F::from(M31::from(256));
        let M31_262144 = E::F::from(M31::from(262144));
        let M31_511 = E::F::from(M31::from(511));
        let M31_512 = E::F::from(M31::from(512));
        let id_col0 = eval.next_trace_mask();
        let msb_col1 = eval.next_trace_mask();
        let mid_limbs_set_col2 = eval.next_trace_mask();
        let value_limb_0_col3 = eval.next_trace_mask();
        let value_limb_1_col4 = eval.next_trace_mask();
        let value_limb_2_col5 = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            memory_address_to_id_lookup_elements,
            E::EF::one(),
            &[read_small_input.clone(), id_col0.clone()],
        ));

        // Cond Decode Small Sign.

        // msb is a bit.
        eval.add_constraint((msb_col1.clone() * (msb_col1.clone() - M31_1.clone())));
        // mid_limbs_set is a bit.
        eval.add_constraint(
            (mid_limbs_set_col2.clone() * (mid_limbs_set_col2.clone() - M31_1.clone())),
        );
        // Cannot have msb equals 0 and mid_limbs_set equals 1.
        eval.add_constraint((mid_limbs_set_col2.clone() * (msb_col1.clone() - M31_1.clone())));

        eval.add_to_relation(RelationEntry::new(
            memory_id_to_big_lookup_elements,
            E::EF::one(),
            &[
                id_col0.clone(),
                value_limb_0_col3.clone(),
                value_limb_1_col4.clone(),
                value_limb_2_col5.clone(),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                (mid_limbs_set_col2.clone() * M31_511.clone()),
                ((M31_136.clone() * msb_col1.clone()) - mid_limbs_set_col2.clone()),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                (msb_col1.clone() * M31_256.clone()),
            ],
        ));

        [
            ((((value_limb_0_col3.clone() + (value_limb_1_col4.clone() * M31_512.clone()))
                + (value_limb_2_col5.clone() * M31_262144.clone()))
                - msb_col1.clone())
                - (M31_134217728.clone() * mid_limbs_set_col2.clone())),
            id_col0.clone(),
        ]
    }
}
