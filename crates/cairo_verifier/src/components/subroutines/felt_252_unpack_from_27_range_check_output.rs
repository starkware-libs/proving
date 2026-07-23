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
        felt_252_unpack_from_27_range_check_output_input_limb_0,
        felt_252_unpack_from_27_range_check_output_input_limb_1,
        felt_252_unpack_from_27_range_check_output_input_limb_2,
        felt_252_unpack_from_27_range_check_output_input_limb_3,
        felt_252_unpack_from_27_range_check_output_input_limb_4,
        felt_252_unpack_from_27_range_check_output_input_limb_5,
        felt_252_unpack_from_27_range_check_output_input_limb_6,
        felt_252_unpack_from_27_range_check_output_input_limb_7,
        felt_252_unpack_from_27_range_check_output_input_limb_8,
        felt_252_unpack_from_27_range_check_output_input_limb_9,
        enabler,
        unpacked_limb_0_col0,
        unpacked_limb_1_col1,
        unpacked_limb_3_col2,
        unpacked_limb_4_col3,
        unpacked_limb_6_col4,
        unpacked_limb_7_col5,
        unpacked_limb_9_col6,
        unpacked_limb_10_col7,
        unpacked_limb_12_col8,
        unpacked_limb_13_col9,
        unpacked_limb_15_col10,
        unpacked_limb_16_col11,
        unpacked_limb_18_col12,
        unpacked_limb_19_col13,
        unpacked_limb_21_col14,
        unpacked_limb_22_col15,
        unpacked_limb_24_col16,
        unpacked_limb_25_col17,
    ] = input.try_into().unwrap();

    let unpacked_tmp_865a8_1_limb_2 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_0) - (unpacked_limb_0_col0))
            - ((unpacked_limb_1_col1) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_5 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_1) - (unpacked_limb_3_col2))
            - ((unpacked_limb_4_col3) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_8 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_2) - (unpacked_limb_6_col4))
            - ((unpacked_limb_7_col5) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_11 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_3) - (unpacked_limb_9_col6))
            - ((unpacked_limb_10_col7) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_14 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_4) - (unpacked_limb_12_col8))
            - ((unpacked_limb_13_col9) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_17 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_5) - (unpacked_limb_15_col10))
            - ((unpacked_limb_16_col11) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_20 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_6) - (unpacked_limb_18_col12))
            - ((unpacked_limb_19_col13) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_23 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_7) - (unpacked_limb_21_col14))
            - ((unpacked_limb_22_col15) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_26 = eval!(
        context,
        (((felt_252_unpack_from_27_range_check_output_input_limb_8) - (unpacked_limb_24_col16))
            - ((unpacked_limb_25_col17) * (512)))
            * (8192)
    );

    let unpacked_tmp_865a8_1_limb_27 =
        eval!(context, felt_252_unpack_from_27_range_check_output_input_limb_9);

    range_check_mem_value_n_28::accumulate_constraints(
        &[
            eval!(context, unpacked_limb_0_col0),
            eval!(context, unpacked_limb_1_col1),
            eval!(context, unpacked_tmp_865a8_1_limb_2),
            eval!(context, unpacked_limb_3_col2),
            eval!(context, unpacked_limb_4_col3),
            eval!(context, unpacked_tmp_865a8_1_limb_5),
            eval!(context, unpacked_limb_6_col4),
            eval!(context, unpacked_limb_7_col5),
            eval!(context, unpacked_tmp_865a8_1_limb_8),
            eval!(context, unpacked_limb_9_col6),
            eval!(context, unpacked_limb_10_col7),
            eval!(context, unpacked_tmp_865a8_1_limb_11),
            eval!(context, unpacked_limb_12_col8),
            eval!(context, unpacked_limb_13_col9),
            eval!(context, unpacked_tmp_865a8_1_limb_14),
            eval!(context, unpacked_limb_15_col10),
            eval!(context, unpacked_limb_16_col11),
            eval!(context, unpacked_tmp_865a8_1_limb_17),
            eval!(context, unpacked_limb_18_col12),
            eval!(context, unpacked_limb_19_col13),
            eval!(context, unpacked_tmp_865a8_1_limb_20),
            eval!(context, unpacked_limb_21_col14),
            eval!(context, unpacked_limb_22_col15),
            eval!(context, unpacked_tmp_865a8_1_limb_23),
            eval!(context, unpacked_limb_24_col16),
            eval!(context, unpacked_limb_25_col17),
            eval!(context, unpacked_tmp_865a8_1_limb_26),
            eval!(context, unpacked_tmp_865a8_1_limb_27),
            eval!(context, enabler),
        ],
        context,
        component_data,
        acc,
    );
    vec![
        eval!(context, unpacked_tmp_865a8_1_limb_2),
        eval!(context, unpacked_tmp_865a8_1_limb_5),
        eval!(context, unpacked_tmp_865a8_1_limb_8),
        eval!(context, unpacked_tmp_865a8_1_limb_11),
        eval!(context, unpacked_tmp_865a8_1_limb_14),
        eval!(context, unpacked_tmp_865a8_1_limb_17),
        eval!(context, unpacked_tmp_865a8_1_limb_20),
        eval!(context, unpacked_tmp_865a8_1_limb_23),
        eval!(context, unpacked_tmp_865a8_1_limb_26),
        eval!(context, unpacked_tmp_865a8_1_limb_27),
    ]
}
