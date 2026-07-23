// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 73;
pub const N_INTERACTION_COLUMNS: usize = 24;

pub const RELATION_USES_PER_ROW: [RelationUse; 5] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 3 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "RangeCheck_4_4_4_4", uses: 3 },
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
        offset1_col5,
        offset2_col6,
        dst_base_fp_col7,
        op0_base_fp_col8,
        op1_imm_col9,
        op1_base_fp_col10,
        res_add_col11,
        ap_update_add_1_col12,
        mem_dst_base_col13,
        mem0_base_col14,
        mem1_base_col15,
        dst_id_col16,
        dst_limb_0_col17,
        dst_limb_1_col18,
        dst_limb_2_col19,
        dst_limb_3_col20,
        dst_limb_4_col21,
        dst_limb_5_col22,
        dst_limb_6_col23,
        dst_limb_7_col24,
        dst_limb_8_col25,
        dst_limb_9_col26,
        dst_limb_10_col27,
        dst_limb_11_col28,
        dst_limb_12_col29,
        dst_limb_13_col30,
        dst_limb_14_col31,
        dst_limb_15_col32,
        dst_delta_ab_inv_col33,
        dst_delta_cd_inv_col34,
        op0_id_col35,
        op0_limb_0_col36,
        op0_limb_1_col37,
        op0_limb_2_col38,
        op0_limb_3_col39,
        op0_limb_4_col40,
        op0_limb_5_col41,
        op0_limb_6_col42,
        op0_limb_7_col43,
        op0_limb_8_col44,
        op0_limb_9_col45,
        op0_limb_10_col46,
        op0_limb_11_col47,
        op0_limb_12_col48,
        op0_limb_13_col49,
        op0_limb_14_col50,
        op0_limb_15_col51,
        op0_delta_ab_inv_col52,
        op0_delta_cd_inv_col53,
        op1_id_col54,
        op1_limb_0_col55,
        op1_limb_1_col56,
        op1_limb_2_col57,
        op1_limb_3_col58,
        op1_limb_4_col59,
        op1_limb_5_col60,
        op1_limb_6_col61,
        op1_limb_7_col62,
        op1_limb_8_col63,
        op1_limb_9_col64,
        op1_limb_10_col65,
        op1_limb_11_col66,
        op1_limb_12_col67,
        op1_limb_13_col68,
        op1_limb_14_col69,
        op1_limb_15_col70,
        op1_delta_ab_inv_col71,
        op1_delta_cd_inv_col72,
    ] = input.try_into().unwrap();

    // Enabler is a bit.
    let constraint_0_value = eval!(context, ((enabler_col0) * (enabler_col0)) - (enabler_col0));
    acc.add_constraint(context, constraint_0_value);

    let [
        decode_instruction_c67a5_output_tmp_48ee6_12_offset0,
        decode_instruction_c67a5_output_tmp_48ee6_12_offset1,
        decode_instruction_c67a5_output_tmp_48ee6_12_offset2,
        decode_instruction_c67a5_output_tmp_48ee6_12_op1_base_ap,
        decode_instruction_c67a5_output_tmp_48ee6_12_res_mul,
    ] = decode_instruction_c67a5::accumulate_constraints(
        &[
            eval!(context, input_pc_col1),
            eval!(context, enabler_col0),
            eval!(context, offset0_col4),
            eval!(context, offset1_col5),
            eval!(context, offset2_col6),
            eval!(context, dst_base_fp_col7),
            eval!(context, op0_base_fp_col8),
            eval!(context, op1_imm_col9),
            eval!(context, op1_base_fp_col10),
            eval!(context, res_add_col11),
            eval!(context, ap_update_add_1_col12),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    // Either flag op1_imm is off or offset2 is equal to 1.
    let constraint_2_value = eval!(
        context,
        (op1_imm_col9) * ((decode_instruction_c67a5_output_tmp_48ee6_12_offset2) - (1))
    );
    acc.add_constraint(context, constraint_2_value);

    // mem_dst_base.
    let constraint_3_value = eval!(
        context,
        (mem_dst_base_col13)
            - (((dst_base_fp_col7) * (input_fp_col3))
                + (((1) - (dst_base_fp_col7)) * (input_ap_col2)))
    );
    acc.add_constraint(context, constraint_3_value);

    // mem0_base.
    let constraint_4_value = eval!(
        context,
        (mem0_base_col14)
            - (((op0_base_fp_col8) * (input_fp_col3))
                + (((1) - (op0_base_fp_col8)) * (input_ap_col2)))
    );
    acc.add_constraint(context, constraint_4_value);

    // mem1_base.
    let constraint_5_value = eval!(
        context,
        (mem1_base_col15)
            - ((((op1_base_fp_col10) * (input_fp_col3))
                + ((decode_instruction_c67a5_output_tmp_48ee6_12_op1_base_ap) * (input_ap_col2)))
                + ((op1_imm_col9) * (input_pc_col1)))
    );
    acc.add_constraint(context, constraint_5_value);

    let [
        qm_31_read_reduced_output_tmp_48ee6_18_limb_0,
        qm_31_read_reduced_output_tmp_48ee6_18_limb_1,
        qm_31_read_reduced_output_tmp_48ee6_18_limb_2,
        qm_31_read_reduced_output_tmp_48ee6_18_limb_3,
    ] = qm_31_read_reduced::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col13) + (decode_instruction_c67a5_output_tmp_48ee6_12_offset0)
            ),
            eval!(context, enabler_col0),
            eval!(context, dst_id_col16),
            eval!(context, dst_limb_0_col17),
            eval!(context, dst_limb_1_col18),
            eval!(context, dst_limb_2_col19),
            eval!(context, dst_limb_3_col20),
            eval!(context, dst_limb_4_col21),
            eval!(context, dst_limb_5_col22),
            eval!(context, dst_limb_6_col23),
            eval!(context, dst_limb_7_col24),
            eval!(context, dst_limb_8_col25),
            eval!(context, dst_limb_9_col26),
            eval!(context, dst_limb_10_col27),
            eval!(context, dst_limb_11_col28),
            eval!(context, dst_limb_12_col29),
            eval!(context, dst_limb_13_col30),
            eval!(context, dst_limb_14_col31),
            eval!(context, dst_limb_15_col32),
            eval!(context, dst_delta_ab_inv_col33),
            eval!(context, dst_delta_cd_inv_col34),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [
        qm_31_read_reduced_output_tmp_48ee6_24_limb_0,
        qm_31_read_reduced_output_tmp_48ee6_24_limb_1,
        qm_31_read_reduced_output_tmp_48ee6_24_limb_2,
        qm_31_read_reduced_output_tmp_48ee6_24_limb_3,
    ] = qm_31_read_reduced::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col14) + (decode_instruction_c67a5_output_tmp_48ee6_12_offset1)
            ),
            eval!(context, enabler_col0),
            eval!(context, op0_id_col35),
            eval!(context, op0_limb_0_col36),
            eval!(context, op0_limb_1_col37),
            eval!(context, op0_limb_2_col38),
            eval!(context, op0_limb_3_col39),
            eval!(context, op0_limb_4_col40),
            eval!(context, op0_limb_5_col41),
            eval!(context, op0_limb_6_col42),
            eval!(context, op0_limb_7_col43),
            eval!(context, op0_limb_8_col44),
            eval!(context, op0_limb_9_col45),
            eval!(context, op0_limb_10_col46),
            eval!(context, op0_limb_11_col47),
            eval!(context, op0_limb_12_col48),
            eval!(context, op0_limb_13_col49),
            eval!(context, op0_limb_14_col50),
            eval!(context, op0_limb_15_col51),
            eval!(context, op0_delta_ab_inv_col52),
            eval!(context, op0_delta_cd_inv_col53),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [
        qm_31_read_reduced_output_tmp_48ee6_30_limb_0,
        qm_31_read_reduced_output_tmp_48ee6_30_limb_1,
        qm_31_read_reduced_output_tmp_48ee6_30_limb_2,
        qm_31_read_reduced_output_tmp_48ee6_30_limb_3,
    ] = qm_31_read_reduced::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col15) + (decode_instruction_c67a5_output_tmp_48ee6_12_offset2)
            ),
            eval!(context, enabler_col0),
            eval!(context, op1_id_col54),
            eval!(context, op1_limb_0_col55),
            eval!(context, op1_limb_1_col56),
            eval!(context, op1_limb_2_col57),
            eval!(context, op1_limb_3_col58),
            eval!(context, op1_limb_4_col59),
            eval!(context, op1_limb_5_col60),
            eval!(context, op1_limb_6_col61),
            eval!(context, op1_limb_7_col62),
            eval!(context, op1_limb_8_col63),
            eval!(context, op1_limb_9_col64),
            eval!(context, op1_limb_10_col65),
            eval!(context, op1_limb_11_col66),
            eval!(context, op1_limb_12_col67),
            eval!(context, op1_limb_13_col68),
            eval!(context, op1_limb_14_col69),
            eval!(context, op1_limb_15_col70),
            eval!(context, op1_delta_ab_inv_col71),
            eval!(context, op1_delta_cd_inv_col72),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    // dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul).
    let constraint_9_value = eval!(
        context,
        ((qm_31_read_reduced_output_tmp_48ee6_18_limb_0)
            - (((((((qm_31_read_reduced_output_tmp_48ee6_24_limb_0)
                * (qm_31_read_reduced_output_tmp_48ee6_30_limb_0))
                - ((qm_31_read_reduced_output_tmp_48ee6_24_limb_1)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_1)))
                + ((2)
                    * (((qm_31_read_reduced_output_tmp_48ee6_24_limb_2)
                        * (qm_31_read_reduced_output_tmp_48ee6_30_limb_2))
                        - ((qm_31_read_reduced_output_tmp_48ee6_24_limb_3)
                            * (qm_31_read_reduced_output_tmp_48ee6_30_limb_3)))))
                - ((qm_31_read_reduced_output_tmp_48ee6_24_limb_2)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_3)))
                - ((qm_31_read_reduced_output_tmp_48ee6_24_limb_3)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_2)))
                * (decode_instruction_c67a5_output_tmp_48ee6_12_res_mul)))
            - (((qm_31_read_reduced_output_tmp_48ee6_24_limb_0)
                + (qm_31_read_reduced_output_tmp_48ee6_30_limb_0))
                * (res_add_col11))
    );
    acc.add_constraint(context, constraint_9_value);

    // dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul).
    let constraint_10_value = eval!(
        context,
        ((qm_31_read_reduced_output_tmp_48ee6_18_limb_1)
            - (((((((qm_31_read_reduced_output_tmp_48ee6_24_limb_0)
                * (qm_31_read_reduced_output_tmp_48ee6_30_limb_1))
                + ((qm_31_read_reduced_output_tmp_48ee6_24_limb_1)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_0)))
                + ((2)
                    * (((qm_31_read_reduced_output_tmp_48ee6_24_limb_2)
                        * (qm_31_read_reduced_output_tmp_48ee6_30_limb_3))
                        + ((qm_31_read_reduced_output_tmp_48ee6_24_limb_3)
                            * (qm_31_read_reduced_output_tmp_48ee6_30_limb_2)))))
                + ((qm_31_read_reduced_output_tmp_48ee6_24_limb_2)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_2)))
                - ((qm_31_read_reduced_output_tmp_48ee6_24_limb_3)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_3)))
                * (decode_instruction_c67a5_output_tmp_48ee6_12_res_mul)))
            - (((qm_31_read_reduced_output_tmp_48ee6_24_limb_1)
                + (qm_31_read_reduced_output_tmp_48ee6_30_limb_1))
                * (res_add_col11))
    );
    acc.add_constraint(context, constraint_10_value);

    // dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul).
    let constraint_11_value = eval!(
        context,
        ((qm_31_read_reduced_output_tmp_48ee6_18_limb_2)
            - ((((((qm_31_read_reduced_output_tmp_48ee6_24_limb_0)
                * (qm_31_read_reduced_output_tmp_48ee6_30_limb_2))
                - ((qm_31_read_reduced_output_tmp_48ee6_24_limb_1)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_3)))
                + ((qm_31_read_reduced_output_tmp_48ee6_24_limb_2)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_0)))
                - ((qm_31_read_reduced_output_tmp_48ee6_24_limb_3)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_1)))
                * (decode_instruction_c67a5_output_tmp_48ee6_12_res_mul)))
            - (((qm_31_read_reduced_output_tmp_48ee6_24_limb_2)
                + (qm_31_read_reduced_output_tmp_48ee6_30_limb_2))
                * (res_add_col11))
    );
    acc.add_constraint(context, constraint_11_value);

    // dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul).
    let constraint_12_value = eval!(
        context,
        ((qm_31_read_reduced_output_tmp_48ee6_18_limb_3)
            - ((((((qm_31_read_reduced_output_tmp_48ee6_24_limb_0)
                * (qm_31_read_reduced_output_tmp_48ee6_30_limb_3))
                + ((qm_31_read_reduced_output_tmp_48ee6_24_limb_1)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_2)))
                + ((qm_31_read_reduced_output_tmp_48ee6_24_limb_2)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_1)))
                + ((qm_31_read_reduced_output_tmp_48ee6_24_limb_3)
                    * (qm_31_read_reduced_output_tmp_48ee6_30_limb_0)))
                * (decode_instruction_c67a5_output_tmp_48ee6_12_res_mul)))
            - (((qm_31_read_reduced_output_tmp_48ee6_24_limb_3)
                + (qm_31_read_reduced_output_tmp_48ee6_30_limb_3))
                * (res_add_col11))
    );
    acc.add_constraint(context, constraint_12_value);

    // Use Opcodes.
    let tuple_13 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col1),
        eval!(context, input_ap_col2),
        eval!(context, input_fp_col3),
    ];
    let numerator_13 = eval!(context, enabler_col0);
    acc.add_to_relation(context, numerator_13, tuple_13);

    // Yield Opcodes.
    let tuple_14 = &[
        eval!(context, 428564188),
        eval!(context, ((input_pc_col1) + (1)) + (op1_imm_col9)),
        eval!(context, (input_ap_col2) + (ap_update_add_1_col12)),
        eval!(context, input_fp_col3),
    ];
    let numerator_14 = eval!(context, -(enabler_col0));
    acc.add_to_relation(context, numerator_14, tuple_14);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "qm_31_add_mul_opcode".to_string()
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
            qm31_from_u32s(442633428, 2121472881, 649949797, 343880041),
            qm31_from_u32s(509742607, 108206962, 717058662, 343880041),
            qm31_from_u32s(576851786, 242424690, 784167526, 343880041),
            qm31_from_u32s(643960965, 376642418, 851276390, 343880041),
            qm31_from_u32s(711070144, 510860146, 918385254, 343880041),
            qm31_from_u32s(778179323, 645077874, 985494118, 343880041),
            qm31_from_u32s(845288502, 779295602, 1052602982, 343880041),
            qm31_from_u32s(375831434, 1987255333, 582841113, 343880101),
            qm31_from_u32s(308722255, 1853037605, 515732249, 343880101),
            qm31_from_u32s(510049792, 108207142, 717058842, 343880101),
            qm31_from_u32s(442940613, 2121473061, 649949977, 343880101),
            qm31_from_u32s(644268150, 376642598, 851276570, 343880101),
            qm31_from_u32s(577158971, 242424870, 784167706, 343880101),
            qm31_from_u32s(778486508, 645078054, 985494298, 343880101),
            qm31_from_u32s(711377329, 510860326, 918385434, 343880101),
            qm31_from_u32s(912704866, 913513510, 1119712026, 343880101),
            qm31_from_u32s(845595687, 779295782, 1052603162, 343880101),
            qm31_from_u32s(1046820829, 1181948906, 1253929694, 343880081),
            qm31_from_u32s(1113930008, 1316166634, 1321038558, 343880081),
            qm31_from_u32s(912602471, 913513450, 1119711966, 343880081),
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
            qm31_from_u32s(1206495569, 482634180, 2049214693, 1941984119),
            qm31_from_u32s(736731316, 1690593731, 1579452644, 1941984119),
            qm31_from_u32s(803840495, 1824811459, 1646561508, 1941984119),
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
        assert_eq!(result_value, QM_31_ADD_MUL_OPCODE_SAMPLE_EVAL_RESULT)
    }
}
