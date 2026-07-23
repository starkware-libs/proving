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
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_0,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_1,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_2,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_3,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_4,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_5,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_6,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_7,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_8,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_9,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_10,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_11,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_12,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_13,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_14,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_15,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_16,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_17,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_18,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_19,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_20,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_21,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_22,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_23,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_24,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_25,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_26,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_27,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_28,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_29,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_30,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_31,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_32,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_33,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_34,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_35,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_36,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_37,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_38,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_39,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_40,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_41,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_42,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_43,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_44,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_45,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_46,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_47,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_48,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_49,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_50,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_51,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_52,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_53,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_54,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_55,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_56,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_57,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_58,
        linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_59,
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

    let carry_0_tmp_290f3_2 = eval!(
        context,
        (((((((((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_0))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_10)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_20)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_30))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_40))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_50))
            - (combination_limb_0_col0))
            - (p_coef_col10))
            * (16)
    );

    let carry_1_tmp_290f3_3 = eval!(
        context,
        ((((((((carry_0_tmp_290f3_2)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_1)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_11)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_21)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_31))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_41))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_51))
            - (combination_limb_1_col1))
            * (16)
    );

    let carry_2_tmp_290f3_4 = eval!(
        context,
        ((((((((carry_1_tmp_290f3_3)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_2)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_12)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_22)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_32))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_42))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_52))
            - (combination_limb_2_col2))
            * (16)
    );

    let carry_3_tmp_290f3_5 = eval!(
        context,
        ((((((((carry_2_tmp_290f3_4)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_3)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_13)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_23)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_33))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_43))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_53))
            - (combination_limb_3_col3))
            * (16)
    );

    let carry_4_tmp_290f3_6 = eval!(
        context,
        ((((((((carry_3_tmp_290f3_5)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_4)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_14)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_24)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_34))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_44))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_54))
            - (combination_limb_4_col4))
            * (16)
    );

    let carry_5_tmp_290f3_7 = eval!(
        context,
        ((((((((carry_4_tmp_290f3_6)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_5)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_15)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_25)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_35))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_45))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_55))
            - (combination_limb_5_col5))
            * (16)
    );

    let carry_6_tmp_290f3_8 = eval!(
        context,
        ((((((((carry_5_tmp_290f3_7)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_6)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_16)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_26)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_36))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_46))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_56))
            - (combination_limb_6_col6))
            * (16)
    );

    let carry_7_tmp_290f3_9 = eval!(
        context,
        (((((((((carry_6_tmp_290f3_8)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_7)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_17)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_27)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_37))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_47))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_57))
            - (combination_limb_7_col7))
            - ((p_coef_col10) * (136)))
            * (16)
    );

    let carry_8_tmp_290f3_10 = eval!(
        context,
        ((((((((carry_7_tmp_290f3_9)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_8)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_18)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_28)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_38))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_48))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_58))
            - (combination_limb_8_col8))
            * (16)
    );

    // final limb constraint.
    let constraint_9_value = eval!(
        context,
        ((((((((carry_8_tmp_290f3_10)
            + ((4) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_9)))
            + ((2) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_19)))
            + ((3) * (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_29)))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_39))
            - (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_49))
            + (linear_combination_n_6_coefs_4_2_3_1_m1_1_input_limb_59))
            - (combination_limb_9_col9))
            - ((p_coef_col10) * (256))
    );
    acc.add_constraint(context, constraint_9_value);

    // Use RangeCheck_4_4_4_4.
    let tuple_10 = &[
        eval!(context, 1027333874),
        eval!(context, (p_coef_col10) + (2)),
        eval!(context, (carry_0_tmp_290f3_2) + (2)),
        eval!(context, (carry_1_tmp_290f3_3) + (2)),
        eval!(context, (carry_2_tmp_290f3_4) + (2)),
    ];
    let numerator_10 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Use RangeCheck_4_4_4_4.
    let tuple_11 = &[
        eval!(context, 1027333874),
        eval!(context, (carry_3_tmp_290f3_5) + (2)),
        eval!(context, (carry_4_tmp_290f3_6) + (2)),
        eval!(context, (carry_5_tmp_290f3_7) + (2)),
        eval!(context, (carry_6_tmp_290f3_8) + (2)),
    ];
    let numerator_11 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Use RangeCheck_4_4.
    let tuple_12 = &[
        eval!(context, 1651211826),
        eval!(context, (carry_7_tmp_290f3_9) + (2)),
        eval!(context, (carry_8_tmp_290f3_10) + (2)),
    ];
    let numerator_12 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_12, tuple_12);
    vec![]
}
