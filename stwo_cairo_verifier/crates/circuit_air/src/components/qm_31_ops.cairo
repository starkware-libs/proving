// This file was created by the AIR team.

use crate::prelude::*;

pub const N_TRACE_COLUMNS: usize = 12;
pub const N_INTERACTION_COLUMNS: usize = 8;
pub const RELATION_USES_PER_ROW: [(felt252, u32); 1] = [('Gate', 2)];

#[derive(Drop, Serde, Copy)]
pub struct Claim {
    pub log_size: u32,
}

pub impl ClaimImpl of ClaimTrait<Claim> {
    fn mix_into(self: @Claim, ref channel: Channel) {
        channel.mix_u64((*(self.log_size)).into());
    }

    fn accumulate_relation_uses(self: @Claim, ref relation_uses: RelationUsesDict) {
        accumulate_relation_uses(ref relation_uses, RELATION_USES_PER_ROW.span(), *self.log_size);
    }
}


#[derive(Drop)]
pub struct Component {
    pub claim: Claim,
    pub claimed_sum: QM31,
    pub common_lookup_elements: CommonLookupElements,
}

pub impl NewComponentImpl of NewComponent<Component> {
    type Claim = Claim;

    fn new(
        claim: @Claim, claimed_sum: QM31, common_lookup_elements: @CommonLookupElements,
    ) -> Component {
        Component {
            claim: *claim, claimed_sum, common_lookup_elements: common_lookup_elements.clone(),
        }
    }
}

pub impl AirComponentImpl of AirComponent<Component> {
    fn evaluate_constraints_at_point(
        self: @Component,
        ref sum: QM31,
        ref preprocessed_mask_values: PreprocessedMaskValues,
        ref trace_mask_values: ColumnSpan<Span<QM31>>,
        ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
        random_coeff: QM31,
    ) {
        let log_size = *(self.claim.log_size);
        let claimed_sum = *self.claimed_sum;
        let column_size = m31(pow2(log_size));
        let mut gate_sum_0: QM31 = Zero::zero();
        let mut numerator_0: QM31 = Zero::zero();
        let mut gate_sum_1: QM31 = Zero::zero();
        let mut numerator_1: QM31 = Zero::zero();
        let mut gate_sum_2: QM31 = Zero::zero();
        let mut numerator_2: QM31 = Zero::zero();
        let qm_31_ops_add_flag = preprocessed_mask_values.get_and_mark_used(QM_31_OPS_ADD_FLAG_IDX);
        let qm_31_ops_mul_flag = preprocessed_mask_values.get_and_mark_used(QM_31_OPS_MUL_FLAG_IDX);
        let qm_31_ops_pointwise_mul_flag = preprocessed_mask_values
            .get_and_mark_used(QM_31_OPS_POINTWISE_MUL_FLAG_IDX);
        let qm_31_ops_sub_flag = preprocessed_mask_values.get_and_mark_used(QM_31_OPS_SUB_FLAG_IDX);
        let qm_31_ops_in_0_address = preprocessed_mask_values
            .get_and_mark_used(QM_31_OPS_IN_0_ADDRESS_IDX);
        let qm_31_ops_in_1_address = preprocessed_mask_values
            .get_and_mark_used(QM_31_OPS_IN_1_ADDRESS_IDX);
        let qm_31_ops_out_address = preprocessed_mask_values
            .get_and_mark_used(QM_31_OPS_OUT_ADDRESS_IDX);
        let qm_31_ops_mults = preprocessed_mask_values.get_and_mark_used(QM_31_OPS_MULTS_IDX);

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
        ]: [Span<QM31>; 12] =
            (*trace_mask_values
            .multi_pop_front()
            .unwrap())
            .unbox();
        let [input_op0_limb0_col0]: [QM31; 1] = (*input_op0_limb0_col0.try_into().unwrap()).unbox();
        let [input_op0_limb1_col1]: [QM31; 1] = (*input_op0_limb1_col1.try_into().unwrap()).unbox();
        let [input_op0_limb2_col2]: [QM31; 1] = (*input_op0_limb2_col2.try_into().unwrap()).unbox();
        let [input_op0_limb3_col3]: [QM31; 1] = (*input_op0_limb3_col3.try_into().unwrap()).unbox();
        let [input_op1_limb0_col4]: [QM31; 1] = (*input_op1_limb0_col4.try_into().unwrap()).unbox();
        let [input_op1_limb1_col5]: [QM31; 1] = (*input_op1_limb1_col5.try_into().unwrap()).unbox();
        let [input_op1_limb2_col6]: [QM31; 1] = (*input_op1_limb2_col6.try_into().unwrap()).unbox();
        let [input_op1_limb3_col7]: [QM31; 1] = (*input_op1_limb3_col7.try_into().unwrap()).unbox();
        let [input_dst_limb0_col8]: [QM31; 1] = (*input_dst_limb0_col8.try_into().unwrap()).unbox();
        let [input_dst_limb1_col9]: [QM31; 1] = (*input_dst_limb1_col9.try_into().unwrap()).unbox();
        let [input_dst_limb2_col10]: [QM31; 1] = (*input_dst_limb2_col10.try_into().unwrap())
            .unbox();
        let [input_dst_limb3_col11]: [QM31; 1] = (*input_dst_limb3_col11.try_into().unwrap())
            .unbox();

