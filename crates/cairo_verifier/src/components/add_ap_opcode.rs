// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 17;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 6] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "RangeCheck_11", uses: 1 },
    RelationUse { relation_id: "RangeCheck_18", uses: 1 },
    RelationUse { relation_id: "VerifyInstruction", uses: 1 },
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
        input_pc_col1,
        input_ap_col2,
        input_fp_col3,
        offset2_col4,
        op1_imm_col5,
        op1_base_fp_col6,
        mem1_base_col7,
        op1_id_col8,
        msb_col9,
        mid_limbs_set_col10,
        op1_limb_0_col11,
        op1_limb_1_col12,
        op1_limb_2_col13,
        remainder_bits_col14,
        partial_limb_msb_col15,
        range_check_29_bot11bits_col16,
    ] = input.try_into().unwrap();

    // Enabler is a bit.
    let constraint_0_value = eval!(context, ((enabler_col0) * (enabler_col0)) - (enabler_col0));
    acc.add_constraint(context, constraint_0_value);

    let [
        decode_instruction_89ffb_output_tmp_44683_6_offset2,
        decode_instruction_89ffb_output_tmp_44683_6_op1_base_ap,
    ] = decode_instruction_89ffb::accumulate_constraints(
        &[
            eval!(context, input_pc_col1),
            eval!(context, enabler_col0),
            eval!(context, offset2_col4),
            eval!(context, op1_imm_col5),
            eval!(context, op1_base_fp_col6),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    // if imm then offset2 is 1.
    let constraint_2_value = eval!(
        context,
        (op1_imm_col5) * ((1) - (decode_instruction_89ffb_output_tmp_44683_6_offset2))
    );
    acc.add_constraint(context, constraint_2_value);

    // mem1_base.
    let constraint_3_value = eval!(
        context,
        (mem1_base_col7)
            - ((((op1_imm_col5) * (input_pc_col1)) + ((op1_base_fp_col6) * (input_fp_col3)))
                + ((decode_instruction_89ffb_output_tmp_44683_6_op1_base_ap) * (input_ap_col2)))
    );
    acc.add_constraint(context, constraint_3_value);

    let [read_small_output_tmp_44683_16_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col7) + (decode_instruction_89ffb_output_tmp_44683_6_offset2)
            ),
            eval!(context, enabler_col0),
            eval!(context, op1_id_col8),
            eval!(context, msb_col9),
            eval!(context, mid_limbs_set_col10),
            eval!(context, op1_limb_0_col11),
            eval!(context, op1_limb_1_col12),
            eval!(context, op1_limb_2_col13),
            eval!(context, remainder_bits_col14),
            eval!(context, partial_limb_msb_col15),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let next_ap_tmp_44683_17 =
        eval!(context, (input_ap_col2) + (read_small_output_tmp_44683_16_limb_0));

    range_check_29::accumulate_constraints(
        &[
            eval!(context, next_ap_tmp_44683_17),
            eval!(context, enabler_col0),
            eval!(context, range_check_29_bot11bits_col16),
        ],
        context,
        component_data,
        acc,
    );

    // Use Opcodes.
    let tuple_7 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col1),
        eval!(context, input_ap_col2),
        eval!(context, input_fp_col3),
    ];
    let numerator_7 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Yield Opcodes.
    let tuple_8 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col1) + ((1) + (op1_imm_col5))),
        eval!(context, next_ap_tmp_44683_17),
        eval!(context, input_fp_col3),
    ];
    let numerator_8 = eval!(context, -(enabler_col0));
    acc.add_to_relation(context, numerator_8, tuple_8);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "add_ap_opcode".to_string()
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
        assert_eq!(result_value, ADD_AP_OPCODE_SAMPLE_EVAL_RESULT)
    }
}
