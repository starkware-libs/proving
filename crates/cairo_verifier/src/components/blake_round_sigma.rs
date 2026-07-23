// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 1;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [multiplicity_0_col0] = input.try_into().unwrap();
    let blake_sigma_0 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_0".to_owned() });
    let blake_sigma_1 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_1".to_owned() });
    let blake_sigma_10 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_10".to_owned() });
    let blake_sigma_11 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_11".to_owned() });
    let blake_sigma_12 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_12".to_owned() });
    let blake_sigma_13 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_13".to_owned() });
    let blake_sigma_14 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_14".to_owned() });
    let blake_sigma_15 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_15".to_owned() });
    let blake_sigma_2 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_2".to_owned() });
    let blake_sigma_3 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_3".to_owned() });
    let blake_sigma_4 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_4".to_owned() });
    let blake_sigma_5 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_5".to_owned() });
    let blake_sigma_6 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_6".to_owned() });
    let blake_sigma_7 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_7".to_owned() });
    let blake_sigma_8 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_8".to_owned() });
    let blake_sigma_9 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_9".to_owned() });
    let seq_4 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_4".to_owned() });

    // Yield BlakeRoundSigma.
    let tuple_0 = &[
        eval!(context, 1805967942),
        eval!(context, seq_4),
        eval!(context, blake_sigma_0),
        eval!(context, blake_sigma_1),
        eval!(context, blake_sigma_2),
        eval!(context, blake_sigma_3),
        eval!(context, blake_sigma_4),
        eval!(context, blake_sigma_5),
        eval!(context, blake_sigma_6),
        eval!(context, blake_sigma_7),
        eval!(context, blake_sigma_8),
        eval!(context, blake_sigma_9),
        eval!(context, blake_sigma_10),
        eval!(context, blake_sigma_11),
        eval!(context, blake_sigma_12),
        eval!(context, blake_sigma_13),
        eval!(context, blake_sigma_14),
        eval!(context, blake_sigma_15),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0_col0));
    acc.add_to_relation(context, numerator_0, tuple_0);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "blake_round_sigma".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
        // Verify this component has 2 ** 4 rows
        let size_bit = component_data.get_n_instances_bit(context, 4);
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
        Some(4)
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
        let trace_columns = [qm31_from_u32s(1659099300, 905558730, 651199673, 1375009625)];
        let interaction_columns = [qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119)];
        let component_data = TestComponentData::from_values(
            &mut context,
            &trace_columns,
            &interaction_columns,
            qm31_from_u32s(1115374022, 1127856551, 489657863, 643630026),
            16,
        );
        let random_coeff =
            context.new_var(qm31_from_u32s(474642921, 876336632, 1911695779, 974600512));
        let interaction_elements = [
            context.new_var(qm31_from_u32s(445623802, 202571636, 1360224996, 131355117)),
            context.new_var(qm31_from_u32s(476823935, 939223384, 62486082, 122423602)),
        ];
        let preprocessed_columns = HashMap::from([
            (
                PreProcessedColumnId { id: "seq_4".to_owned() },
                context.constant(qm31_from_u32s(763482793, 402222854, 1759975343, 865942395)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_0".to_owned() },
                context.constant(qm31_from_u32s(1541575468, 910566768, 1277642954, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_1".to_owned() },
                context.constant(qm31_from_u32s(1474466289, 776349040, 1210534090, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_2".to_owned() },
                context.constant(qm31_from_u32s(1407357110, 642131312, 1143425226, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_3".to_owned() },
                context.constant(qm31_from_u32s(1340247931, 507913584, 1076316362, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_4".to_owned() },
                context.constant(qm31_from_u32s(1810012184, 1447437680, 1546078410, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_5".to_owned() },
                context.constant(qm31_from_u32s(1742903005, 1313219952, 1478969546, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_6".to_owned() },
                context.constant(qm31_from_u32s(1675793826, 1179002224, 1411860682, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_7".to_owned() },
                context.constant(qm31_from_u32s(1608684647, 1044784496, 1344751818, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_8".to_owned() },
                context.constant(qm31_from_u32s(2078448900, 1984308592, 1814513866, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_9".to_owned() },
                context.constant(qm31_from_u32s(2011339721, 1850090864, 1747405002, 337722398)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_10".to_owned() },
                context.constant(qm31_from_u32s(112615900, 18292853, 1092454797, 265412759)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_11".to_owned() },
                context.constant(qm31_from_u32s(179725079, 152510581, 1159563661, 265412759)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_12".to_owned() },
                context.constant(qm31_from_u32s(2125881189, 1897341043, 958237068, 265412759)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_13".to_owned() },
                context.constant(qm31_from_u32s(45506721, 2031558772, 1025345932, 265412759)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_14".to_owned() },
                context.constant(qm31_from_u32s(1991662831, 1628905587, 824019340, 265412759)),
            ),
            (
                PreProcessedColumnId { id: "blake_sigma_15".to_owned() },
                context.constant(qm31_from_u32s(2058772010, 1763123315, 891128204, 265412759)),
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
        assert_eq!(result_value, BLAKE_ROUND_SIGMA_SAMPLE_EVAL_RESULT)
    }
}
