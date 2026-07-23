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
    let [enabler, msb_col0, mid_limbs_set_col1] = input.try_into().unwrap();

    // msb is a bit.
    let constraint_0_value = eval!(context, (msb_col0) * ((msb_col0) - (1)));
    acc.add_constraint(context, constraint_0_value);

    // mid_limbs_set is a bit.
    let constraint_1_value = eval!(context, (mid_limbs_set_col1) * ((mid_limbs_set_col1) - (1)));
    acc.add_constraint(context, constraint_1_value);

    // Cannot have msb equals 0 and mid_limbs_set equals 1.
    let constraint_2_value = eval!(context, (mid_limbs_set_col1) * ((msb_col0) - (1)));
    acc.add_constraint(context, constraint_2_value);
    vec![
        eval!(context, (mid_limbs_set_col1) * (508)),
        eval!(context, (mid_limbs_set_col1) * (511)),
        eval!(context, ((msb_col0) * (136)) - (mid_limbs_set_col1)),
        eval!(context, (msb_col0) * (256)),
    ]
}
