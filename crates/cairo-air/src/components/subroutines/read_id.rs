// This file was created by the AIR team.

use crate::components::prelude::*;

#[derive(Copy, Clone)]
pub struct ReadId {}

impl ReadId {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [read_id_input]: [E::F; 1],
        enabler: E::F,
        id_col0: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        eval: &mut E,
    ) -> [E::F; 0] {
        let memory_address_to_id_relation_id =
            E::F::from(relations::MEMORY_ADDRESS_TO_ID_RELATION_ID);

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[memory_address_to_id_relation_id.clone(), read_id_input.clone(), id_col0.clone()],
        ));

        []
    }
}
