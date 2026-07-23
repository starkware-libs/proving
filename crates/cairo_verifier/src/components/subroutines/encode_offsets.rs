// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "RangeCheck_4_3", uses: 1 },
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
        encode_offsets_input_offset0,
        encode_offsets_input_offset1,
        encode_offsets_input_offset2,
        enabler,
        offset0_low_col0,
        offset0_mid_col1,
        offset1_low_col2,
        offset1_mid_col3,
        offset1_high_col4,
        offset2_low_col5,
        offset2_mid_col6,
        offset2_high_col7,
    ] = input.try_into().unwrap();

    // Reconstructed offset0 is correct.
    let constraint_0_value = eval!(
        context,
        ((offset0_low_col0) + ((offset0_mid_col1) * (512))) - (encode_offsets_input_offset0)
    );
    acc.add_constraint(context, constraint_0_value);

    // Reconstructed offset1 is correct.
    let constraint_1_value = eval!(
        context,
        (((offset1_low_col2) + ((offset1_mid_col3) * (4))) + ((offset1_high_col4) * (2048)))
            - (encode_offsets_input_offset1)
    );
    acc.add_constraint(context, constraint_1_value);

    // Reconstructed offset2 is correct.
    let constraint_2_value = eval!(
        context,
        (((offset2_low_col5) + ((offset2_mid_col6) * (16))) + ((offset2_high_col7) * (8192)))
            - (encode_offsets_input_offset2)
    );
    acc.add_constraint(context, constraint_2_value);

    // Use RangeCheck_7_2_5.
    let tuple_3 = &[
        eval!(context, 371240602),
        eval!(context, offset0_mid_col1),
        eval!(context, offset1_low_col2),
        eval!(context, offset1_high_col4),
    ];
    let numerator_3 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Use RangeCheck_4_3.
    let tuple_4 = &[
        eval!(context, 1567323731),
        eval!(context, offset2_low_col5),
        eval!(context, offset2_high_col7),
    ];
    let numerator_4 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_4, tuple_4);
    vec![
        eval!(context, (offset0_mid_col1) + ((offset1_low_col2) * (128))),
        eval!(context, (offset1_high_col4) + ((offset2_low_col5) * (32))),
    ]
}
