use crate::components::prelude::constraint_eval::*;

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
pub struct DecodeInstructionE03055818C3F043 {}

impl DecodeInstructionE03055818C3F043 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    pub fn evaluate<E: EvalAtRow>(
        decode_instruction_e03055818c3f043_input: E::F,
        eval: &mut E,
        verify_instruction_lookup_elements: &relations::VerifyInstruction,
    ) -> [E::F; 19] {
        let M31_0 = E::F::from(M31::from(0));
        let M31_1 = E::F::from(M31::from(1));
        let M31_2147483646 = E::F::from(M31::from(2147483646));
        let M31_32 = E::F::from(M31::from(32));
        let M31_32767 = E::F::from(M31::from(32767));
        let M31_32768 = E::F::from(M31::from(32768));
        let M31_32769 = E::F::from(M31::from(32769));
        let M31_56 = E::F::from(M31::from(56));
        let M31_8 = E::F::from(M31::from(8));
        let offset0_col0 = eval.next_trace_mask();
        let ap_update_add_1_col1 = eval.next_trace_mask();

        // Flag ap_update_add_1 is a bit.
        eval.add_constraint(
            (ap_update_add_1_col1.clone() * (M31_1.clone() - ap_update_add_1_col1.clone())),
        );
        eval.add_to_relation(RelationEntry::new(
            verify_instruction_lookup_elements,
            E::EF::one(),
            &[
                decode_instruction_e03055818c3f043_input.clone(),
                offset0_col0.clone(),
                M31_32767.clone(),
                M31_32769.clone(),
                M31_56.clone(),
                (M31_8.clone() + (ap_update_add_1_col1.clone() * M31_32.clone())),
            ],
        ));

        eval.finalize_logup_in_pairs();
        [
            (offset0_col0.clone() - M31_32768.clone()),
            M31_2147483646.clone(),
            M31_1.clone(),
            M31_1.clone(),
            M31_1.clone(),
            M31_1.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_1.clone(),
            M31_0.clone(),
            ap_update_add_1_col1.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
        ]
    }
}
