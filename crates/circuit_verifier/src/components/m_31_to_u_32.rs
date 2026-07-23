// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 4;
pub const N_INTERACTION_COLUMNS: usize = 12;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "Gate", uses: 1 },
    RelationUse { relation_id: "RangeCheck_16", uses: 3 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [input_m31_col0, input_u32_limb_0_col1, input_u32_limb_1_col2, inv_or_one_col3] =
        input.try_into().unwrap();
    let m31_to_u32_input_addr = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "m31_to_u32_input_addr".to_owned() });
    let m31_to_u32_multiplicity = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "m31_to_u32_multiplicity".to_owned(),
    });
    let m31_to_u32_output_addr = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "m31_to_u32_output_addr".to_owned() });

    // Use RangeCheck_16.
    let tuple_0 = &[eval!(context, 1008385708), eval!(context, input_u32_limb_0_col1)];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Use RangeCheck_16.
    let tuple_1 = &[eval!(context, 1008385708), eval!(context, input_u32_limb_1_col2)];
    let numerator_1 = eval!(context, 1);
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Use RangeCheck_16.
    let tuple_2 = &[eval!(context, 1008385708), eval!(context, (32767) - (input_u32_limb_1_col2))];
    let numerator_2 = eval!(context, 1);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // input is zero then limb_low is zero.
    let constraint_3_value =
        eval!(context, (((input_m31_col0) * (inv_or_one_col3)) - (1)) * (input_u32_limb_0_col1));
    acc.add_constraint(context, constraint_3_value);

    // input reconstruction.
    let constraint_4_value = eval!(
        context,
        (input_m31_col0) - ((input_u32_limb_0_col1) + ((input_u32_limb_1_col2) * (65536)))
    );
    acc.add_constraint(context, constraint_4_value);

    // Use Gate.
    let tuple_5 = &[
        eval!(context, 378353459),
        eval!(context, m31_to_u32_input_addr),
        eval!(context, input_m31_col0),
    ];
    let numerator_5 = eval!(context, 1);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Yield Gate.
    let tuple_6 = &[
        eval!(context, 378353459),
        eval!(context, m31_to_u32_output_addr),
        eval!(context, input_u32_limb_0_col1),
        eval!(context, input_u32_limb_1_col2),
    ];
    let numerator_6 = eval!(context, -(m31_to_u32_multiplicity));
    acc.add_to_relation(context, numerator_6, tuple_6);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "m_31_to_u_32".to_string()
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
            .get(&PreProcessedColumnId { id: "m31_to_u32_input_addr".to_string() })
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
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
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
                PreProcessedColumnId { id: "m31_to_u32_input_addr".to_owned() },
                context.constant(qm31_from_u32s(15668215, 1851966168, 874056991, 2075313468)),
            ),
            (
                PreProcessedColumnId { id: "m31_to_u32_output_addr".to_owned() },
                context.constant(qm31_from_u32s(701904311, 1125291129, 1904795215, 38357025)),
            ),
            (
                PreProcessedColumnId { id: "m31_to_u32_multiplicity".to_owned() },
                context.constant(qm31_from_u32s(1979029033, 1524573277, 1930122227, 1490762084)),
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
        assert_eq!(result_value, M_31_TO_U_32_SAMPLE_EVAL_RESULT)
    }
}
