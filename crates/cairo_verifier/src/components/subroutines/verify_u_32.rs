// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 3] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
    RelationUse { relation_id: "RangeCheck_7_2_5", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        verify_u_32_input_limb_0,
        verify_u_32_input_limb_1,
        verify_u_32_input_limb_2,
        enabler,
        low_7_ms_bits_col0,
        high_14_ms_bits_col1,
        high_5_ms_bits_col2,
        id_col3,
    ] = input.try_into().unwrap();

    let high_2_ls_bits_tmp_1a63a_2 =
        eval!(context, (verify_u_32_input_limb_2) - ((high_14_ms_bits_col1) * (4)));

    // Use RangeCheck_7_2_5.
    let tuple_1 = &[
        eval!(context, 371240602),
        eval!(context, low_7_ms_bits_col0),
        eval!(context, high_2_ls_bits_tmp_1a63a_2),
        eval!(context, high_5_ms_bits_col2),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);

    mem_verify::accumulate_constraints(
        &[
            eval!(context, verify_u_32_input_limb_0),
            eval!(context, (verify_u_32_input_limb_1) - ((low_7_ms_bits_col0) * (512))),
            eval!(context, (low_7_ms_bits_col0) + ((high_2_ls_bits_tmp_1a63a_2) * (128))),
            eval!(context, (high_14_ms_bits_col1) - ((high_5_ms_bits_col2) * (512))),
            eval!(context, high_5_ms_bits_col2),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, enabler),
            eval!(context, id_col3),
        ],
        context,
        component_data,
        acc,
    );
    vec![]
}
