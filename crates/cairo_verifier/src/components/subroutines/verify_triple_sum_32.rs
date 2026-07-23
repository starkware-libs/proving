// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        verify_triple_sum_32_input_limb_0,
        verify_triple_sum_32_input_limb_1,
        verify_triple_sum_32_input_limb_2,
        verify_triple_sum_32_input_limb_3,
        verify_triple_sum_32_input_limb_4,
        verify_triple_sum_32_input_limb_5,
        verify_triple_sum_32_input_limb_6,
        verify_triple_sum_32_input_limb_7,
        enabler,
    ] = input.try_into().unwrap();

    let carry_low_tmp_a7b7a_0 = eval!(
        context,
        ((((verify_triple_sum_32_input_limb_0) + (verify_triple_sum_32_input_limb_2))
            + (verify_triple_sum_32_input_limb_4))
            - (verify_triple_sum_32_input_limb_6))
            * (32768)
    );

    // carry low is 0 or 1 or 2.
    let constraint_1_value = eval!(
        context,
        ((carry_low_tmp_a7b7a_0) * ((carry_low_tmp_a7b7a_0) - (1)))
            * ((carry_low_tmp_a7b7a_0) - (2))
    );
    acc.add_constraint(context, constraint_1_value);

    let carry_high_tmp_a7b7a_1 = eval!(
        context,
        (((((verify_triple_sum_32_input_limb_1) + (verify_triple_sum_32_input_limb_3))
            + (verify_triple_sum_32_input_limb_5))
            + (carry_low_tmp_a7b7a_0))
            - (verify_triple_sum_32_input_limb_7))
            * (32768)
    );

    // carry high is 0 or 1 or 2.
    let constraint_3_value = eval!(
        context,
        ((carry_high_tmp_a7b7a_1) * ((carry_high_tmp_a7b7a_1) - (1)))
            * ((carry_high_tmp_a7b7a_1) - (2))
    );
    acc.add_constraint(context, constraint_3_value);
    vec![]
}
