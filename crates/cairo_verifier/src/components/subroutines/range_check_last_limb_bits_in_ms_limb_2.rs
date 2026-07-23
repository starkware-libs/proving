// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [range_check_last_limb_bits_in_ms_limb_2_input, enabler, partial_limb_msb_col0] =
        input.try_into().unwrap();

    cond_range_check_2::accumulate_constraints(
        &[
            eval!(context, range_check_last_limb_bits_in_ms_limb_2_input),
            eval!(context, 1),
            eval!(context, enabler),
            eval!(context, partial_limb_msb_col0),
        ],
        context,
        component_data,
        acc,
    );
    vec![]
}
