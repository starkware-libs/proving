// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 4 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 4 },
    RelationUse { relation_id: "RangeCheck_7_2_5", uses: 1 },
    RelationUse { relation_id: "VerifyInstruction", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        decode_blake_opcode_input_pc,
        decode_blake_opcode_input_ap,
        decode_blake_opcode_input_fp,
        enabler,
        offset0_col0,
        offset1_col1,
        offset2_col2,
        dst_base_fp_col3,
        op0_base_fp_col4,
        op1_base_fp_col5,
        ap_update_add_1_col6,
        opcode_extension_col7,
        mem0_base_col8,
        op0_id_col9,
        op0_limb_0_col10,
        op0_limb_1_col11,
        op0_limb_2_col12,
        op0_limb_3_col13,
        partial_limb_msb_col14,
        mem1_base_col15,
        op1_id_col16,
        op1_limb_0_col17,
        op1_limb_1_col18,
        op1_limb_2_col19,
        op1_limb_3_col20,
        partial_limb_msb_col21,
        ap_id_col22,
        ap_limb_0_col23,
        ap_limb_1_col24,
        ap_limb_2_col25,
        ap_limb_3_col26,
        partial_limb_msb_col27,
        mem_dst_base_col28,
        low_16_bits_col29,
        high_16_bits_col30,
        low_7_ms_bits_col31,
        high_14_ms_bits_col32,
        high_5_ms_bits_col33,
        dst_id_col34,
    ] = input.try_into().unwrap();

    let [
        decode_instruction_30129_output_tmp_c2bd8_9_offset0,
        decode_instruction_30129_output_tmp_c2bd8_9_offset1,
        decode_instruction_30129_output_tmp_c2bd8_9_offset2,
        decode_instruction_30129_output_tmp_c2bd8_9_op1_base_ap,
    ] = decode_instruction_30129::accumulate_constraints(
        &[
            eval!(context, decode_blake_opcode_input_pc),
            eval!(context, enabler),
            eval!(context, offset0_col0),
            eval!(context, offset1_col1),
            eval!(context, offset2_col2),
            eval!(context, dst_base_fp_col3),
            eval!(context, op0_base_fp_col4),
            eval!(context, op1_base_fp_col5),
            eval!(context, ap_update_add_1_col6),
            eval!(context, opcode_extension_col7),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    // OpcodeExtension is either Blake or BlakeFinalize.
    let constraint_1_value =
        eval!(context, ((opcode_extension_col7) - (1)) * ((opcode_extension_col7) - (2)));
    acc.add_constraint(context, constraint_1_value);

    // mem0_base.
    let constraint_2_value = eval!(
        context,
        (mem0_base_col8)
            - (((op0_base_fp_col4) * (decode_blake_opcode_input_fp))
                + (((1) - (op0_base_fp_col4)) * (decode_blake_opcode_input_ap)))
    );
    acc.add_constraint(context, constraint_2_value);

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col8) + (decode_instruction_30129_output_tmp_c2bd8_9_offset1)
            ),
            eval!(context, enabler),
            eval!(context, op0_id_col9),
            eval!(context, op0_limb_0_col10),
            eval!(context, op0_limb_1_col11),
            eval!(context, op0_limb_2_col12),
            eval!(context, op0_limb_3_col13),
            eval!(context, partial_limb_msb_col14),
        ],
        context,
        component_data,
        acc,
    );

    // mem1_base.
    let constraint_4_value = eval!(
        context,
        (mem1_base_col15)
            - (((op1_base_fp_col5) * (decode_blake_opcode_input_fp))
                + ((decode_instruction_30129_output_tmp_c2bd8_9_op1_base_ap)
                    * (decode_blake_opcode_input_ap)))
    );
    acc.add_constraint(context, constraint_4_value);

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col15) + (decode_instruction_30129_output_tmp_c2bd8_9_offset2)
            ),
            eval!(context, enabler),
            eval!(context, op1_id_col16),
            eval!(context, op1_limb_0_col17),
            eval!(context, op1_limb_1_col18),
            eval!(context, op1_limb_2_col19),
            eval!(context, op1_limb_3_col20),
            eval!(context, partial_limb_msb_col21),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, decode_blake_opcode_input_ap),
            eval!(context, enabler),
            eval!(context, ap_id_col22),
            eval!(context, ap_limb_0_col23),
            eval!(context, ap_limb_1_col24),
            eval!(context, ap_limb_2_col25),
            eval!(context, ap_limb_3_col26),
            eval!(context, partial_limb_msb_col27),
        ],
        context,
        component_data,
        acc,
    );

    // mem_dst_base.
    let constraint_7_value = eval!(
        context,
        (mem_dst_base_col28)
            - (((dst_base_fp_col3) * (decode_blake_opcode_input_fp))
                + (((1) - (dst_base_fp_col3)) * (decode_blake_opcode_input_ap)))
    );
    acc.add_constraint(context, constraint_7_value);

    read_u_32::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col28) + (decode_instruction_30129_output_tmp_c2bd8_9_offset0)
            ),
            eval!(context, enabler),
            eval!(context, low_16_bits_col29),
            eval!(context, high_16_bits_col30),
            eval!(context, low_7_ms_bits_col31),
            eval!(context, high_14_ms_bits_col32),
            eval!(context, high_5_ms_bits_col33),
            eval!(context, dst_id_col34),
        ],
        context,
        component_data,
        acc,
    );
    vec![
        eval!(
            context,
            (((op0_limb_0_col10) + ((op0_limb_1_col11) * (512))) + ((op0_limb_2_col12) * (262144)))
                + ((op0_limb_3_col13) * (134217728))
        ),
        eval!(
            context,
            (((op1_limb_0_col17) + ((op1_limb_1_col18) * (512))) + ((op1_limb_2_col19) * (262144)))
                + ((op1_limb_3_col20) * (134217728))
        ),
        eval!(
            context,
            (((ap_limb_0_col23) + ((ap_limb_1_col24) * (512))) + ((ap_limb_2_col25) * (262144)))
                + ((ap_limb_3_col26) * (134217728))
        ),
        eval!(context, (opcode_extension_col7) - (1)),
    ]
}
