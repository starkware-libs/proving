// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 47;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 2 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 2 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
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
        offset0_col4,
        dst_base_fp_col5,
        ap_update_add_1_col6,
        mem_dst_base_col7,
        dst_id_col8,
        dst_limb_0_col9,
        dst_limb_1_col10,
        dst_limb_2_col11,
        dst_limb_3_col12,
        dst_limb_4_col13,
        dst_limb_5_col14,
        dst_limb_6_col15,
        dst_limb_7_col16,
        dst_limb_8_col17,
        dst_limb_9_col18,
        dst_limb_10_col19,
        dst_limb_11_col20,
        dst_limb_12_col21,
        dst_limb_13_col22,
        dst_limb_14_col23,
        dst_limb_15_col24,
        dst_limb_16_col25,
        dst_limb_17_col26,
        dst_limb_18_col27,
        dst_limb_19_col28,
        dst_limb_20_col29,
        dst_limb_21_col30,
        dst_limb_22_col31,
        dst_limb_23_col32,
        dst_limb_24_col33,
        dst_limb_25_col34,
        dst_limb_26_col35,
        dst_limb_27_col36,
        dst_sum_inv_col37,
        dst_sum_squares_inv_col38,
        next_pc_id_col39,
        msb_col40,
        mid_limbs_set_col41,
        next_pc_limb_0_col42,
        next_pc_limb_1_col43,
        next_pc_limb_2_col44,
        remainder_bits_col45,
        partial_limb_msb_col46,
    ] = input.try_into().unwrap();

    // Enabler is a bit.
    let constraint_0_value = eval!(context, ((enabler_col0) * (enabler_col0)) - (enabler_col0));
    acc.add_constraint(context, constraint_0_value);

    let [decode_instruction_ad440_output_tmp_7f087_5_offset0] =
        decode_instruction_ad440::accumulate_constraints(
            &[
                eval!(context, input_pc_col1),
                eval!(context, enabler_col0),
                eval!(context, offset0_col4),
                eval!(context, dst_base_fp_col5),
                eval!(context, ap_update_add_1_col6),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    // mem_dst_base.
    let constraint_2_value = eval!(
        context,
        (mem_dst_base_col7)
            - (((dst_base_fp_col5) * (input_fp_col3))
                + (((1) - (dst_base_fp_col5)) * (input_ap_col2)))
    );
    acc.add_constraint(context, constraint_2_value);

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col7) + (decode_instruction_ad440_output_tmp_7f087_5_offset0)
            ),
            eval!(context, enabler_col0),
            eval!(context, dst_id_col8),
            eval!(context, dst_limb_0_col9),
            eval!(context, dst_limb_1_col10),
            eval!(context, dst_limb_2_col11),
            eval!(context, dst_limb_3_col12),
            eval!(context, dst_limb_4_col13),
            eval!(context, dst_limb_5_col14),
            eval!(context, dst_limb_6_col15),
            eval!(context, dst_limb_7_col16),
            eval!(context, dst_limb_8_col17),
            eval!(context, dst_limb_9_col18),
            eval!(context, dst_limb_10_col19),
            eval!(context, dst_limb_11_col20),
            eval!(context, dst_limb_12_col21),
            eval!(context, dst_limb_13_col22),
            eval!(context, dst_limb_14_col23),
            eval!(context, dst_limb_15_col24),
            eval!(context, dst_limb_16_col25),
            eval!(context, dst_limb_17_col26),
            eval!(context, dst_limb_18_col27),
            eval!(context, dst_limb_19_col28),
            eval!(context, dst_limb_20_col29),
            eval!(context, dst_limb_21_col30),
            eval!(context, dst_limb_22_col31),
            eval!(context, dst_limb_23_col32),
            eval!(context, dst_limb_24_col33),
            eval!(context, dst_limb_25_col34),
            eval!(context, dst_limb_26_col35),
            eval!(context, dst_limb_27_col36),
        ],
        context,
        component_data,
        acc,
    );

    let dst_sum_p_zero_tmp_7f087_11 = eval!(
        context,
        ((((((((((((((((((((((((dst_limb_1_col10) + (dst_limb_2_col11))
            + (dst_limb_3_col12))
            + (dst_limb_4_col13))
            + (dst_limb_5_col14))
            + (dst_limb_6_col15))
            + (dst_limb_7_col16))
            + (dst_limb_8_col17))
            + (dst_limb_9_col18))
            + (dst_limb_10_col19))
            + (dst_limb_11_col20))
            + (dst_limb_12_col21))
            + (dst_limb_13_col22))
            + (dst_limb_14_col23))
            + (dst_limb_15_col24))
            + (dst_limb_16_col25))
            + (dst_limb_17_col26))
            + (dst_limb_18_col27))
            + (dst_limb_19_col28))
            + (dst_limb_20_col29))
            + (dst_limb_22_col31))
            + (dst_limb_23_col32))
            + (dst_limb_24_col33))
            + (dst_limb_25_col34))
            + (dst_limb_26_col35)
    );

    // dst doesn't equal 0.
    let constraint_5_value = eval!(
        context,
        (((dst_sum_p_zero_tmp_7f087_11)
            + (((dst_limb_0_col9) + (dst_limb_21_col30)) + (dst_limb_27_col36)))
            * (dst_sum_inv_col37))
            - (1)
    );
    acc.add_constraint(context, constraint_5_value);

    let diff_from_p_tmp_7f087_12 = eval!(context, (dst_limb_0_col9) - (1));

    let diff_from_p_tmp_7f087_13 = eval!(context, (dst_limb_21_col30) - (136));

    let diff_from_p_tmp_7f087_14 = eval!(context, (dst_limb_27_col36) - (256));

    // dst doesn't equal P.
    let constraint_9_value = eval!(
        context,
        (((dst_sum_p_zero_tmp_7f087_11)
            + ((((diff_from_p_tmp_7f087_12) * (diff_from_p_tmp_7f087_12))
                + ((diff_from_p_tmp_7f087_13) * (diff_from_p_tmp_7f087_13)))
                + ((diff_from_p_tmp_7f087_14) * (diff_from_p_tmp_7f087_14))))
            * (dst_sum_squares_inv_col38))
            - (1)
    );
    acc.add_constraint(context, constraint_9_value);

    let [read_small_output_tmp_7f087_24_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(context, (input_pc_col1) + (1)),
            eval!(context, enabler_col0),
            eval!(context, next_pc_id_col39),
            eval!(context, msb_col40),
            eval!(context, mid_limbs_set_col41),
            eval!(context, next_pc_limb_0_col42),
            eval!(context, next_pc_limb_1_col43),
            eval!(context, next_pc_limb_2_col44),
            eval!(context, remainder_bits_col45),
            eval!(context, partial_limb_msb_col46),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    // Use Opcodes.
    let tuple_11 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col1),
        eval!(context, input_ap_col2),
        eval!(context, input_fp_col3),
    ];
    let numerator_11 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Yield Opcodes.
    let tuple_12 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col1) + (read_small_output_tmp_7f087_24_limb_0)),
        eval!(context, (input_ap_col2) + (ap_update_add_1_col6)),
        eval!(context, input_fp_col3),
    ];
    let numerator_12 = eval!(context, -(enabler_col0));
    acc.add_to_relation(context, numerator_12, tuple_12);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "jnz_opcode_taken".to_string()
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
            qm31_from_u32s(1986820986, 913513739, 45970432, 343880178),
            qm31_from_u32s(1919711807, 779296011, 2126345215, 343880177),
            qm31_from_u32s(2121039344, 1181949195, 180188160, 343880178),
            qm31_from_u32s(2053930165, 1047731467, 113079296, 343880178),
            qm31_from_u32s(1718384270, 376642827, 1925018623, 343880177),
            qm31_from_u32s(1651275091, 242425099, 1857909759, 343880177),
            qm31_from_u32s(1852602628, 645078283, 2059236351, 343880177),
            qm31_from_u32s(1785493449, 510860555, 1992127487, 343880177),
            qm31_from_u32s(1449947554, 1987255562, 1656583166, 343880177),
            qm31_from_u32s(1382838375, 1853037834, 1589474302, 343880177),
            qm31_from_u32s(510356977, 108207322, 717059022, 343880161),
            qm31_from_u32s(577466156, 242425050, 784167886, 343880161),
            qm31_from_u32s(376138619, 1987255513, 582841293, 343880161),
            qm31_from_u32s(443247798, 2121473241, 649950157, 343880161),
            qm31_from_u32s(778793693, 645078234, 985494478, 343880161),
            qm31_from_u32s(845902872, 779295962, 1052603342, 343880161),
            qm31_from_u32s(644575335, 376642778, 851276750, 343880161),
            qm31_from_u32s(711684514, 510860506, 918385614, 343880161),
            qm31_from_u32s(1047230409, 1181949146, 1253929934, 343880161),
            qm31_from_u32s(1114339588, 1316166874, 1321038798, 343880161),
            qm31_from_u32s(1717810224, 376642479, 1925018275, 343880061),
            qm31_from_u32s(1650701045, 242424751, 1857909411, 343880061),
            qm31_from_u32s(1583591866, 108207023, 1790800547, 343880061),
            qm31_from_u32s(1516482687, 2121472942, 1723691682, 343880061),
            qm31_from_u32s(1986246940, 913513391, 45970084, 343880062),
            qm31_from_u32s(1919137761, 779295663, 2126344867, 343880061),
            qm31_from_u32s(1852028582, 645077935, 2059236003, 343880061),
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
        assert_eq!(result_value, JNZ_OPCODE_TAKEN_SAMPLE_EVAL_RESULT)
    }
}
