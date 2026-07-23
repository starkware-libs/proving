// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyBitwiseXor_4", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        bitwise_xor_num_bits_4_input_limb_0,
        bitwise_xor_num_bits_4_input_limb_1,
        enabler,
        xor_col0,
    ] = input.try_into().unwrap();

    // Use VerifyBitwiseXor_4.
    let tuple_0 = &[
        eval!(context, 45448144),
        eval!(context, bitwise_xor_num_bits_4_input_limb_0),
        eval!(context, bitwise_xor_num_bits_4_input_limb_1),
        eval!(context, xor_col0),
    ];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);
    vec![]
}
