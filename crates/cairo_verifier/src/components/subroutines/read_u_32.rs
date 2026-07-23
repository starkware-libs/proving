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
        read_u_32_input,
        enabler,
        low_16_bits_col0,
        high_16_bits_col1,
        low_7_ms_bits_col2,
        high_14_ms_bits_col3,
        high_5_ms_bits_col4,
        id_col5,
    ] = input.try_into().unwrap();

    verify_u_32::accumulate_constraints(
        &[
            eval!(context, read_u_32_input),
            eval!(context, low_16_bits_col0),
            eval!(context, high_16_bits_col1),
            eval!(context, enabler),
            eval!(context, low_7_ms_bits_col2),
            eval!(context, high_14_ms_bits_col3),
            eval!(context, high_5_ms_bits_col4),
            eval!(context, id_col5),
        ],
        context,
        component_data,
        acc,
    );
    vec![]
}
