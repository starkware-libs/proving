// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "RangeCheck_11", uses: 1 },
    RelationUse { relation_id: "RangeCheck_18", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [range_check_29_input, enabler, range_check_29_bot11bits_col0] = input.try_into().unwrap();

    // Use RangeCheck_18.
    let tuple_0 = &[
        eval!(context, 1109051422),
        eval!(context, ((range_check_29_input) - (range_check_29_bot11bits_col0)) * (1048576)),
    ];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Use RangeCheck_11.
    let tuple_1 = &[eval!(context, 991608089), eval!(context, range_check_29_bot11bits_col0)];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![]
}
