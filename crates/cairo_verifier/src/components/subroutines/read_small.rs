// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        read_small_input,
        enabler,
        id_col0,
        msb_col1,
        mid_limbs_set_col2,
        value_limb_0_col3,
        value_limb_1_col4,
        value_limb_2_col5,
        remainder_bits_col6,
        partial_limb_msb_col7,
    ] = input.try_into().unwrap();

    read_id::accumulate_constraints(
        &[eval!(context, read_small_input), eval!(context, enabler), eval!(context, id_col0)],
        context,
        component_data,
        acc,
    );

    let [
        decode_small_sign_output_tmp_c806d_5_limb3_7_high_bits,
        decode_small_sign_output_tmp_c806d_5_limbs4_to_20,
        decode_small_sign_output_tmp_c806d_5_limb21,
        decode_small_sign_output_tmp_c806d_5_limb27,
    ] = decode_small_sign::accumulate_constraints(
        &[eval!(context, enabler), eval!(context, msb_col1), eval!(context, mid_limbs_set_col2)],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    cond_range_check_2::accumulate_constraints(
        &[
            eval!(context, remainder_bits_col6),
            eval!(context, 1),
            eval!(context, enabler),
            eval!(context, partial_limb_msb_col7),
        ],
        context,
        component_data,
        acc,
    );

    // Use MemoryIdToBig.
    let tuple_3 = &[
        eval!(context, 1662111297),
        eval!(context, id_col0),
        eval!(context, value_limb_0_col3),
        eval!(context, value_limb_1_col4),
        eval!(context, value_limb_2_col5),
        eval!(
            context,
            (remainder_bits_col6) + (decode_small_sign_output_tmp_c806d_5_limb3_7_high_bits)
        ),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limbs4_to_20),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limb21),
        eval!(context, 0),
        eval!(context, 0),
        eval!(context, 0),
        eval!(context, 0),
        eval!(context, 0),
        eval!(context, decode_small_sign_output_tmp_c806d_5_limb27),
    ];
    let numerator_3 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_3, tuple_3);
    vec![eval!(
        context,
        (((((value_limb_0_col3) + ((value_limb_1_col4) * (512)))
            + ((value_limb_2_col5) * (262144)))
            + ((remainder_bits_col6) * (134217728)))
            - (msb_col1))
            - ((536870912) * (mid_limbs_set_col2))
    )]
}
