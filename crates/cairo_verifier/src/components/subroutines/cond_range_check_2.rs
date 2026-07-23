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
        cond_range_check_2_input_limb_0,
        cond_range_check_2_input_limb_1,
        enabler,
        partial_limb_msb_col0,
    ] = input.try_into().unwrap();

    // msb is a bit or condition is 0.
    let constraint_0_value = eval!(
        context,
        ((partial_limb_msb_col0) * ((1) - (partial_limb_msb_col0)))
            * (cond_range_check_2_input_limb_1)
    );
    acc.add_constraint(context, constraint_0_value);

    let partial_limb_bit_before_msb_tmp_36c3a_1 =
        eval!(context, (cond_range_check_2_input_limb_0) - ((partial_limb_msb_col0) * (2)));

    // bit before msb is a bit or condition is 0.
    let constraint_2_value = eval!(
        context,
        ((partial_limb_bit_before_msb_tmp_36c3a_1)
            * ((1) - (partial_limb_bit_before_msb_tmp_36c3a_1)))
            * (cond_range_check_2_input_limb_1)
    );
    acc.add_constraint(context, constraint_2_value);
    vec![]
}
