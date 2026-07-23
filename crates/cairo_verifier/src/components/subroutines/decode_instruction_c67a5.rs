// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyInstruction", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        decode_instruction_c67a5_input_pc,
        enabler,
        offset0_col0,
        offset1_col1,
        offset2_col2,
        dst_base_fp_col3,
        op0_base_fp_col4,
        op1_imm_col5,
        op1_base_fp_col6,
        res_add_col7,
        ap_update_add_1_col8,
    ] = input.try_into().unwrap();

    // Flag dst_base_fp is a bit.
    let constraint_0_value = eval!(context, (dst_base_fp_col3) * ((1) - (dst_base_fp_col3)));
    acc.add_constraint(context, constraint_0_value);

    // Flag op0_base_fp is a bit.
    let constraint_1_value = eval!(context, (op0_base_fp_col4) * ((1) - (op0_base_fp_col4)));
    acc.add_constraint(context, constraint_1_value);

    // Flag op1_imm is a bit.
    let constraint_2_value = eval!(context, (op1_imm_col5) * ((1) - (op1_imm_col5)));
    acc.add_constraint(context, constraint_2_value);

    // Flag op1_base_fp is a bit.
    let constraint_3_value = eval!(context, (op1_base_fp_col6) * ((1) - (op1_base_fp_col6)));
    acc.add_constraint(context, constraint_3_value);

    let op1_base_ap_tmp_c67a5_9 = eval!(context, ((1) - (op1_imm_col5)) - (op1_base_fp_col6));

    // Flag op1_base_ap is a bit.
    let constraint_5_value =
        eval!(context, (op1_base_ap_tmp_c67a5_9) * ((1) - (op1_base_ap_tmp_c67a5_9)));
    acc.add_constraint(context, constraint_5_value);

    // Flag res_add is a bit.
    let constraint_6_value = eval!(context, (res_add_col7) * ((1) - (res_add_col7)));
    acc.add_constraint(context, constraint_6_value);

    // Flag ap_update_add_1 is a bit.
    let constraint_7_value =
        eval!(context, (ap_update_add_1_col8) * ((1) - (ap_update_add_1_col8)));
    acc.add_constraint(context, constraint_7_value);

    // Use VerifyInstruction.
    let tuple_8 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_c67a5_input_pc),
        eval!(context, offset0_col0),
        eval!(context, offset1_col1),
        eval!(context, offset2_col2),
        eval!(
            context,
            ((((((dst_base_fp_col3) * (8)) + ((op0_base_fp_col4) * (16)))
                + ((op1_imm_col5) * (32)))
                + ((op1_base_fp_col6) * (64)))
                + ((op1_base_ap_tmp_c67a5_9) * (128)))
                + ((res_add_col7) * (256))
        ),
        eval!(context, (((1) - (res_add_col7)) + ((ap_update_add_1_col8) * (32))) + (256)),
        eval!(context, 3),
    ];
    let numerator_8 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_8, tuple_8);
    vec![
        eval!(context, (offset0_col0) - (32768)),
        eval!(context, (offset1_col1) - (32768)),
        eval!(context, (offset2_col2) - (32768)),
        eval!(context, op1_base_ap_tmp_c67a5_9),
        eval!(context, (1) - (res_add_col7)),
    ]
}
