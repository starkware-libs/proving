// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "MemoryAddressToId", uses: 2 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [mem_verify_equal_input_address1, mem_verify_equal_input_address2, enabler, id_col0] =
        input.try_into().unwrap();

    read_id::accumulate_constraints(
        &[
            eval!(context, mem_verify_equal_input_address1),
            eval!(context, enabler),
            eval!(context, id_col0),
        ],
        context,
        component_data,
        acc,
    );

    // Use MemoryAddressToId.
    let tuple_1 = &[
        eval!(context, 1444891767),
        eval!(context, mem_verify_equal_input_address2),
        eval!(context, id_col0),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![]
}
