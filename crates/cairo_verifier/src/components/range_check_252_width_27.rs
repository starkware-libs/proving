// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 20;
pub const N_INTERACTION_COLUMNS: usize = 32;

pub const RELATION_USES_PER_ROW: [RelationUse; 7] = [
    RelationUse { relation_id: "RangeCheck_18", uses: 7 },
    RelationUse { relation_id: "RangeCheck_18_B", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_B", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_C", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_D", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_E", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        enabler_col0,
        input_limb_0_col1,
        input_limb_1_col2,
        input_limb_2_col3,
        input_limb_3_col4,
        input_limb_4_col5,
        input_limb_5_col6,
        input_limb_6_col7,
        input_limb_7_col8,
        input_limb_8_col9,
        input_limb_9_col10,
        limb_0_high_part_col11,
        limb_1_low_part_col12,
        limb_2_high_part_col13,
        limb_3_low_part_col14,
        limb_4_high_part_col15,
        limb_5_low_part_col16,
        limb_6_high_part_col17,
        limb_7_low_part_col18,
        limb_8_high_part_col19,
    ] = input.try_into().unwrap();

    // Enabler is a bit.
    let constraint_0_value = eval!(context, ((enabler_col0) * (enabler_col0)) - (enabler_col0));
    acc.add_constraint(context, constraint_0_value);

    // Use RangeCheck_9_9.
    let tuple_1 = &[
        eval!(context, 517791011),
        eval!(context, limb_0_high_part_col11),
        eval!(context, limb_1_low_part_col12),
    ];
    let numerator_1 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Use RangeCheck_18.
    let tuple_2 = &[
        eval!(context, 1109051422),
        eval!(context, (input_limb_0_col1) - ((limb_0_high_part_col11) * (262144))),
    ];
    let numerator_2 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Use RangeCheck_18.
    let tuple_3 = &[
        eval!(context, 1109051422),
        eval!(context, ((input_limb_1_col2) - (limb_1_low_part_col12)) * (4194304)),
    ];
    let numerator_3 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Use RangeCheck_9_9_B.
    let tuple_4 = &[
        eval!(context, 1897792095),
        eval!(context, limb_2_high_part_col13),
        eval!(context, limb_3_low_part_col14),
    ];
    let numerator_4 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Use RangeCheck_18_B.
    let tuple_5 = &[
        eval!(context, 1424798916),
        eval!(context, (input_limb_2_col3) - ((limb_2_high_part_col13) * (262144))),
    ];
    let numerator_5 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Use RangeCheck_18.
    let tuple_6 = &[
        eval!(context, 1109051422),
        eval!(context, ((input_limb_3_col4) - (limb_3_low_part_col14)) * (4194304)),
    ];
    let numerator_6 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Use RangeCheck_9_9_C.
    let tuple_7 = &[
        eval!(context, 1881014476),
        eval!(context, limb_4_high_part_col15),
        eval!(context, limb_5_low_part_col16),
    ];
    let numerator_7 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Use RangeCheck_18.
    let tuple_8 = &[
        eval!(context, 1109051422),
        eval!(context, (input_limb_4_col5) - ((limb_4_high_part_col15) * (262144))),
    ];
    let numerator_8 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_8, tuple_8);

    // Use RangeCheck_18.
    let tuple_9 = &[
        eval!(context, 1109051422),
        eval!(context, ((input_limb_5_col6) - (limb_5_low_part_col16)) * (4194304)),
    ];
    let numerator_9 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_9, tuple_9);

    // Use RangeCheck_9_9_D.
    let tuple_10 = &[
        eval!(context, 1864236857),
        eval!(context, limb_6_high_part_col17),
        eval!(context, limb_7_low_part_col18),
    ];
    let numerator_10 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Use RangeCheck_18_B.
    let tuple_11 = &[
        eval!(context, 1424798916),
        eval!(context, (input_limb_6_col7) - ((limb_6_high_part_col17) * (262144))),
    ];
    let numerator_11 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Use RangeCheck_18.
    let tuple_12 = &[
        eval!(context, 1109051422),
        eval!(context, ((input_limb_7_col8) - (limb_7_low_part_col18)) * (4194304)),
    ];
    let numerator_12 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_12, tuple_12);

    // Use RangeCheck_9_9_E.
    let tuple_13 = &[
        eval!(context, 1847459238),
        eval!(context, limb_8_high_part_col19),
        eval!(context, input_limb_9_col10),
    ];
    let numerator_13 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_13, tuple_13);

    // Use RangeCheck_18.
    let tuple_14 = &[
        eval!(context, 1109051422),
        eval!(context, (input_limb_8_col9) - ((limb_8_high_part_col19) * (262144))),
    ];
    let numerator_14 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_14, tuple_14);

    // Yield RangeCheck252Width27.
    let tuple_15 = &[
        eval!(context, 1090315331),
        eval!(context, input_limb_0_col1),
        eval!(context, input_limb_1_col2),
        eval!(context, input_limb_2_col3),
        eval!(context, input_limb_3_col4),
        eval!(context, input_limb_4_col5),
        eval!(context, input_limb_5_col6),
        eval!(context, input_limb_6_col7),
        eval!(context, input_limb_7_col8),
        eval!(context, input_limb_8_col9),
        eval!(context, input_limb_9_col10),
    ];
    let numerator_15 = eval!(context, -(enabler_col0));
    acc.add_to_relation(context, numerator_15, tuple_15);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "range_check_252_width_27".to_string()
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
        _preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        None
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
            qm31_from_u32s(870949674, 1959029187, 1713670372, 1941984119),
            qm31_from_u32s(938058853, 2093246915, 1780779236, 1941984119),
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
        let preprocessed_columns = HashMap::from([]);
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
        assert_eq!(result_value, RANGE_CHECK_252_WIDTH_27_SAMPLE_EVAL_RESULT)
    }
}
