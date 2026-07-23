// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 8] = [
    RelationUse { relation_id: "RangeCheck_9_9", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_B", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_C", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_D", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_E", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_F", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_G", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_H", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        range_check_mem_value_n_28_input_limb_0,
        range_check_mem_value_n_28_input_limb_1,
        range_check_mem_value_n_28_input_limb_2,
        range_check_mem_value_n_28_input_limb_3,
        range_check_mem_value_n_28_input_limb_4,
        range_check_mem_value_n_28_input_limb_5,
        range_check_mem_value_n_28_input_limb_6,
        range_check_mem_value_n_28_input_limb_7,
        range_check_mem_value_n_28_input_limb_8,
        range_check_mem_value_n_28_input_limb_9,
        range_check_mem_value_n_28_input_limb_10,
        range_check_mem_value_n_28_input_limb_11,
        range_check_mem_value_n_28_input_limb_12,
        range_check_mem_value_n_28_input_limb_13,
        range_check_mem_value_n_28_input_limb_14,
        range_check_mem_value_n_28_input_limb_15,
        range_check_mem_value_n_28_input_limb_16,
        range_check_mem_value_n_28_input_limb_17,
        range_check_mem_value_n_28_input_limb_18,
        range_check_mem_value_n_28_input_limb_19,
        range_check_mem_value_n_28_input_limb_20,
        range_check_mem_value_n_28_input_limb_21,
        range_check_mem_value_n_28_input_limb_22,
        range_check_mem_value_n_28_input_limb_23,
        range_check_mem_value_n_28_input_limb_24,
        range_check_mem_value_n_28_input_limb_25,
        range_check_mem_value_n_28_input_limb_26,
        range_check_mem_value_n_28_input_limb_27,
        enabler,
    ] = input.try_into().unwrap();

    // Use RangeCheck_9_9.
    let tuple_0 = &[
        eval!(context, 517791011),
        eval!(context, range_check_mem_value_n_28_input_limb_0),
        eval!(context, range_check_mem_value_n_28_input_limb_1),
    ];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Use RangeCheck_9_9_B.
    let tuple_1 = &[
        eval!(context, 1897792095),
        eval!(context, range_check_mem_value_n_28_input_limb_2),
        eval!(context, range_check_mem_value_n_28_input_limb_3),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Use RangeCheck_9_9_C.
    let tuple_2 = &[
        eval!(context, 1881014476),
        eval!(context, range_check_mem_value_n_28_input_limb_4),
        eval!(context, range_check_mem_value_n_28_input_limb_5),
    ];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Use RangeCheck_9_9_D.
    let tuple_3 = &[
        eval!(context, 1864236857),
        eval!(context, range_check_mem_value_n_28_input_limb_6),
        eval!(context, range_check_mem_value_n_28_input_limb_7),
    ];
    let numerator_3 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Use RangeCheck_9_9_E.
    let tuple_4 = &[
        eval!(context, 1847459238),
        eval!(context, range_check_mem_value_n_28_input_limb_8),
        eval!(context, range_check_mem_value_n_28_input_limb_9),
    ];
    let numerator_4 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Use RangeCheck_9_9_F.
    let tuple_5 = &[
        eval!(context, 1830681619),
        eval!(context, range_check_mem_value_n_28_input_limb_10),
        eval!(context, range_check_mem_value_n_28_input_limb_11),
    ];
    let numerator_5 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Use RangeCheck_9_9_G.
    let tuple_6 = &[
        eval!(context, 1813904000),
        eval!(context, range_check_mem_value_n_28_input_limb_12),
        eval!(context, range_check_mem_value_n_28_input_limb_13),
    ];
    let numerator_6 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Use RangeCheck_9_9_H.
    let tuple_7 = &[
        eval!(context, 2065568285),
        eval!(context, range_check_mem_value_n_28_input_limb_14),
        eval!(context, range_check_mem_value_n_28_input_limb_15),
    ];
    let numerator_7 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Use RangeCheck_9_9.
    let tuple_8 = &[
        eval!(context, 517791011),
        eval!(context, range_check_mem_value_n_28_input_limb_16),
        eval!(context, range_check_mem_value_n_28_input_limb_17),
    ];
    let numerator_8 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_8, tuple_8);

    // Use RangeCheck_9_9_B.
    let tuple_9 = &[
        eval!(context, 1897792095),
        eval!(context, range_check_mem_value_n_28_input_limb_18),
        eval!(context, range_check_mem_value_n_28_input_limb_19),
    ];
    let numerator_9 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_9, tuple_9);

    // Use RangeCheck_9_9_C.
    let tuple_10 = &[
        eval!(context, 1881014476),
        eval!(context, range_check_mem_value_n_28_input_limb_20),
        eval!(context, range_check_mem_value_n_28_input_limb_21),
    ];
    let numerator_10 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Use RangeCheck_9_9_D.
    let tuple_11 = &[
        eval!(context, 1864236857),
        eval!(context, range_check_mem_value_n_28_input_limb_22),
        eval!(context, range_check_mem_value_n_28_input_limb_23),
    ];
    let numerator_11 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Use RangeCheck_9_9_E.
    let tuple_12 = &[
        eval!(context, 1847459238),
        eval!(context, range_check_mem_value_n_28_input_limb_24),
        eval!(context, range_check_mem_value_n_28_input_limb_25),
    ];
    let numerator_12 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_12, tuple_12);

    // Use RangeCheck_9_9_F.
    let tuple_13 = &[
        eval!(context, 1830681619),
        eval!(context, range_check_mem_value_n_28_input_limb_26),
        eval!(context, range_check_mem_value_n_28_input_limb_27),
    ];
    let numerator_13 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_13, tuple_13);
    vec![]
}
