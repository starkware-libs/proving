// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "MemoryIdToBig", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        read_positive_known_id_num_bits_29_input,
        enabler,
        value_limb_0_col0,
        value_limb_1_col1,
        value_limb_2_col2,
        value_limb_3_col3,
        partial_limb_msb_col4,
    ] = input.try_into().unwrap();

    range_check_last_limb_bits_in_ms_limb_2::accumulate_constraints(
        &[
            eval!(context, value_limb_3_col3),
            eval!(context, enabler),
            eval!(context, partial_limb_msb_col4),
        ],
        context,
        component_data,
        acc,
    );

    // Use MemoryIdToBig.
    let tuple_1 = &[
        eval!(context, 1662111297),
        eval!(context, read_positive_known_id_num_bits_29_input),
        eval!(context, value_limb_0_col0),
        eval!(context, value_limb_1_col1),
        eval!(context, value_limb_2_col2),
        eval!(context, value_limb_3_col3),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![]
}
