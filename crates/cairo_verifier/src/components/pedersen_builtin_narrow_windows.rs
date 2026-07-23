// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 3;
pub const N_INTERACTION_COLUMNS: usize = 8;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "PedersenAggregatorWindowBits9", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [input_state_0_id_col0, input_state_1_id_col1, output_state_id_col2] =
        input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, &acc.preprocessed_columns);
    let pedersen_builtin_segment_start =
        *acc.public_params.get("pedersen_builtin_segment_start").unwrap();

    let instance_addr_tmp_364c7_0 =
        eval!(context, ((seq) * (3)) + (pedersen_builtin_segment_start));

    read_id::accumulate_constraints(
        &[
            eval!(context, instance_addr_tmp_364c7_0),
            eval!(context, 1),
            eval!(context, input_state_0_id_col0),
        ],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_364c7_0) + (1)),
            eval!(context, 1),
            eval!(context, input_state_1_id_col1),
        ],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_364c7_0) + (2)),
            eval!(context, 1),
            eval!(context, output_state_id_col2),
        ],
        context,
        component_data,
        acc,
    );

    // Use PedersenAggregatorWindowBits9.
    let tuple_4 = &[
        eval!(context, 194336987),
        eval!(context, input_state_0_id_col0),
        eval!(context, input_state_1_id_col1),
        eval!(context, output_state_id_col2),
    ];
    let numerator_4 = eval!(context, 1);
    acc.add_to_relation(context, numerator_4, tuple_4);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "pedersen_builtin_narrow_windows".to_string()
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
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
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
        let preprocessed_columns = HashMap::from([(
            PreProcessedColumnId { id: "seq_15".to_owned() },
            context.constant(qm31_from_u32s(735272696, 1215403647, 795393303, 879304430)),
        )]);
        let public_params = HashMap::from([(
            "pedersen_builtin_segment_start".to_owned(),
            context.constant(1353712625.into()),
        )]);
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
        assert_eq!(result_value, PEDERSEN_BUILTIN_NARROW_WINDOWS_SAMPLE_EVAL_RESULT)
    }
}
