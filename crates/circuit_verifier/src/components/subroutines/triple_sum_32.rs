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
    let [
        triple_sum_32_input_a_limb_0,
        triple_sum_32_input_a_limb_1,
        triple_sum_32_input_b_limb_0,
        triple_sum_32_input_b_limb_1,
        triple_sum_32_input_c_limb_0,
        triple_sum_32_input_c_limb_1,
        enabler,
        triple_sum32_res_limb_0_col0,
        triple_sum32_res_limb_1_col1,
    ] = input.try_into().unwrap();

    verify_triple_sum_32::accumulate_constraints(
        &[
            eval!(context, triple_sum_32_input_a_limb_0),
            eval!(context, triple_sum_32_input_a_limb_1),
            eval!(context, triple_sum_32_input_b_limb_0),
            eval!(context, triple_sum_32_input_b_limb_1),
            eval!(context, triple_sum_32_input_c_limb_0),
            eval!(context, triple_sum_32_input_c_limb_1),
            eval!(context, triple_sum32_res_limb_0_col0),
            eval!(context, triple_sum32_res_limb_1_col1),
            eval!(context, enabler),
        ],
        context,
        component_data,
        acc,
    );
    vec![]
}
