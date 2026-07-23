// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 20;
pub const N_INTERACTION_COLUMNS: usize = 24;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "Gate", uses: 3 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 8 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_a_limb_0_col0,
        input_a_limb_1_col1,
        input_b_limb_0_col2,
        input_b_limb_1_col3,
        input_c_limb_0_col4,
        input_c_limb_1_col5,
        input_a_xor_b_xor_c_limb_0_col6,
        input_a_xor_b_xor_c_limb_1_col7,
        ms_8_bits_col8,
        ms_8_bits_col9,
        ms_8_bits_col10,
        ms_8_bits_col11,
        ms_8_bits_col12,
        ms_8_bits_col13,
        ms_8_bits_col14,
        ms_8_bits_col15,
        xor_col16,
        xor_col17,
        xor_col18,
        xor_col19,
    ] = input.try_into().unwrap();
    let triple_xor_input_addr_0 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "triple_xor_input_addr_0".to_owned(),
    });
    let triple_xor_input_addr_1 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "triple_xor_input_addr_1".to_owned(),
    });
    let triple_xor_input_addr_2 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "triple_xor_input_addr_2".to_owned(),
    });
    let triple_xor_multiplicity = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "triple_xor_multiplicity".to_owned(),
    });
    let triple_xor_output_addr = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "triple_xor_output_addr".to_owned() });

    let [split_16_low_part_size_8_output_tmp_4ec2c_1_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, input_a_limb_0_col0),
                eval!(context, 1),
                eval!(context, ms_8_bits_col8),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4ec2c_3_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, input_a_limb_1_col1),
                eval!(context, 1),
                eval!(context, ms_8_bits_col9),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4ec2c_5_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, input_b_limb_0_col2),
                eval!(context, 1),
                eval!(context, ms_8_bits_col10),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4ec2c_7_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, input_b_limb_1_col3),
                eval!(context, 1),
                eval!(context, ms_8_bits_col11),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4ec2c_9_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, input_c_limb_0_col4),
                eval!(context, 1),
                eval!(context, ms_8_bits_col12),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4ec2c_11_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, input_c_limb_1_col5),
                eval!(context, 1),
                eval!(context, ms_8_bits_col13),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4ec2c_13_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, input_a_xor_b_xor_c_limb_0_col6),
                eval!(context, 1),
                eval!(context, ms_8_bits_col14),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4ec2c_15_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[
                eval!(context, input_a_xor_b_xor_c_limb_1_col7),
                eval!(context, 1),
                eval!(context, ms_8_bits_col15),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_8_output_tmp_4ec2c_1_limb_0),
            eval!(context, split_16_low_part_size_8_output_tmp_4ec2c_5_limb_0),
            eval!(context, 1),
            eval!(context, xor_col16),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, ms_8_bits_col8),
            eval!(context, ms_8_bits_col10),
            eval!(context, 1),
            eval!(context, xor_col17),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_8_output_tmp_4ec2c_3_limb_0),
            eval!(context, split_16_low_part_size_8_output_tmp_4ec2c_7_limb_0),
            eval!(context, 1),
            eval!(context, xor_col18),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, ms_8_bits_col9),
            eval!(context, ms_8_bits_col11),
            eval!(context, 1),
            eval!(context, xor_col19),
        ],
        context,
        component_data,
        acc,
    );

    // Use VerifyBitwiseXor_8.
    let tuple_12 = &[
        eval!(context, 112558620),
        eval!(context, xor_col16),
        eval!(context, split_16_low_part_size_8_output_tmp_4ec2c_9_limb_0),
        eval!(context, split_16_low_part_size_8_output_tmp_4ec2c_13_limb_0),
    ];
    let numerator_12 = eval!(context, 1);
    acc.add_to_relation(context, numerator_12, tuple_12);

    // Use VerifyBitwiseXor_8.
    let tuple_13 = &[
        eval!(context, 112558620),
        eval!(context, xor_col17),
        eval!(context, ms_8_bits_col12),
        eval!(context, ms_8_bits_col14),
    ];
    let numerator_13 = eval!(context, 1);
    acc.add_to_relation(context, numerator_13, tuple_13);

    // Use VerifyBitwiseXor_8.
    let tuple_14 = &[
        eval!(context, 112558620),
        eval!(context, xor_col18),
        eval!(context, split_16_low_part_size_8_output_tmp_4ec2c_11_limb_0),
        eval!(context, split_16_low_part_size_8_output_tmp_4ec2c_15_limb_0),
    ];
    let numerator_14 = eval!(context, 1);
    acc.add_to_relation(context, numerator_14, tuple_14);

    // Use VerifyBitwiseXor_8.
    let tuple_15 = &[
        eval!(context, 112558620),
        eval!(context, xor_col19),
        eval!(context, ms_8_bits_col13),
        eval!(context, ms_8_bits_col15),
    ];
    let numerator_15 = eval!(context, 1);
    acc.add_to_relation(context, numerator_15, tuple_15);

    // Use Gate.
    let tuple_16 = &[
        eval!(context, 378353459),
        eval!(context, triple_xor_input_addr_0),
        eval!(context, input_a_limb_0_col0),
        eval!(context, input_a_limb_1_col1),
    ];
    let numerator_16 = eval!(context, 1);
    acc.add_to_relation(context, numerator_16, tuple_16);

    // Use Gate.
    let tuple_17 = &[
        eval!(context, 378353459),
        eval!(context, triple_xor_input_addr_1),
        eval!(context, input_b_limb_0_col2),
        eval!(context, input_b_limb_1_col3),
    ];
    let numerator_17 = eval!(context, 1);
    acc.add_to_relation(context, numerator_17, tuple_17);

    // Use Gate.
    let tuple_18 = &[
        eval!(context, 378353459),
        eval!(context, triple_xor_input_addr_2),
        eval!(context, input_c_limb_0_col4),
        eval!(context, input_c_limb_1_col5),
    ];
    let numerator_18 = eval!(context, 1);
    acc.add_to_relation(context, numerator_18, tuple_18);

    // Yield Gate.
    let tuple_19 = &[
        eval!(context, 378353459),
        eval!(context, triple_xor_output_addr),
        eval!(context, input_a_xor_b_xor_c_limb_0_col6),
        eval!(context, input_a_xor_b_xor_c_limb_1_col7),
    ];
    let numerator_19 = eval!(context, -(triple_xor_multiplicity));
    acc.add_to_relation(context, numerator_19, tuple_19);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "triple_xor".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &RELATION_USES_PER_ROW
    }

    fn log_size(
        &self,
        preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        preprocessed_column_log_sizes
            .get(&PreProcessedColumnId { id: "triple_xor_input_addr_0".to_string() })
            .cloned()
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use circuits::context::Context;
    use circuits::ivalue::qm31_from_u32s;
    use circuits_stark_verifier::constraint_eval::*;
    use circuits_stark_verifier::test_utils::TestComponentData;
    use stwo::core::fields::qm31::QM31;

    use super::Component;
    #[allow(unused_imports)]
    use crate::components::prelude::PreProcessedColumnId;
    use crate::sample_evaluations::*;

    #[test]
    fn test_evaluation_result() {
        let component = Component {};
        let mut context: Context<QM31> = Default::default();
        context.enable_assert_eq_on_eval();
        let trace_columns = [
            qm31_from_u32s(1659099300, 905558730, 651199673, 1375009625),
            qm31_from_u32s(1591990121, 771341002, 584090809, 1375009625),
            qm31_from_u32s(1793317658, 1173994186, 785417401, 1375009625),
            qm31_from_u32s(1726208479, 1039776458, 718308537, 1375009625),
            qm31_from_u32s(1390662584, 368687818, 382764217, 1375009625),
            qm31_from_u32s(1323553405, 234470090, 315655353, 1375009625),
            qm31_from_u32s(1524880942, 637123274, 516981945, 1375009625),
            qm31_from_u32s(1457771763, 502905546, 449873081, 1375009625),
            qm31_from_u32s(48489085, 1979300555, 1188070585, 1375009625),
            qm31_from_u32s(2128863553, 1845082826, 1120961721, 1375009625),
            qm31_from_u32s(1852335767, 645078115, 2059236183, 343880121),
            qm31_from_u32s(1919444946, 779295843, 2126345047, 343880121),
            qm31_from_u32s(1986554125, 913513571, 45970264, 343880122),
            qm31_from_u32s(2053663304, 1047731299, 113079128, 343880122),
            qm31_from_u32s(1583899051, 108207203, 1790800727, 343880121),
            qm31_from_u32s(1651008230, 242424931, 1857909591, 343880121),
            qm31_from_u32s(1718117409, 376642659, 1925018455, 343880121),
            qm31_from_u32s(1785226588, 510860387, 1992127319, 343880121),
            qm31_from_u32s(1315462335, 1718819938, 1522365270, 343880121),
            qm31_from_u32s(1382571514, 1853037666, 1589474134, 343880121),
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
            qm31_from_u32s(1206495569, 482634180, 2049214693, 1941984119),
            qm31_from_u32s(736731316, 1690593731, 1579452644, 1941984119),
            qm31_from_u32s(803840495, 1824811459, 1646561508, 1941984119),
        ];
        let component_data = TestComponentData::from_values(
            &mut context,
            &trace_columns,
            &interaction_columns,
            qm31_from_u32s(1115374022, 1127856551, 489657863, 643630026),
            32768,
        );
        let random_coeff =
            context.new_var(qm31_from_u32s(474642921, 876336632, 1911695779, 974600512));
        let interaction_elements = [
            context.new_var(qm31_from_u32s(445623802, 202571636, 1360224996, 131355117)),
            context.new_var(qm31_from_u32s(476823935, 939223384, 62486082, 122423602)),
        ];
        let preprocessed_columns = HashMap::from([
            (
                PreProcessedColumnId { id: "triple_xor_input_addr_0".to_owned() },
                context.constant(qm31_from_u32s(609298445, 1319370969, 1526988810, 301130926)),
            ),
            (
                PreProcessedColumnId { id: "triple_xor_input_addr_1".to_owned() },
                context.constant(qm31_from_u32s(542189266, 1185153241, 1459879946, 301130926)),
            ),
            (
                PreProcessedColumnId { id: "triple_xor_input_addr_2".to_owned() },
                context.constant(qm31_from_u32s(475080087, 1050935513, 1392771082, 301130926)),
            ),
            (
                PreProcessedColumnId { id: "triple_xor_output_addr".to_owned() },
                context.constant(qm31_from_u32s(2078058264, 1287289382, 1925271066, 560922030)),
            ),
            (
                PreProcessedColumnId { id: "triple_xor_multiplicity".to_owned() },
                context.constant(qm31_from_u32s(576605629, 937297661, 250894038, 1499736593)),
            ),
        ]);
        let public_params = HashMap::from([]);
        let mut accumulator = CompositionConstraintAccumulator::new(
            &mut context,
            preprocessed_columns,
            public_params,
            random_coeff,
            interaction_elements,
        );
        component.evaluate(&mut context, &component_data, &mut accumulator);
        let claimed_sum =
            context.new_var(qm31_from_u32s(1398335417, 314974026, 1722107152, 821933968));
        accumulator.finalize_logup_in_pairs(
            &mut context,
            <TestComponentData as ComponentDataTrait<QM31>>::interaction_columns(&component_data),
            &component_data,
            claimed_sum,
        );

        let result = accumulator.finalize();
        let result_value = context.get(result);
        assert_eq!(result_value, TRIPLE_XOR_SAMPLE_EVAL_RESULT)
    }
}
