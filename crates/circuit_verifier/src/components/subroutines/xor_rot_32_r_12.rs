// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "VerifyBitwiseXor_12", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_4", uses: 2 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        xor_rot_32_r_12_input_limb_0,
        xor_rot_32_r_12_input_limb_1,
        xor_rot_32_r_12_input_limb_2,
        xor_rot_32_r_12_input_limb_3,
        enabler,
        ms_4_bits_col0,
        ms_4_bits_col1,
        ms_4_bits_col2,
        ms_4_bits_col3,
        xor_col4,
        xor_col5,
        xor_col6,
        xor_col7,
    ] = input.try_into().unwrap();

    let [split_16_low_part_size_12_output_tmp_b89d6_1_limb_0] =
        split_16_low_part_size_12::accumulate_constraints(
            &[
                eval!(context, xor_rot_32_r_12_input_limb_0),
                eval!(context, enabler),
                eval!(context, ms_4_bits_col0),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_12_output_tmp_b89d6_3_limb_0] =
        split_16_low_part_size_12::accumulate_constraints(
            &[
                eval!(context, xor_rot_32_r_12_input_limb_1),
                eval!(context, enabler),
                eval!(context, ms_4_bits_col1),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_12_output_tmp_b89d6_5_limb_0] =
        split_16_low_part_size_12::accumulate_constraints(
            &[
                eval!(context, xor_rot_32_r_12_input_limb_2),
                eval!(context, enabler),
                eval!(context, ms_4_bits_col2),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_12_output_tmp_b89d6_7_limb_0] =
        split_16_low_part_size_12::accumulate_constraints(
            &[
                eval!(context, xor_rot_32_r_12_input_limb_3),
                eval!(context, enabler),
                eval!(context, ms_4_bits_col3),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    bitwise_xor_num_bits_12::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_12_output_tmp_b89d6_1_limb_0),
            eval!(context, split_16_low_part_size_12_output_tmp_b89d6_5_limb_0),
            eval!(context, enabler),
            eval!(context, xor_col4),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_4::accumulate_constraints(
        &[
            eval!(context, ms_4_bits_col0),
            eval!(context, ms_4_bits_col2),
            eval!(context, enabler),
            eval!(context, xor_col5),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_12::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_12_output_tmp_b89d6_3_limb_0),
            eval!(context, split_16_low_part_size_12_output_tmp_b89d6_7_limb_0),
            eval!(context, enabler),
            eval!(context, xor_col6),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_4::accumulate_constraints(
        &[
            eval!(context, ms_4_bits_col1),
            eval!(context, ms_4_bits_col3),
            eval!(context, enabler),
            eval!(context, xor_col7),
        ],
        context,
        component_data,
        acc,
    );

    let xor_rot_12_output_tmp_b89d6_16_limb_0 = eval!(context, (xor_col5) + ((xor_col6) * (16)));

    let xor_rot_12_output_tmp_b89d6_16_limb_1 = eval!(context, (xor_col7) + ((xor_col4) * (16)));
    vec![
        eval!(context, xor_rot_12_output_tmp_b89d6_16_limb_0),
        eval!(context, xor_rot_12_output_tmp_b89d6_16_limb_1),
    ]
}
