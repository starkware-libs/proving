// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "RangeCheck_4_4", uses: 1 },
    RelationUse { relation_id: "RangeCheck_4_4_4_4", uses: 2 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_0,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_1,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_2,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_3,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_4,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_5,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_6,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_7,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_8,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_9,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_10,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_11,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_12,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_13,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_14,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_15,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_16,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_17,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_18,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_19,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_20,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_21,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_22,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_23,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_24,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_25,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_26,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_27,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_28,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_29,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_30,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_31,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_32,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_33,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_34,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_35,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_36,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_37,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_38,
        linear_combination_n_4_coefs_4_2_m2_1_input_limb_39,
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

    let carry_0_tmp_5073a_2 = eval!(
        context,
        (((((((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_0))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_10)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_20)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_30))
            - (combination_limb_0_col0))
            - (p_coef_col10))
            * (16)
    );

    let carry_1_tmp_5073a_3 = eval!(
        context,
        ((((((carry_0_tmp_5073a_2)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_1)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_11)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_21)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_31))
            - (combination_limb_1_col1))
            * (16)
    );

    let carry_2_tmp_5073a_4 = eval!(
        context,
        ((((((carry_1_tmp_5073a_3)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_2)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_12)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_22)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_32))
            - (combination_limb_2_col2))
            * (16)
    );

    let carry_3_tmp_5073a_5 = eval!(
        context,
        ((((((carry_2_tmp_5073a_4)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_3)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_13)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_23)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_33))
            - (combination_limb_3_col3))
            * (16)
    );

    let carry_4_tmp_5073a_6 = eval!(
        context,
        ((((((carry_3_tmp_5073a_5)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_4)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_14)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_24)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_34))
            - (combination_limb_4_col4))
            * (16)
    );

    let carry_5_tmp_5073a_7 = eval!(
        context,
        ((((((carry_4_tmp_5073a_6)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_5)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_15)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_25)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_35))
            - (combination_limb_5_col5))
            * (16)
    );

    let carry_6_tmp_5073a_8 = eval!(
        context,
        ((((((carry_5_tmp_5073a_7)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_6)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_16)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_26)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_36))
            - (combination_limb_6_col6))
            * (16)
    );

    let carry_7_tmp_5073a_9 = eval!(
        context,
        (((((((carry_6_tmp_5073a_8)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_7)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_17)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_27)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_37))
            - (combination_limb_7_col7))
            - ((p_coef_col10) * (136)))
            * (16)
    );

    let carry_8_tmp_5073a_10 = eval!(
        context,
        ((((((carry_7_tmp_5073a_9)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_8)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_18)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_28)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_38))
            - (combination_limb_8_col8))
            * (16)
    );

    // final limb constraint.
    let constraint_9_value = eval!(
        context,
        ((((((carry_8_tmp_5073a_10)
            + ((4) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_9)))
            + ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_19)))
            - ((2) * (linear_combination_n_4_coefs_4_2_m2_1_input_limb_29)))
            + (linear_combination_n_4_coefs_4_2_m2_1_input_limb_39))
            - (combination_limb_9_col9))
            - ((p_coef_col10) * (256))
    );
    acc.add_constraint(context, constraint_9_value);

    // Use RangeCheck_4_4_4_4.
    let tuple_10 = &[
        eval!(context, 1027333874),
        eval!(context, (p_coef_col10) + (3)),
        eval!(context, (carry_0_tmp_5073a_2) + (3)),
        eval!(context, (carry_1_tmp_5073a_3) + (3)),
        eval!(context, (carry_2_tmp_5073a_4) + (3)),
    ];
    let numerator_10 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Use RangeCheck_4_4_4_4.
    let tuple_11 = &[
        eval!(context, 1027333874),
        eval!(context, (carry_3_tmp_5073a_5) + (3)),
        eval!(context, (carry_4_tmp_5073a_6) + (3)),
        eval!(context, (carry_5_tmp_5073a_7) + (3)),
        eval!(context, (carry_6_tmp_5073a_8) + (3)),
    ];
    let numerator_11 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Use RangeCheck_4_4.
    let tuple_12 = &[
        eval!(context, 1651211826),
        eval!(context, (carry_7_tmp_5073a_9) + (3)),
        eval!(context, (carry_8_tmp_5073a_10) + (3)),
    ];
    let numerator_12 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_12, tuple_12);
    vec![]
}
