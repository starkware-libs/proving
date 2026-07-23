// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 3] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
    RelationUse { relation_id: "RangeCheck_4_4_4_4", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        qm_31_read_reduced_input,
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
        value_limb_11_col12,
        value_limb_12_col13,
        value_limb_13_col14,
        value_limb_14_col15,
        value_limb_15_col16,
        delta_ab_inv_col17,
        delta_cd_inv_col18,
    ] = input.try_into().unwrap();

    read_positive_num_bits_144::accumulate_constraints(
        &[
            eval!(context, qm_31_read_reduced_input),
            eval!(context, enabler),
            eval!(context, id_col0),
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
            eval!(context, value_limb_11_col12),
            eval!(context, value_limb_12_col13),
            eval!(context, value_limb_13_col14),
            eval!(context, value_limb_14_col15),
            eval!(context, value_limb_15_col16),
        ],
        context,
        component_data,
        acc,
    );

    // Use RangeCheck_4_4_4_4.
    let tuple_1 = &[
        eval!(context, 1027333874),
        eval!(context, value_limb_3_col4),
        eval!(context, value_limb_7_col8),
        eval!(context, value_limb_11_col12),
        eval!(context, value_limb_15_col16),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);

    // delta_ab doesn't equal 0.
    let constraint_2_value = eval!(
        context,
        (((((((value_limb_0_col1) + (value_limb_1_col2)) + (value_limb_2_col3))
            + (value_limb_3_col4))
            - (1548))
            * (((((value_limb_4_col5) + (value_limb_5_col6)) + (value_limb_6_col7))
                + (value_limb_7_col8))
                - (1548)))
            * (delta_ab_inv_col17))
            - (1)
    );
    acc.add_constraint(context, constraint_2_value);

    // delta_cd doesn't equal 0.
    let constraint_3_value = eval!(
        context,
        (((((((value_limb_8_col9) + (value_limb_9_col10)) + (value_limb_10_col11))
            + (value_limb_11_col12))
            - (1548))
            * (((((value_limb_12_col13) + (value_limb_13_col14)) + (value_limb_14_col15))
                + (value_limb_15_col16))
                - (1548)))
            * (delta_cd_inv_col18))
            - (1)
    );
    acc.add_constraint(context, constraint_3_value);
    vec![
        eval!(
            context,
            (((value_limb_0_col1) + ((value_limb_1_col2) * (512)))
                + ((value_limb_2_col3) * (262144)))
                + ((value_limb_3_col4) * (134217728))
        ),
        eval!(
            context,
            (((value_limb_4_col5) + ((value_limb_5_col6) * (512)))
                + ((value_limb_6_col7) * (262144)))
                + ((value_limb_7_col8) * (134217728))
        ),
        eval!(
            context,
            (((value_limb_8_col9) + ((value_limb_9_col10) * (512)))
                + ((value_limb_10_col11) * (262144)))
                + ((value_limb_11_col12) * (134217728))
        ),
        eval!(
            context,
            (((value_limb_12_col13) + ((value_limb_13_col14) * (512)))
                + ((value_limb_14_col15) * (262144)))
                + ((value_limb_15_col16) * (134217728))
        ),
    ]
}
