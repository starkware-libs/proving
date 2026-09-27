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
    let qm_31_ops_add_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm_31_ops_add_flag".to_owned() });
    let qm_31_ops_in_0_address = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "qm_31_ops_in_0_address".to_owned() });
    let qm_31_ops_in_1_address = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "qm_31_ops_in_1_address".to_owned() });
    let qm_31_ops_mul_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm_31_ops_mul_flag".to_owned() });
    let qm_31_ops_mults =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm_31_ops_mults".to_owned() });
    let qm_31_ops_out_address = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "qm_31_ops_out_address".to_owned() });
    let qm_31_ops_pointwise_mul_flag = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "qm_31_ops_pointwise_mul_flag".to_owned(),
    });
    let qm_31_ops_sub_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm_31_ops_sub_flag".to_owned() });

    let constraint_0_value = eval!(
        context,
        (input_dst_limb0_col8)
            - ((((((((((input_op0_limb0_col0) * (input_op1_limb0_col4))
                - ((input_op0_limb1_col1) * (input_op1_limb1_col5)))
                + ((2)
                    * (((input_op0_limb2_col2) * (input_op1_limb2_col6))
                        - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))))
                - ((input_op0_limb2_col2) * (input_op1_limb3_col7)))
                - ((input_op0_limb3_col3) * (input_op1_limb2_col6)))
                * (qm_31_ops_mul_flag))
                + (((input_op0_limb0_col0) + (input_op1_limb0_col4)) * (qm_31_ops_add_flag)))
                + (((input_op0_limb0_col0) - (input_op1_limb0_col4)) * (qm_31_ops_sub_flag)))
                + (((input_op0_limb0_col0) * (input_op1_limb0_col4))
                    * (qm_31_ops_pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_0_value);

    let constraint_1_value = eval!(
        context,
        (input_dst_limb1_col9)
            - ((((((((((input_op0_limb0_col0) * (input_op1_limb1_col5))
                + ((input_op0_limb1_col1) * (input_op1_limb0_col4)))
                + ((2)
                    * (((input_op0_limb2_col2) * (input_op1_limb3_col7))
                        + ((input_op0_limb3_col3) * (input_op1_limb2_col6)))))
                + ((input_op0_limb2_col2) * (input_op1_limb2_col6)))
                - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))
                * (qm_31_ops_mul_flag))
                + (((input_op0_limb1_col1) + (input_op1_limb1_col5)) * (qm_31_ops_add_flag)))
                + (((input_op0_limb1_col1) - (input_op1_limb1_col5)) * (qm_31_ops_sub_flag)))
                + (((input_op0_limb1_col1) * (input_op1_limb1_col5))
                    * (qm_31_ops_pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_1_value);

    let constraint_2_value = eval!(
        context,
        (input_dst_limb2_col10)
            - (((((((((input_op0_limb0_col0) * (input_op1_limb2_col6))
                - ((input_op0_limb1_col1) * (input_op1_limb3_col7)))
                + ((input_op0_limb2_col2) * (input_op1_limb0_col4)))
                - ((input_op0_limb3_col3) * (input_op1_limb1_col5)))
                * (qm_31_ops_mul_flag))
                + (((input_op0_limb2_col2) + (input_op1_limb2_col6)) * (qm_31_ops_add_flag)))
                + (((input_op0_limb2_col2) - (input_op1_limb2_col6)) * (qm_31_ops_sub_flag)))
                + (((input_op0_limb2_col2) * (input_op1_limb2_col6))
                    * (qm_31_ops_pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_2_value);

    let constraint_3_value = eval!(
        context,
        (input_dst_limb3_col11)
            - (((((((((input_op0_limb0_col0) * (input_op1_limb3_col7))
                + ((input_op0_limb1_col1) * (input_op1_limb2_col6)))
                + ((input_op0_limb2_col2) * (input_op1_limb1_col5)))
                + ((input_op0_limb3_col3) * (input_op1_limb0_col4)))
                * (qm_31_ops_mul_flag))
                + (((input_op0_limb3_col3) + (input_op1_limb3_col7)) * (qm_31_ops_add_flag)))
                + (((input_op0_limb3_col3) - (input_op1_limb3_col7)) * (qm_31_ops_sub_flag)))
                + (((input_op0_limb3_col3) * (input_op1_limb3_col7))
                    * (qm_31_ops_pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_3_value);

    // Use Gate.
    let tuple_4 = &[
        eval!(context, 378353459),
        eval!(context, qm_31_ops_in_0_address),
        eval!(context, input_op0_limb0_col0),
        eval!(context, input_op0_limb1_col1),
        eval!(context, input_op0_limb2_col2),
        eval!(context, input_op0_limb3_col3),
    ];
    let numerator_4 = eval!(context, 1);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Use Gate.
    let tuple_5 = &[
        eval!(context, 378353459),
        eval!(context, qm_31_ops_in_1_address),
        eval!(context, input_op1_limb0_col4),
        eval!(context, input_op1_limb1_col5),
        eval!(context, input_op1_limb2_col6),
        eval!(context, input_op1_limb3_col7),
    ];
    let numerator_5 = eval!(context, 1);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Yield Gate.
    let tuple_6 = &[
        eval!(context, 378353459),
        eval!(context, qm_31_ops_out_address),
        eval!(context, input_dst_limb0_col8),
        eval!(context, input_dst_limb1_col9),
        eval!(context, input_dst_limb2_col10),
        eval!(context, input_dst_limb3_col11),
    ];
    let numerator_6 = eval!(context, -(qm_31_ops_mults));
    acc.add_to_relation(context, numerator_6, tuple_6);
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
                .get(&PreProcessedColumnId { id: "qm_31_ops_add_flag".to_string() })
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
                PreProcessedColumnId { id: "qm_31_ops_add_flag".to_owned() },
                context.constant(qm31_from_u32s(2008763856, 668586075, 986260244, 1154698137)),
            ),
            (
                PreProcessedColumnId { id: "qm_31_ops_mul_flag".to_owned() },
                context.constant(qm31_from_u32s(753532226, 1668588607, 2021383940, 940498869)),
            ),
            (
                PreProcessedColumnId { id: "qm_31_ops_pointwise_mul_flag".to_owned() },
                context.constant(qm31_from_u32s(1658621201, 1657657148, 1342332119, 2034171678)),
            ),
            (
                PreProcessedColumnId { id: "qm_31_ops_sub_flag".to_owned() },
                context.constant(qm31_from_u32s(346603561, 1505146370, 374195948, 1196742422)),
            ),
            (
                PreProcessedColumnId { id: "qm_31_ops_in_0_address".to_owned() },
                context.constant(qm31_from_u32s(1444382797, 1354185417, 705047099, 132239089)),
            ),
            (
                PreProcessedColumnId { id: "qm_31_ops_in_1_address".to_owned() },
                context.constant(qm31_from_u32s(585273626, 1140883031, 1920880217, 1007275653)),
            ),
            (
                PreProcessedColumnId { id: "qm_31_ops_out_address".to_owned() },
                context.constant(qm31_from_u32s(1287652242, 435165403, 1148348826, 1979403697)),
            ),
            (
                PreProcessedColumnId { id: "qm_31_ops_mults".to_owned() },
                context.constant(qm31_from_u32s(1372962279, 1246081592, 1017358753, 1525168544)),
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
