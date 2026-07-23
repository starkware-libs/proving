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
        linear_combination_n_1_coefs_2_input_limb_0,
        linear_combination_n_1_coefs_2_input_limb_1,
        linear_combination_n_1_coefs_2_input_limb_2,
        linear_combination_n_1_coefs_2_input_limb_3,
        linear_combination_n_1_coefs_2_input_limb_4,
        linear_combination_n_1_coefs_2_input_limb_5,
        linear_combination_n_1_coefs_2_input_limb_6,
        linear_combination_n_1_coefs_2_input_limb_7,
        linear_combination_n_1_coefs_2_input_limb_8,
        linear_combination_n_1_coefs_2_input_limb_9,
        enabler,
        combination_limb_0_col0,
        combination_limb_1_col1,
        combination_limb_2_col2,
        combination_limb_3_col3,
        combination_limb_4_col4,
        combination_limb_5_col5,
        combination_limb_6_col6,
        combination_limb_7_col7,
        combination_limb_8_col8,
        combination_limb_9_col9,
        p_coef_col10,
    ] = input.try_into().unwrap();

    let carry_0_tmp_e3263_2 = eval!(
        context,
        ((((2) * (linear_combination_n_1_coefs_2_input_limb_0)) - (combination_limb_0_col0))
            - (p_coef_col10))
            * (16)
    );

    let carry_1_tmp_e3263_3 = eval!(
        context,
        (((carry_0_tmp_e3263_2) + ((2) * (linear_combination_n_1_coefs_2_input_limb_1)))
            - (combination_limb_1_col1))
            * (16)
    );

    let carry_2_tmp_e3263_4 = eval!(
        context,
        (((carry_1_tmp_e3263_3) + ((2) * (linear_combination_n_1_coefs_2_input_limb_2)))
            - (combination_limb_2_col2))
            * (16)
    );

    let carry_3_tmp_e3263_5 = eval!(
        context,
        (((carry_2_tmp_e3263_4) + ((2) * (linear_combination_n_1_coefs_2_input_limb_3)))
            - (combination_limb_3_col3))
            * (16)
    );

    let carry_4_tmp_e3263_6 = eval!(
        context,
        (((carry_3_tmp_e3263_5) + ((2) * (linear_combination_n_1_coefs_2_input_limb_4)))
            - (combination_limb_4_col4))
            * (16)
    );

    let carry_5_tmp_e3263_7 = eval!(
        context,
        (((carry_4_tmp_e3263_6) + ((2) * (linear_combination_n_1_coefs_2_input_limb_5)))
            - (combination_limb_5_col5))
            * (16)
    );

    let carry_6_tmp_e3263_8 = eval!(
        context,
        (((carry_5_tmp_e3263_7) + ((2) * (linear_combination_n_1_coefs_2_input_limb_6)))
            - (combination_limb_6_col6))
            * (16)
    );

    let carry_7_tmp_e3263_9 = eval!(
        context,
        ((((carry_6_tmp_e3263_8) + ((2) * (linear_combination_n_1_coefs_2_input_limb_7)))
            - (combination_limb_7_col7))
            - ((p_coef_col10) * (136)))
            * (16)
    );

    let carry_8_tmp_e3263_10 = eval!(
        context,
        (((carry_7_tmp_e3263_9) + ((2) * (linear_combination_n_1_coefs_2_input_limb_8)))
            - (combination_limb_8_col8))
            * (16)
    );

    // final limb constraint.
    let constraint_9_value = eval!(
        context,
        (((carry_8_tmp_e3263_10) + ((2) * (linear_combination_n_1_coefs_2_input_limb_9)))
            - (combination_limb_9_col9))
            - ((p_coef_col10) * (256))
    );
    acc.add_constraint(context, constraint_9_value);

    // carry constraint 0.
    let constraint_10_value =
        eval!(context, (((p_coef_col10) * (p_coef_col10)) * (p_coef_col10)) - (p_coef_col10));
    acc.add_constraint(context, constraint_10_value);

    let biased_carry_1_tmp_e3263_11 = eval!(context, carry_0_tmp_e3263_2);

    // carry constraint 1.
    let constraint_12_value = eval!(
        context,
        (((biased_carry_1_tmp_e3263_11) * (biased_carry_1_tmp_e3263_11))
            * (biased_carry_1_tmp_e3263_11))
            - (biased_carry_1_tmp_e3263_11)
    );
    acc.add_constraint(context, constraint_12_value);

    let biased_carry_2_tmp_e3263_12 = eval!(context, carry_1_tmp_e3263_3);

    // carry constraint 2.
    let constraint_14_value = eval!(
        context,
        (((biased_carry_2_tmp_e3263_12) * (biased_carry_2_tmp_e3263_12))
            * (biased_carry_2_tmp_e3263_12))
            - (biased_carry_2_tmp_e3263_12)
    );
    acc.add_constraint(context, constraint_14_value);

    let biased_carry_3_tmp_e3263_13 = eval!(context, carry_2_tmp_e3263_4);

    // carry constraint 3.
    let constraint_16_value = eval!(
        context,
        (((biased_carry_3_tmp_e3263_13) * (biased_carry_3_tmp_e3263_13))
            * (biased_carry_3_tmp_e3263_13))
            - (biased_carry_3_tmp_e3263_13)
    );
    acc.add_constraint(context, constraint_16_value);

    let biased_carry_4_tmp_e3263_14 = eval!(context, carry_3_tmp_e3263_5);

    // carry constraint 4.
    let constraint_18_value = eval!(
        context,
        (((biased_carry_4_tmp_e3263_14) * (biased_carry_4_tmp_e3263_14))
            * (biased_carry_4_tmp_e3263_14))
            - (biased_carry_4_tmp_e3263_14)
    );
    acc.add_constraint(context, constraint_18_value);

    let biased_carry_5_tmp_e3263_15 = eval!(context, carry_4_tmp_e3263_6);

    // carry constraint 5.
    let constraint_20_value = eval!(
        context,
        (((biased_carry_5_tmp_e3263_15) * (biased_carry_5_tmp_e3263_15))
            * (biased_carry_5_tmp_e3263_15))
            - (biased_carry_5_tmp_e3263_15)
    );
    acc.add_constraint(context, constraint_20_value);

    let biased_carry_6_tmp_e3263_16 = eval!(context, carry_5_tmp_e3263_7);

    // carry constraint 6.
    let constraint_22_value = eval!(
        context,
        (((biased_carry_6_tmp_e3263_16) * (biased_carry_6_tmp_e3263_16))
            * (biased_carry_6_tmp_e3263_16))
            - (biased_carry_6_tmp_e3263_16)
    );
    acc.add_constraint(context, constraint_22_value);

    let biased_carry_7_tmp_e3263_17 = eval!(context, carry_6_tmp_e3263_8);

    // carry constraint 7.
    let constraint_24_value = eval!(
        context,
        (((biased_carry_7_tmp_e3263_17) * (biased_carry_7_tmp_e3263_17))
            * (biased_carry_7_tmp_e3263_17))
            - (biased_carry_7_tmp_e3263_17)
    );
    acc.add_constraint(context, constraint_24_value);

    let biased_carry_8_tmp_e3263_18 = eval!(context, carry_7_tmp_e3263_9);

    // carry constraint 8.
    let constraint_26_value = eval!(
        context,
        (((biased_carry_8_tmp_e3263_18) * (biased_carry_8_tmp_e3263_18))
            * (biased_carry_8_tmp_e3263_18))
            - (biased_carry_8_tmp_e3263_18)
    );
    acc.add_constraint(context, constraint_26_value);

    let biased_carry_9_tmp_e3263_19 = eval!(context, carry_8_tmp_e3263_10);

    // carry constraint 9.
    let constraint_28_value = eval!(
        context,
        (((biased_carry_9_tmp_e3263_19) * (biased_carry_9_tmp_e3263_19))
            * (biased_carry_9_tmp_e3263_19))
            - (biased_carry_9_tmp_e3263_19)
    );
    acc.add_constraint(context, constraint_28_value);
    vec![]
}
