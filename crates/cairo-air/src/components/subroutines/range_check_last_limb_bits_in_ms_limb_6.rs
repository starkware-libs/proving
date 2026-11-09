// This file was created by the AIR team.

use crate::components::prelude::*;

#[derive(Copy, Clone)]
pub struct RangeCheckLastLimbBitsInMsLimb6 {}

impl RangeCheckLastLimbBitsInMsLimb6 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [range_check_last_limb_bits_in_ms_limb_6_input]: [E::F; 1],
        enabler: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        eval: &mut E,
    ) -> [E::F; 0] {
        let range_check_6_relation_id = E::F::from(relations::RANGE_CHECK_6_RELATION_ID);

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                range_check_6_relation_id.clone(),
                range_check_last_limb_bits_in_ms_limb_6_input.clone(),
            ],
        ));

        []
    }
}
