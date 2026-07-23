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
    let [
        handle_opcodes_input_pc,
        handle_opcodes_input_fp,
        handle_opcodes_input_dst_base_fp,
        handle_opcodes_input_op0_base_fp,
        handle_opcodes_input_op1_base_fp,
        handle_opcodes_input_pc_update_jump,
        handle_opcodes_input_opcode_call,
        handle_opcodes_input_opcode_ret,
        handle_opcodes_input_opcode_assert_eq,
        handle_opcodes_input_res_op1,
        handle_opcodes_input_instruction_size,
        handle_opcodes_input_offset0,
        handle_opcodes_input_offset1,
        handle_opcodes_input_offset2,
        handle_opcodes_input_dst_limb_0,
        handle_opcodes_input_dst_limb_1,
        handle_opcodes_input_dst_limb_2,
        handle_opcodes_input_dst_limb_3,
        handle_opcodes_input_dst_limb_4,
        handle_opcodes_input_dst_limb_5,
        handle_opcodes_input_dst_limb_6,
        handle_opcodes_input_dst_limb_7,
        handle_opcodes_input_dst_limb_8,
        handle_opcodes_input_dst_limb_9,
        handle_opcodes_input_dst_limb_10,
        handle_opcodes_input_dst_limb_11,
        handle_opcodes_input_dst_limb_12,
        handle_opcodes_input_dst_limb_13,
        handle_opcodes_input_dst_limb_14,
        handle_opcodes_input_dst_limb_15,
        handle_opcodes_input_dst_limb_16,
        handle_opcodes_input_dst_limb_17,
        handle_opcodes_input_dst_limb_18,
        handle_opcodes_input_dst_limb_19,
        handle_opcodes_input_dst_limb_20,
        handle_opcodes_input_dst_limb_21,
        handle_opcodes_input_dst_limb_22,
        handle_opcodes_input_dst_limb_23,
        handle_opcodes_input_dst_limb_24,
        handle_opcodes_input_dst_limb_25,
        handle_opcodes_input_dst_limb_26,
        handle_opcodes_input_dst_limb_27,
        handle_opcodes_input_op0_limb_0,
        handle_opcodes_input_op0_limb_1,
        handle_opcodes_input_op0_limb_2,
        handle_opcodes_input_op0_limb_3,
        handle_opcodes_input_op0_limb_4,
        handle_opcodes_input_op0_limb_5,
        handle_opcodes_input_op0_limb_6,
        handle_opcodes_input_op0_limb_7,
        handle_opcodes_input_op0_limb_8,
        handle_opcodes_input_op0_limb_9,
        handle_opcodes_input_op0_limb_10,
        handle_opcodes_input_op0_limb_11,
        handle_opcodes_input_op0_limb_12,
        handle_opcodes_input_op0_limb_13,
        handle_opcodes_input_op0_limb_14,
        handle_opcodes_input_op0_limb_15,
        handle_opcodes_input_op0_limb_16,
        handle_opcodes_input_op0_limb_17,
        handle_opcodes_input_op0_limb_18,
        handle_opcodes_input_op0_limb_19,
        handle_opcodes_input_op0_limb_20,
        handle_opcodes_input_op0_limb_21,
        handle_opcodes_input_op0_limb_22,
        handle_opcodes_input_op0_limb_23,
        handle_opcodes_input_op0_limb_24,
        handle_opcodes_input_op0_limb_25,
        handle_opcodes_input_op0_limb_26,
        handle_opcodes_input_op0_limb_27,
        handle_opcodes_input_res_limb_0,
        handle_opcodes_input_res_limb_1,
        handle_opcodes_input_res_limb_2,
        handle_opcodes_input_res_limb_3,
        handle_opcodes_input_res_limb_4,
        handle_opcodes_input_res_limb_5,
        handle_opcodes_input_res_limb_6,
        handle_opcodes_input_res_limb_7,
        handle_opcodes_input_res_limb_8,
        handle_opcodes_input_res_limb_9,
        handle_opcodes_input_res_limb_10,
        handle_opcodes_input_res_limb_11,
        handle_opcodes_input_res_limb_12,
        handle_opcodes_input_res_limb_13,
        handle_opcodes_input_res_limb_14,
        handle_opcodes_input_res_limb_15,
        handle_opcodes_input_res_limb_16,
        handle_opcodes_input_res_limb_17,
        handle_opcodes_input_res_limb_18,
        handle_opcodes_input_res_limb_19,
        handle_opcodes_input_res_limb_20,
        handle_opcodes_input_res_limb_21,
        handle_opcodes_input_res_limb_22,
        handle_opcodes_input_res_limb_23,
        handle_opcodes_input_res_limb_24,
        handle_opcodes_input_res_limb_25,
        handle_opcodes_input_res_limb_26,
        handle_opcodes_input_res_limb_27,
        enabler,
        partial_limb_msb_col0,
        partial_limb_msb_col1,
    ] = input.try_into().unwrap();

    let constraint_0_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_0) - (handle_opcodes_input_dst_limb_0))
    );
    acc.add_constraint(context, constraint_0_value);

    let constraint_1_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_1) - (handle_opcodes_input_dst_limb_1))
    );
    acc.add_constraint(context, constraint_1_value);

    let constraint_2_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_2) - (handle_opcodes_input_dst_limb_2))
    );
    acc.add_constraint(context, constraint_2_value);

    let constraint_3_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_3) - (handle_opcodes_input_dst_limb_3))
    );
    acc.add_constraint(context, constraint_3_value);

    let constraint_4_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_4) - (handle_opcodes_input_dst_limb_4))
    );
    acc.add_constraint(context, constraint_4_value);

    let constraint_5_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_5) - (handle_opcodes_input_dst_limb_5))
    );
    acc.add_constraint(context, constraint_5_value);

    let constraint_6_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_6) - (handle_opcodes_input_dst_limb_6))
    );
    acc.add_constraint(context, constraint_6_value);

    let constraint_7_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_7) - (handle_opcodes_input_dst_limb_7))
    );
    acc.add_constraint(context, constraint_7_value);

    let constraint_8_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_8) - (handle_opcodes_input_dst_limb_8))
    );
    acc.add_constraint(context, constraint_8_value);

    let constraint_9_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_9) - (handle_opcodes_input_dst_limb_9))
    );
    acc.add_constraint(context, constraint_9_value);

    let constraint_10_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_10) - (handle_opcodes_input_dst_limb_10))
    );
    acc.add_constraint(context, constraint_10_value);

    let constraint_11_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_11) - (handle_opcodes_input_dst_limb_11))
    );
    acc.add_constraint(context, constraint_11_value);

    let constraint_12_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_12) - (handle_opcodes_input_dst_limb_12))
    );
    acc.add_constraint(context, constraint_12_value);

    let constraint_13_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_13) - (handle_opcodes_input_dst_limb_13))
    );
    acc.add_constraint(context, constraint_13_value);

    let constraint_14_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_14) - (handle_opcodes_input_dst_limb_14))
    );
    acc.add_constraint(context, constraint_14_value);

    let constraint_15_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_15) - (handle_opcodes_input_dst_limb_15))
    );
    acc.add_constraint(context, constraint_15_value);

    let constraint_16_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_16) - (handle_opcodes_input_dst_limb_16))
    );
    acc.add_constraint(context, constraint_16_value);

    let constraint_17_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_17) - (handle_opcodes_input_dst_limb_17))
    );
    acc.add_constraint(context, constraint_17_value);

    let constraint_18_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_18) - (handle_opcodes_input_dst_limb_18))
    );
    acc.add_constraint(context, constraint_18_value);

    let constraint_19_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_19) - (handle_opcodes_input_dst_limb_19))
    );
    acc.add_constraint(context, constraint_19_value);

    let constraint_20_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_20) - (handle_opcodes_input_dst_limb_20))
    );
    acc.add_constraint(context, constraint_20_value);

    let constraint_21_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_21) - (handle_opcodes_input_dst_limb_21))
    );
    acc.add_constraint(context, constraint_21_value);

    let constraint_22_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_22) - (handle_opcodes_input_dst_limb_22))
    );
    acc.add_constraint(context, constraint_22_value);

    let constraint_23_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_23) - (handle_opcodes_input_dst_limb_23))
    );
    acc.add_constraint(context, constraint_23_value);

    let constraint_24_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_24) - (handle_opcodes_input_dst_limb_24))
    );
    acc.add_constraint(context, constraint_24_value);

    let constraint_25_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_25) - (handle_opcodes_input_dst_limb_25))
    );
    acc.add_constraint(context, constraint_25_value);

    let constraint_26_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_26) - (handle_opcodes_input_dst_limb_26))
    );
    acc.add_constraint(context, constraint_26_value);

    let constraint_27_value = eval!(
        context,
        (handle_opcodes_input_opcode_assert_eq)
            * ((handle_opcodes_input_res_limb_27) - (handle_opcodes_input_dst_limb_27))
    );
    acc.add_constraint(context, constraint_27_value);

    // ret opcode offset0 equals -2.
    let constraint_28_value =
        eval!(context, (handle_opcodes_input_opcode_ret) * ((handle_opcodes_input_offset0) + (2)));
    acc.add_constraint(context, constraint_28_value);

    // ret opcode offset2 equals -1.
    let constraint_29_value =
        eval!(context, (handle_opcodes_input_opcode_ret) * ((handle_opcodes_input_offset2) + (1)));
    acc.add_constraint(context, constraint_29_value);

    // ret opcode flags pc_update_jump and dst_base_fp and op1_base_fp_and_res_op1 are on.
    let constraint_30_value = eval!(
        context,
        (handle_opcodes_input_opcode_ret)
            * (((((4) - (handle_opcodes_input_pc_update_jump))
                - (handle_opcodes_input_dst_base_fp))
                - (handle_opcodes_input_op1_base_fp))
                - (handle_opcodes_input_res_op1))
    );
    acc.add_constraint(context, constraint_30_value);

    // call opcode offset0 equals 0.
    let constraint_31_value =
        eval!(context, (handle_opcodes_input_opcode_call) * (handle_opcodes_input_offset0));
    acc.add_constraint(context, constraint_31_value);

    // call opcode offset1 equals 1.
    let constraint_32_value =
        eval!(context, (handle_opcodes_input_opcode_call) * ((1) - (handle_opcodes_input_offset1)));
    acc.add_constraint(context, constraint_32_value);

    // call opcode flags op0_base_fp and dst_base_fp are off.
    let constraint_33_value = eval!(
        context,
        (handle_opcodes_input_opcode_call)
            * ((handle_opcodes_input_op0_base_fp) + (handle_opcodes_input_dst_base_fp))
    );
    acc.add_constraint(context, constraint_33_value);

    let [cond_felt_252_as_addr_output_tmp_fe176_2] = cond_felt_252_as_addr::accumulate_constraints(
        &[
            eval!(context, handle_opcodes_input_dst_limb_0),
            eval!(context, handle_opcodes_input_dst_limb_1),
            eval!(context, handle_opcodes_input_dst_limb_2),
            eval!(context, handle_opcodes_input_dst_limb_3),
            eval!(context, handle_opcodes_input_dst_limb_4),
            eval!(context, handle_opcodes_input_dst_limb_5),
            eval!(context, handle_opcodes_input_dst_limb_6),
            eval!(context, handle_opcodes_input_dst_limb_7),
            eval!(context, handle_opcodes_input_dst_limb_8),
            eval!(context, handle_opcodes_input_dst_limb_9),
            eval!(context, handle_opcodes_input_dst_limb_10),
            eval!(context, handle_opcodes_input_dst_limb_11),
            eval!(context, handle_opcodes_input_dst_limb_12),
            eval!(context, handle_opcodes_input_dst_limb_13),
            eval!(context, handle_opcodes_input_dst_limb_14),
            eval!(context, handle_opcodes_input_dst_limb_15),
            eval!(context, handle_opcodes_input_dst_limb_16),
            eval!(context, handle_opcodes_input_dst_limb_17),
            eval!(context, handle_opcodes_input_dst_limb_18),
            eval!(context, handle_opcodes_input_dst_limb_19),
            eval!(context, handle_opcodes_input_dst_limb_20),
            eval!(context, handle_opcodes_input_dst_limb_21),
            eval!(context, handle_opcodes_input_dst_limb_22),
            eval!(context, handle_opcodes_input_dst_limb_23),
            eval!(context, handle_opcodes_input_dst_limb_24),
            eval!(context, handle_opcodes_input_dst_limb_25),
            eval!(context, handle_opcodes_input_dst_limb_26),
            eval!(context, handle_opcodes_input_dst_limb_27),
            eval!(context, handle_opcodes_input_opcode_call),
            eval!(context, enabler),
            eval!(context, partial_limb_msb_col0),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let constraint_35_value = eval!(
        context,
        (handle_opcodes_input_opcode_call)
            * ((cond_felt_252_as_addr_output_tmp_fe176_2) - (handle_opcodes_input_fp))
    );
    acc.add_constraint(context, constraint_35_value);

    let [cond_felt_252_as_addr_output_tmp_fe176_5] = cond_felt_252_as_addr::accumulate_constraints(
        &[
            eval!(context, handle_opcodes_input_op0_limb_0),
            eval!(context, handle_opcodes_input_op0_limb_1),
            eval!(context, handle_opcodes_input_op0_limb_2),
            eval!(context, handle_opcodes_input_op0_limb_3),
            eval!(context, handle_opcodes_input_op0_limb_4),
            eval!(context, handle_opcodes_input_op0_limb_5),
            eval!(context, handle_opcodes_input_op0_limb_6),
            eval!(context, handle_opcodes_input_op0_limb_7),
            eval!(context, handle_opcodes_input_op0_limb_8),
            eval!(context, handle_opcodes_input_op0_limb_9),
            eval!(context, handle_opcodes_input_op0_limb_10),
            eval!(context, handle_opcodes_input_op0_limb_11),
            eval!(context, handle_opcodes_input_op0_limb_12),
            eval!(context, handle_opcodes_input_op0_limb_13),
            eval!(context, handle_opcodes_input_op0_limb_14),
            eval!(context, handle_opcodes_input_op0_limb_15),
            eval!(context, handle_opcodes_input_op0_limb_16),
            eval!(context, handle_opcodes_input_op0_limb_17),
            eval!(context, handle_opcodes_input_op0_limb_18),
            eval!(context, handle_opcodes_input_op0_limb_19),
            eval!(context, handle_opcodes_input_op0_limb_20),
            eval!(context, handle_opcodes_input_op0_limb_21),
            eval!(context, handle_opcodes_input_op0_limb_22),
            eval!(context, handle_opcodes_input_op0_limb_23),
            eval!(context, handle_opcodes_input_op0_limb_24),
            eval!(context, handle_opcodes_input_op0_limb_25),
            eval!(context, handle_opcodes_input_op0_limb_26),
            eval!(context, handle_opcodes_input_op0_limb_27),
            eval!(context, handle_opcodes_input_opcode_call),
            eval!(context, enabler),
            eval!(context, partial_limb_msb_col1),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let constraint_37_value = eval!(
        context,
        (handle_opcodes_input_opcode_call)
            * ((cond_felt_252_as_addr_output_tmp_fe176_5)
                - ((handle_opcodes_input_pc) + (handle_opcodes_input_instruction_size)))
    );
    acc.add_constraint(context, constraint_37_value);
    vec![]
}
