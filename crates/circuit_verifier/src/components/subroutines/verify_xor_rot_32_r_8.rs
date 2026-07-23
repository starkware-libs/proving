// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 4 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        verify_xor_rot_32_r_8_input_limb_0,
        verify_xor_rot_32_r_8_input_limb_1,
        verify_xor_rot_32_r_8_input_limb_2,
        verify_xor_rot_32_r_8_input_limb_3,
        verify_xor_rot_32_r_8_input_limb_4,
        verify_xor_rot_32_r_8_input_limb_5,
        enabler,
        ms_8_bits_col0,
        ms_8_bits_col1,
        ms_8_bits_col2,
        ms_8_bits_col3,
        ms_8_bits_col4,
        ms_8_bits_col5,
    ] = input.try_into().unwrap();

    let [split_16_low_part_size_8_output_tmp_e2dc4_1_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_8_input_limb_0),
                eval!(context, enabler),
                eval!(context, ms_8_bits_col0),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_e2dc4_3_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_8_input_limb_1),
                eval!(context, enabler),
                eval!(context, ms_8_bits_col1),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_e2dc4_5_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_8_input_limb_2),
                eval!(context, enabler),
                eval!(context, ms_8_bits_col2),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_e2dc4_7_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_8_input_limb_3),
                eval!(context, enabler),
                eval!(context, ms_8_bits_col3),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_e2dc4_9_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_8_input_limb_4),
                eval!(context, enabler),
                eval!(context, ms_8_bits_col4),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_e2dc4_11_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_8_input_limb_5),
                eval!(context, enabler),
                eval!(context, ms_8_bits_col5),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    // Use VerifyBitwiseXor_8.
    let tuple_6 = &[
        eval!(context, 112558620),
        eval!(context, ms_8_bits_col0),
        eval!(context, ms_8_bits_col2),
        eval!(context, split_16_low_part_size_8_output_tmp_e2dc4_9_limb_0),
    ];
    let numerator_6 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Use VerifyBitwiseXor_8.
    let tuple_7 = &[
        eval!(context, 112558620),
        eval!(context, split_16_low_part_size_8_output_tmp_e2dc4_3_limb_0),
        eval!(context, split_16_low_part_size_8_output_tmp_e2dc4_7_limb_0),
        eval!(context, ms_8_bits_col4),
    ];
    let numerator_7 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Use VerifyBitwiseXor_8.
    let tuple_8 = &[
        eval!(context, 112558620),
        eval!(context, ms_8_bits_col1),
        eval!(context, ms_8_bits_col3),
        eval!(context, split_16_low_part_size_8_output_tmp_e2dc4_11_limb_0),
    ];
    let numerator_8 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_8, tuple_8);

    // Use VerifyBitwiseXor_8.
    let tuple_9 = &[
        eval!(context, 112558620),
        eval!(context, split_16_low_part_size_8_output_tmp_e2dc4_1_limb_0),
        eval!(context, split_16_low_part_size_8_output_tmp_e2dc4_5_limb_0),
        eval!(context, ms_8_bits_col5),
    ];
    let numerator_9 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_9, tuple_9);
    vec![]
}
