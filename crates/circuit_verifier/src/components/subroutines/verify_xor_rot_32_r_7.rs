// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "VerifyBitwiseXor_7", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_9", uses: 2 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        verify_xor_rot_32_r_7_input_limb_0,
        verify_xor_rot_32_r_7_input_limb_1,
        verify_xor_rot_32_r_7_input_limb_2,
        verify_xor_rot_32_r_7_input_limb_3,
        verify_xor_rot_32_r_7_input_limb_4,
        verify_xor_rot_32_r_7_input_limb_5,
        enabler,
        ms_9_bits_col0,
        ms_9_bits_col1,
        ms_9_bits_col2,
        ms_9_bits_col3,
        ms_7_bits_col4,
        ms_7_bits_col5,
    ] = input.try_into().unwrap();

    let [split_16_low_part_size_7_output_tmp_f88f9_1_limb_0] =
        split_16_low_part_size_7::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_7_input_limb_0),
                eval!(context, enabler),
                eval!(context, ms_9_bits_col0),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_7_output_tmp_f88f9_3_limb_0] =
        split_16_low_part_size_7::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_7_input_limb_1),
                eval!(context, enabler),
                eval!(context, ms_9_bits_col1),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_7_output_tmp_f88f9_5_limb_0] =
        split_16_low_part_size_7::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_7_input_limb_2),
                eval!(context, enabler),
                eval!(context, ms_9_bits_col2),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_7_output_tmp_f88f9_7_limb_0] =
        split_16_low_part_size_7::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_7_input_limb_3),
                eval!(context, enabler),
                eval!(context, ms_9_bits_col3),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_9_output_tmp_f88f9_9_limb_0] =
        split_16_low_part_size_9::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_7_input_limb_4),
                eval!(context, enabler),
                eval!(context, ms_7_bits_col4),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_9_output_tmp_f88f9_11_limb_0] =
        split_16_low_part_size_9::accumulate_constraints(
            &[
                eval!(context, verify_xor_rot_32_r_7_input_limb_5),
                eval!(context, enabler),
                eval!(context, ms_7_bits_col5),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    // Use VerifyBitwiseXor_9.
    let tuple_6 = &[
        eval!(context, 95781001),
        eval!(context, ms_9_bits_col0),
        eval!(context, ms_9_bits_col2),
        eval!(context, split_16_low_part_size_9_output_tmp_f88f9_9_limb_0),
    ];
    let numerator_6 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Use VerifyBitwiseXor_7.
    let tuple_7 = &[
        eval!(context, 62225763),
        eval!(context, split_16_low_part_size_7_output_tmp_f88f9_3_limb_0),
        eval!(context, split_16_low_part_size_7_output_tmp_f88f9_7_limb_0),
        eval!(context, ms_7_bits_col4),
    ];
    let numerator_7 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Use VerifyBitwiseXor_9.
    let tuple_8 = &[
        eval!(context, 95781001),
        eval!(context, ms_9_bits_col1),
        eval!(context, ms_9_bits_col3),
        eval!(context, split_16_low_part_size_9_output_tmp_f88f9_11_limb_0),
    ];
    let numerator_8 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_8, tuple_8);

    // Use VerifyBitwiseXor_7.
    let tuple_9 = &[
        eval!(context, 62225763),
        eval!(context, split_16_low_part_size_7_output_tmp_f88f9_1_limb_0),
        eval!(context, split_16_low_part_size_7_output_tmp_f88f9_5_limb_0),
        eval!(context, ms_7_bits_col5),
    ];
    let numerator_9 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_9, tuple_9);
    vec![]
}
