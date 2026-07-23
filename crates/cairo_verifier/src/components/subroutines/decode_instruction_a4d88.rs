// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyInstruction", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [decode_instruction_a4d88_input_pc, enabler] = input.try_into().unwrap();

    // Use VerifyInstruction.
    let tuple_0 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_a4d88_input_pc),
        eval!(context, 32766),
        eval!(context, 32767),
        eval!(context, 32767),
        eval!(context, 88),
        eval!(context, 130),
    ];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);
    vec![]
}
