// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "RangeCheck_9_9", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_B", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_C", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_D", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        range_check_mem_value_n_8_input_limb_0,
        range_check_mem_value_n_8_input_limb_1,
        range_check_mem_value_n_8_input_limb_2,
        range_check_mem_value_n_8_input_limb_3,
        range_check_mem_value_n_8_input_limb_4,
        range_check_mem_value_n_8_input_limb_5,
        range_check_mem_value_n_8_input_limb_6,
        range_check_mem_value_n_8_input_limb_7,
        enabler,
    ] = input.try_into().unwrap();

    // Use RangeCheck_9_9.
    let tuple_0 = &[
        eval!(context, 517791011),
        eval!(context, range_check_mem_value_n_8_input_limb_0),
        eval!(context, range_check_mem_value_n_8_input_limb_1),
    ];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Use RangeCheck_9_9_B.
    let tuple_1 = &[
        eval!(context, 1897792095),
        eval!(context, range_check_mem_value_n_8_input_limb_2),
        eval!(context, range_check_mem_value_n_8_input_limb_3),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Use RangeCheck_9_9_C.
    let tuple_2 = &[
        eval!(context, 1881014476),
        eval!(context, range_check_mem_value_n_8_input_limb_4),
        eval!(context, range_check_mem_value_n_8_input_limb_5),
    ];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Use RangeCheck_9_9_D.
    let tuple_3 = &[
        eval!(context, 1864236857),
        eval!(context, range_check_mem_value_n_8_input_limb_6),
        eval!(context, range_check_mem_value_n_8_input_limb_7),
    ];
    let numerator_3 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_3, tuple_3);
    vec![]
}
