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
        decode_instruction_ed841_input_pc,
        enabler,
        offset0_col0,
        offset1_col1,
        offset2_col2,
        dst_base_fp_col3,
        op0_base_fp_col4,
        ap_update_add_1_col5,
    ] = input.try_into().unwrap();

    // Flag dst_base_fp is a bit.
    let constraint_0_value = eval!(context, (dst_base_fp_col3) * ((1) - (dst_base_fp_col3)));
    acc.add_constraint(context, constraint_0_value);

    // Flag op0_base_fp is a bit.
    let constraint_1_value = eval!(context, (op0_base_fp_col4) * ((1) - (op0_base_fp_col4)));
    acc.add_constraint(context, constraint_1_value);

    // Flag ap_update_add_1 is a bit.
    let constraint_2_value =
        eval!(context, (ap_update_add_1_col5) * ((1) - (ap_update_add_1_col5)));
    acc.add_constraint(context, constraint_2_value);

    // Use VerifyInstruction.
    let tuple_3 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_ed841_input_pc),
        eval!(context, offset0_col0),
        eval!(context, offset1_col1),
        eval!(context, offset2_col2),
        eval!(context, ((dst_base_fp_col3) * (8)) + ((op0_base_fp_col4) * (16))),
        eval!(context, ((ap_update_add_1_col5) * (32)) + (256)),
    ];
    let numerator_3 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_3, tuple_3);
    vec![
        eval!(context, (offset0_col0) - (32768)),
        eval!(context, (offset1_col1) - (32768)),
        eval!(context, (offset2_col2) - (32768)),
    ]
}
