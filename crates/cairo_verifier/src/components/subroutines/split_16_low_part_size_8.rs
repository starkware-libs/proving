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
    let [split_16_low_part_size_8_input, enabler, ms_8_bits_col0] = input.try_into().unwrap();
    vec![eval!(context, (split_16_low_part_size_8_input) - ((ms_8_bits_col0) * (256)))]
}
