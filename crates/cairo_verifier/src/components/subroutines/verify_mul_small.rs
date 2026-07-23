// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "RangeCheck_11", uses: 3 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        verify_mul_small_input_a_limb_0,
        verify_mul_small_input_a_limb_1,
        verify_mul_small_input_a_limb_2,
        verify_mul_small_input_a_limb_3,
        verify_mul_small_input_b_limb_0,
        verify_mul_small_input_b_limb_1,
        verify_mul_small_input_b_limb_2,
        verify_mul_small_input_b_limb_3,
        verify_mul_small_input_c_limb_0,
        verify_mul_small_input_c_limb_1,
        verify_mul_small_input_c_limb_2,
        verify_mul_small_input_c_limb_3,
        verify_mul_small_input_c_limb_4,
        verify_mul_small_input_c_limb_5,
        verify_mul_small_input_c_limb_6,
        verify_mul_small_input_c_limb_7,
        enabler,
        carry_1_col0,
        carry_3_col1,
        carry_5_col2,
    ] = input.try_into().unwrap();

    // Use RangeCheck_11.
    let tuple_0 = &[eval!(context, 991608089), eval!(context, carry_1_col0)];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // carry 1 definition.
    let constraint_1_value = eval!(
        context,
        ((carry_1_col0) * (262144))
            - ((((verify_mul_small_input_a_limb_0) * (verify_mul_small_input_b_limb_0))
                - (verify_mul_small_input_c_limb_0))
                + (((((verify_mul_small_input_a_limb_0) * (verify_mul_small_input_b_limb_1))
                    + ((verify_mul_small_input_a_limb_1) * (verify_mul_small_input_b_limb_0)))
                    - (verify_mul_small_input_c_limb_1))
                    * (512)))
    );
    acc.add_constraint(context, constraint_1_value);

    // Use RangeCheck_11.
    let tuple_2 = &[eval!(context, 991608089), eval!(context, carry_3_col1)];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // carry 3 definition.
    let constraint_3_value = eval!(
        context,
        ((carry_3_col1) * (262144))
            - (((carry_1_col0)
                + (((((verify_mul_small_input_a_limb_0) * (verify_mul_small_input_b_limb_2))
                    + ((verify_mul_small_input_a_limb_1) * (verify_mul_small_input_b_limb_1)))
                    + ((verify_mul_small_input_a_limb_2) * (verify_mul_small_input_b_limb_0)))
                    - (verify_mul_small_input_c_limb_2)))
                + (((((((verify_mul_small_input_a_limb_0)
                    * (verify_mul_small_input_b_limb_3))
                    + ((verify_mul_small_input_a_limb_1) * (verify_mul_small_input_b_limb_2)))
                    + ((verify_mul_small_input_a_limb_2) * (verify_mul_small_input_b_limb_1)))
                    + ((verify_mul_small_input_a_limb_3) * (verify_mul_small_input_b_limb_0)))
                    - (verify_mul_small_input_c_limb_3))
                    * (512)))
    );
    acc.add_constraint(context, constraint_3_value);

    // Use RangeCheck_11.
    let tuple_4 = &[eval!(context, 991608089), eval!(context, carry_5_col2)];
    let numerator_4 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // carry 5 definition.
    let constraint_5_value = eval!(
        context,
        ((carry_5_col2) * (262144))
            - (((carry_3_col1)
                + (((((verify_mul_small_input_a_limb_1) * (verify_mul_small_input_b_limb_3))
                    + ((verify_mul_small_input_a_limb_2) * (verify_mul_small_input_b_limb_2)))
                    + ((verify_mul_small_input_a_limb_3) * (verify_mul_small_input_b_limb_1)))
                    - (verify_mul_small_input_c_limb_4)))
                + (((((verify_mul_small_input_a_limb_2) * (verify_mul_small_input_b_limb_3))
                    + ((verify_mul_small_input_a_limb_3) * (verify_mul_small_input_b_limb_2)))
                    - (verify_mul_small_input_c_limb_5))
                    * (512)))
    );
    acc.add_constraint(context, constraint_5_value);

    // final limb constraint.
    let constraint_6_value = eval!(
        context,
        (((carry_5_col2)
            + ((verify_mul_small_input_a_limb_3) * (verify_mul_small_input_b_limb_3)))
            - ((verify_mul_small_input_c_limb_7) * (512)))
            - (verify_mul_small_input_c_limb_6)
    );
    acc.add_constraint(context, constraint_6_value);
    vec![]
}
