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
    let poseidon_round_keys_0 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_0".to_owned() });
    let poseidon_round_keys_1 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_1".to_owned() });
    let poseidon_round_keys_10 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_10".to_owned() });
    let poseidon_round_keys_11 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_11".to_owned() });
    let poseidon_round_keys_12 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_12".to_owned() });
    let poseidon_round_keys_13 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_13".to_owned() });
    let poseidon_round_keys_14 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_14".to_owned() });
    let poseidon_round_keys_15 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_15".to_owned() });
    let poseidon_round_keys_16 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_16".to_owned() });
    let poseidon_round_keys_17 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_17".to_owned() });
    let poseidon_round_keys_18 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_18".to_owned() });
    let poseidon_round_keys_19 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_19".to_owned() });
    let poseidon_round_keys_2 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_2".to_owned() });
    let poseidon_round_keys_20 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_20".to_owned() });
    let poseidon_round_keys_21 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_21".to_owned() });
    let poseidon_round_keys_22 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_22".to_owned() });
    let poseidon_round_keys_23 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_23".to_owned() });
    let poseidon_round_keys_24 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_24".to_owned() });
    let poseidon_round_keys_25 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_25".to_owned() });
    let poseidon_round_keys_26 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_26".to_owned() });
    let poseidon_round_keys_27 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_27".to_owned() });
    let poseidon_round_keys_28 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_28".to_owned() });
    let poseidon_round_keys_29 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_29".to_owned() });
    let poseidon_round_keys_3 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_3".to_owned() });
    let poseidon_round_keys_4 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_4".to_owned() });
    let poseidon_round_keys_5 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_5".to_owned() });
    let poseidon_round_keys_6 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_6".to_owned() });
    let poseidon_round_keys_7 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_7".to_owned() });
    let poseidon_round_keys_8 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_8".to_owned() });
    let poseidon_round_keys_9 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_9".to_owned() });
    let seq_6 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_6".to_owned() });

    // Yield PoseidonRoundKeys.
    let tuple_0 = &[
        eval!(context, 1024310512),
        eval!(context, seq_6),
        eval!(context, poseidon_round_keys_0),
        eval!(context, poseidon_round_keys_1),
        eval!(context, poseidon_round_keys_2),
        eval!(context, poseidon_round_keys_3),
        eval!(context, poseidon_round_keys_4),
        eval!(context, poseidon_round_keys_5),
        eval!(context, poseidon_round_keys_6),
        eval!(context, poseidon_round_keys_7),
        eval!(context, poseidon_round_keys_8),
        eval!(context, poseidon_round_keys_9),
        eval!(context, poseidon_round_keys_10),
        eval!(context, poseidon_round_keys_11),
        eval!(context, poseidon_round_keys_12),
        eval!(context, poseidon_round_keys_13),
        eval!(context, poseidon_round_keys_14),
        eval!(context, poseidon_round_keys_15),
        eval!(context, poseidon_round_keys_16),
        eval!(context, poseidon_round_keys_17),
        eval!(context, poseidon_round_keys_18),
        eval!(context, poseidon_round_keys_19),
        eval!(context, poseidon_round_keys_20),
        eval!(context, poseidon_round_keys_21),
        eval!(context, poseidon_round_keys_22),
        eval!(context, poseidon_round_keys_23),
        eval!(context, poseidon_round_keys_24),
        eval!(context, poseidon_round_keys_25),
        eval!(context, poseidon_round_keys_26),
        eval!(context, poseidon_round_keys_27),
        eval!(context, poseidon_round_keys_28),
        eval!(context, poseidon_round_keys_29),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0_col0));
    acc.add_to_relation(context, numerator_0, tuple_0);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "poseidon_round_keys".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
        // Verify this component has 2 ** 6 rows
        let size_bit = component_data.get_n_instances_bit(context, 6);
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
        Some(6)
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
            64,
        );
        let random_coeff =
            context.new_var(qm31_from_u32s(474642921, 876336632, 1911695779, 974600512));
        let interaction_elements = [
            context.new_var(qm31_from_u32s(445623802, 202571636, 1360224996, 131355117)),
            context.new_var(qm31_from_u32s(476823935, 939223384, 62486082, 122423602)),
        ];
        let preprocessed_columns = HashMap::from([
            (
                PreProcessedColumnId { id: "seq_6".to_owned() },
                context.constant(qm31_from_u32s(897701151, 670658310, 1894193071, 865942395)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_0".to_owned() },
                context.constant(qm31_from_u32s(876887147, 1637604496, 697065836, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_1".to_owned() },
                context.constant(qm31_from_u32s(809777968, 1503386768, 629956972, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_2".to_owned() },
                context.constant(qm31_from_u32s(1011105505, 1906039952, 831283564, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_3".to_owned() },
                context.constant(qm31_from_u32s(943996326, 1771822224, 764174700, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_4".to_owned() },
                context.constant(qm31_from_u32s(1145323863, 26991761, 965501293, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_5".to_owned() },
                context.constant(qm31_from_u32s(1078214684, 2040257680, 898392428, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_6".to_owned() },
                context.constant(qm31_from_u32s(1279542221, 295427217, 1099719021, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_7".to_owned() },
                context.constant(qm31_from_u32s(1212433042, 161209489, 1032610157, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_8".to_owned() },
                context.constant(qm31_from_u32s(340013715, 563862672, 160194924, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_9".to_owned() },
                context.constant(qm31_from_u32s(272904536, 429644944, 93086060, 1600795770)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_10".to_owned() },
                context.constant(qm31_from_u32s(953889868, 561726076, 1919855750, 479512227)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_11".to_owned() },
                context.constant(qm31_from_u32s(1020999047, 695943804, 1986964614, 479512227)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_12".to_owned() },
                context.constant(qm31_from_u32s(1088108226, 830161532, 2054073478, 479512227)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_13".to_owned() },
                context.constant(qm31_from_u32s(1155217405, 964379260, 2121182342, 479512227)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_14".to_owned() },
                context.constant(qm31_from_u32s(1222326584, 1098596988, 40807559, 479512228)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_15".to_owned() },
                context.constant(qm31_from_u32s(1289435763, 1232814716, 107916423, 479512228)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_16".to_owned() },
                context.constant(qm31_from_u32s(1356544942, 1367032444, 175025287, 479512228)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_17".to_owned() },
                context.constant(qm31_from_u32s(1423654121, 1501250172, 242134151, 479512228)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_18".to_owned() },
                context.constant(qm31_from_u32s(417016436, 1635467899, 1382984837, 479512227)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_19".to_owned() },
                context.constant(qm31_from_u32s(484125615, 1769685627, 1450093701, 479512227)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_20".to_owned() },
                context.constant(qm31_from_u32s(1088415411, 830161712, 2054073658, 479512287)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_21".to_owned() },
                context.constant(qm31_from_u32s(1021306232, 695943984, 1986964794, 479512287)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_22".to_owned() },
                context.constant(qm31_from_u32s(1222633769, 1098597168, 40807739, 479512288)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_23".to_owned() },
                context.constant(qm31_from_u32s(1155524590, 964379440, 2121182522, 479512287)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_24".to_owned() },
                context.constant(qm31_from_u32s(1356852127, 1367032624, 175025467, 479512288)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_25".to_owned() },
                context.constant(qm31_from_u32s(1289742948, 1232814896, 107916603, 479512288)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_26".to_owned() },
                context.constant(qm31_from_u32s(1491070485, 1635468080, 309243195, 479512288)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_27".to_owned() },
                context.constant(qm31_from_u32s(1423961306, 1501250352, 242134331, 479512288)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_28".to_owned() },
                context.constant(qm31_from_u32s(551541979, 1903903535, 1517202745, 479512287)),
            ),
            (
                PreProcessedColumnId { id: "poseidon_round_keys_29".to_owned() },
                context.constant(qm31_from_u32s(484432800, 1769685807, 1450093881, 479512287)),
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
        assert_eq!(result_value, POSEIDON_ROUND_KEYS_SAMPLE_EVAL_RESULT)
    }
}
