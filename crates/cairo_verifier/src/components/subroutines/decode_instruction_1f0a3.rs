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
    let [
        decode_instruction_1f0a3_input_pc,
        enabler,
        offset2_col0,
        op1_base_fp_col1,
        ap_update_add_1_col2,
    ] = input.try_into().unwrap();

    // Flag op1_base_fp is a bit.
    let constraint_0_value = eval!(context, (op1_base_fp_col1) * ((1) - (op1_base_fp_col1)));
    acc.add_constraint(context, constraint_0_value);

    // Flag ap_update_add_1 is a bit.
    let constraint_1_value =
        eval!(context, (ap_update_add_1_col2) * ((1) - (ap_update_add_1_col2)));
    acc.add_constraint(context, constraint_1_value);

    // Use VerifyInstruction.
    let tuple_2 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_1f0a3_input_pc),
        eval!(context, 32767),
        eval!(context, 32767),
        eval!(context, offset2_col0),
        eval!(context, ((24) + ((op1_base_fp_col1) * (64))) + (((1) - (op1_base_fp_col1)) * (128))),
        eval!(context, (2) + ((ap_update_add_1_col2) * (32))),
    ];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);
    vec![eval!(context, (offset2_col0) - (32768)), eval!(context, (1) - (op1_base_fp_col1))]
}
