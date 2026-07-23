// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        read_positive_num_bits_99_input,
        enabler,
        id_col0,
        value_limb_0_col1,
        value_limb_1_col2,
        value_limb_2_col3,
        value_limb_3_col4,
        value_limb_4_col5,
        value_limb_5_col6,
        value_limb_6_col7,
        value_limb_7_col8,
        value_limb_8_col9,
        value_limb_9_col10,
        value_limb_10_col11,
    ] = input.try_into().unwrap();

    read_id::accumulate_constraints(
        &[
            eval!(context, read_positive_num_bits_99_input),
            eval!(context, enabler),
            eval!(context, id_col0),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_known_id_num_bits_99::accumulate_constraints(
        &[
            eval!(context, id_col0),
            eval!(context, enabler),
            eval!(context, value_limb_0_col1),
            eval!(context, value_limb_1_col2),
            eval!(context, value_limb_2_col3),
            eval!(context, value_limb_3_col4),
            eval!(context, value_limb_4_col5),
            eval!(context, value_limb_5_col6),
            eval!(context, value_limb_6_col7),
            eval!(context, value_limb_7_col8),
            eval!(context, value_limb_8_col9),
            eval!(context, value_limb_9_col10),
            eval!(context, value_limb_10_col11),
        ],
        context,
        component_data,
        acc,
    );
    vec![]
}
