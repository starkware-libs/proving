// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 267;
pub const N_INTERACTION_COLUMNS: usize = 108;

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
) {
    let [
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
        sub_p_bit_col252,
        carry_0_col253,
        carry_1_col254,
        carry_2_col255,
        carry_3_col256,
        carry_4_col257,
        carry_5_col258,
        carry_6_col259,
        carry_7_col260,
        carry_8_col261,
        carry_9_col262,
        carry_10_col263,
        carry_11_col264,
        carry_12_col265,
        carry_13_col266,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, &acc.preprocessed_columns);
    let add_mod_builtin_segment_start =
        *acc.public_params.get("add_mod_builtin_segment_start").unwrap();

    mod_utils::accumulate_constraints(
        &[
            eval!(context, add_mod_builtin_segment_start),
            eval!(context, seq),
            eval!(context, 1),
            eval!(context, is_instance_0_col0),
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
            eval!(context, values_ptr_id_col49),
            eval!(context, values_ptr_limb_0_col50),
            eval!(context, values_ptr_limb_1_col51),
            eval!(context, values_ptr_limb_2_col52),
            eval!(context, values_ptr_limb_3_col53),
            eval!(context, partial_limb_msb_col54),
            eval!(context, offsets_ptr_id_col55),
            eval!(context, offsets_ptr_limb_0_col56),
            eval!(context, offsets_ptr_limb_1_col57),
            eval!(context, offsets_ptr_limb_2_col58),
            eval!(context, offsets_ptr_limb_3_col59),
            eval!(context, partial_limb_msb_col60),
            eval!(context, offsets_ptr_prev_id_col61),
            eval!(context, offsets_ptr_prev_limb_0_col62),
            eval!(context, offsets_ptr_prev_limb_1_col63),
            eval!(context, offsets_ptr_prev_limb_2_col64),
            eval!(context, offsets_ptr_prev_limb_3_col65),
            eval!(context, partial_limb_msb_col66),
            eval!(context, n_id_col67),
            eval!(context, n_limb_0_col68),
            eval!(context, n_limb_1_col69),
            eval!(context, n_limb_2_col70),
            eval!(context, n_limb_3_col71),
            eval!(context, partial_limb_msb_col72),
            eval!(context, n_prev_id_col73),
            eval!(context, n_prev_limb_0_col74),
            eval!(context, n_prev_limb_1_col75),
            eval!(context, n_prev_limb_2_col76),
            eval!(context, n_prev_limb_3_col77),
            eval!(context, partial_limb_msb_col78),
            eval!(context, values_ptr_prev_id_col79),
            eval!(context, p_prev0_id_col80),
            eval!(context, p_prev1_id_col81),
            eval!(context, p_prev2_id_col82),
            eval!(context, p_prev3_id_col83),
            eval!(context, offsets_a_id_col84),
            eval!(context, msb_col85),
            eval!(context, mid_limbs_set_col86),
            eval!(context, offsets_a_limb_0_col87),
            eval!(context, offsets_a_limb_1_col88),
            eval!(context, offsets_a_limb_2_col89),
            eval!(context, remainder_bits_col90),
            eval!(context, partial_limb_msb_col91),
            eval!(context, offsets_b_id_col92),
            eval!(context, msb_col93),
            eval!(context, mid_limbs_set_col94),
            eval!(context, offsets_b_limb_0_col95),
            eval!(context, offsets_b_limb_1_col96),
            eval!(context, offsets_b_limb_2_col97),
            eval!(context, remainder_bits_col98),
            eval!(context, partial_limb_msb_col99),
            eval!(context, offsets_c_id_col100),
            eval!(context, msb_col101),
            eval!(context, mid_limbs_set_col102),
            eval!(context, offsets_c_limb_0_col103),
            eval!(context, offsets_c_limb_1_col104),
            eval!(context, offsets_c_limb_2_col105),
            eval!(context, remainder_bits_col106),
            eval!(context, partial_limb_msb_col107),
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

    // make sure sub_p_bit is 0 or 1..
    let constraint_1_value = eval!(context, ((sub_p_bit_col252) - (1)) * (sub_p_bit_col252));
    acc.add_constraint(context, constraint_1_value);

    // carry_0.
    let constraint_2_value = eval!(
        context,
        (carry_0_col253)
            - (((((((a0_limb_0_col109) + (b0_limb_0_col157)) - (c0_limb_0_col205))
                - ((p0_limb_0_col2) * (sub_p_bit_col252)))
                + ((512)
                    * ((((a0_limb_1_col110) + (b0_limb_1_col158)) - (c0_limb_1_col206))
                        - ((p0_limb_1_col3) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a0_limb_2_col111) + (b0_limb_2_col159)) - (c0_limb_2_col207))
                        - ((p0_limb_2_col4) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_2_value);

    // carry is 0 or 1 or -1..
    let constraint_3_value =
        eval!(context, (carry_0_col253) * (((carry_0_col253) * (carry_0_col253)) - (1)));
    acc.add_constraint(context, constraint_3_value);

    // carry_1.
    let constraint_4_value = eval!(
        context,
        (carry_1_col254)
            - (((((carry_0_col253)
                + ((((a0_limb_3_col112) + (b0_limb_3_col160)) - (c0_limb_3_col208))
                    - ((p0_limb_3_col5) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a0_limb_4_col113) + (b0_limb_4_col161)) - (c0_limb_4_col209))
                        - ((p0_limb_4_col6) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a0_limb_5_col114) + (b0_limb_5_col162)) - (c0_limb_5_col210))
                        - ((p0_limb_5_col7) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_4_value);

    // carry is 0 or 1 or -1..
    let constraint_5_value =
        eval!(context, (carry_1_col254) * (((carry_1_col254) * (carry_1_col254)) - (1)));
    acc.add_constraint(context, constraint_5_value);

    // carry_2.
    let constraint_6_value = eval!(
        context,
        (carry_2_col255)
            - (((((carry_1_col254)
                + ((((a0_limb_6_col115) + (b0_limb_6_col163)) - (c0_limb_6_col211))
                    - ((p0_limb_6_col8) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a0_limb_7_col116) + (b0_limb_7_col164)) - (c0_limb_7_col212))
                        - ((p0_limb_7_col9) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a0_limb_8_col117) + (b0_limb_8_col165)) - (c0_limb_8_col213))
                        - ((p0_limb_8_col10) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_6_value);

    // carry is 0 or 1 or -1..
    let constraint_7_value =
        eval!(context, (carry_2_col255) * (((carry_2_col255) * (carry_2_col255)) - (1)));
    acc.add_constraint(context, constraint_7_value);

    // carry_3.
    let constraint_8_value = eval!(
        context,
        (carry_3_col256)
            - (((((carry_2_col255)
                + ((((a0_limb_9_col118) + (b0_limb_9_col166)) - (c0_limb_9_col214))
                    - ((p0_limb_9_col11) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a0_limb_10_col119) + (b0_limb_10_col167)) - (c0_limb_10_col215))
                        - ((p0_limb_10_col12) * (sub_p_bit_col252)))))
                + ((32768)
                    * ((((a1_limb_0_col121) + (b1_limb_0_col169)) - (c1_limb_0_col217))
                        - ((p1_limb_0_col14) * (sub_p_bit_col252)))))
                * (128))
    );
    acc.add_constraint(context, constraint_8_value);

    // carry is 0 or 1 or -1..
    let constraint_9_value =
        eval!(context, (carry_3_col256) * (((carry_3_col256) * (carry_3_col256)) - (1)));
    acc.add_constraint(context, constraint_9_value);

    // carry_4.
    let constraint_10_value = eval!(
        context,
        (carry_4_col257)
            - (((((carry_3_col256)
                + ((((a1_limb_1_col122) + (b1_limb_1_col170)) - (c1_limb_1_col218))
                    - ((p1_limb_1_col15) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a1_limb_2_col123) + (b1_limb_2_col171)) - (c1_limb_2_col219))
                        - ((p1_limb_2_col16) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a1_limb_3_col124) + (b1_limb_3_col172)) - (c1_limb_3_col220))
                        - ((p1_limb_3_col17) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_10_value);

    // carry is 0 or 1 or -1..
    let constraint_11_value =
        eval!(context, (carry_4_col257) * (((carry_4_col257) * (carry_4_col257)) - (1)));
    acc.add_constraint(context, constraint_11_value);

    // carry_5.
    let constraint_12_value = eval!(
        context,
        (carry_5_col258)
            - (((((carry_4_col257)
                + ((((a1_limb_4_col125) + (b1_limb_4_col173)) - (c1_limb_4_col221))
                    - ((p1_limb_4_col18) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a1_limb_5_col126) + (b1_limb_5_col174)) - (c1_limb_5_col222))
                        - ((p1_limb_5_col19) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a1_limb_6_col127) + (b1_limb_6_col175)) - (c1_limb_6_col223))
                        - ((p1_limb_6_col20) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_12_value);

    // carry is 0 or 1 or -1..
    let constraint_13_value =
        eval!(context, (carry_5_col258) * (((carry_5_col258) * (carry_5_col258)) - (1)));
    acc.add_constraint(context, constraint_13_value);

    // carry_6.
    let constraint_14_value = eval!(
        context,
        (carry_6_col259)
            - (((((carry_5_col258)
                + ((((a1_limb_7_col128) + (b1_limb_7_col176)) - (c1_limb_7_col224))
                    - ((p1_limb_7_col21) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a1_limb_8_col129) + (b1_limb_8_col177)) - (c1_limb_8_col225))
                        - ((p1_limb_8_col22) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a1_limb_9_col130) + (b1_limb_9_col178)) - (c1_limb_9_col226))
                        - ((p1_limb_9_col23) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_14_value);

    // carry is 0 or 1 or -1..
    let constraint_15_value =
        eval!(context, (carry_6_col259) * (((carry_6_col259) * (carry_6_col259)) - (1)));
    acc.add_constraint(context, constraint_15_value);

    // carry_7.
    let constraint_16_value = eval!(
        context,
        (carry_7_col260)
            - (((((carry_6_col259)
                + ((((a1_limb_10_col131) + (b1_limb_10_col179)) - (c1_limb_10_col227))
                    - ((p1_limb_10_col24) * (sub_p_bit_col252))))
                + ((64)
                    * ((((a2_limb_0_col133) + (b2_limb_0_col181)) - (c2_limb_0_col229))
                        - ((p2_limb_0_col26) * (sub_p_bit_col252)))))
                + ((32768)
                    * ((((a2_limb_1_col134) + (b2_limb_1_col182)) - (c2_limb_1_col230))
                        - ((p2_limb_1_col27) * (sub_p_bit_col252)))))
                * (128))
    );
    acc.add_constraint(context, constraint_16_value);

    // carry is 0 or 1 or -1..
    let constraint_17_value =
        eval!(context, (carry_7_col260) * (((carry_7_col260) * (carry_7_col260)) - (1)));
    acc.add_constraint(context, constraint_17_value);

    // carry_8.
    let constraint_18_value = eval!(
        context,
        (carry_8_col261)
            - (((((carry_7_col260)
                + ((((a2_limb_2_col135) + (b2_limb_2_col183)) - (c2_limb_2_col231))
                    - ((p2_limb_2_col28) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a2_limb_3_col136) + (b2_limb_3_col184)) - (c2_limb_3_col232))
                        - ((p2_limb_3_col29) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a2_limb_4_col137) + (b2_limb_4_col185)) - (c2_limb_4_col233))
                        - ((p2_limb_4_col30) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_18_value);

    // carry is 0 or 1 or -1..
    let constraint_19_value =
        eval!(context, (carry_8_col261) * (((carry_8_col261) * (carry_8_col261)) - (1)));
    acc.add_constraint(context, constraint_19_value);

    // carry_9.
    let constraint_20_value = eval!(
        context,
        (carry_9_col262)
            - (((((carry_8_col261)
                + ((((a2_limb_5_col138) + (b2_limb_5_col186)) - (c2_limb_5_col234))
                    - ((p2_limb_5_col31) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a2_limb_6_col139) + (b2_limb_6_col187)) - (c2_limb_6_col235))
                        - ((p2_limb_6_col32) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a2_limb_7_col140) + (b2_limb_7_col188)) - (c2_limb_7_col236))
                        - ((p2_limb_7_col33) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_20_value);

    // carry is 0 or 1 or -1..
    let constraint_21_value =
        eval!(context, (carry_9_col262) * (((carry_9_col262) * (carry_9_col262)) - (1)));
    acc.add_constraint(context, constraint_21_value);

    // carry_10.
    let constraint_22_value = eval!(
        context,
        (carry_10_col263)
            - (((((carry_9_col262)
                + ((((a2_limb_8_col141) + (b2_limb_8_col189)) - (c2_limb_8_col237))
                    - ((p2_limb_8_col34) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a2_limb_9_col142) + (b2_limb_9_col190)) - (c2_limb_9_col238))
                        - ((p2_limb_9_col35) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a2_limb_10_col143) + (b2_limb_10_col191)) - (c2_limb_10_col239))
                        - ((p2_limb_10_col36) * (sub_p_bit_col252)))))
                * (128))
    );
    acc.add_constraint(context, constraint_22_value);

    // carry is 0 or 1 or -1..
    let constraint_23_value =
        eval!(context, (carry_10_col263) * (((carry_10_col263) * (carry_10_col263)) - (1)));
    acc.add_constraint(context, constraint_23_value);

    // carry_11.
    let constraint_24_value = eval!(
        context,
        (carry_11_col264)
            - (((((carry_10_col263)
                + ((((a3_limb_0_col145) + (b3_limb_0_col193)) - (c3_limb_0_col241))
                    - ((p3_limb_0_col38) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a3_limb_1_col146) + (b3_limb_1_col194)) - (c3_limb_1_col242))
                        - ((p3_limb_1_col39) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a3_limb_2_col147) + (b3_limb_2_col195)) - (c3_limb_2_col243))
                        - ((p3_limb_2_col40) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_24_value);

    // carry is 0 or 1 or -1..
    let constraint_25_value =
        eval!(context, (carry_11_col264) * (((carry_11_col264) * (carry_11_col264)) - (1)));
    acc.add_constraint(context, constraint_25_value);

    // carry_12.
    let constraint_26_value = eval!(
        context,
        (carry_12_col265)
            - (((((carry_11_col264)
                + ((((a3_limb_3_col148) + (b3_limb_3_col196)) - (c3_limb_3_col244))
                    - ((p3_limb_3_col41) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a3_limb_4_col149) + (b3_limb_4_col197)) - (c3_limb_4_col245))
                        - ((p3_limb_4_col42) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a3_limb_5_col150) + (b3_limb_5_col198)) - (c3_limb_5_col246))
                        - ((p3_limb_5_col43) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_26_value);

    // carry is 0 or 1 or -1..
    let constraint_27_value =
        eval!(context, (carry_12_col265) * (((carry_12_col265) * (carry_12_col265)) - (1)));
    acc.add_constraint(context, constraint_27_value);

    // carry_13.
    let constraint_28_value = eval!(
        context,
        (carry_13_col266)
            - (((((carry_12_col265)
                + ((((a3_limb_6_col151) + (b3_limb_6_col199)) - (c3_limb_6_col247))
                    - ((p3_limb_6_col44) * (sub_p_bit_col252))))
                + ((512)
                    * ((((a3_limb_7_col152) + (b3_limb_7_col200)) - (c3_limb_7_col248))
                        - ((p3_limb_7_col45) * (sub_p_bit_col252)))))
                + ((262144)
                    * ((((a3_limb_8_col153) + (b3_limb_8_col201)) - (c3_limb_8_col249))
                        - ((p3_limb_8_col46) * (sub_p_bit_col252)))))
                * (16))
    );
    acc.add_constraint(context, constraint_28_value);

    // carry is 0 or 1 or -1..
    let constraint_29_value =
        eval!(context, (carry_13_col266) * (((carry_13_col266) * (carry_13_col266)) - (1)));
    acc.add_constraint(context, constraint_29_value);

    // last carry needs to be 0..
    let constraint_30_value = eval!(
        context,
        ((carry_13_col266)
            + ((((a3_limb_9_col154) + (b3_limb_9_col202)) - (c3_limb_9_col250))
                - ((p3_limb_9_col47) * (sub_p_bit_col252))))
            + ((512)
                * ((((a3_limb_10_col155) + (b3_limb_10_col203)) - (c3_limb_10_col251))
                    - ((p3_limb_10_col48) * (sub_p_bit_col252))))
    );
    acc.add_constraint(context, constraint_30_value);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "add_mod_builtin".to_string()
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
            qm31_from_u32s(1651889461, 242425459, 1857910119, 343880297),
            qm31_from_u32s(779367739, 645078582, 985494826, 343880277),
            qm31_from_u32s(846476918, 779296310, 1052603690, 343880277),
            qm31_from_u32s(913586097, 913514038, 1119712554, 343880277),
            qm31_from_u32s(980695276, 1047731766, 1186821418, 343880277),
            qm31_from_u32s(510931023, 108207670, 717059370, 343880277),
            qm31_from_u32s(578040202, 242425398, 784168234, 343880277),
            qm31_from_u32s(645149381, 376643126, 851277098, 343880277),
            qm31_from_u32s(712258560, 510860854, 918385962, 343880277),
            qm31_from_u32s(1316241171, 1718820406, 1522365738, 343880277),
            qm31_from_u32s(1383350350, 1853038134, 1589474602, 343880277),
            qm31_from_u32s(1340598866, 536394231, 1198633759, 502514173),
            qm31_from_u32s(1407708045, 670611959, 1265742623, 502514173),
            qm31_from_u32s(1474817224, 804829687, 1332851487, 502514173),
            qm31_from_u32s(1541926403, 939047415, 1399960351, 502514173),
            qm31_from_u32s(1072162150, 2147006966, 930198302, 502514173),
            qm31_from_u32s(1139271329, 133741047, 997307167, 502514173),
            qm31_from_u32s(1206380508, 267958775, 1064416031, 502514173),
            qm31_from_u32s(1273489687, 402176503, 1131524895, 502514173),
            qm31_from_u32s(1877472298, 1610136055, 1735504671, 502514173),
            qm31_from_u32s(1944581477, 1744353783, 1802613535, 502514173),
            qm31_from_u32s(669619552, 1341700661, 527545181, 502514194),
            qm31_from_u32s(602510373, 1207482933, 460436317, 502514194),
            qm31_from_u32s(535401194, 1073265205, 393327453, 502514194),
            qm31_from_u32s(468292015, 939047477, 326218589, 502514194),
            qm31_from_u32s(401182836, 804829749, 259109725, 502514194),
            qm31_from_u32s(334073657, 670612021, 192000861, 502514194),
            qm31_from_u32s(266964478, 536394293, 124891997, 502514194),
            qm31_from_u32s(199855299, 402176565, 57783133, 502514194),
            qm31_from_u32s(132746120, 267958837, 2138157916, 502514193),
            qm31_from_u32s(65636941, 133741109, 2071049052, 502514193),
            qm31_from_u32s(2146113804, 2147007087, 2003940247, 502514213),
            qm31_from_u32s(65739336, 133741169, 2071049112, 502514213),
            qm31_from_u32s(2011895446, 1878571631, 1869722519, 502514213),
            qm31_from_u32s(2079004625, 2012789359, 1936831383, 502514213),
            qm31_from_u32s(267066873, 536394353, 124892057, 502514214),
            qm31_from_u32s(334176052, 670612081, 192000921, 502514214),
            qm31_from_u32s(132848515, 267958897, 2138157976, 502514213),
            qm31_from_u32s(199957694, 402176625, 57783193, 502514214),
            qm31_from_u32s(1609240372, 1073265263, 1467069335, 502514213),
            qm31_from_u32s(1676349551, 1207482991, 1534178199, 502514213),
            qm31_from_u32s(1475124409, 804829867, 1332851667, 502514233),
            qm31_from_u32s(1408015230, 670612139, 1265742803, 502514233),
            qm31_from_u32s(1609342767, 1073265323, 1467069395, 502514233),
            qm31_from_u32s(1542233588, 939047595, 1399960531, 502514233),
            qm31_from_u32s(1206687693, 267958955, 1064416211, 502514233),
            qm31_from_u32s(1139578514, 133741227, 997307347, 502514233),
            qm31_from_u32s(1340906051, 536394411, 1198633939, 502514233),
            qm31_from_u32s(1273796872, 402176683, 1131525075, 502514233),
            qm31_from_u32s(2011997841, 1878571691, 1869722579, 502514233),
            qm31_from_u32s(1944888662, 1744353963, 1802613715, 502514233),
            qm31_from_u32s(1877062718, 1610135815, 1735504431, 502514093),
            qm31_from_u32s(1944171897, 1744353543, 1802613295, 502514093),
            qm31_from_u32s(2011281076, 1878571271, 1869722159, 502514093),
            qm31_from_u32s(2078390255, 2012788999, 1936831023, 502514093),
            qm31_from_u32s(2145499434, 2147006727, 2003939887, 502514093),
            qm31_from_u32s(65124966, 133740809, 2071048752, 502514093),
            qm31_from_u32s(132234145, 267958537, 2138157616, 502514093),
            qm31_from_u32s(199343324, 402176265, 57782833, 502514094),
            qm31_from_u32s(1340189286, 536393991, 1198633519, 502514093),
            qm31_from_u32s(1407298465, 670611719, 1265742383, 502514093),
            qm31_from_u32s(1206073323, 267958595, 1064415851, 502514113),
            qm31_from_u32s(1138964144, 133740867, 997306987, 502514113),
            qm31_from_u32s(1071854965, 2147006786, 930198122, 502514113),
            qm31_from_u32s(1004745786, 2012789058, 863089258, 502514113),
            qm31_from_u32s(1474510039, 804829507, 1332851307, 502514113),
            qm31_from_u32s(1407400860, 670611779, 1265742443, 502514113),
            qm31_from_u32s(1340291681, 536394051, 1198633579, 502514113),
            qm31_from_u32s(1273182502, 402176323, 1131524715, 502514113),
            qm31_from_u32s(1742946755, 1341700419, 1601286763, 502514113),
            qm31_from_u32s(1675837576, 1207482691, 1534177899, 502514113),
            qm31_from_u32s(535094009, 1073265025, 393327273, 502514134),
            qm31_from_u32s(602203188, 1207482753, 460436137, 502514134),
            qm31_from_u32s(400875651, 804829569, 259109545, 502514134),
            qm31_from_u32s(467984830, 939047297, 326218409, 502514134),
            qm31_from_u32s(266657293, 536394113, 124891817, 502514134),
            qm31_from_u32s(333766472, 670611841, 192000681, 502514134),
            qm31_from_u32s(132438935, 267958657, 2138157736, 502514133),
            qm31_from_u32s(199548114, 402176385, 57782953, 502514134),
            qm31_from_u32s(2145704224, 2147006847, 2003940007, 502514133),
            qm31_from_u32s(65329756, 133740929, 2071048872, 502514133),
            qm31_from_u32s(2011588261, 1878571451, 1869722339, 502514153),
            qm31_from_u32s(1944479082, 1744353723, 1802613475, 502514153),
            qm31_from_u32s(2145806619, 2147006907, 2003940067, 502514153),
            qm31_from_u32s(2078697440, 2012789179, 1936831203, 502514153),
            qm31_from_u32s(132541330, 267958717, 2138157796, 502514153),
            qm31_from_u32s(65432151, 133740989, 2071048932, 502514153),
            qm31_from_u32s(266759688, 536394173, 124891877, 502514154),
            qm31_from_u32s(199650509, 402176445, 57783013, 502514154),
            qm31_from_u32s(1474714829, 804829627, 1332851427, 502514153),
            qm31_from_u32s(1407605650, 670611899, 1265742563, 502514153),
            qm31_from_u32s(266042923, 536393753, 124891457, 502514014),
            qm31_from_u32s(333152102, 670611481, 192000321, 502514014),
            qm31_from_u32s(400261281, 804829209, 259109185, 502514014),
            qm31_from_u32s(467370460, 939046937, 326218049, 502514014),
            qm31_from_u32s(2145089854, 2147006487, 2003939647, 502514013),
            qm31_from_u32s(64715386, 133740569, 2071048512, 502514013),
            qm31_from_u32s(131824565, 267958297, 2138157376, 502514013),
            qm31_from_u32s(198933744, 402176025, 57782593, 502514014),
            qm31_from_u32s(1876653138, 1610135575, 1735504191, 502514013),
            qm31_from_u32s(1943762317, 1744353303, 1802613055, 502514013),
            qm31_from_u32s(1742537175, 1341700179, 1601286523, 502514033),
            qm31_from_u32s(1675427996, 1207482451, 1534177659, 502514033),
            qm31_from_u32s(1608318817, 1073264723, 1467068795, 502514033),
            qm31_from_u32s(1541209638, 939046995, 1399959931, 502514033),
            qm31_from_u32s(1474100459, 804829267, 1332851067, 502514033),
            qm31_from_u32s(1406991280, 670611539, 1265742203, 502514033),
            qm31_from_u32s(1339882101, 536393811, 1198633339, 502514033),
            qm31_from_u32s(1272772922, 402176083, 1131524475, 502514033),
            qm31_from_u32s(131926960, 267958357, 2138157436, 502514033),
            qm31_from_u32s(64817781, 133740629, 2071048572, 502514033),
            qm31_from_u32s(1491955610, 670690004, 1265820668, 502540188),
            qm31_from_u32s(1424846431, 536472276, 1198711804, 502540188),
            qm31_from_u32s(1357737252, 402254548, 1131602940, 502540188),
            qm31_from_u32s(1290628073, 268036820, 1064494076, 502540188),
            qm31_from_u32s(1223518894, 133819092, 997385212, 502540188),
            qm31_from_u32s(1156409715, 2147085011, 930276347, 502540188),
            qm31_from_u32s(1089300536, 2012867283, 863167483, 502540188),
            qm31_from_u32s(1022191357, 1878649555, 796058619, 502540188),
            qm31_from_u32s(955082178, 1744431827, 728949755, 502540188),
            qm31_from_u32s(887972999, 1610214099, 661840891, 502540188),
            qm31_from_u32s(15491601, 2012867234, 1936909257, 502540171),
            qm31_from_u32s(82600780, 2147084962, 2004018121, 502540171),
            qm31_from_u32s(149709959, 133819043, 2071126986, 502540171),
            qm31_from_u32s(216819138, 268036771, 2138235850, 502540171),
            qm31_from_u32s(1894538532, 1475996321, 1668473801, 502540171),
            qm31_from_u32s(1961647711, 1610214049, 1735582665, 502540171),
            qm31_from_u32s(2028756890, 1744431777, 1802691529, 502540171),
            qm31_from_u32s(2095866069, 1878649505, 1869800393, 502540171),
            qm31_from_u32s(552365033, 939125411, 326296523, 502540172),
            qm31_from_u32s(619474212, 1073343139, 393405387, 502540172),
            qm31_from_u32s(149976820, 133819211, 2071127154, 502540227),
            qm31_from_u32s(82867641, 2147085130, 2004018289, 502540227),
            qm31_from_u32s(284195178, 402254667, 57861235, 502540228),
            qm31_from_u32s(217085999, 268036939, 2138236018, 502540227),
            qm31_from_u32s(2029023751, 1744431945, 1802691697, 502540227),
            qm31_from_u32s(1961914572, 1610214217, 1735582833, 502540227),
            qm31_from_u32s(15758462, 2012867402, 1936909425, 502540227),
            qm31_from_u32s(2096132930, 1878649673, 1869800561, 502540227),
            qm31_from_u32s(686850252, 1207561035, 460514419, 502540228),
            qm31_from_u32s(619741073, 1073343307, 393405555, 502540228),
            qm31_from_u32s(820966215, 1475996431, 594732087, 502540208),
            qm31_from_u32s(888075394, 1610214159, 661840951, 502540208),
            qm31_from_u32s(686747857, 1207560975, 460514359, 502540208),
            qm31_from_u32s(753857036, 1341778703, 527623223, 502540208),
            qm31_from_u32s(1089402931, 2012867343, 863167543, 502540208),
            qm31_from_u32s(1156512110, 2147085071, 930276407, 502540208),
            qm31_from_u32s(955184573, 1744431887, 728949815, 502540208),
            qm31_from_u32s(1022293752, 1878649615, 796058679, 502540208),
            qm31_from_u32s(284092783, 402254607, 57861175, 502540208),
            qm31_from_u32s(351201962, 536472335, 124970039, 502540208),
            qm31_from_u32s(2028449705, 1744431597, 1802691349, 502540111),
            qm31_from_u32s(1961340526, 1610213869, 1735582485, 502540111),
            qm31_from_u32s(1894231347, 1475996141, 1668473621, 502540111),
            qm31_from_u32s(1827122168, 1341778413, 1601364757, 502540111),
            qm31_from_u32s(149402774, 133818863, 2071126806, 502540111),
            qm31_from_u32s(82293595, 2147084782, 2004017941, 502540111),
            qm31_from_u32s(15184416, 2012867054, 1936909077, 502540111),
            qm31_from_u32s(2095558884, 1878649325, 1869800213, 502540111),
            qm31_from_u32s(417839490, 670689775, 192078615, 502540112),
            qm31_from_u32s(350730311, 536472047, 124969751, 502540112),
            qm31_from_u32s(551955453, 939125171, 326296283, 502540092),
            qm31_from_u32s(619064632, 1073342899, 393405147, 502540092),
            qm31_from_u32s(686173811, 1207560627, 460514011, 502540092),
            qm31_from_u32s(753282990, 1341778355, 527622875, 502540092),
            qm31_from_u32s(820392169, 1475996083, 594731739, 502540092),
            qm31_from_u32s(887501348, 1610213811, 661840603, 502540092),
            qm31_from_u32s(954610527, 1744431539, 728949467, 502540092),
            qm31_from_u32s(1021719706, 1878649267, 796058331, 502540092),
            qm31_from_u32s(15082021, 2012866994, 1936909017, 502540091),
            qm31_from_u32s(82191200, 2147084722, 2004017881, 502540091),
            qm31_from_u32s(686480996, 1207560807, 460514191, 502540152),
            qm31_from_u32s(619371817, 1073343079, 393405327, 502540152),
            qm31_from_u32s(820699354, 1475996263, 594731919, 502540152),
            qm31_from_u32s(753590175, 1341778535, 527623055, 502540152),
            qm31_from_u32s(954917712, 1744431719, 728949647, 502540152),
            qm31_from_u32s(887808533, 1610213991, 661840783, 502540152),
            qm31_from_u32s(1089136070, 2012867175, 863167375, 502540152),
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
            qm31_from_u32s(2047663051, 1045625204, 60375153, 1261630211),
            qm31_from_u32s(906909403, 911407535, 1067008171, 1261630230),
            qm31_from_u32s(974018582, 1045625263, 1134117035, 1261630230),
            qm31_from_u32s(772691045, 642972079, 932790443, 1261630230),
            qm31_from_u32s(839800224, 777189807, 999899307, 1261630230),
            qm31_from_u32s(1175346119, 1448278447, 1335443627, 1261630230),
            qm31_from_u32s(1242455298, 1582496175, 1402552491, 1261630230),
            qm31_from_u32s(1041127761, 1179842991, 1201225899, 1261630230),
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
            "add_mod_builtin_segment_start".to_owned(),
            context.constant(1165333106.into()),
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
        assert_eq!(result_value, ADD_MOD_BUILTIN_SAMPLE_EVAL_RESULT)
    }
}
