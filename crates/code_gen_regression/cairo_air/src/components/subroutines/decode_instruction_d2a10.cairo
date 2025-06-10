use core::num::traits::Zero;
use crate::components::CairoComponent;
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


pub fn decode_instruction_d2a10_evaluate(
    input: [
        QM31
    ; 1],
    offset2_col0: QM31,
    op1_imm_col1: QM31,
    op1_base_fp_col2: QM31,
    verify_instruction_lookup_elements: @crate::VerifyInstructionElements,
    ref verify_instruction_sum_0: QM31,
    ref sum: QM31,
    domain_vanishing_eval_inv: QM31,
    random_coeff: QM31,
    ) -> [
    QM31
; 2] {
    let [decode_instruction_d2a10_input_pc] = input;

    [
        (offset2_col0 - qm31_const::<32768, 0, 0, 0>()),
        ((qm31_const::<1, 0, 0, 0>() - op1_imm_col1) - op1_base_fp_col2)
    ]
}
