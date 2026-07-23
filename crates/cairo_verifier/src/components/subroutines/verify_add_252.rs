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
        verify_add_252_input_a_limb_0,
        verify_add_252_input_a_limb_1,
        verify_add_252_input_a_limb_2,
        verify_add_252_input_a_limb_3,
        verify_add_252_input_a_limb_4,
        verify_add_252_input_a_limb_5,
        verify_add_252_input_a_limb_6,
        verify_add_252_input_a_limb_7,
        verify_add_252_input_a_limb_8,
        verify_add_252_input_a_limb_9,
        verify_add_252_input_a_limb_10,
        verify_add_252_input_a_limb_11,
        verify_add_252_input_a_limb_12,
        verify_add_252_input_a_limb_13,
        verify_add_252_input_a_limb_14,
        verify_add_252_input_a_limb_15,
        verify_add_252_input_a_limb_16,
        verify_add_252_input_a_limb_17,
        verify_add_252_input_a_limb_18,
        verify_add_252_input_a_limb_19,
        verify_add_252_input_a_limb_20,
        verify_add_252_input_a_limb_21,
        verify_add_252_input_a_limb_22,
        verify_add_252_input_a_limb_23,
        verify_add_252_input_a_limb_24,
        verify_add_252_input_a_limb_25,
        verify_add_252_input_a_limb_26,
        verify_add_252_input_a_limb_27,
        verify_add_252_input_b_limb_0,
        verify_add_252_input_b_limb_1,
        verify_add_252_input_b_limb_2,
        verify_add_252_input_b_limb_3,
        verify_add_252_input_b_limb_4,
        verify_add_252_input_b_limb_5,
        verify_add_252_input_b_limb_6,
        verify_add_252_input_b_limb_7,
        verify_add_252_input_b_limb_8,
        verify_add_252_input_b_limb_9,
        verify_add_252_input_b_limb_10,
        verify_add_252_input_b_limb_11,
        verify_add_252_input_b_limb_12,
        verify_add_252_input_b_limb_13,
        verify_add_252_input_b_limb_14,
        verify_add_252_input_b_limb_15,
        verify_add_252_input_b_limb_16,
        verify_add_252_input_b_limb_17,
        verify_add_252_input_b_limb_18,
        verify_add_252_input_b_limb_19,
        verify_add_252_input_b_limb_20,
        verify_add_252_input_b_limb_21,
        verify_add_252_input_b_limb_22,
        verify_add_252_input_b_limb_23,
        verify_add_252_input_b_limb_24,
        verify_add_252_input_b_limb_25,
        verify_add_252_input_b_limb_26,
        verify_add_252_input_b_limb_27,
        verify_add_252_input_c_limb_0,
        verify_add_252_input_c_limb_1,
        verify_add_252_input_c_limb_2,
        verify_add_252_input_c_limb_3,
        verify_add_252_input_c_limb_4,
        verify_add_252_input_c_limb_5,
        verify_add_252_input_c_limb_6,
        verify_add_252_input_c_limb_7,
        verify_add_252_input_c_limb_8,
        verify_add_252_input_c_limb_9,
        verify_add_252_input_c_limb_10,
        verify_add_252_input_c_limb_11,
        verify_add_252_input_c_limb_12,
        verify_add_252_input_c_limb_13,
        verify_add_252_input_c_limb_14,
        verify_add_252_input_c_limb_15,
        verify_add_252_input_c_limb_16,
        verify_add_252_input_c_limb_17,
        verify_add_252_input_c_limb_18,
        verify_add_252_input_c_limb_19,
        verify_add_252_input_c_limb_20,
        verify_add_252_input_c_limb_21,
        verify_add_252_input_c_limb_22,
        verify_add_252_input_c_limb_23,
        verify_add_252_input_c_limb_24,
        verify_add_252_input_c_limb_25,
        verify_add_252_input_c_limb_26,
        verify_add_252_input_c_limb_27,
        enabler,
        sub_p_bit_col0,
    ] = input.try_into().unwrap();

    // sub_p_bit is a bit.
    let constraint_0_value = eval!(context, (sub_p_bit_col0) * ((sub_p_bit_col0) - (1)));
    acc.add_constraint(context, constraint_0_value);

    let carry_tmp_6572d_1 = eval!(
        context,
        ((((verify_add_252_input_a_limb_2) + (verify_add_252_input_b_limb_2))
            + (((((verify_add_252_input_a_limb_1) + (verify_add_252_input_b_limb_1))
                + (((((verify_add_252_input_a_limb_0) + (verify_add_252_input_b_limb_0))
                    - (verify_add_252_input_c_limb_0))
                    - (sub_p_bit_col0))
                    * (4194304)))
                - (verify_add_252_input_c_limb_1))
                * (4194304)))
            - (verify_add_252_input_c_limb_2))
            * (4194304)
    );

    let constraint_2_value =
        eval!(context, (carry_tmp_6572d_1) * (((carry_tmp_6572d_1) * (carry_tmp_6572d_1)) - (1)));
    acc.add_constraint(context, constraint_2_value);

    let carry_tmp_6572d_2 = eval!(
        context,
        ((((verify_add_252_input_a_limb_5) + (verify_add_252_input_b_limb_5))
            + (((((verify_add_252_input_a_limb_4) + (verify_add_252_input_b_limb_4))
                + (((((verify_add_252_input_a_limb_3) + (verify_add_252_input_b_limb_3))
                    + (carry_tmp_6572d_1))
                    - (verify_add_252_input_c_limb_3))
                    * (4194304)))
                - (verify_add_252_input_c_limb_4))
                * (4194304)))
            - (verify_add_252_input_c_limb_5))
            * (4194304)
    );

    let constraint_4_value =
        eval!(context, (carry_tmp_6572d_2) * (((carry_tmp_6572d_2) * (carry_tmp_6572d_2)) - (1)));
    acc.add_constraint(context, constraint_4_value);

    let carry_tmp_6572d_3 = eval!(
        context,
        ((((verify_add_252_input_a_limb_8) + (verify_add_252_input_b_limb_8))
            + (((((verify_add_252_input_a_limb_7) + (verify_add_252_input_b_limb_7))
                + (((((verify_add_252_input_a_limb_6) + (verify_add_252_input_b_limb_6))
                    + (carry_tmp_6572d_2))
                    - (verify_add_252_input_c_limb_6))
                    * (4194304)))
                - (verify_add_252_input_c_limb_7))
                * (4194304)))
            - (verify_add_252_input_c_limb_8))
            * (4194304)
    );

    let constraint_6_value =
        eval!(context, (carry_tmp_6572d_3) * (((carry_tmp_6572d_3) * (carry_tmp_6572d_3)) - (1)));
    acc.add_constraint(context, constraint_6_value);

    let carry_tmp_6572d_4 = eval!(
        context,
        ((((verify_add_252_input_a_limb_11) + (verify_add_252_input_b_limb_11))
            + (((((verify_add_252_input_a_limb_10) + (verify_add_252_input_b_limb_10))
                + (((((verify_add_252_input_a_limb_9) + (verify_add_252_input_b_limb_9))
                    + (carry_tmp_6572d_3))
                    - (verify_add_252_input_c_limb_9))
                    * (4194304)))
                - (verify_add_252_input_c_limb_10))
                * (4194304)))
            - (verify_add_252_input_c_limb_11))
            * (4194304)
    );

    let constraint_8_value =
        eval!(context, (carry_tmp_6572d_4) * (((carry_tmp_6572d_4) * (carry_tmp_6572d_4)) - (1)));
    acc.add_constraint(context, constraint_8_value);

    let carry_tmp_6572d_5 = eval!(
        context,
        ((((verify_add_252_input_a_limb_14) + (verify_add_252_input_b_limb_14))
            + (((((verify_add_252_input_a_limb_13) + (verify_add_252_input_b_limb_13))
                + (((((verify_add_252_input_a_limb_12) + (verify_add_252_input_b_limb_12))
                    + (carry_tmp_6572d_4))
                    - (verify_add_252_input_c_limb_12))
                    * (4194304)))
                - (verify_add_252_input_c_limb_13))
                * (4194304)))
            - (verify_add_252_input_c_limb_14))
            * (4194304)
    );

    let constraint_10_value =
        eval!(context, (carry_tmp_6572d_5) * (((carry_tmp_6572d_5) * (carry_tmp_6572d_5)) - (1)));
    acc.add_constraint(context, constraint_10_value);

    let carry_tmp_6572d_6 = eval!(
        context,
        ((((verify_add_252_input_a_limb_17) + (verify_add_252_input_b_limb_17))
            + (((((verify_add_252_input_a_limb_16) + (verify_add_252_input_b_limb_16))
                + (((((verify_add_252_input_a_limb_15) + (verify_add_252_input_b_limb_15))
                    + (carry_tmp_6572d_5))
                    - (verify_add_252_input_c_limb_15))
                    * (4194304)))
                - (verify_add_252_input_c_limb_16))
                * (4194304)))
            - (verify_add_252_input_c_limb_17))
            * (4194304)
    );

    let constraint_12_value =
        eval!(context, (carry_tmp_6572d_6) * (((carry_tmp_6572d_6) * (carry_tmp_6572d_6)) - (1)));
    acc.add_constraint(context, constraint_12_value);

    let carry_tmp_6572d_7 = eval!(
        context,
        ((((verify_add_252_input_a_limb_20) + (verify_add_252_input_b_limb_20))
            + (((((verify_add_252_input_a_limb_19) + (verify_add_252_input_b_limb_19))
                + (((((verify_add_252_input_a_limb_18) + (verify_add_252_input_b_limb_18))
                    + (carry_tmp_6572d_6))
                    - (verify_add_252_input_c_limb_18))
                    * (4194304)))
                - (verify_add_252_input_c_limb_19))
                * (4194304)))
            - (verify_add_252_input_c_limb_20))
            * (4194304)
    );

    let constraint_14_value =
        eval!(context, (carry_tmp_6572d_7) * (((carry_tmp_6572d_7) * (carry_tmp_6572d_7)) - (1)));
    acc.add_constraint(context, constraint_14_value);

    let carry_tmp_6572d_8 = eval!(
        context,
        ((((verify_add_252_input_a_limb_23) + (verify_add_252_input_b_limb_23))
            + (((((verify_add_252_input_a_limb_22) + (verify_add_252_input_b_limb_22))
                + ((((((verify_add_252_input_a_limb_21)
                    + (verify_add_252_input_b_limb_21))
                    + (carry_tmp_6572d_7))
                    - (verify_add_252_input_c_limb_21))
                    - ((136) * (sub_p_bit_col0)))
                    * (4194304)))
                - (verify_add_252_input_c_limb_22))
                * (4194304)))
            - (verify_add_252_input_c_limb_23))
            * (4194304)
    );

    let constraint_16_value =
        eval!(context, (carry_tmp_6572d_8) * (((carry_tmp_6572d_8) * (carry_tmp_6572d_8)) - (1)));
    acc.add_constraint(context, constraint_16_value);

    let carry_tmp_6572d_9 = eval!(
        context,
        ((((verify_add_252_input_a_limb_26) + (verify_add_252_input_b_limb_26))
            + (((((verify_add_252_input_a_limb_25) + (verify_add_252_input_b_limb_25))
                + (((((verify_add_252_input_a_limb_24) + (verify_add_252_input_b_limb_24))
                    + (carry_tmp_6572d_8))
                    - (verify_add_252_input_c_limb_24))
                    * (4194304)))
                - (verify_add_252_input_c_limb_25))
                * (4194304)))
            - (verify_add_252_input_c_limb_26))
            * (4194304)
    );

    let constraint_18_value =
        eval!(context, (carry_tmp_6572d_9) * (((carry_tmp_6572d_9) * (carry_tmp_6572d_9)) - (1)));
    acc.add_constraint(context, constraint_18_value);

    let constraint_19_value = eval!(
        context,
        ((((verify_add_252_input_a_limb_27) + (verify_add_252_input_b_limb_27))
            + (carry_tmp_6572d_9))
            - (verify_add_252_input_c_limb_27))
            - ((256) * (sub_p_bit_col0))
    );
    acc.add_constraint(context, constraint_19_value);
    vec![]
}
