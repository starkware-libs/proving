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
        cond_felt_252_as_rel_imm_input_limb_0,
        cond_felt_252_as_rel_imm_input_limb_1,
        cond_felt_252_as_rel_imm_input_limb_2,
        cond_felt_252_as_rel_imm_input_limb_3,
        cond_felt_252_as_rel_imm_input_limb_4,
        cond_felt_252_as_rel_imm_input_limb_5,
        cond_felt_252_as_rel_imm_input_limb_6,
        cond_felt_252_as_rel_imm_input_limb_7,
        cond_felt_252_as_rel_imm_input_limb_8,
        cond_felt_252_as_rel_imm_input_limb_9,
        cond_felt_252_as_rel_imm_input_limb_10,
        cond_felt_252_as_rel_imm_input_limb_11,
        cond_felt_252_as_rel_imm_input_limb_12,
        cond_felt_252_as_rel_imm_input_limb_13,
        cond_felt_252_as_rel_imm_input_limb_14,
        cond_felt_252_as_rel_imm_input_limb_15,
        cond_felt_252_as_rel_imm_input_limb_16,
        cond_felt_252_as_rel_imm_input_limb_17,
        cond_felt_252_as_rel_imm_input_limb_18,
        cond_felt_252_as_rel_imm_input_limb_19,
        cond_felt_252_as_rel_imm_input_limb_20,
        cond_felt_252_as_rel_imm_input_limb_21,
        cond_felt_252_as_rel_imm_input_limb_22,
        cond_felt_252_as_rel_imm_input_limb_23,
        cond_felt_252_as_rel_imm_input_limb_24,
        cond_felt_252_as_rel_imm_input_limb_25,
        cond_felt_252_as_rel_imm_input_limb_26,
        cond_felt_252_as_rel_imm_input_limb_27,
        cond_felt_252_as_rel_imm_input_limb_28,
        enabler,
        msb_col0,
        mid_limbs_set_col1,
        partial_limb_msb_col2,
    ] = input.try_into().unwrap();

    let [
        decode_small_sign_output_tmp_37f57_2_limb3_7_high_bits,
        decode_small_sign_output_tmp_37f57_2_limbs4_to_20,
        decode_small_sign_output_tmp_37f57_2_limb21,
        decode_small_sign_output_tmp_37f57_2_limb27,
    ] = decode_small_sign::accumulate_constraints(
        &[eval!(context, enabler), eval!(context, msb_col0), eval!(context, mid_limbs_set_col1)],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let remainder_bits_tmp_37f57_3 = eval!(
        context,
        (cond_felt_252_as_rel_imm_input_limb_3)
            - (decode_small_sign_output_tmp_37f57_2_limb3_7_high_bits)
    );

    cond_range_check_2::accumulate_constraints(
        &[
            eval!(context, remainder_bits_tmp_37f57_3),
            eval!(context, cond_felt_252_as_rel_imm_input_limb_28),
            eval!(context, enabler),
            eval!(context, partial_limb_msb_col2),
        ],
        context,
        component_data,
        acc,
    );

    // When the condition holds, limbs 4-20 must be zero or 0x1ff.
    let constraint_3_value = eval!(
        context,
        (cond_felt_252_as_rel_imm_input_limb_28)
            * ((((((((((((((((((cond_felt_252_as_rel_imm_input_limb_4)
                + (cond_felt_252_as_rel_imm_input_limb_5))
                + (cond_felt_252_as_rel_imm_input_limb_6))
                + (cond_felt_252_as_rel_imm_input_limb_7))
                + (cond_felt_252_as_rel_imm_input_limb_8))
                + (cond_felt_252_as_rel_imm_input_limb_9))
                + (cond_felt_252_as_rel_imm_input_limb_10))
                + (cond_felt_252_as_rel_imm_input_limb_11))
                + (cond_felt_252_as_rel_imm_input_limb_12))
                + (cond_felt_252_as_rel_imm_input_limb_13))
                + (cond_felt_252_as_rel_imm_input_limb_14))
                + (cond_felt_252_as_rel_imm_input_limb_15))
                + (cond_felt_252_as_rel_imm_input_limb_16))
                + (cond_felt_252_as_rel_imm_input_limb_17))
                + (cond_felt_252_as_rel_imm_input_limb_18))
                + (cond_felt_252_as_rel_imm_input_limb_19))
                + (cond_felt_252_as_rel_imm_input_limb_20))
                - ((decode_small_sign_output_tmp_37f57_2_limbs4_to_20) * (17)))
    );
    acc.add_constraint(context, constraint_3_value);

    // When the condition holds, limb 21 must be 0x0, 0x88 or 0x87.
    let constraint_4_value = eval!(
        context,
        (cond_felt_252_as_rel_imm_input_limb_28)
            * ((cond_felt_252_as_rel_imm_input_limb_21)
                - (decode_small_sign_output_tmp_37f57_2_limb21))
    );
    acc.add_constraint(context, constraint_4_value);

    // When the condition holds, limbs 22-26 must be zero.
    let constraint_5_value = eval!(
        context,
        (cond_felt_252_as_rel_imm_input_limb_28)
            * (((((cond_felt_252_as_rel_imm_input_limb_22)
                + (cond_felt_252_as_rel_imm_input_limb_23))
                + (cond_felt_252_as_rel_imm_input_limb_24))
                + (cond_felt_252_as_rel_imm_input_limb_25))
                + (cond_felt_252_as_rel_imm_input_limb_26))
    );
    acc.add_constraint(context, constraint_5_value);

    // When the condition holds, limb 27 must be 0x0 or 0x100.
    let constraint_6_value = eval!(
        context,
        (cond_felt_252_as_rel_imm_input_limb_28)
            * ((cond_felt_252_as_rel_imm_input_limb_27)
                - (decode_small_sign_output_tmp_37f57_2_limb27))
    );
    acc.add_constraint(context, constraint_6_value);
    vec![eval!(
        context,
        (((((cond_felt_252_as_rel_imm_input_limb_0)
            + ((cond_felt_252_as_rel_imm_input_limb_1) * (512)))
            + ((cond_felt_252_as_rel_imm_input_limb_2) * (262144)))
            + ((remainder_bits_tmp_37f57_3) * (134217728)))
            - (msb_col0))
            - ((536870912) * (mid_limbs_set_col1))
    )]
}
