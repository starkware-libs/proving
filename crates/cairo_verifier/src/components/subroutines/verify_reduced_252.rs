// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "RangeCheck_8", uses: 2 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        verify_reduced_252_input_limb_0,
        verify_reduced_252_input_limb_1,
        verify_reduced_252_input_limb_2,
        verify_reduced_252_input_limb_3,
        verify_reduced_252_input_limb_4,
        verify_reduced_252_input_limb_5,
        verify_reduced_252_input_limb_6,
        verify_reduced_252_input_limb_7,
        verify_reduced_252_input_limb_8,
        verify_reduced_252_input_limb_9,
        verify_reduced_252_input_limb_10,
        verify_reduced_252_input_limb_11,
        verify_reduced_252_input_limb_12,
        verify_reduced_252_input_limb_13,
        verify_reduced_252_input_limb_14,
        verify_reduced_252_input_limb_15,
        verify_reduced_252_input_limb_16,
        verify_reduced_252_input_limb_17,
        verify_reduced_252_input_limb_18,
        verify_reduced_252_input_limb_19,
        verify_reduced_252_input_limb_20,
        verify_reduced_252_input_limb_21,
        verify_reduced_252_input_limb_22,
        verify_reduced_252_input_limb_23,
        verify_reduced_252_input_limb_24,
        verify_reduced_252_input_limb_25,
        verify_reduced_252_input_limb_26,
        verify_reduced_252_input_limb_27,
        enabler,
        ms_limb_is_max_col0,
        ms_and_mid_limbs_are_max_col1,
        rc_input_col2,
    ] = input.try_into().unwrap();

    // ms_max is bit.
    let constraint_0_value = eval!(context, (ms_limb_is_max_col0) * ((1) - (ms_limb_is_max_col0)));
    acc.add_constraint(context, constraint_0_value);

    // both_max is bit.
    let constraint_1_value =
        eval!(context, (ms_and_mid_limbs_are_max_col1) * ((1) - (ms_and_mid_limbs_are_max_col1)));
    acc.add_constraint(context, constraint_1_value);

    // Use RangeCheck_8.
    let tuple_2 = &[
        eval!(context, 1420243005),
        eval!(context, (verify_reduced_252_input_limb_27) - (ms_limb_is_max_col0)),
    ];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // If the MS limb is max, high limbs should be 0.
    let constraint_3_value = eval!(
        context,
        (ms_limb_is_max_col0)
            * (((((verify_reduced_252_input_limb_22) + (verify_reduced_252_input_limb_23))
                + (verify_reduced_252_input_limb_24))
                + (verify_reduced_252_input_limb_25))
                + (verify_reduced_252_input_limb_26))
    );
    acc.add_constraint(context, constraint_3_value);

    // rc_input.
    let constraint_4_value = eval!(
        context,
        (rc_input_col2)
            - ((ms_limb_is_max_col0)
                * (((120) + (verify_reduced_252_input_limb_21)) - (ms_and_mid_limbs_are_max_col1)))
    );
    acc.add_constraint(context, constraint_4_value);

    // Use RangeCheck_8.
    let tuple_5 = &[eval!(context, 1420243005), eval!(context, rc_input_col2)];
    let numerator_5 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // If the MS and mid limbs are max, low limbs should be 0.
    let constraint_6_value = eval!(
        context,
        (ms_and_mid_limbs_are_max_col1)
            * (((((((((((((((((((((verify_reduced_252_input_limb_0)
                + (verify_reduced_252_input_limb_1))
                + (verify_reduced_252_input_limb_2))
                + (verify_reduced_252_input_limb_3))
                + (verify_reduced_252_input_limb_4))
                + (verify_reduced_252_input_limb_5))
                + (verify_reduced_252_input_limb_6))
                + (verify_reduced_252_input_limb_7))
                + (verify_reduced_252_input_limb_8))
                + (verify_reduced_252_input_limb_9))
                + (verify_reduced_252_input_limb_10))
                + (verify_reduced_252_input_limb_11))
                + (verify_reduced_252_input_limb_12))
                + (verify_reduced_252_input_limb_13))
                + (verify_reduced_252_input_limb_14))
                + (verify_reduced_252_input_limb_15))
                + (verify_reduced_252_input_limb_16))
                + (verify_reduced_252_input_limb_17))
                + (verify_reduced_252_input_limb_18))
                + (verify_reduced_252_input_limb_19))
                + (verify_reduced_252_input_limb_20))
    );
    acc.add_constraint(context, constraint_6_value);
    vec![]
}
