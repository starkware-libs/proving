// Constraints version: 7ad41409

use core::num::traits::Zero;
use crate::components::CairoComponent;
use crate::components::subroutines::decode_instruction_d2a10::decode_instruction_d2a10_evaluate;
use crate::components::subroutines::read_small::read_small_evaluate;
use crate::utils::U32Impl;
use stwo_constraint_framework::{
    PreprocessedColumn, PreprocessedColumnSet, PreprocessedMaskValues, PreprocessedMaskValuesImpl,
    PreprocessedColumnSetImpl, LookupElementsImpl,
};
use stwo_verifier_core::channel::{Channel, ChannelTrait};
use stwo_verifier_core::circle::CirclePoint;
use stwo_verifier_core::circle::CirclePointIndexTrait;
use stwo_verifier_core::circle::CirclePointQM31AddCirclePointM31Trait;
use stwo_verifier_core::fields::Invertible;
use stwo_verifier_core::fields::m31::{m31, M31};
use stwo_verifier_core::fields::qm31::{qm31_const, QM31, QM31Impl, QM31Serde, QM31Zero};
use stwo_verifier_core::poly::circle::CanonicCosetImpl;
use stwo_verifier_core::utils::{ArrayImpl, pow2};
use stwo_verifier_core::{ColumnArray, ColumnSpan, TreeArray};

pub const N_TRACE_COLUMNS: usize = 15;


#[derive(Drop)]
pub struct Component {
    pub claim: Claim,
    pub interaction_claim: InteractionClaim,
    pub verify_instruction_lookup_elements: crate::VerifyInstructionElements,
    pub memory_address_to_id_lookup_elements: crate::MemoryAddressToIdElements,
    pub memory_id_to_big_lookup_elements: crate::MemoryIdToBigElements,
    pub range_check_19_lookup_elements: crate::RangeCheck_19Elements,
    pub range_check_8_lookup_elements: crate::RangeCheck_8Elements,
    pub opcodes_lookup_elements: crate::OpcodesElements,
}

pub impl ComponentImpl of CairoComponent<Component> {
    fn mask_points(
        self: @Component,
        ref preprocessed_column_set: PreprocessedColumnSet,
        ref trace_mask_points: ColumnArray<Array<CirclePoint<QM31>>>,
        ref interaction_trace_mask_points: ColumnArray<Array<CirclePoint<QM31>>>,
        point: CirclePoint<QM31>,
    ) {
        let log_size = *(self.claim.log_size);
        let trace_gen = CanonicCosetImpl::new(log_size).coset.step_size;
        let point_offset_neg_1 = point.add_circle_point_m31(-trace_gen.mul(1).to_point());
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point]);
        interaction_trace_mask_points.append(array![point_offset_neg_1, point]);
        interaction_trace_mask_points.append(array![point_offset_neg_1, point]);
        interaction_trace_mask_points.append(array![point_offset_neg_1, point]);
        interaction_trace_mask_points.append(array![point_offset_neg_1, point]);
    }

    fn max_constraint_log_degree_bound(self: @Component) -> u32 {
        *(self.claim.log_size) + 1
    }

    fn evaluate_constraints_at_point(
        self: @Component,
        ref sum: QM31,
        ref preprocessed_mask_values: PreprocessedMaskValues,
        ref trace_mask_values: ColumnSpan<Span<QM31>>,
        ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
        random_coeff: QM31,
        point: CirclePoint<QM31>,
    ) {
        let log_size = *(self.claim.log_size);
        let trace_domain = CanonicCosetImpl::new(log_size);
        let domain_vanishing_eval_inv = trace_domain.eval_vanishing(point).inverse();
        let claimed_sum = *self.interaction_claim.claimed_sum;
        let column_size = m31(pow2(log_size));
        let mut verify_instruction_sum_0: QM31 = Zero::zero();
        let mut memory_address_to_id_sum_1: QM31 = Zero::zero();
        let mut memory_id_to_big_sum_2: QM31 = Zero::zero();
        let mut range_check_19_sum_3: QM31 = Zero::zero();
        let mut range_check_8_sum_4: QM31 = Zero::zero();
        let mut opcodes_sum_5: QM31 = Zero::zero();
        let mut opcodes_sum_6: QM31 = Zero::zero();

        let [
            input_pc_col0,
            input_ap_col1,
            input_fp_col2,
            offset2_col3,
            op1_imm_col4,
            op1_base_fp_col5,
            mem1_base_col6,
            op1_id_col7,
            msb_col8,
            mid_limbs_set_col9,
            op1_limb_0_col10,
            op1_limb_1_col11,
            op1_limb_2_col12,
            next_ap_bot8bits_col13,
            enabler
        ]: [Span<QM31>; 15] =
            (*trace_mask_values
            .multi_pop_front()
            .unwrap())
            .unbox();
        let [input_pc_col0]: [QM31; 1] = (*input_pc_col0.try_into().unwrap()).unbox();
        let [input_ap_col1]: [QM31; 1] = (*input_ap_col1.try_into().unwrap()).unbox();
        let [input_fp_col2]: [QM31; 1] = (*input_fp_col2.try_into().unwrap()).unbox();
        let [offset2_col3]: [QM31; 1] = (*offset2_col3.try_into().unwrap()).unbox();
        let [op1_imm_col4]: [QM31; 1] = (*op1_imm_col4.try_into().unwrap()).unbox();
        let [op1_base_fp_col5]: [QM31; 1] = (*op1_base_fp_col5.try_into().unwrap()).unbox();
        let [mem1_base_col6]: [QM31; 1] = (*mem1_base_col6.try_into().unwrap()).unbox();
        let [op1_id_col7]: [QM31; 1] = (*op1_id_col7.try_into().unwrap()).unbox();
        let [msb_col8]: [QM31; 1] = (*msb_col8.try_into().unwrap()).unbox();
        let [mid_limbs_set_col9]: [QM31; 1] = (*mid_limbs_set_col9.try_into().unwrap()).unbox();
        let [op1_limb_0_col10]: [QM31; 1] = (*op1_limb_0_col10.try_into().unwrap()).unbox();
        let [op1_limb_1_col11]: [QM31; 1] = (*op1_limb_1_col11.try_into().unwrap()).unbox();
        let [op1_limb_2_col12]: [QM31; 1] = (*op1_limb_2_col12.try_into().unwrap()).unbox();
        let [next_ap_bot8bits_col13]: [QM31; 1] = (*next_ap_bot8bits_col13.try_into().unwrap())
            .unbox();
        let [enabler]: [QM31; 1] = (*enabler.try_into().unwrap()).unbox();

        core::internal::revoke_ap_tracking();
    }
}
