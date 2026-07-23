// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 8;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        multiplicity_0_col0,
        multiplicity_1_col1,
        multiplicity_2_col2,
        multiplicity_3_col3,
        multiplicity_4_col4,
        multiplicity_5_col5,
        multiplicity_6_col6,
        multiplicity_7_col7,
    ] = input.try_into().unwrap();
    let range_check_9_9_column_0 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "range_check_9_9_column_0".to_owned(),
    });
    let range_check_9_9_column_1 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "range_check_9_9_column_1".to_owned(),
    });

    // Yield RangeCheck_9_9.
    let tuple_0 = &[
        eval!(context, 517791011),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0_col0));
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Yield RangeCheck_9_9_B.
    let tuple_1 = &[
        eval!(context, 1897792095),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_1 = eval!(context, -(multiplicity_1_col1));
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Yield RangeCheck_9_9_C.
    let tuple_2 = &[
        eval!(context, 1881014476),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_2 = eval!(context, -(multiplicity_2_col2));
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Yield RangeCheck_9_9_D.
    let tuple_3 = &[
        eval!(context, 1864236857),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_3 = eval!(context, -(multiplicity_3_col3));
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Yield RangeCheck_9_9_E.
    let tuple_4 = &[
        eval!(context, 1847459238),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_4 = eval!(context, -(multiplicity_4_col4));
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Yield RangeCheck_9_9_F.
    let tuple_5 = &[
        eval!(context, 1830681619),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_5 = eval!(context, -(multiplicity_5_col5));
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Yield RangeCheck_9_9_G.
    let tuple_6 = &[
        eval!(context, 1813904000),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_6 = eval!(context, -(multiplicity_6_col6));
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Yield RangeCheck_9_9_H.
    let tuple_7 = &[
        eval!(context, 2065568285),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_7 = eval!(context, -(multiplicity_7_col7));
    acc.add_to_relation(context, numerator_7, tuple_7);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "range_check_9_9".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
        // Verify this component has 2 ** 18 rows
        let size_bit = component_data.get_n_instances_bit(context, 18);
        eq(context, size_bit, context.one());
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
        Some(18)
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
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
            qm31_from_u32s(1206495569, 482634180, 2049214693, 1941984119),
        ];
        let component_data = TestComponentData::from_values(
            &mut context,
            &trace_columns,
            &interaction_columns,
            qm31_from_u32s(1115374022, 1127856551, 489657863, 643630026),
            262144,
        );
        let random_coeff =
            context.new_var(qm31_from_u32s(474642921, 876336632, 1911695779, 974600512));
        let interaction_elements = [
            context.new_var(qm31_from_u32s(445623802, 202571636, 1360224996, 131355117)),
            context.new_var(qm31_from_u32s(476823935, 939223384, 62486082, 122423602)),
        ];
        let preprocessed_columns = HashMap::from([
            (
                PreProcessedColumnId { id: "range_check_9_9_column_0".to_owned() },
                context.constant(qm31_from_u32s(601843633, 18326218, 429617579, 1687843200)),
            ),
            (
                PreProcessedColumnId { id: "range_check_9_9_column_1".to_owned() },
                context.constant(qm31_from_u32s(534734454, 2031592137, 362508714, 1687843200)),
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
        assert_eq!(result_value, RANGE_CHECK_9_9_SAMPLE_EVAL_RESULT)
    }
}
