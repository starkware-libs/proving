// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 53;
pub const N_INTERACTION_COLUMNS: usize = 36;

pub const RELATION_USES_PER_ROW: [RelationUse; 6] = [
    RelationUse { relation_id: "VerifyBitwiseXor_12", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_4", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_7", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 4 },
    RelationUse { relation_id: "VerifyBitwiseXor_8_B", uses: 4 },
    RelationUse { relation_id: "VerifyBitwiseXor_9", uses: 2 },
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
        input_limb_10_col11,
        input_limb_11_col12,
        triple_sum32_res_limb_0_col13,
        triple_sum32_res_limb_1_col14,
        ms_8_bits_col15,
        ms_8_bits_col16,
        ms_8_bits_col17,
        ms_8_bits_col18,
        xor_col19,
        xor_col20,
        xor_col21,
        xor_col22,
        triple_sum32_res_limb_0_col23,
        triple_sum32_res_limb_1_col24,
        ms_4_bits_col25,
        ms_4_bits_col26,
        ms_4_bits_col27,
        ms_4_bits_col28,
        xor_col29,
        xor_col30,
        xor_col31,
        xor_col32,
        triple_sum32_res_limb_0_col33,
        triple_sum32_res_limb_1_col34,
        ms_8_bits_col35,
        ms_8_bits_col36,
        ms_8_bits_col37,
        ms_8_bits_col38,
        xor_col39,
        xor_col40,
        xor_col41,
        xor_col42,
        triple_sum32_res_limb_0_col43,
        triple_sum32_res_limb_1_col44,
        ms_9_bits_col45,
        ms_9_bits_col46,
        ms_9_bits_col47,
        ms_9_bits_col48,
        xor_col49,
        xor_col50,
        xor_col51,
        xor_col52,
    ] = input.try_into().unwrap();

    // Enabler is a bit.
    let constraint_0_value = eval!(context, ((enabler_col0) * (enabler_col0)) - (enabler_col0));
    acc.add_constraint(context, constraint_0_value);

    triple_sum_32::accumulate_constraints(
        &[
            eval!(context, input_limb_0_col1),
            eval!(context, input_limb_1_col2),
            eval!(context, input_limb_2_col3),
            eval!(context, input_limb_3_col4),
            eval!(context, input_limb_8_col9),
            eval!(context, input_limb_9_col10),
            eval!(context, enabler_col0),
            eval!(context, triple_sum32_res_limb_0_col13),
            eval!(context, triple_sum32_res_limb_1_col14),
        ],
        context,
        component_data,
        acc,
    );

    let [xor_rot_32_r_16_output_tmp_4da06_21_limb_0, xor_rot_32_r_16_output_tmp_4da06_21_limb_1] =
        xor_rot_32_r_16::accumulate_constraints(
            &[
                eval!(context, triple_sum32_res_limb_0_col13),
                eval!(context, triple_sum32_res_limb_1_col14),
                eval!(context, input_limb_6_col7),
                eval!(context, input_limb_7_col8),
                eval!(context, enabler_col0),
                eval!(context, ms_8_bits_col15),
                eval!(context, ms_8_bits_col16),
                eval!(context, ms_8_bits_col17),
                eval!(context, ms_8_bits_col18),
                eval!(context, xor_col19),
                eval!(context, xor_col20),
                eval!(context, xor_col21),
                eval!(context, xor_col22),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    triple_sum_32::accumulate_constraints(
        &[
            eval!(context, input_limb_4_col5),
            eval!(context, input_limb_5_col6),
            eval!(context, xor_rot_32_r_16_output_tmp_4da06_21_limb_0),
            eval!(context, xor_rot_32_r_16_output_tmp_4da06_21_limb_1),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, enabler_col0),
            eval!(context, triple_sum32_res_limb_0_col23),
            eval!(context, triple_sum32_res_limb_1_col24),
        ],
        context,
        component_data,
        acc,
    );

    let [xor_rot_32_r_12_output_tmp_4da06_43_limb_0, xor_rot_32_r_12_output_tmp_4da06_43_limb_1] =
        xor_rot_32_r_12::accumulate_constraints(
            &[
                eval!(context, input_limb_2_col3),
                eval!(context, input_limb_3_col4),
                eval!(context, triple_sum32_res_limb_0_col23),
                eval!(context, triple_sum32_res_limb_1_col24),
                eval!(context, enabler_col0),
                eval!(context, ms_4_bits_col25),
                eval!(context, ms_4_bits_col26),
                eval!(context, ms_4_bits_col27),
                eval!(context, ms_4_bits_col28),
                eval!(context, xor_col29),
                eval!(context, xor_col30),
                eval!(context, xor_col31),
                eval!(context, xor_col32),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    triple_sum_32::accumulate_constraints(
        &[
            eval!(context, triple_sum32_res_limb_0_col13),
            eval!(context, triple_sum32_res_limb_1_col14),
            eval!(context, xor_rot_32_r_12_output_tmp_4da06_43_limb_0),
            eval!(context, xor_rot_32_r_12_output_tmp_4da06_43_limb_1),
            eval!(context, input_limb_10_col11),
            eval!(context, input_limb_11_col12),
            eval!(context, enabler_col0),
            eval!(context, triple_sum32_res_limb_0_col33),
            eval!(context, triple_sum32_res_limb_1_col34),
        ],
        context,
        component_data,
        acc,
    );

    let [xor_rot_32_r_8_output_tmp_4da06_65_limb_0, xor_rot_32_r_8_output_tmp_4da06_65_limb_1] =
        xor_rot_32_r_8::accumulate_constraints(
            &[
                eval!(context, triple_sum32_res_limb_0_col33),
                eval!(context, triple_sum32_res_limb_1_col34),
                eval!(context, xor_rot_32_r_16_output_tmp_4da06_21_limb_0),
                eval!(context, xor_rot_32_r_16_output_tmp_4da06_21_limb_1),
                eval!(context, enabler_col0),
                eval!(context, ms_8_bits_col35),
                eval!(context, ms_8_bits_col36),
                eval!(context, ms_8_bits_col37),
                eval!(context, ms_8_bits_col38),
                eval!(context, xor_col39),
                eval!(context, xor_col40),
                eval!(context, xor_col41),
                eval!(context, xor_col42),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    triple_sum_32::accumulate_constraints(
        &[
            eval!(context, triple_sum32_res_limb_0_col23),
            eval!(context, triple_sum32_res_limb_1_col24),
            eval!(context, xor_rot_32_r_8_output_tmp_4da06_65_limb_0),
            eval!(context, xor_rot_32_r_8_output_tmp_4da06_65_limb_1),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, enabler_col0),
            eval!(context, triple_sum32_res_limb_0_col43),
            eval!(context, triple_sum32_res_limb_1_col44),
        ],
        context,
        component_data,
        acc,
    );

    let [xor_rot_32_r_7_output_tmp_4da06_87_limb_0, xor_rot_32_r_7_output_tmp_4da06_87_limb_1] =
        xor_rot_32_r_7::accumulate_constraints(
            &[
                eval!(context, xor_rot_32_r_12_output_tmp_4da06_43_limb_0),
                eval!(context, xor_rot_32_r_12_output_tmp_4da06_43_limb_1),
                eval!(context, triple_sum32_res_limb_0_col43),
                eval!(context, triple_sum32_res_limb_1_col44),
                eval!(context, enabler_col0),
                eval!(context, ms_9_bits_col45),
                eval!(context, ms_9_bits_col46),
                eval!(context, ms_9_bits_col47),
                eval!(context, ms_9_bits_col48),
                eval!(context, xor_col49),
                eval!(context, xor_col50),
                eval!(context, xor_col51),
                eval!(context, xor_col52),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    // Yield BlakeG.
    let tuple_9 = &[
        eval!(context, 1139985212),
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
        eval!(context, input_limb_10_col11),
        eval!(context, input_limb_11_col12),
        eval!(context, triple_sum32_res_limb_0_col33),
        eval!(context, triple_sum32_res_limb_1_col34),
        eval!(context, xor_rot_32_r_7_output_tmp_4da06_87_limb_0),
        eval!(context, xor_rot_32_r_7_output_tmp_4da06_87_limb_1),
        eval!(context, triple_sum32_res_limb_0_col43),
        eval!(context, triple_sum32_res_limb_1_col44),
        eval!(context, xor_rot_32_r_8_output_tmp_4da06_65_limb_0),
        eval!(context, xor_rot_32_r_8_output_tmp_4da06_65_limb_1),
    ];
    let numerator_9 = eval!(context, -(enabler_col0));
    acc.add_to_relation(context, numerator_9, tuple_9);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "blake_g".to_string()
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
            qm31_from_u32s(1784919403, 510860207, 1992127139, 343880061),
            qm31_from_u32s(1180936792, 1450384302, 1388147362, 343880061),
            qm31_from_u32s(1113827613, 1316166574, 1321038498, 343880061),
            qm31_from_u32s(241305891, 1718819697, 448623205, 343880041),
            qm31_from_u32s(308415070, 1853037425, 515732069, 343880041),
            qm31_from_u32s(375524249, 1987255153, 582840933, 343880041),
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
            qm31_from_u32s(1542041464, 1153722820, 237275366, 1941984120),
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
        assert_eq!(result_value, BLAKE_G_SAMPLE_EVAL_RESULT)
    }
}
