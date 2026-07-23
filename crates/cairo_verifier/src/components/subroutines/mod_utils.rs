// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 29 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 24 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        mod_utils_input_first_addr,
        mod_utils_input_instance_num,
        enabler,
        is_instance_0_col0,
        p0_id_col1,
        p0_limb_0_col2,
        p0_limb_1_col3,
        p0_limb_2_col4,
        p0_limb_3_col5,
        p0_limb_4_col6,
        p0_limb_5_col7,
        p0_limb_6_col8,
        p0_limb_7_col9,
        p0_limb_8_col10,
        p0_limb_9_col11,
        p0_limb_10_col12,
        p1_id_col13,
        p1_limb_0_col14,
        p1_limb_1_col15,
        p1_limb_2_col16,
        p1_limb_3_col17,
        p1_limb_4_col18,
        p1_limb_5_col19,
        p1_limb_6_col20,
        p1_limb_7_col21,
        p1_limb_8_col22,
        p1_limb_9_col23,
        p1_limb_10_col24,
        p2_id_col25,
        p2_limb_0_col26,
        p2_limb_1_col27,
        p2_limb_2_col28,
        p2_limb_3_col29,
        p2_limb_4_col30,
        p2_limb_5_col31,
        p2_limb_6_col32,
        p2_limb_7_col33,
        p2_limb_8_col34,
        p2_limb_9_col35,
        p2_limb_10_col36,
        p3_id_col37,
        p3_limb_0_col38,
        p3_limb_1_col39,
        p3_limb_2_col40,
        p3_limb_3_col41,
        p3_limb_4_col42,
        p3_limb_5_col43,
        p3_limb_6_col44,
        p3_limb_7_col45,
        p3_limb_8_col46,
        p3_limb_9_col47,
        p3_limb_10_col48,
        values_ptr_id_col49,
        values_ptr_limb_0_col50,
        values_ptr_limb_1_col51,
        values_ptr_limb_2_col52,
        values_ptr_limb_3_col53,
        partial_limb_msb_col54,
        offsets_ptr_id_col55,
        offsets_ptr_limb_0_col56,
        offsets_ptr_limb_1_col57,
        offsets_ptr_limb_2_col58,
        offsets_ptr_limb_3_col59,
        partial_limb_msb_col60,
        offsets_ptr_prev_id_col61,
        offsets_ptr_prev_limb_0_col62,
        offsets_ptr_prev_limb_1_col63,
        offsets_ptr_prev_limb_2_col64,
        offsets_ptr_prev_limb_3_col65,
        partial_limb_msb_col66,
        n_id_col67,
        n_limb_0_col68,
        n_limb_1_col69,
        n_limb_2_col70,
        n_limb_3_col71,
        partial_limb_msb_col72,
        n_prev_id_col73,
        n_prev_limb_0_col74,
        n_prev_limb_1_col75,
        n_prev_limb_2_col76,
        n_prev_limb_3_col77,
        partial_limb_msb_col78,
        values_ptr_prev_id_col79,
        p_prev0_id_col80,
        p_prev1_id_col81,
        p_prev2_id_col82,
        p_prev3_id_col83,
        offsets_a_id_col84,
        msb_col85,
        mid_limbs_set_col86,
        offsets_a_limb_0_col87,
        offsets_a_limb_1_col88,
        offsets_a_limb_2_col89,
        remainder_bits_col90,
        partial_limb_msb_col91,
        offsets_b_id_col92,
        msb_col93,
        mid_limbs_set_col94,
        offsets_b_limb_0_col95,
        offsets_b_limb_1_col96,
        offsets_b_limb_2_col97,
        remainder_bits_col98,
        partial_limb_msb_col99,
        offsets_c_id_col100,
        msb_col101,
        mid_limbs_set_col102,
        offsets_c_limb_0_col103,
        offsets_c_limb_1_col104,
        offsets_c_limb_2_col105,
        remainder_bits_col106,
        partial_limb_msb_col107,
        a0_id_col108,
        a0_limb_0_col109,
        a0_limb_1_col110,
        a0_limb_2_col111,
        a0_limb_3_col112,
        a0_limb_4_col113,
        a0_limb_5_col114,
        a0_limb_6_col115,
        a0_limb_7_col116,
        a0_limb_8_col117,
        a0_limb_9_col118,
        a0_limb_10_col119,
        a1_id_col120,
        a1_limb_0_col121,
        a1_limb_1_col122,
        a1_limb_2_col123,
        a1_limb_3_col124,
        a1_limb_4_col125,
        a1_limb_5_col126,
        a1_limb_6_col127,
        a1_limb_7_col128,
        a1_limb_8_col129,
        a1_limb_9_col130,
        a1_limb_10_col131,
        a2_id_col132,
        a2_limb_0_col133,
        a2_limb_1_col134,
        a2_limb_2_col135,
        a2_limb_3_col136,
        a2_limb_4_col137,
        a2_limb_5_col138,
        a2_limb_6_col139,
        a2_limb_7_col140,
        a2_limb_8_col141,
        a2_limb_9_col142,
        a2_limb_10_col143,
        a3_id_col144,
        a3_limb_0_col145,
        a3_limb_1_col146,
        a3_limb_2_col147,
        a3_limb_3_col148,
        a3_limb_4_col149,
        a3_limb_5_col150,
        a3_limb_6_col151,
        a3_limb_7_col152,
        a3_limb_8_col153,
        a3_limb_9_col154,
        a3_limb_10_col155,
        b0_id_col156,
        b0_limb_0_col157,
        b0_limb_1_col158,
        b0_limb_2_col159,
        b0_limb_3_col160,
        b0_limb_4_col161,
        b0_limb_5_col162,
        b0_limb_6_col163,
        b0_limb_7_col164,
        b0_limb_8_col165,
        b0_limb_9_col166,
        b0_limb_10_col167,
        b1_id_col168,
        b1_limb_0_col169,
        b1_limb_1_col170,
        b1_limb_2_col171,
        b1_limb_3_col172,
        b1_limb_4_col173,
        b1_limb_5_col174,
        b1_limb_6_col175,
        b1_limb_7_col176,
        b1_limb_8_col177,
        b1_limb_9_col178,
        b1_limb_10_col179,
        b2_id_col180,
        b2_limb_0_col181,
        b2_limb_1_col182,
        b2_limb_2_col183,
        b2_limb_3_col184,
        b2_limb_4_col185,
        b2_limb_5_col186,
        b2_limb_6_col187,
        b2_limb_7_col188,
        b2_limb_8_col189,
        b2_limb_9_col190,
        b2_limb_10_col191,
        b3_id_col192,
        b3_limb_0_col193,
        b3_limb_1_col194,
        b3_limb_2_col195,
        b3_limb_3_col196,
        b3_limb_4_col197,
        b3_limb_5_col198,
        b3_limb_6_col199,
        b3_limb_7_col200,
        b3_limb_8_col201,
        b3_limb_9_col202,
        b3_limb_10_col203,
        c0_id_col204,
        c0_limb_0_col205,
        c0_limb_1_col206,
        c0_limb_2_col207,
        c0_limb_3_col208,
        c0_limb_4_col209,
        c0_limb_5_col210,
        c0_limb_6_col211,
        c0_limb_7_col212,
        c0_limb_8_col213,
        c0_limb_9_col214,
        c0_limb_10_col215,
        c1_id_col216,
        c1_limb_0_col217,
        c1_limb_1_col218,
        c1_limb_2_col219,
        c1_limb_3_col220,
        c1_limb_4_col221,
        c1_limb_5_col222,
        c1_limb_6_col223,
        c1_limb_7_col224,
        c1_limb_8_col225,
        c1_limb_9_col226,
        c1_limb_10_col227,
        c2_id_col228,
        c2_limb_0_col229,
        c2_limb_1_col230,
        c2_limb_2_col231,
        c2_limb_3_col232,
        c2_limb_4_col233,
        c2_limb_5_col234,
        c2_limb_6_col235,
        c2_limb_7_col236,
        c2_limb_8_col237,
        c2_limb_9_col238,
        c2_limb_10_col239,
        c3_id_col240,
        c3_limb_0_col241,
        c3_limb_1_col242,
        c3_limb_2_col243,
        c3_limb_3_col244,
        c3_limb_4_col245,
        c3_limb_5_col246,
        c3_limb_6_col247,
        c3_limb_7_col248,
        c3_limb_8_col249,
        c3_limb_9_col250,
        c3_limb_10_col251,
    ] = input.try_into().unwrap();

    // is_instance_0 is 0 or 1..
    let constraint_0_value = eval!(context, (is_instance_0_col0) * ((is_instance_0_col0) - (1)));
    acc.add_constraint(context, constraint_0_value);

    // is_instance_0 is 0 when instance_num is not 0..
    let constraint_1_value = eval!(context, (is_instance_0_col0) * (mod_utils_input_instance_num));
    acc.add_constraint(context, constraint_1_value);

    let is_instance_0_minus_1_tmp_e0805_1 = eval!(context, (is_instance_0_col0) - (1));

    let instance_addr_tmp_e0805_2 =
        eval!(context, (mod_utils_input_first_addr) + ((7) * (mod_utils_input_instance_num)));

    let prev_instance_addr_tmp_e0805_3 =
        eval!(context, (instance_addr_tmp_e0805_2) + ((7) * (is_instance_0_minus_1_tmp_e0805_1)));

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(context, instance_addr_tmp_e0805_2),
            eval!(context, enabler),
            eval!(context, p0_id_col1),
            eval!(context, p0_limb_0_col2),
            eval!(context, p0_limb_1_col3),
            eval!(context, p0_limb_2_col4),
            eval!(context, p0_limb_3_col5),
            eval!(context, p0_limb_4_col6),
            eval!(context, p0_limb_5_col7),
            eval!(context, p0_limb_6_col8),
            eval!(context, p0_limb_7_col9),
            eval!(context, p0_limb_8_col10),
            eval!(context, p0_limb_9_col11),
            eval!(context, p0_limb_10_col12),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_e0805_2) + (1)),
            eval!(context, enabler),
            eval!(context, p1_id_col13),
            eval!(context, p1_limb_0_col14),
            eval!(context, p1_limb_1_col15),
            eval!(context, p1_limb_2_col16),
            eval!(context, p1_limb_3_col17),
            eval!(context, p1_limb_4_col18),
            eval!(context, p1_limb_5_col19),
            eval!(context, p1_limb_6_col20),
            eval!(context, p1_limb_7_col21),
            eval!(context, p1_limb_8_col22),
            eval!(context, p1_limb_9_col23),
            eval!(context, p1_limb_10_col24),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_e0805_2) + (2)),
            eval!(context, enabler),
            eval!(context, p2_id_col25),
            eval!(context, p2_limb_0_col26),
            eval!(context, p2_limb_1_col27),
            eval!(context, p2_limb_2_col28),
            eval!(context, p2_limb_3_col29),
            eval!(context, p2_limb_4_col30),
            eval!(context, p2_limb_5_col31),
            eval!(context, p2_limb_6_col32),
            eval!(context, p2_limb_7_col33),
            eval!(context, p2_limb_8_col34),
            eval!(context, p2_limb_9_col35),
            eval!(context, p2_limb_10_col36),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_e0805_2) + (3)),
            eval!(context, enabler),
            eval!(context, p3_id_col37),
            eval!(context, p3_limb_0_col38),
            eval!(context, p3_limb_1_col39),
            eval!(context, p3_limb_2_col40),
            eval!(context, p3_limb_3_col41),
            eval!(context, p3_limb_4_col42),
            eval!(context, p3_limb_5_col43),
            eval!(context, p3_limb_6_col44),
            eval!(context, p3_limb_7_col45),
            eval!(context, p3_limb_8_col46),
            eval!(context, p3_limb_9_col47),
            eval!(context, p3_limb_10_col48),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_e0805_2) + (4)),
            eval!(context, enabler),
            eval!(context, values_ptr_id_col49),
            eval!(context, values_ptr_limb_0_col50),
            eval!(context, values_ptr_limb_1_col51),
            eval!(context, values_ptr_limb_2_col52),
            eval!(context, values_ptr_limb_3_col53),
            eval!(context, partial_limb_msb_col54),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_e0805_2) + (5)),
            eval!(context, enabler),
            eval!(context, offsets_ptr_id_col55),
            eval!(context, offsets_ptr_limb_0_col56),
            eval!(context, offsets_ptr_limb_1_col57),
            eval!(context, offsets_ptr_limb_2_col58),
            eval!(context, offsets_ptr_limb_3_col59),
            eval!(context, partial_limb_msb_col60),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (prev_instance_addr_tmp_e0805_3) + (5)),
            eval!(context, enabler),
            eval!(context, offsets_ptr_prev_id_col61),
            eval!(context, offsets_ptr_prev_limb_0_col62),
            eval!(context, offsets_ptr_prev_limb_1_col63),
            eval!(context, offsets_ptr_prev_limb_2_col64),
            eval!(context, offsets_ptr_prev_limb_3_col65),
            eval!(context, partial_limb_msb_col66),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_e0805_2) + (6)),
            eval!(context, enabler),
            eval!(context, n_id_col67),
            eval!(context, n_limb_0_col68),
            eval!(context, n_limb_1_col69),
            eval!(context, n_limb_2_col70),
            eval!(context, n_limb_3_col71),
            eval!(context, partial_limb_msb_col72),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (prev_instance_addr_tmp_e0805_3) + (6)),
            eval!(context, enabler),
            eval!(context, n_prev_id_col73),
            eval!(context, n_prev_limb_0_col74),
            eval!(context, n_prev_limb_1_col75),
            eval!(context, n_prev_limb_2_col76),
            eval!(context, n_prev_limb_3_col77),
            eval!(context, partial_limb_msb_col78),
        ],
        context,
        component_data,
        acc,
    );

    let n_prev_minus_1_tmp_e0805_59 = eval!(
        context,
        ((((n_prev_limb_0_col74) + ((n_prev_limb_1_col75) * (512)))
            + ((n_prev_limb_2_col76) * (262144)))
            + ((n_prev_limb_3_col77) * (134217728)))
            - (1)
    );

    let offsets_ptr_tmp_e0805_60 = eval!(
        context,
        (((offsets_ptr_limb_0_col56) + ((offsets_ptr_limb_1_col57) * (512)))
            + ((offsets_ptr_limb_2_col58) * (262144)))
            + ((offsets_ptr_limb_3_col59) * (134217728))
    );

    let block_reset_condition_tmp_e0805_61 =
        eval!(context, (n_prev_minus_1_tmp_e0805_59) * (is_instance_0_minus_1_tmp_e0805_1));

    // Progression of n between instances..
    let constraint_17_value = eval!(
        context,
        (block_reset_condition_tmp_e0805_61)
            * ((n_prev_minus_1_tmp_e0805_59)
                - ((((n_limb_0_col68) + ((n_limb_1_col69) * (512)))
                    + ((n_limb_2_col70) * (262144)))
                    + ((n_limb_3_col71) * (134217728))))
    );
    acc.add_constraint(context, constraint_17_value);

    // Progression of offsets_ptr between instances..
    let constraint_18_value = eval!(
        context,
        (block_reset_condition_tmp_e0805_61)
            * (((offsets_ptr_tmp_e0805_60) - (3))
                - ((((offsets_ptr_prev_limb_0_col62)
                    + ((offsets_ptr_prev_limb_1_col63) * (512)))
                    + ((offsets_ptr_prev_limb_2_col64) * (262144)))
                    + ((offsets_ptr_prev_limb_3_col65) * (134217728))))
    );
    acc.add_constraint(context, constraint_18_value);

    mem_cond_verify_equal_known_id::accumulate_constraints(
        &[
            eval!(context, (prev_instance_addr_tmp_e0805_3) + (4)),
            eval!(context, values_ptr_id_col49),
            eval!(context, block_reset_condition_tmp_e0805_61),
            eval!(context, enabler),
            eval!(context, values_ptr_prev_id_col79),
        ],
        context,
        component_data,
        acc,
    );

    mem_cond_verify_equal_known_id::accumulate_constraints(
        &[
            eval!(context, prev_instance_addr_tmp_e0805_3),
            eval!(context, p0_id_col1),
            eval!(context, block_reset_condition_tmp_e0805_61),
            eval!(context, enabler),
            eval!(context, p_prev0_id_col80),
        ],
        context,
        component_data,
        acc,
    );

    mem_cond_verify_equal_known_id::accumulate_constraints(
        &[
            eval!(context, (prev_instance_addr_tmp_e0805_3) + (1)),
            eval!(context, p1_id_col13),
            eval!(context, block_reset_condition_tmp_e0805_61),
            eval!(context, enabler),
            eval!(context, p_prev1_id_col81),
        ],
        context,
        component_data,
        acc,
    );

    mem_cond_verify_equal_known_id::accumulate_constraints(
        &[
            eval!(context, (prev_instance_addr_tmp_e0805_3) + (2)),
            eval!(context, p2_id_col25),
            eval!(context, block_reset_condition_tmp_e0805_61),
            eval!(context, enabler),
            eval!(context, p_prev2_id_col82),
        ],
        context,
        component_data,
        acc,
    );

    mem_cond_verify_equal_known_id::accumulate_constraints(
        &[
            eval!(context, (prev_instance_addr_tmp_e0805_3) + (3)),
            eval!(context, p3_id_col37),
            eval!(context, block_reset_condition_tmp_e0805_61),
            eval!(context, enabler),
            eval!(context, p_prev3_id_col83),
        ],
        context,
        component_data,
        acc,
    );

    let [read_small_output_tmp_e0805_81_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(context, offsets_ptr_tmp_e0805_60),
            eval!(context, enabler),
            eval!(context, offsets_a_id_col84),
            eval!(context, msb_col85),
            eval!(context, mid_limbs_set_col86),
            eval!(context, offsets_a_limb_0_col87),
            eval!(context, offsets_a_limb_1_col88),
            eval!(context, offsets_a_limb_2_col89),
            eval!(context, remainder_bits_col90),
            eval!(context, partial_limb_msb_col91),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [read_small_output_tmp_e0805_91_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(context, (offsets_ptr_tmp_e0805_60) + (1)),
            eval!(context, enabler),
            eval!(context, offsets_b_id_col92),
            eval!(context, msb_col93),
            eval!(context, mid_limbs_set_col94),
            eval!(context, offsets_b_limb_0_col95),
            eval!(context, offsets_b_limb_1_col96),
            eval!(context, offsets_b_limb_2_col97),
            eval!(context, remainder_bits_col98),
            eval!(context, partial_limb_msb_col99),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [read_small_output_tmp_e0805_101_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(context, (offsets_ptr_tmp_e0805_60) + (2)),
            eval!(context, enabler),
            eval!(context, offsets_c_id_col100),
            eval!(context, msb_col101),
            eval!(context, mid_limbs_set_col102),
            eval!(context, offsets_c_limb_0_col103),
            eval!(context, offsets_c_limb_1_col104),
            eval!(context, offsets_c_limb_2_col105),
            eval!(context, remainder_bits_col106),
            eval!(context, partial_limb_msb_col107),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let values_ptr_tmp_e0805_102 = eval!(
        context,
        (((values_ptr_limb_0_col50) + ((values_ptr_limb_1_col51) * (512)))
            + ((values_ptr_limb_2_col52) * (262144)))
            + ((values_ptr_limb_3_col53) * (134217728))
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(context, (values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_81_limb_0)),
            eval!(context, enabler),
            eval!(context, a0_id_col108),
            eval!(context, a0_limb_0_col109),
            eval!(context, a0_limb_1_col110),
            eval!(context, a0_limb_2_col111),
            eval!(context, a0_limb_3_col112),
            eval!(context, a0_limb_4_col113),
            eval!(context, a0_limb_5_col114),
            eval!(context, a0_limb_6_col115),
            eval!(context, a0_limb_7_col116),
            eval!(context, a0_limb_8_col117),
            eval!(context, a0_limb_9_col118),
            eval!(context, a0_limb_10_col119),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_81_limb_0)) + (1)
            ),
            eval!(context, enabler),
            eval!(context, a1_id_col120),
            eval!(context, a1_limb_0_col121),
            eval!(context, a1_limb_1_col122),
            eval!(context, a1_limb_2_col123),
            eval!(context, a1_limb_3_col124),
            eval!(context, a1_limb_4_col125),
            eval!(context, a1_limb_5_col126),
            eval!(context, a1_limb_6_col127),
            eval!(context, a1_limb_7_col128),
            eval!(context, a1_limb_8_col129),
            eval!(context, a1_limb_9_col130),
            eval!(context, a1_limb_10_col131),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_81_limb_0)) + (2)
            ),
            eval!(context, enabler),
            eval!(context, a2_id_col132),
            eval!(context, a2_limb_0_col133),
            eval!(context, a2_limb_1_col134),
            eval!(context, a2_limb_2_col135),
            eval!(context, a2_limb_3_col136),
            eval!(context, a2_limb_4_col137),
            eval!(context, a2_limb_5_col138),
            eval!(context, a2_limb_6_col139),
            eval!(context, a2_limb_7_col140),
            eval!(context, a2_limb_8_col141),
            eval!(context, a2_limb_9_col142),
            eval!(context, a2_limb_10_col143),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_81_limb_0)) + (3)
            ),
            eval!(context, enabler),
            eval!(context, a3_id_col144),
            eval!(context, a3_limb_0_col145),
            eval!(context, a3_limb_1_col146),
            eval!(context, a3_limb_2_col147),
            eval!(context, a3_limb_3_col148),
            eval!(context, a3_limb_4_col149),
            eval!(context, a3_limb_5_col150),
            eval!(context, a3_limb_6_col151),
            eval!(context, a3_limb_7_col152),
            eval!(context, a3_limb_8_col153),
            eval!(context, a3_limb_9_col154),
            eval!(context, a3_limb_10_col155),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(context, (values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_91_limb_0)),
            eval!(context, enabler),
            eval!(context, b0_id_col156),
            eval!(context, b0_limb_0_col157),
            eval!(context, b0_limb_1_col158),
            eval!(context, b0_limb_2_col159),
            eval!(context, b0_limb_3_col160),
            eval!(context, b0_limb_4_col161),
            eval!(context, b0_limb_5_col162),
            eval!(context, b0_limb_6_col163),
            eval!(context, b0_limb_7_col164),
            eval!(context, b0_limb_8_col165),
            eval!(context, b0_limb_9_col166),
            eval!(context, b0_limb_10_col167),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_91_limb_0)) + (1)
            ),
            eval!(context, enabler),
            eval!(context, b1_id_col168),
            eval!(context, b1_limb_0_col169),
            eval!(context, b1_limb_1_col170),
            eval!(context, b1_limb_2_col171),
            eval!(context, b1_limb_3_col172),
            eval!(context, b1_limb_4_col173),
            eval!(context, b1_limb_5_col174),
            eval!(context, b1_limb_6_col175),
            eval!(context, b1_limb_7_col176),
            eval!(context, b1_limb_8_col177),
            eval!(context, b1_limb_9_col178),
            eval!(context, b1_limb_10_col179),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_91_limb_0)) + (2)
            ),
            eval!(context, enabler),
            eval!(context, b2_id_col180),
            eval!(context, b2_limb_0_col181),
            eval!(context, b2_limb_1_col182),
            eval!(context, b2_limb_2_col183),
            eval!(context, b2_limb_3_col184),
            eval!(context, b2_limb_4_col185),
            eval!(context, b2_limb_5_col186),
            eval!(context, b2_limb_6_col187),
            eval!(context, b2_limb_7_col188),
            eval!(context, b2_limb_8_col189),
            eval!(context, b2_limb_9_col190),
            eval!(context, b2_limb_10_col191),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_91_limb_0)) + (3)
            ),
            eval!(context, enabler),
            eval!(context, b3_id_col192),
            eval!(context, b3_limb_0_col193),
            eval!(context, b3_limb_1_col194),
            eval!(context, b3_limb_2_col195),
            eval!(context, b3_limb_3_col196),
            eval!(context, b3_limb_4_col197),
            eval!(context, b3_limb_5_col198),
            eval!(context, b3_limb_6_col199),
            eval!(context, b3_limb_7_col200),
            eval!(context, b3_limb_8_col201),
            eval!(context, b3_limb_9_col202),
            eval!(context, b3_limb_10_col203),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(context, (values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_101_limb_0)),
            eval!(context, enabler),
            eval!(context, c0_id_col204),
            eval!(context, c0_limb_0_col205),
            eval!(context, c0_limb_1_col206),
            eval!(context, c0_limb_2_col207),
            eval!(context, c0_limb_3_col208),
            eval!(context, c0_limb_4_col209),
            eval!(context, c0_limb_5_col210),
            eval!(context, c0_limb_6_col211),
            eval!(context, c0_limb_7_col212),
            eval!(context, c0_limb_8_col213),
            eval!(context, c0_limb_9_col214),
            eval!(context, c0_limb_10_col215),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_101_limb_0)) + (1)
            ),
            eval!(context, enabler),
            eval!(context, c1_id_col216),
            eval!(context, c1_limb_0_col217),
            eval!(context, c1_limb_1_col218),
            eval!(context, c1_limb_2_col219),
            eval!(context, c1_limb_3_col220),
            eval!(context, c1_limb_4_col221),
            eval!(context, c1_limb_5_col222),
            eval!(context, c1_limb_6_col223),
            eval!(context, c1_limb_7_col224),
            eval!(context, c1_limb_8_col225),
            eval!(context, c1_limb_9_col226),
            eval!(context, c1_limb_10_col227),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_101_limb_0)) + (2)
            ),
            eval!(context, enabler),
            eval!(context, c2_id_col228),
            eval!(context, c2_limb_0_col229),
            eval!(context, c2_limb_1_col230),
            eval!(context, c2_limb_2_col231),
            eval!(context, c2_limb_3_col232),
            eval!(context, c2_limb_4_col233),
            eval!(context, c2_limb_5_col234),
            eval!(context, c2_limb_6_col235),
            eval!(context, c2_limb_7_col236),
            eval!(context, c2_limb_8_col237),
            eval!(context, c2_limb_9_col238),
            eval!(context, c2_limb_10_col239),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_99::accumulate_constraints(
        &[
            eval!(
                context,
                ((values_ptr_tmp_e0805_102) + (read_small_output_tmp_e0805_101_limb_0)) + (3)
            ),
            eval!(context, enabler),
            eval!(context, c3_id_col240),
            eval!(context, c3_limb_0_col241),
            eval!(context, c3_limb_1_col242),
            eval!(context, c3_limb_2_col243),
            eval!(context, c3_limb_3_col244),
            eval!(context, c3_limb_4_col245),
            eval!(context, c3_limb_5_col246),
            eval!(context, c3_limb_6_col247),
            eval!(context, c3_limb_7_col248),
            eval!(context, c3_limb_8_col249),
            eval!(context, c3_limb_9_col250),
            eval!(context, c3_limb_10_col251),
        ],
        context,
        component_data,
        acc,
    );
    vec![]
}
