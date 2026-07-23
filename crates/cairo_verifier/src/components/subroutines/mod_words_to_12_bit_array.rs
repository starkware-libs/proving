// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "RangeCheck_3_6_6_3", uses: 5 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        mod_words_to_12_bit_array_input_limb_0,
        mod_words_to_12_bit_array_input_limb_1,
        mod_words_to_12_bit_array_input_limb_2,
        mod_words_to_12_bit_array_input_limb_3,
        mod_words_to_12_bit_array_input_limb_4,
        mod_words_to_12_bit_array_input_limb_5,
        mod_words_to_12_bit_array_input_limb_6,
        mod_words_to_12_bit_array_input_limb_7,
        mod_words_to_12_bit_array_input_limb_8,
        mod_words_to_12_bit_array_input_limb_9,
        mod_words_to_12_bit_array_input_limb_10,
        mod_words_to_12_bit_array_input_limb_28,
        mod_words_to_12_bit_array_input_limb_29,
        mod_words_to_12_bit_array_input_limb_30,
        mod_words_to_12_bit_array_input_limb_31,
        mod_words_to_12_bit_array_input_limb_32,
        mod_words_to_12_bit_array_input_limb_33,
        mod_words_to_12_bit_array_input_limb_34,
        mod_words_to_12_bit_array_input_limb_35,
        mod_words_to_12_bit_array_input_limb_36,
        mod_words_to_12_bit_array_input_limb_37,
        mod_words_to_12_bit_array_input_limb_38,
        enabler,
        limb1b_0_col0,
        limb2b_0_col1,
        limb5b_0_col2,
        limb6b_0_col3,
        limb9b_0_col4,
        limb1b_1_col5,
        limb2b_1_col6,
        limb5b_1_col7,
        limb6b_1_col8,
        limb9b_1_col9,
    ] = input.try_into().unwrap();

    let limb1a_0_tmp_2aff4_1 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_1) - ((limb1b_0_col0) * (8)));

    let limb2a_0_tmp_2aff4_3 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_2) - ((limb2b_0_col1) * (64)));

    // Use RangeCheck_3_6_6_3.
    let tuple_2 = &[
        eval!(context, 1005786011),
        eval!(context, limb1a_0_tmp_2aff4_1),
        eval!(context, limb1b_0_col0),
        eval!(context, limb2a_0_tmp_2aff4_3),
        eval!(context, limb2b_0_col1),
    ];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);

    let limb5a_0_tmp_2aff4_5 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_5) - ((limb5b_0_col2) * (8)));

    let limb6a_0_tmp_2aff4_7 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_6) - ((limb6b_0_col3) * (64)));

    // Use RangeCheck_3_6_6_3.
    let tuple_5 = &[
        eval!(context, 1005786011),
        eval!(context, limb5a_0_tmp_2aff4_5),
        eval!(context, limb5b_0_col2),
        eval!(context, limb6a_0_tmp_2aff4_7),
        eval!(context, limb6b_0_col3),
    ];
    let numerator_5 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_5, tuple_5);

    let limb9a_0_tmp_2aff4_9 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_9) - ((limb9b_0_col4) * (8)));

    let limb1a_1_tmp_2aff4_11 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_29) - ((limb1b_1_col5) * (8)));

    let limb2a_1_tmp_2aff4_13 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_30) - ((limb2b_1_col6) * (64)));

    // Use RangeCheck_3_6_6_3.
    let tuple_9 = &[
        eval!(context, 1005786011),
        eval!(context, limb1a_1_tmp_2aff4_11),
        eval!(context, limb1b_1_col5),
        eval!(context, limb2a_1_tmp_2aff4_13),
        eval!(context, limb2b_1_col6),
    ];
    let numerator_9 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_9, tuple_9);

    let limb5a_1_tmp_2aff4_15 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_33) - ((limb5b_1_col7) * (8)));

    let limb6a_1_tmp_2aff4_17 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_34) - ((limb6b_1_col8) * (64)));

    // Use RangeCheck_3_6_6_3.
    let tuple_12 = &[
        eval!(context, 1005786011),
        eval!(context, limb5a_1_tmp_2aff4_15),
        eval!(context, limb5b_1_col7),
        eval!(context, limb6a_1_tmp_2aff4_17),
        eval!(context, limb6b_1_col8),
    ];
    let numerator_12 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_12, tuple_12);

    let limb9a_1_tmp_2aff4_19 =
        eval!(context, (mod_words_to_12_bit_array_input_limb_37) - ((limb9b_1_col9) * (8)));

    // Use RangeCheck_3_6_6_3.
    let tuple_14 = &[
        eval!(context, 1005786011),
        eval!(context, limb9a_0_tmp_2aff4_9),
        eval!(context, limb9b_0_col4),
        eval!(context, limb9b_1_col9),
        eval!(context, limb9a_1_tmp_2aff4_19),
    ];
    let numerator_14 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_14, tuple_14);
    vec![
        eval!(context, (mod_words_to_12_bit_array_input_limb_0) + ((512) * (limb1a_0_tmp_2aff4_1))),
        eval!(context, (limb1b_0_col0) + ((64) * (limb2a_0_tmp_2aff4_3))),
        eval!(context, (limb2b_0_col1) + ((8) * (mod_words_to_12_bit_array_input_limb_3))),
        eval!(context, (mod_words_to_12_bit_array_input_limb_4) + ((512) * (limb5a_0_tmp_2aff4_5))),
        eval!(context, (limb5b_0_col2) + ((64) * (limb6a_0_tmp_2aff4_7))),
        eval!(context, (limb6b_0_col3) + ((8) * (mod_words_to_12_bit_array_input_limb_7))),
        eval!(context, (mod_words_to_12_bit_array_input_limb_8) + ((512) * (limb9a_0_tmp_2aff4_9))),
        eval!(context, (limb9b_0_col4) + ((64) * (mod_words_to_12_bit_array_input_limb_10))),
        eval!(
            context,
            (mod_words_to_12_bit_array_input_limb_28) + ((512) * (limb1a_1_tmp_2aff4_11))
        ),
        eval!(context, (limb1b_1_col5) + ((64) * (limb2a_1_tmp_2aff4_13))),
        eval!(context, (limb2b_1_col6) + ((8) * (mod_words_to_12_bit_array_input_limb_31))),
        eval!(
            context,
            (mod_words_to_12_bit_array_input_limb_32) + ((512) * (limb5a_1_tmp_2aff4_15))
        ),
        eval!(context, (limb5b_1_col7) + ((64) * (limb6a_1_tmp_2aff4_17))),
        eval!(context, (limb6b_1_col8) + ((8) * (mod_words_to_12_bit_array_input_limb_35))),
        eval!(
            context,
            (mod_words_to_12_bit_array_input_limb_36) + ((512) * (limb9a_1_tmp_2aff4_19))
        ),
        eval!(context, (limb9b_1_col9) + ((64) * (mod_words_to_12_bit_array_input_limb_38))),
    ]
}
