// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 12;
pub const N_INTERACTION_COLUMNS: usize = 8;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] = [RelationUse { relation_id: "Gate", uses: 2 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_op0_limb0_col0,
        input_op0_limb1_col1,
        input_op0_limb2_col2,
        input_op0_limb3_col3,
        input_op1_limb0_col4,
        input_op1_limb1_col5,
        input_op1_limb2_col6,
        input_op1_limb3_col7,
        input_dst_limb0_col8,
        input_dst_limb1_col9,
        input_dst_limb2_col10,
        input_dst_limb3_col11,
    ] = input.try_into().unwrap();
    let qm31_ops_add_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() });
    let qm31_ops_in0_address = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_in0_address".to_owned() });
    let qm31_ops_in1_address = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_in1_address".to_owned() });
    let qm31_ops_mul_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() });
    let qm31_ops_mults =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_mults".to_owned() });
    let qm31_ops_out_address = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_out_address".to_owned() });
    let qm31_ops_pointwise_mul_flag = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "qm31_ops_pointwise_mul_flag".to_owned(),
    });
    let qm31_ops_sub_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() });

    // all flags sum to 1.
    let constraint_0_value = eval!(
        context,
        ((((qm31_ops_add_flag) + (qm31_ops_sub_flag)) + (qm31_ops_mul_flag))
            + (qm31_ops_pointwise_mul_flag))
            - (1)
    );
    acc.add_constraint(context, constraint_0_value);

    // add_flag is a bit.
    let constraint_1_value = eval!(context, (qm31_ops_add_flag) * ((qm31_ops_add_flag) - (1)));
    acc.add_constraint(context, constraint_1_value);

    // sub_flag is a bit.
    let constraint_2_value = eval!(context, (qm31_ops_sub_flag) * ((qm31_ops_sub_flag) - (1)));
    acc.add_constraint(context, constraint_2_value);

    // mul_flag is a bit.
    let constraint_3_value = eval!(context, (qm31_ops_mul_flag) * ((qm31_ops_mul_flag) - (1)));
    acc.add_constraint(context, constraint_3_value);

    // pointwise_mul_flag is a bit.
    let constraint_4_value =
        eval!(context, (qm31_ops_pointwise_mul_flag) * ((qm31_ops_pointwise_mul_flag) - (1)));
    acc.add_constraint(context, constraint_4_value);

    let constraint_5_value = eval!(
        context,
        (input_dst_limb0_col8)
            - ((((((((((input_op0_limb0_col0) * (input_op1_limb0_col4))
                - ((input_op0_limb1_col1) * (input_op1_limb1_col5)))
                + ((2)
                    * (((input_op0_limb2_col2) * (input_op1_limb2_col6))
                        - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))))
                - ((input_op0_limb2_col2) * (input_op1_limb3_col7)))
                - ((input_op0_limb3_col3) * (input_op1_limb2_col6)))
                * (qm31_ops_mul_flag))
                + (((input_op0_limb0_col0) + (input_op1_limb0_col4)) * (qm31_ops_add_flag)))
                + (((input_op0_limb0_col0) - (input_op1_limb0_col4)) * (qm31_ops_sub_flag)))
                + (((input_op0_limb0_col0) * (input_op1_limb0_col4))
                    * (qm31_ops_pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_5_value);

    let constraint_6_value = eval!(
        context,
        (input_dst_limb1_col9)
            - ((((((((((input_op0_limb0_col0) * (input_op1_limb1_col5))
                + ((input_op0_limb1_col1) * (input_op1_limb0_col4)))
                + ((2)
                    * (((input_op0_limb2_col2) * (input_op1_limb3_col7))
                        + ((input_op0_limb3_col3) * (input_op1_limb2_col6)))))
                + ((input_op0_limb2_col2) * (input_op1_limb2_col6)))
                - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))
                * (qm31_ops_mul_flag))
                + (((input_op0_limb1_col1) + (input_op1_limb1_col5)) * (qm31_ops_add_flag)))
                + (((input_op0_limb1_col1) - (input_op1_limb1_col5)) * (qm31_ops_sub_flag)))
                + (((input_op0_limb1_col1) * (input_op1_limb1_col5))
                    * (qm31_ops_pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_6_value);

    let constraint_7_value = eval!(
        context,
        (input_dst_limb2_col10)
            - (((((((((input_op0_limb0_col0) * (input_op1_limb2_col6))
                - ((input_op0_limb1_col1) * (input_op1_limb3_col7)))
                + ((input_op0_limb2_col2) * (input_op1_limb0_col4)))
                - ((input_op0_limb3_col3) * (input_op1_limb1_col5)))
                * (qm31_ops_mul_flag))
                + (((input_op0_limb2_col2) + (input_op1_limb2_col6)) * (qm31_ops_add_flag)))
                + (((input_op0_limb2_col2) - (input_op1_limb2_col6)) * (qm31_ops_sub_flag)))
                + (((input_op0_limb2_col2) * (input_op1_limb2_col6))
                    * (qm31_ops_pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_7_value);

    let constraint_8_value = eval!(
        context,
        (input_dst_limb3_col11)
            - (((((((((input_op0_limb0_col0) * (input_op1_limb3_col7))
                + ((input_op0_limb1_col1) * (input_op1_limb2_col6)))
                + ((input_op0_limb2_col2) * (input_op1_limb1_col5)))
                + ((input_op0_limb3_col3) * (input_op1_limb0_col4)))
                * (qm31_ops_mul_flag))
                + (((input_op0_limb3_col3) + (input_op1_limb3_col7)) * (qm31_ops_add_flag)))
                + (((input_op0_limb3_col3) - (input_op1_limb3_col7)) * (qm31_ops_sub_flag)))
                + (((input_op0_limb3_col3) * (input_op1_limb3_col7))
                    * (qm31_ops_pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_8_value);

    // Use Gate.
    let tuple_9 = &[
        eval!(context, 378353459),
        eval!(context, qm31_ops_in0_address),
        eval!(context, input_op0_limb0_col0),
        eval!(context, input_op0_limb1_col1),
        eval!(context, input_op0_limb2_col2),
        eval!(context, input_op0_limb3_col3),
    ];
    let numerator_9 = eval!(context, 1);
    acc.add_to_relation(context, numerator_9, tuple_9);

    // Use Gate.
    let tuple_10 = &[
        eval!(context, 378353459),
        eval!(context, qm31_ops_in1_address),
        eval!(context, input_op1_limb0_col4),
        eval!(context, input_op1_limb1_col5),
        eval!(context, input_op1_limb2_col6),
        eval!(context, input_op1_limb3_col7),
    ];
    let numerator_10 = eval!(context, 1);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Yield Gate.
    let tuple_11 = &[
        eval!(context, 378353459),
        eval!(context, qm31_ops_out_address),
        eval!(context, input_dst_limb0_col8),
        eval!(context, input_dst_limb1_col9),
        eval!(context, input_dst_limb2_col10),
        eval!(context, input_dst_limb3_col11),
    ];
    let numerator_11 = eval!(context, -(qm31_ops_mults));
    acc.add_to_relation(context, numerator_11, tuple_11);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "qm_31_ops".to_string()
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
        Some(
            *preprocessed_column_log_sizes
                .get(&PreProcessedColumnId { id: "qm31_ops_add_flag".to_string() })
                .unwrap(),
        )
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
        let preprocessed_columns = HashMap::from([
            (
                PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() },
                context.constant(qm31_from_u32s(1527140322, 858930457, 73068685, 1080204029)),
            ),
            (
                PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() },
                context.constant(qm31_from_u32s(1106251446, 17903294, 1375115415, 1357632339)),
            ),
            (
                PreProcessedColumnId { id: "qm31_ops_pointwise_mul_flag".to_owned() },
                context.constant(qm31_from_u32s(485880212, 563142412, 2027283405, 872250142)),
            ),
            (
                PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() },
                context.constant(qm31_from_u32s(631320763, 551670292, 1783831999, 688564846)),
            ),
            (
                PreProcessedColumnId { id: "qm31_ops_in0_address".to_owned() },
                context.constant(qm31_from_u32s(944321702, 2104082059, 1058357559, 650219243)),
            ),
            (
                PreProcessedColumnId { id: "qm31_ops_in1_address".to_owned() },
                context.constant(qm31_from_u32s(721894235, 818083030, 1176389319, 1875149912)),
            ),
            (
                PreProcessedColumnId { id: "qm31_ops_out_address".to_owned() },
                context.constant(qm31_from_u32s(1284595166, 878257086, 777884389, 531365363)),
            ),
            (
                PreProcessedColumnId { id: "qm31_ops_mults".to_owned() },
                context.constant(qm31_from_u32s(2068309461, 1074848526, 422232906, 2078266109)),
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
        assert_eq!(result_value, QM_31_OPS_SAMPLE_EVAL_RESULT)
    }
}
