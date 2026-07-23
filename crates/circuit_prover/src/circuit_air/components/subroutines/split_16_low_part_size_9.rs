// This file was created by the AIR team.

use crate::circuit_air::components::prelude::*;

#[derive(Copy, Clone)]
pub struct Split16LowPartSize9 {}

impl Split16LowPartSize9 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [split_16_low_part_size_9_input]: [E::F; 1],
        enabler: E::F,
        ms_7_bits_col0: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        eval: &mut E,
    ) -> [E::F; 1] {
        let M31_512 = E::F::from(M31::from(512));

        [(split_16_low_part_size_9_input.clone() - (ms_7_bits_col0.clone() * M31_512.clone()))]
    }
}