        core::internal::revoke_ap_tracking();

        // Constraint -
        let constraint_eval = ((input_dst_limb0_col8
            - (((((((((input_op0_limb0_col0 * input_op1_limb0_col4)
                - (input_op0_limb1_col1 * input_op1_limb1_col5))
                + (qm31_const::<2, 0, 0, 0>()
                    * ((input_op0_limb2_col2 * input_op1_limb2_col6)
                        - (input_op0_limb3_col3 * input_op1_limb3_col7))))
                - (input_op0_limb2_col2 * input_op1_limb3_col7))
                - (input_op0_limb3_col3 * input_op1_limb2_col6))
                * qm_31_ops_mul_flag)
                + ((input_op0_limb0_col0 + input_op1_limb0_col4) * qm_31_ops_add_flag))
                + ((input_op0_limb0_col0 - input_op1_limb0_col4) * qm_31_ops_sub_flag))
                + ((input_op0_limb0_col0 * input_op1_limb0_col4) * qm_31_ops_pointwise_mul_flag))));
        sum = sum * random_coeff + constraint_eval;

        // Constraint -
        let constraint_eval = ((input_dst_limb1_col9
            - (((((((((input_op0_limb0_col0 * input_op1_limb1_col5)
                + (input_op0_limb1_col1 * input_op1_limb0_col4))
                + (qm31_const::<2, 0, 0, 0>()
                    * ((input_op0_limb2_col2 * input_op1_limb3_col7)
                        + (input_op0_limb3_col3 * input_op1_limb2_col6))))
                + (input_op0_limb2_col2 * input_op1_limb2_col6))
                - (input_op0_limb3_col3 * input_op1_limb3_col7))
                * qm_31_ops_mul_flag)
                + ((input_op0_limb1_col1 + input_op1_limb1_col5) * qm_31_ops_add_flag))
                + ((input_op0_limb1_col1 - input_op1_limb1_col5) * qm_31_ops_sub_flag))
                + ((input_op0_limb1_col1 * input_op1_limb1_col5) * qm_31_ops_pointwise_mul_flag))));
        sum = sum * random_coeff + constraint_eval;

        // Constraint -
        let constraint_eval = ((input_dst_limb2_col10
            - ((((((((input_op0_limb0_col0 * input_op1_limb2_col6)
                - (input_op0_limb1_col1 * input_op1_limb3_col7))
                + (input_op0_limb2_col2 * input_op1_limb0_col4))
                - (input_op0_limb3_col3 * input_op1_limb1_col5))
                * qm_31_ops_mul_flag)
                + ((input_op0_limb2_col2 + input_op1_limb2_col6) * qm_31_ops_add_flag))
                + ((input_op0_limb2_col2 - input_op1_limb2_col6) * qm_31_ops_sub_flag))
                + ((input_op0_limb2_col2 * input_op1_limb2_col6) * qm_31_ops_pointwise_mul_flag))));
        sum = sum * random_coeff + constraint_eval;

        // Constraint -
        let constraint_eval = ((input_dst_limb3_col11
            - ((((((((input_op0_limb0_col0 * input_op1_limb3_col7)
                + (input_op0_limb1_col1 * input_op1_limb2_col6))
                + (input_op0_limb2_col2 * input_op1_limb1_col5))
                + (input_op0_limb3_col3 * input_op1_limb0_col4))
                * qm_31_ops_mul_flag)
                + ((input_op0_limb3_col3 + input_op1_limb3_col7) * qm_31_ops_add_flag))
                + ((input_op0_limb3_col3 - input_op1_limb3_col7) * qm_31_ops_sub_flag))
                + ((input_op0_limb3_col3 * input_op1_limb3_col7) * qm_31_ops_pointwise_mul_flag))));
        sum = sum * random_coeff + constraint_eval;

        gate_sum_0 = self
            .common_lookup_elements
            .combine_qm31(
                [
                    qm31_const::<378353459, 0, 0, 0>(), qm_31_ops_in_0_address,
                    input_op0_limb0_col0, input_op0_limb1_col1, input_op0_limb2_col2,
                    input_op0_limb3_col3,
                ]
                    .span(),
            );
        numerator_0 = qm31_const::<1, 0, 0, 0>();

