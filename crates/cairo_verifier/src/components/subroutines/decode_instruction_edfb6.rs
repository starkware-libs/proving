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
    let [decode_instruction_edfb6_input_pc, enabler, offset2_col0, op1_base_fp_col1] =
        input.try_into().unwrap();

    // Flag op1_base_fp is a bit.
    let constraint_0_value = eval!(context, (op1_base_fp_col1) * ((1) - (op1_base_fp_col1)));
    acc.add_constraint(context, constraint_0_value);

    // Use VerifyInstruction.
    let tuple_1 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_edfb6_input_pc),
        eval!(context, 32768),
        eval!(context, 32769),
        eval!(context, offset2_col0),
        eval!(context, ((op1_base_fp_col1) * (64)) + (((1) - (op1_base_fp_col1)) * (128))),
        eval!(context, 66),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![eval!(context, (offset2_col0) - (32768)), eval!(context, (1) - (op1_base_fp_col1))]
}
