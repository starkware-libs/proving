// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 89;
pub const N_INTERACTION_COLUMNS: usize = 76;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 5 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 5 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 1 },
    RelationUse { relation_id: "VerifyBitwiseXor_9", uses: 27 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        op0_id_col0,
        op0_limb_0_col1,
        op0_limb_1_col2,
        op0_limb_2_col3,
        op0_limb_3_col4,
        op0_limb_4_col5,
        op0_limb_5_col6,
        op0_limb_6_col7,
        op0_limb_7_col8,
        op0_limb_8_col9,
        op0_limb_9_col10,
        op0_limb_10_col11,
        op0_limb_11_col12,
        op0_limb_12_col13,
        op0_limb_13_col14,
        op0_limb_14_col15,
        op0_limb_15_col16,
        op0_limb_16_col17,
        op0_limb_17_col18,
        op0_limb_18_col19,
        op0_limb_19_col20,
        op0_limb_20_col21,
        op0_limb_21_col22,
        op0_limb_22_col23,
        op0_limb_23_col24,
        op0_limb_24_col25,
        op0_limb_25_col26,
        op0_limb_26_col27,
        op0_limb_27_col28,
        op1_id_col29,
        op1_limb_0_col30,
        op1_limb_1_col31,
        op1_limb_2_col32,
        op1_limb_3_col33,
        op1_limb_4_col34,
        op1_limb_5_col35,
        op1_limb_6_col36,
        op1_limb_7_col37,
        op1_limb_8_col38,
        op1_limb_9_col39,
        op1_limb_10_col40,
        op1_limb_11_col41,
        op1_limb_12_col42,
        op1_limb_13_col43,
        op1_limb_14_col44,
        op1_limb_15_col45,
        op1_limb_16_col46,
        op1_limb_17_col47,
        op1_limb_18_col48,
        op1_limb_19_col49,
        op1_limb_20_col50,
        op1_limb_21_col51,
        op1_limb_22_col52,
        op1_limb_23_col53,
        op1_limb_24_col54,
        op1_limb_25_col55,
        op1_limb_26_col56,
        op1_limb_27_col57,
        xor_col58,
        xor_col59,
        xor_col60,
        xor_col61,
        xor_col62,
        xor_col63,
        xor_col64,
        xor_col65,
        xor_col66,
        xor_col67,
        xor_col68,
        xor_col69,
        xor_col70,
        xor_col71,
        xor_col72,
        xor_col73,
        xor_col74,
        xor_col75,
        xor_col76,
        xor_col77,
        xor_col78,
        xor_col79,
        xor_col80,
        xor_col81,
        xor_col82,
        xor_col83,
        xor_col84,
        xor_col85,
        and_id_col86,
        xor_id_col87,
        or_id_col88,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, &acc.preprocessed_columns);
    let bitwise_builtin_segment_start =
        *acc.public_params.get("bitwise_builtin_segment_start").unwrap();

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(context, (bitwise_builtin_segment_start) + ((seq) * (5))),
            eval!(context, 1),
            eval!(context, op0_id_col0),
            eval!(context, op0_limb_0_col1),
            eval!(context, op0_limb_1_col2),
            eval!(context, op0_limb_2_col3),
            eval!(context, op0_limb_3_col4),
            eval!(context, op0_limb_4_col5),
            eval!(context, op0_limb_5_col6),
            eval!(context, op0_limb_6_col7),
            eval!(context, op0_limb_7_col8),
            eval!(context, op0_limb_8_col9),
            eval!(context, op0_limb_9_col10),
            eval!(context, op0_limb_10_col11),
            eval!(context, op0_limb_11_col12),
            eval!(context, op0_limb_12_col13),
            eval!(context, op0_limb_13_col14),
            eval!(context, op0_limb_14_col15),
            eval!(context, op0_limb_15_col16),
            eval!(context, op0_limb_16_col17),
            eval!(context, op0_limb_17_col18),
            eval!(context, op0_limb_18_col19),
            eval!(context, op0_limb_19_col20),
            eval!(context, op0_limb_20_col21),
            eval!(context, op0_limb_21_col22),
            eval!(context, op0_limb_22_col23),
            eval!(context, op0_limb_23_col24),
            eval!(context, op0_limb_24_col25),
            eval!(context, op0_limb_25_col26),
            eval!(context, op0_limb_26_col27),
            eval!(context, op0_limb_27_col28),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(context, ((bitwise_builtin_segment_start) + ((seq) * (5))) + (1)),
            eval!(context, 1),
            eval!(context, op1_id_col29),
            eval!(context, op1_limb_0_col30),
            eval!(context, op1_limb_1_col31),
            eval!(context, op1_limb_2_col32),
            eval!(context, op1_limb_3_col33),
            eval!(context, op1_limb_4_col34),
            eval!(context, op1_limb_5_col35),
            eval!(context, op1_limb_6_col36),
            eval!(context, op1_limb_7_col37),
            eval!(context, op1_limb_8_col38),
            eval!(context, op1_limb_9_col39),
            eval!(context, op1_limb_10_col40),
            eval!(context, op1_limb_11_col41),
            eval!(context, op1_limb_12_col42),
            eval!(context, op1_limb_13_col43),
            eval!(context, op1_limb_14_col44),
            eval!(context, op1_limb_15_col45),
            eval!(context, op1_limb_16_col46),
            eval!(context, op1_limb_17_col47),
            eval!(context, op1_limb_18_col48),
            eval!(context, op1_limb_19_col49),
            eval!(context, op1_limb_20_col50),
            eval!(context, op1_limb_21_col51),
            eval!(context, op1_limb_22_col52),
            eval!(context, op1_limb_23_col53),
            eval!(context, op1_limb_24_col54),
            eval!(context, op1_limb_25_col55),
            eval!(context, op1_limb_26_col56),
            eval!(context, op1_limb_27_col57),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_0_col1),
            eval!(context, op1_limb_0_col30),
            eval!(context, 1),
            eval!(context, xor_col58),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_12 =
        eval!(context, (1073741824) * (((op0_limb_0_col1) + (op1_limb_0_col30)) - (xor_col58)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_1_col2),
            eval!(context, op1_limb_1_col31),
            eval!(context, 1),
            eval!(context, xor_col59),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_15 =
        eval!(context, (1073741824) * (((op0_limb_1_col2) + (op1_limb_1_col31)) - (xor_col59)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_2_col3),
            eval!(context, op1_limb_2_col32),
            eval!(context, 1),
            eval!(context, xor_col60),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_18 =
        eval!(context, (1073741824) * (((op0_limb_2_col3) + (op1_limb_2_col32)) - (xor_col60)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_3_col4),
            eval!(context, op1_limb_3_col33),
            eval!(context, 1),
            eval!(context, xor_col61),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_21 =
        eval!(context, (1073741824) * (((op0_limb_3_col4) + (op1_limb_3_col33)) - (xor_col61)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_4_col5),
            eval!(context, op1_limb_4_col34),
            eval!(context, 1),
            eval!(context, xor_col62),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_24 =
        eval!(context, (1073741824) * (((op0_limb_4_col5) + (op1_limb_4_col34)) - (xor_col62)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_5_col6),
            eval!(context, op1_limb_5_col35),
            eval!(context, 1),
            eval!(context, xor_col63),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_27 =
        eval!(context, (1073741824) * (((op0_limb_5_col6) + (op1_limb_5_col35)) - (xor_col63)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_6_col7),
            eval!(context, op1_limb_6_col36),
            eval!(context, 1),
            eval!(context, xor_col64),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_30 =
        eval!(context, (1073741824) * (((op0_limb_6_col7) + (op1_limb_6_col36)) - (xor_col64)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_7_col8),
            eval!(context, op1_limb_7_col37),
            eval!(context, 1),
            eval!(context, xor_col65),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_33 =
        eval!(context, (1073741824) * (((op0_limb_7_col8) + (op1_limb_7_col37)) - (xor_col65)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_8_col9),
            eval!(context, op1_limb_8_col38),
            eval!(context, 1),
            eval!(context, xor_col66),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_36 =
        eval!(context, (1073741824) * (((op0_limb_8_col9) + (op1_limb_8_col38)) - (xor_col66)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_9_col10),
            eval!(context, op1_limb_9_col39),
            eval!(context, 1),
            eval!(context, xor_col67),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_39 =
        eval!(context, (1073741824) * (((op0_limb_9_col10) + (op1_limb_9_col39)) - (xor_col67)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_10_col11),
            eval!(context, op1_limb_10_col40),
            eval!(context, 1),
            eval!(context, xor_col68),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_42 =
        eval!(context, (1073741824) * (((op0_limb_10_col11) + (op1_limb_10_col40)) - (xor_col68)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_11_col12),
            eval!(context, op1_limb_11_col41),
            eval!(context, 1),
            eval!(context, xor_col69),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_45 =
        eval!(context, (1073741824) * (((op0_limb_11_col12) + (op1_limb_11_col41)) - (xor_col69)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_12_col13),
            eval!(context, op1_limb_12_col42),
            eval!(context, 1),
            eval!(context, xor_col70),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_48 =
        eval!(context, (1073741824) * (((op0_limb_12_col13) + (op1_limb_12_col42)) - (xor_col70)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_13_col14),
            eval!(context, op1_limb_13_col43),
            eval!(context, 1),
            eval!(context, xor_col71),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_51 =
        eval!(context, (1073741824) * (((op0_limb_13_col14) + (op1_limb_13_col43)) - (xor_col71)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_14_col15),
            eval!(context, op1_limb_14_col44),
            eval!(context, 1),
            eval!(context, xor_col72),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_54 =
        eval!(context, (1073741824) * (((op0_limb_14_col15) + (op1_limb_14_col44)) - (xor_col72)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_15_col16),
            eval!(context, op1_limb_15_col45),
            eval!(context, 1),
            eval!(context, xor_col73),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_57 =
        eval!(context, (1073741824) * (((op0_limb_15_col16) + (op1_limb_15_col45)) - (xor_col73)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_16_col17),
            eval!(context, op1_limb_16_col46),
            eval!(context, 1),
            eval!(context, xor_col74),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_60 =
        eval!(context, (1073741824) * (((op0_limb_16_col17) + (op1_limb_16_col46)) - (xor_col74)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_17_col18),
            eval!(context, op1_limb_17_col47),
            eval!(context, 1),
            eval!(context, xor_col75),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_63 =
        eval!(context, (1073741824) * (((op0_limb_17_col18) + (op1_limb_17_col47)) - (xor_col75)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_18_col19),
            eval!(context, op1_limb_18_col48),
            eval!(context, 1),
            eval!(context, xor_col76),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_66 =
        eval!(context, (1073741824) * (((op0_limb_18_col19) + (op1_limb_18_col48)) - (xor_col76)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_19_col20),
            eval!(context, op1_limb_19_col49),
            eval!(context, 1),
            eval!(context, xor_col77),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_69 =
        eval!(context, (1073741824) * (((op0_limb_19_col20) + (op1_limb_19_col49)) - (xor_col77)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_20_col21),
            eval!(context, op1_limb_20_col50),
            eval!(context, 1),
            eval!(context, xor_col78),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_72 =
        eval!(context, (1073741824) * (((op0_limb_20_col21) + (op1_limb_20_col50)) - (xor_col78)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_21_col22),
            eval!(context, op1_limb_21_col51),
            eval!(context, 1),
            eval!(context, xor_col79),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_75 =
        eval!(context, (1073741824) * (((op0_limb_21_col22) + (op1_limb_21_col51)) - (xor_col79)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_22_col23),
            eval!(context, op1_limb_22_col52),
            eval!(context, 1),
            eval!(context, xor_col80),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_78 =
        eval!(context, (1073741824) * (((op0_limb_22_col23) + (op1_limb_22_col52)) - (xor_col80)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_23_col24),
            eval!(context, op1_limb_23_col53),
            eval!(context, 1),
            eval!(context, xor_col81),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_81 =
        eval!(context, (1073741824) * (((op0_limb_23_col24) + (op1_limb_23_col53)) - (xor_col81)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_24_col25),
            eval!(context, op1_limb_24_col54),
            eval!(context, 1),
            eval!(context, xor_col82),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_84 =
        eval!(context, (1073741824) * (((op0_limb_24_col25) + (op1_limb_24_col54)) - (xor_col82)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_25_col26),
            eval!(context, op1_limb_25_col55),
            eval!(context, 1),
            eval!(context, xor_col83),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_87 =
        eval!(context, (1073741824) * (((op0_limb_25_col26) + (op1_limb_25_col55)) - (xor_col83)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_26_col27),
            eval!(context, op1_limb_26_col56),
            eval!(context, 1),
            eval!(context, xor_col84),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_90 =
        eval!(context, (1073741824) * (((op0_limb_26_col27) + (op1_limb_26_col56)) - (xor_col84)));

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, op0_limb_27_col28),
            eval!(context, op1_limb_27_col57),
            eval!(context, 1),
            eval!(context, xor_col85),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_b8fb8_93 =
        eval!(context, (1073741824) * (((op0_limb_27_col28) + (op1_limb_27_col57)) - (xor_col85)));

    mem_verify::accumulate_constraints(
        &[
            eval!(context, ((bitwise_builtin_segment_start) + ((seq) * (5))) + (2)),
            eval!(context, and_tmp_b8fb8_12),
            eval!(context, and_tmp_b8fb8_15),
            eval!(context, and_tmp_b8fb8_18),
            eval!(context, and_tmp_b8fb8_21),
            eval!(context, and_tmp_b8fb8_24),
            eval!(context, and_tmp_b8fb8_27),
            eval!(context, and_tmp_b8fb8_30),
            eval!(context, and_tmp_b8fb8_33),
            eval!(context, and_tmp_b8fb8_36),
            eval!(context, and_tmp_b8fb8_39),
            eval!(context, and_tmp_b8fb8_42),
            eval!(context, and_tmp_b8fb8_45),
            eval!(context, and_tmp_b8fb8_48),
            eval!(context, and_tmp_b8fb8_51),
            eval!(context, and_tmp_b8fb8_54),
            eval!(context, and_tmp_b8fb8_57),
            eval!(context, and_tmp_b8fb8_60),
            eval!(context, and_tmp_b8fb8_63),
            eval!(context, and_tmp_b8fb8_66),
            eval!(context, and_tmp_b8fb8_69),
            eval!(context, and_tmp_b8fb8_72),
            eval!(context, and_tmp_b8fb8_75),
            eval!(context, and_tmp_b8fb8_78),
            eval!(context, and_tmp_b8fb8_81),
            eval!(context, and_tmp_b8fb8_84),
            eval!(context, and_tmp_b8fb8_87),
            eval!(context, and_tmp_b8fb8_90),
            eval!(context, and_tmp_b8fb8_93),
            eval!(context, 1),
            eval!(context, and_id_col86),
        ],
        context,
        component_data,
        acc,
    );

    mem_verify::accumulate_constraints(
        &[
            eval!(context, ((bitwise_builtin_segment_start) + ((seq) * (5))) + (3)),
            eval!(context, xor_col58),
            eval!(context, xor_col59),
            eval!(context, xor_col60),
            eval!(context, xor_col61),
            eval!(context, xor_col62),
            eval!(context, xor_col63),
            eval!(context, xor_col64),
            eval!(context, xor_col65),
            eval!(context, xor_col66),
            eval!(context, xor_col67),
            eval!(context, xor_col68),
            eval!(context, xor_col69),
            eval!(context, xor_col70),
            eval!(context, xor_col71),
            eval!(context, xor_col72),
            eval!(context, xor_col73),
            eval!(context, xor_col74),
            eval!(context, xor_col75),
            eval!(context, xor_col76),
            eval!(context, xor_col77),
            eval!(context, xor_col78),
            eval!(context, xor_col79),
            eval!(context, xor_col80),
            eval!(context, xor_col81),
            eval!(context, xor_col82),
            eval!(context, xor_col83),
            eval!(context, xor_col84),
            eval!(context, xor_col85),
            eval!(context, 1),
            eval!(context, xor_id_col87),
        ],
        context,
        component_data,
        acc,
    );

    mem_verify::accumulate_constraints(
        &[
            eval!(context, ((bitwise_builtin_segment_start) + ((seq) * (5))) + (4)),
            eval!(context, (and_tmp_b8fb8_12) + (xor_col58)),
            eval!(context, (and_tmp_b8fb8_15) + (xor_col59)),
            eval!(context, (and_tmp_b8fb8_18) + (xor_col60)),
            eval!(context, (and_tmp_b8fb8_21) + (xor_col61)),
            eval!(context, (and_tmp_b8fb8_24) + (xor_col62)),
            eval!(context, (and_tmp_b8fb8_27) + (xor_col63)),
            eval!(context, (and_tmp_b8fb8_30) + (xor_col64)),
            eval!(context, (and_tmp_b8fb8_33) + (xor_col65)),
            eval!(context, (and_tmp_b8fb8_36) + (xor_col66)),
            eval!(context, (and_tmp_b8fb8_39) + (xor_col67)),
            eval!(context, (and_tmp_b8fb8_42) + (xor_col68)),
            eval!(context, (and_tmp_b8fb8_45) + (xor_col69)),
            eval!(context, (and_tmp_b8fb8_48) + (xor_col70)),
            eval!(context, (and_tmp_b8fb8_51) + (xor_col71)),
            eval!(context, (and_tmp_b8fb8_54) + (xor_col72)),
            eval!(context, (and_tmp_b8fb8_57) + (xor_col73)),
            eval!(context, (and_tmp_b8fb8_60) + (xor_col74)),
            eval!(context, (and_tmp_b8fb8_63) + (xor_col75)),
            eval!(context, (and_tmp_b8fb8_66) + (xor_col76)),
            eval!(context, (and_tmp_b8fb8_69) + (xor_col77)),
            eval!(context, (and_tmp_b8fb8_72) + (xor_col78)),
            eval!(context, (and_tmp_b8fb8_75) + (xor_col79)),
            eval!(context, (and_tmp_b8fb8_78) + (xor_col80)),
            eval!(context, (and_tmp_b8fb8_81) + (xor_col81)),
            eval!(context, (and_tmp_b8fb8_84) + (xor_col82)),
            eval!(context, (and_tmp_b8fb8_87) + (xor_col83)),
            eval!(context, (and_tmp_b8fb8_90) + (xor_col84)),
            eval!(context, (and_tmp_b8fb8_93) + (xor_col85)),
            eval!(context, 1),
            eval!(context, or_id_col88),
        ],
        context,
        component_data,
        acc,
    );
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "bitwise_builtin".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &RELATION_USES_PER_ROW
    }

    fn log_size(
        &self,
        _preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        None
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use circuits::context::Context;
    use circuits::ivalue::qm31_from_u32s;
    use circuits_stark_verifier::constraint_eval::*;
    use circuits_stark_verifier::test_utils::TestComponentData;
    use stwo::core::fields::qm31::QM31;

    use super::Component;
    #[allow(unused_imports)]
    use crate::components::prelude::PreProcessedColumnId;
    use crate::sample_evaluations::*;

    #[test]
    fn test_evaluation_result() {
        let component = Component {};
        let mut context: Context<QM31> = Default::default();
        context.enable_assert_eq_on_eval();
        let trace_columns = [
            qm31_from_u32s(1659099300, 905558730, 651199673, 1375009625),
            qm31_from_u32s(1591990121, 771341002, 584090809, 1375009625),
            qm31_from_u32s(1793317658, 1173994186, 785417401, 1375009625),
            qm31_from_u32s(1726208479, 1039776458, 718308537, 1375009625),
            qm31_from_u32s(1390662584, 368687818, 382764217, 1375009625),
            qm31_from_u32s(1323553405, 234470090, 315655353, 1375009625),
            qm31_from_u32s(1524880942, 637123274, 516981945, 1375009625),
            qm31_from_u32s(1457771763, 502905546, 449873081, 1375009625),
            qm31_from_u32s(48489085, 1979300555, 1188070585, 1375009625),
            qm31_from_u32s(2128863553, 1845082826, 1120961721, 1375009625),
            qm31_from_u32s(1852335767, 645078115, 2059236183, 343880121),
            qm31_from_u32s(1919444946, 779295843, 2126345047, 343880121),
            qm31_from_u32s(1986554125, 913513571, 45970264, 343880122),
            qm31_from_u32s(2053663304, 1047731299, 113079128, 343880122),
            qm31_from_u32s(1583899051, 108207203, 1790800727, 343880121),
            qm31_from_u32s(1651008230, 242424931, 1857909591, 343880121),
            qm31_from_u32s(1718117409, 376642659, 1925018455, 343880121),
            qm31_from_u32s(1785226588, 510860387, 1992127319, 343880121),
            qm31_from_u32s(1315462335, 1718819938, 1522365270, 343880121),
            qm31_from_u32s(1382571514, 1853037666, 1589474134, 343880121),
            qm31_from_u32s(1986820986, 913513739, 45970432, 343880178),
            qm31_from_u32s(1919711807, 779296011, 2126345215, 343880177),
            qm31_from_u32s(2121039344, 1181949195, 180188160, 343880178),
            qm31_from_u32s(2053930165, 1047731467, 113079296, 343880178),
            qm31_from_u32s(1718384270, 376642827, 1925018623, 343880177),
            qm31_from_u32s(1651275091, 242425099, 1857909759, 343880177),
            qm31_from_u32s(1852602628, 645078283, 2059236351, 343880177),
            qm31_from_u32s(1785493449, 510860555, 1992127487, 343880177),
            qm31_from_u32s(1449947554, 1987255562, 1656583166, 343880177),
            qm31_from_u32s(1382838375, 1853037834, 1589474302, 343880177),
            qm31_from_u32s(510356977, 108207322, 717059022, 343880161),
            qm31_from_u32s(577466156, 242425050, 784167886, 343880161),
            qm31_from_u32s(376138619, 1987255513, 582841293, 343880161),
            qm31_from_u32s(443247798, 2121473241, 649950157, 343880161),
            qm31_from_u32s(778793693, 645078234, 985494478, 343880161),
            qm31_from_u32s(845902872, 779295962, 1052603342, 343880161),
            qm31_from_u32s(644575335, 376642778, 851276750, 343880161),
            qm31_from_u32s(711684514, 510860506, 918385614, 343880161),
            qm31_from_u32s(1047230409, 1181949146, 1253929934, 343880161),
            qm31_from_u32s(1114339588, 1316166874, 1321038798, 343880161),
            qm31_from_u32s(1717810224, 376642479, 1925018275, 343880061),
            qm31_from_u32s(1650701045, 242424751, 1857909411, 343880061),
            qm31_from_u32s(1583591866, 108207023, 1790800547, 343880061),
            qm31_from_u32s(1516482687, 2121472942, 1723691682, 343880061),
            qm31_from_u32s(1986246940, 913513391, 45970084, 343880062),
            qm31_from_u32s(1919137761, 779295663, 2126344867, 343880061),
            qm31_from_u32s(1852028582, 645077935, 2059236003, 343880061),
            qm31_from_u32s(1784919403, 510860207, 1992127139, 343880061),
            qm31_from_u32s(1180936792, 1450384302, 1388147362, 343880061),
            qm31_from_u32s(1113827613, 1316166574, 1321038498, 343880061),
            qm31_from_u32s(241305891, 1718819697, 448623205, 343880041),
            qm31_from_u32s(308415070, 1853037425, 515732069, 343880041),
            qm31_from_u32s(375524249, 1987255153, 582840933, 343880041),
            qm31_from_u32s(442633428, 2121472881, 649949797, 343880041),
            qm31_from_u32s(509742607, 108206962, 717058662, 343880041),
            qm31_from_u32s(576851786, 242424690, 784167526, 343880041),
            qm31_from_u32s(643960965, 376642418, 851276390, 343880041),
            qm31_from_u32s(711070144, 510860146, 918385254, 343880041),
            qm31_from_u32s(778179323, 645077874, 985494118, 343880041),
            qm31_from_u32s(845288502, 779295602, 1052602982, 343880041),
            qm31_from_u32s(375831434, 1987255333, 582841113, 343880101),
            qm31_from_u32s(308722255, 1853037605, 515732249, 343880101),
            qm31_from_u32s(510049792, 108207142, 717058842, 343880101),
            qm31_from_u32s(442940613, 2121473061, 649949977, 343880101),
            qm31_from_u32s(644268150, 376642598, 851276570, 343880101),
            qm31_from_u32s(577158971, 242424870, 784167706, 343880101),
            qm31_from_u32s(778486508, 645078054, 985494298, 343880101),
            qm31_from_u32s(711377329, 510860326, 918385434, 343880101),
            qm31_from_u32s(912704866, 913513510, 1119712026, 343880101),
            qm31_from_u32s(845595687, 779295782, 1052603162, 343880101),
            qm31_from_u32s(1046820829, 1181948906, 1253929694, 343880081),
            qm31_from_u32s(1113930008, 1316166634, 1321038558, 343880081),
            qm31_from_u32s(912602471, 913513450, 1119711966, 343880081),
            qm31_from_u32s(979711650, 1047731178, 1186820830, 343880081),
            qm31_from_u32s(778384113, 645077994, 985494238, 343880081),
            qm31_from_u32s(845493292, 779295722, 1052603102, 343880081),
            qm31_from_u32s(644165755, 376642538, 851276510, 343880081),
            qm31_from_u32s(711274934, 510860266, 918385374, 343880081),
            qm31_from_u32s(1583694261, 108207083, 1790800607, 343880081),
            qm31_from_u32s(1650803440, 242424811, 1857909471, 343880081),
            qm31_from_u32s(108388425, 1450385012, 314406248, 343880298),
            qm31_from_u32s(41279246, 1316167284, 247297384, 343880298),
            qm31_from_u32s(2121653714, 1181949555, 180188520, 343880298),
            qm31_from_u32s(2054544535, 1047731827, 113079656, 343880298),
            qm31_from_u32s(1987435356, 913514099, 45970792, 343880298),
            qm31_from_u32s(1920326177, 779296371, 2126345575, 343880297),
            qm31_from_u32s(1853216998, 645078643, 2059236711, 343880297),
            qm31_from_u32s(1786107819, 510860915, 1992127847, 343880297),
            qm31_from_u32s(1718998640, 376643187, 1925018983, 343880297),
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
            qm31_from_u32s(1206495569, 482634180, 2049214693, 1941984119),
            qm31_from_u32s(736731316, 1690593731, 1579452644, 1941984119),
            qm31_from_u32s(803840495, 1824811459, 1646561508, 1941984119),
            qm31_from_u32s(870949674, 1959029187, 1713670372, 1941984119),
            qm31_from_u32s(938058853, 2093246915, 1780779236, 1941984119),
            qm31_from_u32s(1542041464, 1153722820, 237275366, 1941984120),
            qm31_from_u32s(1609150643, 1287940548, 304384230, 1941984120),
            qm31_from_u32s(1577898798, 106101108, 1738096752, 1261630210),
            qm31_from_u32s(1510789619, 2119367027, 1670987887, 1261630210),
            qm31_from_u32s(1443680440, 1985149299, 1603879023, 1261630210),
            qm31_from_u32s(1376571261, 1850931571, 1536770159, 1261630210),
            qm31_from_u32s(1309462082, 1716713843, 1469661295, 1261630210),
            qm31_from_u32s(1242352903, 1582496115, 1402552431, 1261630210),
            qm31_from_u32s(1175243724, 1448278387, 1335443567, 1261630210),
            qm31_from_u32s(1108134545, 1314060659, 1268334703, 1261630210),
            qm31_from_u32s(2114772230, 1179842932, 127484017, 1261630211),
        ];
        let component_data = TestComponentData::from_values(
            &mut context,
            &trace_columns,
            &interaction_columns,
            qm31_from_u32s(1115374022, 1127856551, 489657863, 643630026),
            32768,
        );
        let random_coeff =
            context.new_var(qm31_from_u32s(474642921, 876336632, 1911695779, 974600512));
        let interaction_elements = [
            context.new_var(qm31_from_u32s(445623802, 202571636, 1360224996, 131355117)),
            context.new_var(qm31_from_u32s(476823935, 939223384, 62486082, 122423602)),
        ];
        let preprocessed_columns = HashMap::from([(
            PreProcessedColumnId { id: "seq_15".to_owned() },
            context.constant(qm31_from_u32s(735272696, 1215403647, 795393303, 879304430)),
        )]);
        let public_params = HashMap::from([(
            "bitwise_builtin_segment_start".to_owned(),
            context.constant(434121993.into()),
        )]);
        let mut accumulator = CompositionConstraintAccumulator::new(
            &mut context,
            preprocessed_columns,
            public_params,
            random_coeff,
            interaction_elements,
        );
        component.evaluate(&mut context, &component_data, &mut accumulator);
        let claimed_sum =
            context.new_var(qm31_from_u32s(1398335417, 314974026, 1722107152, 821933968));
        accumulator.finalize_logup_in_pairs(
            &mut context,
            <TestComponentData as ComponentDataTrait<QM31>>::interaction_columns(&component_data),
            &component_data,
            claimed_sum,
        );

        let result = accumulator.finalize();
        let result_value = context.get(result);
        assert_eq!(result_value, BITWISE_BUILTIN_SAMPLE_EVAL_RESULT)
    }
}
