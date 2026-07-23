// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "TripleXor32", uses: 8 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        create_blake_output_input_limb_0,
        create_blake_output_input_limb_1,
        create_blake_output_input_limb_2,
        create_blake_output_input_limb_3,
        create_blake_output_input_limb_4,
        create_blake_output_input_limb_5,
        create_blake_output_input_limb_6,
        create_blake_output_input_limb_7,
        create_blake_output_input_limb_8,
        create_blake_output_input_limb_9,
        create_blake_output_input_limb_10,
        create_blake_output_input_limb_11,
        create_blake_output_input_limb_12,
        create_blake_output_input_limb_13,
        create_blake_output_input_limb_14,
        create_blake_output_input_limb_15,
        create_blake_output_input_limb_16,
        create_blake_output_input_limb_17,
        create_blake_output_input_limb_18,
        create_blake_output_input_limb_19,
        create_blake_output_input_limb_20,
        create_blake_output_input_limb_21,
        create_blake_output_input_limb_22,
        create_blake_output_input_limb_23,
        create_blake_output_input_limb_24,
        create_blake_output_input_limb_25,
        create_blake_output_input_limb_26,
        create_blake_output_input_limb_27,
        create_blake_output_input_limb_28,
        create_blake_output_input_limb_29,
        create_blake_output_input_limb_30,
        create_blake_output_input_limb_31,
        create_blake_output_input_limb_32,
        create_blake_output_input_limb_33,
        create_blake_output_input_limb_34,
        create_blake_output_input_limb_35,
        create_blake_output_input_limb_36,
        create_blake_output_input_limb_37,
        create_blake_output_input_limb_38,
        create_blake_output_input_limb_39,
        create_blake_output_input_limb_40,
        create_blake_output_input_limb_41,
        create_blake_output_input_limb_42,
        create_blake_output_input_limb_43,
        create_blake_output_input_limb_44,
        create_blake_output_input_limb_45,
        create_blake_output_input_limb_46,
        create_blake_output_input_limb_47,
        enabler,
        triple_xor_32_output_limb_0_col0,
        triple_xor_32_output_limb_1_col1,
        triple_xor_32_output_limb_0_col2,
        triple_xor_32_output_limb_1_col3,
        triple_xor_32_output_limb_0_col4,
        triple_xor_32_output_limb_1_col5,
        triple_xor_32_output_limb_0_col6,
        triple_xor_32_output_limb_1_col7,
        triple_xor_32_output_limb_0_col8,
        triple_xor_32_output_limb_1_col9,
        triple_xor_32_output_limb_0_col10,
        triple_xor_32_output_limb_1_col11,
        triple_xor_32_output_limb_0_col12,
        triple_xor_32_output_limb_1_col13,
        triple_xor_32_output_limb_0_col14,
        triple_xor_32_output_limb_1_col15,
    ] = input.try_into().unwrap();

    // Use TripleXor32.
    let tuple_0 = &[
        eval!(context, 990559919),
        eval!(context, create_blake_output_input_limb_16),
        eval!(context, create_blake_output_input_limb_17),
        eval!(context, create_blake_output_input_limb_32),
        eval!(context, create_blake_output_input_limb_33),
        eval!(context, create_blake_output_input_limb_0),
        eval!(context, create_blake_output_input_limb_1),
        eval!(context, triple_xor_32_output_limb_0_col0),
        eval!(context, triple_xor_32_output_limb_1_col1),
    ];
    let numerator_0 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Use TripleXor32.
    let tuple_1 = &[
        eval!(context, 990559919),
        eval!(context, create_blake_output_input_limb_18),
        eval!(context, create_blake_output_input_limb_19),
        eval!(context, create_blake_output_input_limb_34),
        eval!(context, create_blake_output_input_limb_35),
        eval!(context, create_blake_output_input_limb_2),
        eval!(context, create_blake_output_input_limb_3),
        eval!(context, triple_xor_32_output_limb_0_col2),
        eval!(context, triple_xor_32_output_limb_1_col3),
    ];
    let numerator_1 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Use TripleXor32.
    let tuple_2 = &[
        eval!(context, 990559919),
        eval!(context, create_blake_output_input_limb_20),
        eval!(context, create_blake_output_input_limb_21),
        eval!(context, create_blake_output_input_limb_36),
        eval!(context, create_blake_output_input_limb_37),
        eval!(context, create_blake_output_input_limb_4),
        eval!(context, create_blake_output_input_limb_5),
        eval!(context, triple_xor_32_output_limb_0_col4),
        eval!(context, triple_xor_32_output_limb_1_col5),
    ];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Use TripleXor32.
    let tuple_3 = &[
        eval!(context, 990559919),
        eval!(context, create_blake_output_input_limb_22),
        eval!(context, create_blake_output_input_limb_23),
        eval!(context, create_blake_output_input_limb_38),
        eval!(context, create_blake_output_input_limb_39),
        eval!(context, create_blake_output_input_limb_6),
        eval!(context, create_blake_output_input_limb_7),
        eval!(context, triple_xor_32_output_limb_0_col6),
        eval!(context, triple_xor_32_output_limb_1_col7),
    ];
    let numerator_3 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Use TripleXor32.
    let tuple_4 = &[
        eval!(context, 990559919),
        eval!(context, create_blake_output_input_limb_24),
        eval!(context, create_blake_output_input_limb_25),
        eval!(context, create_blake_output_input_limb_40),
        eval!(context, create_blake_output_input_limb_41),
        eval!(context, create_blake_output_input_limb_8),
        eval!(context, create_blake_output_input_limb_9),
        eval!(context, triple_xor_32_output_limb_0_col8),
        eval!(context, triple_xor_32_output_limb_1_col9),
    ];
    let numerator_4 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Use TripleXor32.
    let tuple_5 = &[
        eval!(context, 990559919),
        eval!(context, create_blake_output_input_limb_26),
        eval!(context, create_blake_output_input_limb_27),
        eval!(context, create_blake_output_input_limb_42),
        eval!(context, create_blake_output_input_limb_43),
        eval!(context, create_blake_output_input_limb_10),
        eval!(context, create_blake_output_input_limb_11),
        eval!(context, triple_xor_32_output_limb_0_col10),
        eval!(context, triple_xor_32_output_limb_1_col11),
    ];
    let numerator_5 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Use TripleXor32.
    let tuple_6 = &[
        eval!(context, 990559919),
        eval!(context, create_blake_output_input_limb_28),
        eval!(context, create_blake_output_input_limb_29),
        eval!(context, create_blake_output_input_limb_44),
        eval!(context, create_blake_output_input_limb_45),
        eval!(context, create_blake_output_input_limb_12),
        eval!(context, create_blake_output_input_limb_13),
        eval!(context, triple_xor_32_output_limb_0_col12),
        eval!(context, triple_xor_32_output_limb_1_col13),
    ];
    let numerator_6 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Use TripleXor32.
    let tuple_7 = &[
        eval!(context, 990559919),
        eval!(context, create_blake_output_input_limb_30),
        eval!(context, create_blake_output_input_limb_31),
        eval!(context, create_blake_output_input_limb_46),
        eval!(context, create_blake_output_input_limb_47),
        eval!(context, create_blake_output_input_limb_14),
        eval!(context, create_blake_output_input_limb_15),
        eval!(context, triple_xor_32_output_limb_0_col14),
        eval!(context, triple_xor_32_output_limb_1_col15),
    ];
    let numerator_7 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_7, tuple_7);
    vec![]
}
