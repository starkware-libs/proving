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
        cond_felt_252_as_addr_input_limb_0,
        cond_felt_252_as_addr_input_limb_1,
        cond_felt_252_as_addr_input_limb_2,
        cond_felt_252_as_addr_input_limb_3,
        cond_felt_252_as_addr_input_limb_4,
        cond_felt_252_as_addr_input_limb_5,
        cond_felt_252_as_addr_input_limb_6,
        cond_felt_252_as_addr_input_limb_7,
        cond_felt_252_as_addr_input_limb_8,
        cond_felt_252_as_addr_input_limb_9,
        cond_felt_252_as_addr_input_limb_10,
        cond_felt_252_as_addr_input_limb_11,
        cond_felt_252_as_addr_input_limb_12,
        cond_felt_252_as_addr_input_limb_13,
        cond_felt_252_as_addr_input_limb_14,
        cond_felt_252_as_addr_input_limb_15,
        cond_felt_252_as_addr_input_limb_16,
        cond_felt_252_as_addr_input_limb_17,
        cond_felt_252_as_addr_input_limb_18,
        cond_felt_252_as_addr_input_limb_19,
        cond_felt_252_as_addr_input_limb_20,
        cond_felt_252_as_addr_input_limb_21,
        cond_felt_252_as_addr_input_limb_22,
        cond_felt_252_as_addr_input_limb_23,
        cond_felt_252_as_addr_input_limb_24,
        cond_felt_252_as_addr_input_limb_25,
        cond_felt_252_as_addr_input_limb_26,
        cond_felt_252_as_addr_input_limb_27,
        cond_felt_252_as_addr_input_limb_28,
        enabler,
        partial_limb_msb_col0,
    ] = input.try_into().unwrap();

    // When the condition holds, the high limbs must be zero for an address.
    let constraint_0_value = eval!(
        context,
        (cond_felt_252_as_addr_input_limb_28)
            * ((((((((((((((((((((((((cond_felt_252_as_addr_input_limb_4)
                + (cond_felt_252_as_addr_input_limb_5))
                + (cond_felt_252_as_addr_input_limb_6))
                + (cond_felt_252_as_addr_input_limb_7))
                + (cond_felt_252_as_addr_input_limb_8))
                + (cond_felt_252_as_addr_input_limb_9))
                + (cond_felt_252_as_addr_input_limb_10))
                + (cond_felt_252_as_addr_input_limb_11))
                + (cond_felt_252_as_addr_input_limb_12))
                + (cond_felt_252_as_addr_input_limb_13))
                + (cond_felt_252_as_addr_input_limb_14))
                + (cond_felt_252_as_addr_input_limb_15))
                + (cond_felt_252_as_addr_input_limb_16))
                + (cond_felt_252_as_addr_input_limb_17))
                + (cond_felt_252_as_addr_input_limb_18))
                + (cond_felt_252_as_addr_input_limb_19))
                + (cond_felt_252_as_addr_input_limb_20))
                + (cond_felt_252_as_addr_input_limb_21))
                + (cond_felt_252_as_addr_input_limb_22))
                + (cond_felt_252_as_addr_input_limb_23))
                + (cond_felt_252_as_addr_input_limb_24))
                + (cond_felt_252_as_addr_input_limb_25))
                + (cond_felt_252_as_addr_input_limb_26))
                + (cond_felt_252_as_addr_input_limb_27))
    );
    acc.add_constraint(context, constraint_0_value);

    cond_range_check_2::accumulate_constraints(
        &[
            eval!(context, cond_felt_252_as_addr_input_limb_3),
            eval!(context, cond_felt_252_as_addr_input_limb_28),
            eval!(context, enabler),
            eval!(context, partial_limb_msb_col0),
        ],
        context,
        component_data,
        acc,
    );
    vec![eval!(
        context,
        (((cond_felt_252_as_addr_input_limb_0) + ((cond_felt_252_as_addr_input_limb_1) * (512)))
            + ((cond_felt_252_as_addr_input_limb_2) * (262144)))
            + ((cond_felt_252_as_addr_input_limb_3) * (134217728))
    )]
}
