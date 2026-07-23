// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyInstruction", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [decode_instruction_7dcb2_input_pc, enabler, ap_update_add_1_col0] =
        input.try_into().unwrap();

    // Flag ap_update_add_1 is a bit.
    let constraint_0_value =
        eval!(context, (ap_update_add_1_col0) * ((1) - (ap_update_add_1_col0)));
    acc.add_constraint(context, constraint_0_value);

    // Use VerifyInstruction.
    let tuple_1 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_7dcb2_input_pc),
        eval!(context, 32767),
        eval!(context, 32767),
        eval!(context, 32769),
        eval!(context, 56),
        eval!(context, (4) + ((ap_update_add_1_col0) * (32))),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![]
}
