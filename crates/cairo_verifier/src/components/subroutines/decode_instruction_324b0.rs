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
        decode_instruction_324b0_input_pc,
        enabler,
        offset0_col0,
        dst_base_fp_col1,
        ap_update_add_1_col2,
    ] = input.try_into().unwrap();

    // Flag dst_base_fp is a bit.
    let constraint_0_value = eval!(context, (dst_base_fp_col1) * ((1) - (dst_base_fp_col1)));
    acc.add_constraint(context, constraint_0_value);

    // Flag ap_update_add_1 is a bit.
    let constraint_1_value =
        eval!(context, (ap_update_add_1_col2) * ((1) - (ap_update_add_1_col2)));
    acc.add_constraint(context, constraint_1_value);

    // Use VerifyInstruction.
    let tuple_2 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_324b0_input_pc),
        eval!(context, offset0_col0),
        eval!(context, 32767),
        eval!(context, 32769),
        eval!(context, (((dst_base_fp_col1) * (8)) + (16)) + (32)),
        eval!(context, ((ap_update_add_1_col2) * (32)) + (256)),
    ];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);
    vec![eval!(context, (offset0_col0) - (32768))]
}
