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
    let [
        mem_cond_verify_equal_known_id_input_limb_0,
        mem_cond_verify_equal_known_id_input_limb_1,
        mem_cond_verify_equal_known_id_input_limb_2,
        enabler,
        id_col0,
    ] = input.try_into().unwrap();

    read_id::accumulate_constraints(
        &[
            eval!(context, mem_cond_verify_equal_known_id_input_limb_0),
            eval!(context, enabler),
            eval!(context, id_col0),
        ],
        context,
        component_data,
        acc,
    );

    // The two ids are equal if the condition is met.
    let constraint_1_value = eval!(
        context,
        ((id_col0) - (mem_cond_verify_equal_known_id_input_limb_1))
            * (mem_cond_verify_equal_known_id_input_limb_2)
    );
    acc.add_constraint(context, constraint_1_value);
    vec![]
}
