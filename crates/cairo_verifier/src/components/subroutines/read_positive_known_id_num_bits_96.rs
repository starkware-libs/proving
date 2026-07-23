// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
    RelationUse { relation_id: "RangeCheck_6", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        read_positive_known_id_num_bits_96_input,
        enabler,
        value_limb_0_col0,
        value_limb_1_col1,
        value_limb_2_col2,
        value_limb_3_col3,
        value_limb_4_col4,
        value_limb_5_col5,
        value_limb_6_col6,
        value_limb_7_col7,
        value_limb_8_col8,
        value_limb_9_col9,
        value_limb_10_col10,
    ] = input.try_into().unwrap();

    range_check_last_limb_bits_in_ms_limb_6::accumulate_constraints(
        &[eval!(context, value_limb_10_col10), eval!(context, enabler)],
        context,
        component_data,
        acc,
    );

    // Use MemoryIdToBig.
    let tuple_1 = &[
        eval!(context, 1662111297),
        eval!(context, read_positive_known_id_num_bits_96_input),
        eval!(context, value_limb_0_col0),
        eval!(context, value_limb_1_col1),
        eval!(context, value_limb_2_col2),
        eval!(context, value_limb_3_col3),
        eval!(context, value_limb_4_col4),
        eval!(context, value_limb_5_col5),
        eval!(context, value_limb_6_col6),
        eval!(context, value_limb_7_col7),
        eval!(context, value_limb_8_col8),
        eval!(context, value_limb_9_col9),
        eval!(context, value_limb_10_col10),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![]
}
