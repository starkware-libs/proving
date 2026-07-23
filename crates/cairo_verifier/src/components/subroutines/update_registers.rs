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
    let [
        update_registers_input_pc,
        update_registers_input_ap,
        update_registers_input_fp,
        update_registers_input_pc_update_jump,
        update_registers_input_pc_update_jump_rel,
        update_registers_input_pc_update_jnz,
        update_registers_input_ap_update_add,
        update_registers_input_ap_update_add_1,
        update_registers_input_opcode_call,
        update_registers_input_opcode_ret,
        update_registers_input_pc_update_regular,
        update_registers_input_fp_update_regular,
        update_registers_input_instruction_size,
        update_registers_input_dst_limb_0,
        update_registers_input_dst_limb_1,
        update_registers_input_dst_limb_2,
        update_registers_input_dst_limb_3,
        update_registers_input_dst_limb_4,
        update_registers_input_dst_limb_5,
        update_registers_input_dst_limb_6,
        update_registers_input_dst_limb_7,
        update_registers_input_dst_limb_8,
        update_registers_input_dst_limb_9,
        update_registers_input_dst_limb_10,
        update_registers_input_dst_limb_11,
        update_registers_input_dst_limb_12,
        update_registers_input_dst_limb_13,
        update_registers_input_dst_limb_14,
        update_registers_input_dst_limb_15,
        update_registers_input_dst_limb_16,
        update_registers_input_dst_limb_17,
        update_registers_input_dst_limb_18,
        update_registers_input_dst_limb_19,
        update_registers_input_dst_limb_20,
        update_registers_input_dst_limb_21,
        update_registers_input_dst_limb_22,
        update_registers_input_dst_limb_23,
        update_registers_input_dst_limb_24,
        update_registers_input_dst_limb_25,
        update_registers_input_dst_limb_26,
        update_registers_input_dst_limb_27,
        update_registers_input_op1_limb_0,
        update_registers_input_op1_limb_1,
        update_registers_input_op1_limb_2,
        update_registers_input_op1_limb_3,
        update_registers_input_op1_limb_4,
        update_registers_input_op1_limb_5,
        update_registers_input_op1_limb_6,
        update_registers_input_op1_limb_7,
        update_registers_input_op1_limb_8,
        update_registers_input_op1_limb_9,
        update_registers_input_op1_limb_10,
        update_registers_input_op1_limb_11,
        update_registers_input_op1_limb_12,
        update_registers_input_op1_limb_13,
        update_registers_input_op1_limb_14,
        update_registers_input_op1_limb_15,
        update_registers_input_op1_limb_16,
        update_registers_input_op1_limb_17,
        update_registers_input_op1_limb_18,
        update_registers_input_op1_limb_19,
        update_registers_input_op1_limb_20,
        update_registers_input_op1_limb_21,
        update_registers_input_op1_limb_22,
        update_registers_input_op1_limb_23,
        update_registers_input_op1_limb_24,
        update_registers_input_op1_limb_25,
        update_registers_input_op1_limb_26,
        update_registers_input_op1_limb_27,
        update_registers_input_res_limb_0,
        update_registers_input_res_limb_1,
        update_registers_input_res_limb_2,
        update_registers_input_res_limb_3,
        update_registers_input_res_limb_4,
        update_registers_input_res_limb_5,
        update_registers_input_res_limb_6,
        update_registers_input_res_limb_7,
        update_registers_input_res_limb_8,
        update_registers_input_res_limb_9,
        update_registers_input_res_limb_10,
        update_registers_input_res_limb_11,
        update_registers_input_res_limb_12,
        update_registers_input_res_limb_13,
        update_registers_input_res_limb_14,
        update_registers_input_res_limb_15,
        update_registers_input_res_limb_16,
        update_registers_input_res_limb_17,
        update_registers_input_res_limb_18,
        update_registers_input_res_limb_19,
        update_registers_input_res_limb_20,
        update_registers_input_res_limb_21,
        update_registers_input_res_limb_22,
        update_registers_input_res_limb_23,
        update_registers_input_res_limb_24,
        update_registers_input_res_limb_25,
        update_registers_input_res_limb_26,
        update_registers_input_res_limb_27,
        enabler,
        partial_limb_msb_col0,
        msb_col1,
        mid_limbs_set_col2,
        partial_limb_msb_col3,
        dst_sum_squares_inv_col4,
        dst_sum_inv_col5,
        op1_as_rel_imm_cond_col6,
        msb_col7,
        mid_limbs_set_col8,
        partial_limb_msb_col9,
        next_pc_jnz_col10,
        next_pc_col11,
        next_ap_col12,
        range_check_29_bot11bits_col13,
        next_fp_col14,
    ] = input.try_into().unwrap();

    let [cond_felt_252_as_addr_output_tmp_1f3b5_2] = cond_felt_252_as_addr::accumulate_constraints(
        &[
            eval!(context, update_registers_input_dst_limb_0),
            eval!(context, update_registers_input_dst_limb_1),
            eval!(context, update_registers_input_dst_limb_2),
            eval!(context, update_registers_input_dst_limb_3),
            eval!(context, update_registers_input_dst_limb_4),
            eval!(context, update_registers_input_dst_limb_5),
            eval!(context, update_registers_input_dst_limb_6),
            eval!(context, update_registers_input_dst_limb_7),
            eval!(context, update_registers_input_dst_limb_8),
            eval!(context, update_registers_input_dst_limb_9),
            eval!(context, update_registers_input_dst_limb_10),
            eval!(context, update_registers_input_dst_limb_11),
            eval!(context, update_registers_input_dst_limb_12),
            eval!(context, update_registers_input_dst_limb_13),
            eval!(context, update_registers_input_dst_limb_14),
            eval!(context, update_registers_input_dst_limb_15),
            eval!(context, update_registers_input_dst_limb_16),
            eval!(context, update_registers_input_dst_limb_17),
            eval!(context, update_registers_input_dst_limb_18),
            eval!(context, update_registers_input_dst_limb_19),
            eval!(context, update_registers_input_dst_limb_20),
            eval!(context, update_registers_input_dst_limb_21),
            eval!(context, update_registers_input_dst_limb_22),
            eval!(context, update_registers_input_dst_limb_23),
            eval!(context, update_registers_input_dst_limb_24),
            eval!(context, update_registers_input_dst_limb_25),
            eval!(context, update_registers_input_dst_limb_26),
            eval!(context, update_registers_input_dst_limb_27),
            eval!(context, update_registers_input_opcode_ret),
            eval!(context, enabler),
            eval!(context, partial_limb_msb_col0),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [cond_felt_252_as_rel_imm_output_tmp_1f3b5_9] =
        cond_felt_252_as_rel_imm::accumulate_constraints(
            &[
                eval!(context, update_registers_input_res_limb_0),
                eval!(context, update_registers_input_res_limb_1),
                eval!(context, update_registers_input_res_limb_2),
                eval!(context, update_registers_input_res_limb_3),
                eval!(context, update_registers_input_res_limb_4),
                eval!(context, update_registers_input_res_limb_5),
                eval!(context, update_registers_input_res_limb_6),
                eval!(context, update_registers_input_res_limb_7),
                eval!(context, update_registers_input_res_limb_8),
                eval!(context, update_registers_input_res_limb_9),
                eval!(context, update_registers_input_res_limb_10),
                eval!(context, update_registers_input_res_limb_11),
                eval!(context, update_registers_input_res_limb_12),
                eval!(context, update_registers_input_res_limb_13),
                eval!(context, update_registers_input_res_limb_14),
                eval!(context, update_registers_input_res_limb_15),
                eval!(context, update_registers_input_res_limb_16),
                eval!(context, update_registers_input_res_limb_17),
                eval!(context, update_registers_input_res_limb_18),
                eval!(context, update_registers_input_res_limb_19),
                eval!(context, update_registers_input_res_limb_20),
                eval!(context, update_registers_input_res_limb_21),
                eval!(context, update_registers_input_res_limb_22),
                eval!(context, update_registers_input_res_limb_23),
                eval!(context, update_registers_input_res_limb_24),
                eval!(context, update_registers_input_res_limb_25),
                eval!(context, update_registers_input_res_limb_26),
                eval!(context, update_registers_input_res_limb_27),
                eval!(
                    context,
                    (update_registers_input_pc_update_jump_rel)
                        + (update_registers_input_ap_update_add)
                ),
                eval!(context, enabler),
                eval!(context, msb_col1),
                eval!(context, mid_limbs_set_col2),
                eval!(context, partial_limb_msb_col3),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let diff_from_p_tmp_1f3b5_10 = eval!(context, (update_registers_input_dst_limb_0) - (1));

    let diff_from_p_tmp_1f3b5_11 = eval!(context, (update_registers_input_dst_limb_21) - (136));

    let diff_from_p_tmp_1f3b5_12 = eval!(context, (update_registers_input_dst_limb_27) - (256));

    // dst_not_p.
    let constraint_5_value = eval!(
        context,
        ((((((((((((((((((((((((((((((diff_from_p_tmp_1f3b5_10)
            * (diff_from_p_tmp_1f3b5_10))
            + (update_registers_input_dst_limb_1))
            + (update_registers_input_dst_limb_2))
            + (update_registers_input_dst_limb_3))
            + (update_registers_input_dst_limb_4))
            + (update_registers_input_dst_limb_5))
            + (update_registers_input_dst_limb_6))
            + (update_registers_input_dst_limb_7))
            + (update_registers_input_dst_limb_8))
            + (update_registers_input_dst_limb_9))
            + (update_registers_input_dst_limb_10))
            + (update_registers_input_dst_limb_11))
            + (update_registers_input_dst_limb_12))
            + (update_registers_input_dst_limb_13))
            + (update_registers_input_dst_limb_14))
            + (update_registers_input_dst_limb_15))
            + (update_registers_input_dst_limb_16))
            + (update_registers_input_dst_limb_17))
            + (update_registers_input_dst_limb_18))
            + (update_registers_input_dst_limb_19))
            + (update_registers_input_dst_limb_20))
            + ((diff_from_p_tmp_1f3b5_11) * (diff_from_p_tmp_1f3b5_11)))
            + (update_registers_input_dst_limb_22))
            + (update_registers_input_dst_limb_23))
            + (update_registers_input_dst_limb_24))
            + (update_registers_input_dst_limb_25))
            + (update_registers_input_dst_limb_26))
            + ((diff_from_p_tmp_1f3b5_12) * (diff_from_p_tmp_1f3b5_12)))
            * (dst_sum_squares_inv_col4))
            - (1)
    );
    acc.add_constraint(context, constraint_5_value);

    let dst_sum_tmp_1f3b5_13 = eval!(
        context,
        (((((((((((((((((((((((((((update_registers_input_dst_limb_0)
            + (update_registers_input_dst_limb_1))
            + (update_registers_input_dst_limb_2))
            + (update_registers_input_dst_limb_3))
            + (update_registers_input_dst_limb_4))
            + (update_registers_input_dst_limb_5))
            + (update_registers_input_dst_limb_6))
            + (update_registers_input_dst_limb_7))
            + (update_registers_input_dst_limb_8))
            + (update_registers_input_dst_limb_9))
            + (update_registers_input_dst_limb_10))
            + (update_registers_input_dst_limb_11))
            + (update_registers_input_dst_limb_12))
            + (update_registers_input_dst_limb_13))
            + (update_registers_input_dst_limb_14))
            + (update_registers_input_dst_limb_15))
            + (update_registers_input_dst_limb_16))
            + (update_registers_input_dst_limb_17))
            + (update_registers_input_dst_limb_18))
            + (update_registers_input_dst_limb_19))
            + (update_registers_input_dst_limb_20))
            + (update_registers_input_dst_limb_21))
            + (update_registers_input_dst_limb_22))
            + (update_registers_input_dst_limb_23))
            + (update_registers_input_dst_limb_24))
            + (update_registers_input_dst_limb_25))
            + (update_registers_input_dst_limb_26))
            + (update_registers_input_dst_limb_27)
    );

    // op1_as_rel_imm_cond.
    let constraint_7_value = eval!(
        context,
        (op1_as_rel_imm_cond_col6)
            - ((update_registers_input_pc_update_jnz) * (dst_sum_tmp_1f3b5_13))
    );
    acc.add_constraint(context, constraint_7_value);

    let [cond_felt_252_as_rel_imm_output_tmp_1f3b5_21] =
        cond_felt_252_as_rel_imm::accumulate_constraints(
            &[
                eval!(context, update_registers_input_op1_limb_0),
                eval!(context, update_registers_input_op1_limb_1),
                eval!(context, update_registers_input_op1_limb_2),
                eval!(context, update_registers_input_op1_limb_3),
                eval!(context, update_registers_input_op1_limb_4),
                eval!(context, update_registers_input_op1_limb_5),
                eval!(context, update_registers_input_op1_limb_6),
                eval!(context, update_registers_input_op1_limb_7),
                eval!(context, update_registers_input_op1_limb_8),
                eval!(context, update_registers_input_op1_limb_9),
                eval!(context, update_registers_input_op1_limb_10),
                eval!(context, update_registers_input_op1_limb_11),
                eval!(context, update_registers_input_op1_limb_12),
                eval!(context, update_registers_input_op1_limb_13),
                eval!(context, update_registers_input_op1_limb_14),
                eval!(context, update_registers_input_op1_limb_15),
                eval!(context, update_registers_input_op1_limb_16),
                eval!(context, update_registers_input_op1_limb_17),
                eval!(context, update_registers_input_op1_limb_18),
                eval!(context, update_registers_input_op1_limb_19),
                eval!(context, update_registers_input_op1_limb_20),
                eval!(context, update_registers_input_op1_limb_21),
                eval!(context, update_registers_input_op1_limb_22),
                eval!(context, update_registers_input_op1_limb_23),
                eval!(context, update_registers_input_op1_limb_24),
                eval!(context, update_registers_input_op1_limb_25),
                eval!(context, update_registers_input_op1_limb_26),
                eval!(context, update_registers_input_op1_limb_27),
                eval!(context, op1_as_rel_imm_cond_col6),
                eval!(context, enabler),
                eval!(context, msb_col7),
                eval!(context, mid_limbs_set_col8),
                eval!(context, partial_limb_msb_col9),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    // Constraint1 for conditional jump.
    let constraint_9_value = eval!(
        context,
        ((next_pc_jnz_col10)
            - ((update_registers_input_pc) + (cond_felt_252_as_rel_imm_output_tmp_1f3b5_21)))
            * (dst_sum_tmp_1f3b5_13)
    );
    acc.add_constraint(context, constraint_9_value);

    // Constraint2 for conditional jump.
    let constraint_10_value = eval!(
        context,
        ((next_pc_jnz_col10)
            - ((update_registers_input_pc) + (update_registers_input_instruction_size)))
            * (((dst_sum_tmp_1f3b5_13) * (dst_sum_inv_col5)) - (1))
    );
    acc.add_constraint(context, constraint_10_value);

    // next_pc.
    let constraint_11_value = eval!(
        context,
        (next_pc_col11)
            - (((((update_registers_input_pc_update_regular)
                * ((update_registers_input_pc) + (update_registers_input_instruction_size)))
                + ((update_registers_input_pc_update_jump)
                    * (cond_felt_252_as_rel_imm_output_tmp_1f3b5_9)))
                + ((update_registers_input_pc_update_jump_rel)
                    * ((update_registers_input_pc)
                        + (cond_felt_252_as_rel_imm_output_tmp_1f3b5_9))))
                + ((update_registers_input_pc_update_jnz) * (next_pc_jnz_col10)))
    );
    acc.add_constraint(context, constraint_11_value);

    // next_ap.
    let constraint_12_value = eval!(
        context,
        (next_ap_col12)
            - ((((update_registers_input_ap)
                + ((update_registers_input_ap_update_add)
                    * (cond_felt_252_as_rel_imm_output_tmp_1f3b5_9)))
                + (update_registers_input_ap_update_add_1))
                + ((update_registers_input_opcode_call) * (2)))
    );
    acc.add_constraint(context, constraint_12_value);

    range_check_29::accumulate_constraints(
        &[
            eval!(context, next_ap_col12),
            eval!(context, enabler),
            eval!(context, range_check_29_bot11bits_col13),
        ],
        context,
        component_data,
        acc,
    );

    // next_fp.
    let constraint_14_value = eval!(
        context,
        (next_fp_col14)
            - ((((update_registers_input_fp_update_regular) * (update_registers_input_fp))
                + ((update_registers_input_opcode_ret)
                    * (cond_felt_252_as_addr_output_tmp_1f3b5_2)))
                + ((update_registers_input_opcode_call) * ((update_registers_input_ap) + (2))))
    );
    acc.add_constraint(context, constraint_14_value);
    vec![]
}