        gate_sum_1 = self
            .common_lookup_elements
            .combine_qm31(
                [
                    qm31_const::<378353459, 0, 0, 0>(), qm_31_ops_in_1_address,
                    input_op1_limb0_col4, input_op1_limb1_col5, input_op1_limb2_col6,
                    input_op1_limb3_col7,
                ]
                    .span(),
            );
        numerator_1 = qm31_const::<1, 0, 0, 0>();

        gate_sum_2 = self
            .common_lookup_elements
            .combine_qm31(
                [
                    qm31_const::<378353459, 0, 0, 0>(), qm_31_ops_out_address, input_dst_limb0_col8,
                    input_dst_limb1_col9, input_dst_limb2_col10, input_dst_limb3_col11,
                ]
                    .span(),
            );
        numerator_2 = qm_31_ops_mults;

        lookup_constraints(
            ref sum,
            random_coeff,
            claimed_sum,
            numerator_0,
            numerator_1,
            numerator_2,
            column_size,
            ref interaction_trace_mask_values,
            gate_sum_0,
            gate_sum_1,
            gate_sum_2,
        );
    }
}


fn lookup_constraints(
    ref sum: QM31,
    random_coeff: QM31,
    claimed_sum: QM31,
    numerator_0: QM31,
    numerator_1: QM31,
    numerator_2: QM31,
    column_size: M31,
    ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
    gate_sum_0: QM31,
    gate_sum_1: QM31,
    gate_sum_2: QM31,
) {
    let [
        trace_2_col0,
        trace_2_col1,
        trace_2_col2,
        trace_2_col3,
        trace_2_col4,
        trace_2_col5,
        trace_2_col6,
        trace_2_col7,
    ]: [Span<QM31>; 8] =
        (*interaction_trace_mask_values
        .multi_pop_front()
        .unwrap())
        .unbox();

    let [trace_2_col0]: [QM31; 1] = (*trace_2_col0.try_into().unwrap()).unbox();
    let [trace_2_col1]: [QM31; 1] = (*trace_2_col1.try_into().unwrap()).unbox();
    let [trace_2_col2]: [QM31; 1] = (*trace_2_col2.try_into().unwrap()).unbox();
    let [trace_2_col3]: [QM31; 1] = (*trace_2_col3.try_into().unwrap()).unbox();
    let [trace_2_col4_neg1, trace_2_col4]: [QM31; 2] = (*trace_2_col4.try_into().unwrap()).unbox();
    let [trace_2_col5_neg1, trace_2_col5]: [QM31; 2] = (*trace_2_col5.try_into().unwrap()).unbox();
    let [trace_2_col6_neg1, trace_2_col6]: [QM31; 2] = (*trace_2_col6.try_into().unwrap()).unbox();
    let [trace_2_col7_neg1, trace_2_col7]: [QM31; 2] = (*trace_2_col7.try_into().unwrap()).unbox();

    core::internal::revoke_ap_tracking();

    let constraint_eval = (((QM31Impl::from_partial_evals(
        [trace_2_col0, trace_2_col1, trace_2_col2, trace_2_col3],
    ))
        * gate_sum_0
        * gate_sum_1)
        - (gate_sum_0 * numerator_1)
        - (gate_sum_1 * numerator_0));
    sum = sum * random_coeff + constraint_eval;

    let constraint_eval = (((QM31Impl::from_partial_evals(
        [trace_2_col4, trace_2_col5, trace_2_col6, trace_2_col7],
    )
        - QM31Impl::from_partial_evals([trace_2_col0, trace_2_col1, trace_2_col2, trace_2_col3])
        - QM31Impl::from_partial_evals(
            [trace_2_col4_neg1, trace_2_col5_neg1, trace_2_col6_neg1, trace_2_col7_neg1],
        )
        + (claimed_sum * (column_size.inverse().into())))
        * gate_sum_2)
        + numerator_2);
    sum = sum * random_coeff + constraint_eval;
}
#[cfg(and(test, feature: "qm31_opcode"))]
mod tests {
    use core::array::ArrayImpl;
    use core::num::traits::Zero;
    use stwo_constraint_framework::AirComponent;
    #[allow(unused_imports)]
    use stwo_constraint_framework::test_utils::{make_interaction_trace, preprocessed_mask_add};
    #[allow(unused_imports)]
    use stwo_constraint_framework::{
        LookupElementsTrait, PreprocessedMaskValues, PreprocessedMaskValuesTrait,
    };
    use stwo_verifier_core::fields::qm31::{QM31, QM31Impl, QM31Trait, qm31_const};
    use crate::components::sample_evaluations::*;
    #[allow(unused_imports)]
    use crate::preprocessed_columns::*;
    use super::{Claim, Component};

