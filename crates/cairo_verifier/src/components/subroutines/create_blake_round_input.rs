// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 8 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 8 },
    RelationUse { relation_id: "RangeCheck_7_2_5", uses: 8 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 4 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        create_blake_round_input_input_limb_0,
        create_blake_round_input_input_limb_1,
        create_blake_round_input_input_limb_2,
        create_blake_round_input_input_limb_3,
        enabler,
        low_16_bits_col0,
        high_16_bits_col1,
        low_7_ms_bits_col2,
        high_14_ms_bits_col3,
        high_5_ms_bits_col4,
        state_0_id_col5,
        low_16_bits_col6,
        high_16_bits_col7,
        low_7_ms_bits_col8,
        high_14_ms_bits_col9,
        high_5_ms_bits_col10,
        state_1_id_col11,
        low_16_bits_col12,
        high_16_bits_col13,
        low_7_ms_bits_col14,
        high_14_ms_bits_col15,
        high_5_ms_bits_col16,
        state_2_id_col17,
        low_16_bits_col18,
        high_16_bits_col19,
        low_7_ms_bits_col20,
        high_14_ms_bits_col21,
        high_5_ms_bits_col22,
        state_3_id_col23,
        low_16_bits_col24,
        high_16_bits_col25,
        low_7_ms_bits_col26,
        high_14_ms_bits_col27,
        high_5_ms_bits_col28,
        state_4_id_col29,
        low_16_bits_col30,
        high_16_bits_col31,
        low_7_ms_bits_col32,
        high_14_ms_bits_col33,
        high_5_ms_bits_col34,
        state_5_id_col35,
        low_16_bits_col36,
        high_16_bits_col37,
        low_7_ms_bits_col38,
        high_14_ms_bits_col39,
        high_5_ms_bits_col40,
        state_6_id_col41,
        low_16_bits_col42,
        high_16_bits_col43,
        low_7_ms_bits_col44,
        high_14_ms_bits_col45,
        high_5_ms_bits_col46,
        state_7_id_col47,
        ms_8_bits_col48,
        ms_8_bits_col49,
        xor_col50,
        xor_col51,
        xor_col52,
        xor_col53,
    ] = input.try_into().unwrap();

    read_u_32::accumulate_constraints(
        &[
            eval!(context, create_blake_round_input_input_limb_0),
            eval!(context, enabler),
            eval!(context, low_16_bits_col0),
            eval!(context, high_16_bits_col1),
            eval!(context, low_7_ms_bits_col2),
            eval!(context, high_14_ms_bits_col3),
            eval!(context, high_5_ms_bits_col4),
            eval!(context, state_0_id_col5),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (create_blake_round_input_input_limb_0) + (1)),
            eval!(context, enabler),
            eval!(context, low_16_bits_col6),
            eval!(context, high_16_bits_col7),
            eval!(context, low_7_ms_bits_col8),
            eval!(context, high_14_ms_bits_col9),
            eval!(context, high_5_ms_bits_col10),
            eval!(context, state_1_id_col11),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (create_blake_round_input_input_limb_0) + (2)),
            eval!(context, enabler),
            eval!(context, low_16_bits_col12),
            eval!(context, high_16_bits_col13),
            eval!(context, low_7_ms_bits_col14),
            eval!(context, high_14_ms_bits_col15),
            eval!(context, high_5_ms_bits_col16),
            eval!(context, state_2_id_col17),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (create_blake_round_input_input_limb_0) + (3)),
            eval!(context, enabler),
            eval!(context, low_16_bits_col18),
            eval!(context, high_16_bits_col19),
            eval!(context, low_7_ms_bits_col20),
            eval!(context, high_14_ms_bits_col21),
            eval!(context, high_5_ms_bits_col22),
            eval!(context, state_3_id_col23),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (create_blake_round_input_input_limb_0) + (4)),
            eval!(context, enabler),
            eval!(context, low_16_bits_col24),
            eval!(context, high_16_bits_col25),
            eval!(context, low_7_ms_bits_col26),
            eval!(context, high_14_ms_bits_col27),
            eval!(context, high_5_ms_bits_col28),
            eval!(context, state_4_id_col29),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (create_blake_round_input_input_limb_0) + (5)),
            eval!(context, enabler),
            eval!(context, low_16_bits_col30),
            eval!(context, high_16_bits_col31),
            eval!(context, low_7_ms_bits_col32),
            eval!(context, high_14_ms_bits_col33),
            eval!(context, high_5_ms_bits_col34),
            eval!(context, state_5_id_col35),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (create_blake_round_input_input_limb_0) + (6)),
            eval!(context, enabler),
            eval!(context, low_16_bits_col36),
            eval!(context, high_16_bits_col37),
            eval!(context, low_7_ms_bits_col38),
            eval!(context, high_14_ms_bits_col39),
            eval!(context, high_5_ms_bits_col40),
            eval!(context, state_6_id_col41),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (create_blake_round_input_input_limb_0) + (7)),
            eval!(context, enabler),
            eval!(context, low_16_bits_col42),
            eval!(context, high_16_bits_col43),
            eval!(context, low_7_ms_bits_col44),
            eval!(context, high_14_ms_bits_col45),
            eval!(context, high_5_ms_bits_col46),
            eval!(context, state_7_id_col47),
        ],
        context,
        component_data,
        acc,
    );

    let [split_16_low_part_size_8_output_tmp_4204d_89_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, create_blake_round_input_input_limb_1),
                eval!(context, enabler),
                eval!(context, ms_8_bits_col48),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4204d_91_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, create_blake_round_input_input_limb_2),
                eval!(context, enabler),
                eval!(context, ms_8_bits_col49),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_8_output_tmp_4204d_89_limb_0),
            eval!(context, 127),
            eval!(context, enabler),
            eval!(context, xor_col50),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, ms_8_bits_col48),
            eval!(context, 82),
            eval!(context, enabler),
            eval!(context, xor_col51),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_8_output_tmp_4204d_91_limb_0),
            eval!(context, 14),
            eval!(context, enabler),
            eval!(context, xor_col52),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, ms_8_bits_col49),
            eval!(context, 81),
            eval!(context, enabler),
            eval!(context, xor_col53),
        ],
        context,
        component_data,
        acc,
    );
    vec![
        eval!(context, (xor_col50) + ((xor_col51) * (256))),
        eval!(context, (xor_col52) + ((xor_col53) * (256))),
        eval!(
            context,
            ((create_blake_round_input_input_limb_3) * (9812))
                + (((1) - (create_blake_round_input_input_limb_3)) * (55723))
        ),
        eval!(
            context,
            ((create_blake_round_input_input_limb_3) * (57468))
                + (((1) - (create_blake_round_input_input_limb_3)) * (8067))
        ),
    ]
}
