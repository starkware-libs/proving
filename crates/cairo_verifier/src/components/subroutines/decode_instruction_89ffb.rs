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
    let [decode_instruction_89ffb_input_pc, enabler, offset2_col0, op1_imm_col1, op1_base_fp_col2] =
        input.try_into().unwrap();

    // Flag op1_imm is a bit.
    let constraint_0_value = eval!(context, (op1_imm_col1) * ((1) - (op1_imm_col1)));
    acc.add_constraint(context, constraint_0_value);

    // Flag op1_base_fp is a bit.
    let constraint_1_value = eval!(context, (op1_base_fp_col2) * ((1) - (op1_base_fp_col2)));
    acc.add_constraint(context, constraint_1_value);

    let op1_base_ap_tmp_89ffb_5 = eval!(context, ((1) - (op1_imm_col1)) - (op1_base_fp_col2));

    // Flag op1_base_ap is a bit.
    let constraint_3_value =
        eval!(context, (op1_base_ap_tmp_89ffb_5) * ((1) - (op1_base_ap_tmp_89ffb_5)));
    acc.add_constraint(context, constraint_3_value);

    // Use VerifyInstruction.
    let tuple_4 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_89ffb_input_pc),
        eval!(context, 32767),
        eval!(context, 32767),
        eval!(context, offset2_col0),
        eval!(
            context,
            (((24) + ((op1_imm_col1) * (32))) + ((op1_base_fp_col2) * (64)))
                + ((op1_base_ap_tmp_89ffb_5) * (128))
        ),
        eval!(context, 16),
    ];
    let numerator_4 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_4, tuple_4);
    vec![eval!(context, (offset2_col0) - (32768)), eval!(context, op1_base_ap_tmp_89ffb_5)]
}
