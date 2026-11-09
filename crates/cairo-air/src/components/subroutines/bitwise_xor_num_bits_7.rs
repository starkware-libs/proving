// This file was created by the AIR team.

use crate::components::prelude::*;

#[derive(Copy, Clone)]
pub struct BitwiseXorNumBits7 {}

impl BitwiseXorNumBits7 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [bitwise_xor_num_bits_7_input_limb_0, bitwise_xor_num_bits_7_input_limb_1]: [E::F; 2],
        enabler: E::F,
        xor_col0: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        eval: &mut E,
    ) -> [E::F; 0] {
        let verify_bitwise_xor_7_relation_id =
            E::F::from(relations::VERIFY_BITWISE_XOR_7_RELATION_ID);

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                verify_bitwise_xor_7_relation_id.clone(),
                bitwise_xor_num_bits_7_input_limb_0.clone(),
                bitwise_xor_num_bits_7_input_limb_1.clone(),
                xor_col0.clone(),
            ],
        ));

        []
    }
}
