// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "MemoryAddressToId", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [read_id_input, enabler, id_col0] = input.try_into().unwrap();

    // Use MemoryAddressToId.
    let tuple_0 =
        &[eval!(context, 1444891767), eval!(context, read_id_input), eval!(context, id_col0)];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);
    vec![]
}
