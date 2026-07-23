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
        decode_instruction_a1c8b_input_pc,
        enabler,
        offset0_col0,
        offset1_col1,
        offset2_col2,
        dst_base_fp_col3,
        op0_base_fp_col4,
        op1_imm_col5,
        op1_base_fp_col6,
        op1_base_ap_col7,
        res_add_col8,
        res_mul_col9,
        pc_update_jump_col10,
        pc_update_jump_rel_col11,
        pc_update_jnz_col12,
        ap_update_add_col13,
        ap_update_add_1_col14,
        opcode_call_col15,
        opcode_ret_col16,
        opcode_assert_eq_col17,
    ] = input.try_into().unwrap();

    // Flag dst_base_fp is a bit.
    let constraint_0_value = eval!(context, (dst_base_fp_col3) * ((1) - (dst_base_fp_col3)));
    acc.add_constraint(context, constraint_0_value);

    // Flag op0_base_fp is a bit.
    let constraint_1_value = eval!(context, (op0_base_fp_col4) * ((1) - (op0_base_fp_col4)));
    acc.add_constraint(context, constraint_1_value);

    // Flag op1_imm is a bit.
    let constraint_2_value = eval!(context, (op1_imm_col5) * ((1) - (op1_imm_col5)));
    acc.add_constraint(context, constraint_2_value);

    // Flag op1_base_fp is a bit.
    let constraint_3_value = eval!(context, (op1_base_fp_col6) * ((1) - (op1_base_fp_col6)));
    acc.add_constraint(context, constraint_3_value);

    // Flag op1_base_ap is a bit.
    let constraint_4_value = eval!(context, (op1_base_ap_col7) * ((1) - (op1_base_ap_col7)));
    acc.add_constraint(context, constraint_4_value);

    // Flag res_add is a bit.
    let constraint_5_value = eval!(context, (res_add_col8) * ((1) - (res_add_col8)));
    acc.add_constraint(context, constraint_5_value);

    // Flag res_mul is a bit.
    let constraint_6_value = eval!(context, (res_mul_col9) * ((1) - (res_mul_col9)));
    acc.add_constraint(context, constraint_6_value);

    // Flag pc_update_jump is a bit.
    let constraint_7_value =
        eval!(context, (pc_update_jump_col10) * ((1) - (pc_update_jump_col10)));
    acc.add_constraint(context, constraint_7_value);

    // Flag pc_update_jump_rel is a bit.
    let constraint_8_value =
        eval!(context, (pc_update_jump_rel_col11) * ((1) - (pc_update_jump_rel_col11)));
    acc.add_constraint(context, constraint_8_value);

    // Flag pc_update_jnz is a bit.
    let constraint_9_value = eval!(context, (pc_update_jnz_col12) * ((1) - (pc_update_jnz_col12)));
    acc.add_constraint(context, constraint_9_value);

    // Flag ap_update_add is a bit.
    let constraint_10_value = eval!(context, (ap_update_add_col13) * ((1) - (ap_update_add_col13)));
    acc.add_constraint(context, constraint_10_value);

    // Flag ap_update_add_1 is a bit.
    let constraint_11_value =
        eval!(context, (ap_update_add_1_col14) * ((1) - (ap_update_add_1_col14)));
    acc.add_constraint(context, constraint_11_value);

    // Flag opcode_call is a bit.
    let constraint_12_value = eval!(context, (opcode_call_col15) * ((1) - (opcode_call_col15)));
    acc.add_constraint(context, constraint_12_value);

    // Flag opcode_ret is a bit.
    let constraint_13_value = eval!(context, (opcode_ret_col16) * ((1) - (opcode_ret_col16)));
    acc.add_constraint(context, constraint_13_value);

    // Flag opcode_assert_eq is a bit.
    let constraint_14_value =
        eval!(context, (opcode_assert_eq_col17) * ((1) - (opcode_assert_eq_col17)));
    acc.add_constraint(context, constraint_14_value);

    // Use VerifyInstruction.
    let tuple_15 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_a1c8b_input_pc),
        eval!(context, offset0_col0),
        eval!(context, offset1_col1),
        eval!(context, offset2_col2),
        eval!(
            context,
            ((((((dst_base_fp_col3) * (8)) + ((op0_base_fp_col4) * (16)))
                + ((op1_imm_col5) * (32)))
                + ((op1_base_fp_col6) * (64)))
                + ((op1_base_ap_col7) * (128)))
                + ((res_add_col8) * (256))
        ),
        eval!(
            context,
            ((((((((res_mul_col9) + ((pc_update_jump_col10) * (2)))
                + ((pc_update_jump_rel_col11) * (4)))
                + ((pc_update_jnz_col12) * (8)))
                + ((ap_update_add_col13) * (16)))
                + ((ap_update_add_1_col14) * (32)))
                + ((opcode_call_col15) * (64)))
                + ((opcode_ret_col16) * (128)))
                + ((opcode_assert_eq_col17) * (256))
        ),
    ];
    let numerator_15 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_15, tuple_15);
    vec![
        eval!(context, (offset0_col0) - (32768)),
        eval!(context, (offset1_col1) - (32768)),
        eval!(context, (offset2_col2) - (32768)),
    ]
}