    #[test]
    fn test_evaluation_result() {
        let component = Component {
            claim: Claim { log_size: 15 },
            claimed_sum: qm31_const::<1398335417, 314974026, 1722107152, 821933968>(),
            common_lookup_elements: LookupElementsTrait::from_z_alpha(
                qm31_const::<445623802, 202571636, 1360224996, 131355117>(),
                qm31_const::<476823935, 939223384, 62486082, 122423602>(),
            ),
        };
        let mut sum: QM31 = Zero::zero();

        let mut preprocessed_trace = PreprocessedMaskValues { values: Default::default() };
        let mut preprocessed_trace = preprocessed_mask_add(
            preprocessed_trace,
            QM_31_OPS_ADD_FLAG_IDX,
            qm31_const::<2008763856, 668586075, 986260244, 1154698137>(),
        );
        let mut preprocessed_trace = preprocessed_mask_add(
            preprocessed_trace,
            QM_31_OPS_MUL_FLAG_IDX,
            qm31_const::<753532226, 1668588607, 2021383940, 940498869>(),
        );
        let mut preprocessed_trace = preprocessed_mask_add(
            preprocessed_trace,
            QM_31_OPS_POINTWISE_MUL_FLAG_IDX,
            qm31_const::<1658621201, 1657657148, 1342332119, 2034171678>(),
        );
        let mut preprocessed_trace = preprocessed_mask_add(
            preprocessed_trace,
            QM_31_OPS_SUB_FLAG_IDX,
            qm31_const::<346603561, 1505146370, 374195948, 1196742422>(),
        );
        let mut preprocessed_trace = preprocessed_mask_add(
            preprocessed_trace,
            QM_31_OPS_IN_0_ADDRESS_IDX,
            qm31_const::<1444382797, 1354185417, 705047099, 132239089>(),
        );
        let mut preprocessed_trace = preprocessed_mask_add(
            preprocessed_trace,
            QM_31_OPS_IN_1_ADDRESS_IDX,
            qm31_const::<585273626, 1140883031, 1920880217, 1007275653>(),
        );
        let mut preprocessed_trace = preprocessed_mask_add(
            preprocessed_trace,
            QM_31_OPS_OUT_ADDRESS_IDX,
            qm31_const::<1287652242, 435165403, 1148348826, 1979403697>(),
        );
        let mut preprocessed_trace = preprocessed_mask_add(
            preprocessed_trace,
            QM_31_OPS_MULTS_IDX,
            qm31_const::<1372962279, 1246081592, 1017358753, 1525168544>(),
        );

        let mut trace_columns = [
            [qm31_const::<1659099300, 905558730, 651199673, 1375009625>()].span(),
            [qm31_const::<1591990121, 771341002, 584090809, 1375009625>()].span(),
            [qm31_const::<1793317658, 1173994186, 785417401, 1375009625>()].span(),
            [qm31_const::<1726208479, 1039776458, 718308537, 1375009625>()].span(),
            [qm31_const::<1390662584, 368687818, 382764217, 1375009625>()].span(),
            [qm31_const::<1323553405, 234470090, 315655353, 1375009625>()].span(),
            [qm31_const::<1524880942, 637123274, 516981945, 1375009625>()].span(),
            [qm31_const::<1457771763, 502905546, 449873081, 1375009625>()].span(),
            [qm31_const::<48489085, 1979300555, 1188070585, 1375009625>()].span(),
            [qm31_const::<2128863553, 1845082826, 1120961721, 1375009625>()].span(),
            [qm31_const::<1852335767, 645078115, 2059236183, 343880121>()].span(),
            [qm31_const::<1919444946, 779295843, 2126345047, 343880121>()].span(),
        ]
            .span();
        let interaction_values = array![
            qm31_const::<1005168032, 79980996, 1847888101, 1941984119>(),
            qm31_const::<1072277211, 214198724, 1914996965, 1941984119>(),
        ];
        let mut interaction_columns = make_interaction_trace(
            interaction_values, qm31_const::<1115374022, 1127856551, 489657863, 643630026>(),
        );
        component
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_trace,
                ref trace_columns,
                ref interaction_columns,
                qm31_const::<474642921, 876336632, 1911695779, 974600512>(),
            );
        preprocessed_trace.validate_usage();
        assert_eq!(sum, QM31Trait::from_fixed_array(QM_31_OPS_SAMPLE_EVAL_RESULT))
    }
}
