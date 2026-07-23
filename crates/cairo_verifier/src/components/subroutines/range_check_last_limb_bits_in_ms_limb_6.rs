// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "RangeCheck_6", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [range_check_last_limb_bits_in_ms_limb_6_input, enabler] = input.try_into().unwrap();

    // Use RangeCheck_6.
    let tuple_0 = &[
        eval!(context, 1185356339),
        eval!(context, range_check_last_limb_bits_in_ms_limb_6_input),
    ];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);
    vec![]
}
