use stwo_verifier_utils::zip_eq::zip_eq;
use crate::components;
use crate::multiverifier_consts::{COMPONENT_LOG_SIZES, PREPROCESSED_COLUMN_LOG_SIZES};
use crate::per_component::PerComponentTrait;
use crate::preprocessed_columns::{
    BLAKE_G_GATE_INPUT_ADDR_A_IDX, EQ_IN0_ADDRESS_IDX, M_31_TO_U_32_INPUT_ADDR_IDX,
    QM_31_OPS_IN_0_ADDRESS_IDX, TRIPLE_XOR_INPUT_ADDR_0_IDX,
};

/// Derives each component's log size from the preprocessed column log sizes, in the committed
/// (`ComponentList` declaration) order of `COMPONENT_LOG_SIZES`. Variable-size components read
/// the log size of one of their preprocessed columns (all columns of a component share its log
/// size); fixed-size components return their `LOG_SIZE` constant.
fn derive_component_log_sizes(preprocessed_column_log_sizes: Span<u32>) -> Array<u32> {
    array![
        *preprocessed_column_log_sizes.at(EQ_IN0_ADDRESS_IDX), // eq
        *preprocessed_column_log_sizes.at(QM_31_OPS_IN_0_ADDRESS_IDX), // qm_31_ops
        *preprocessed_column_log_sizes.at(TRIPLE_XOR_INPUT_ADDR_0_IDX), // triple_xor
        *preprocessed_column_log_sizes.at(M_31_TO_U_32_INPUT_ADDR_IDX), // m_31_to_u_32
        *preprocessed_column_log_sizes.at(BLAKE_G_GATE_INPUT_ADDR_A_IDX), // blake_g_gate
        components::verify_bitwise_xor_8::LOG_SIZE, // verify_bitwise_xor_8
        components::verify_bitwise_xor_12::LOG_SIZE, // verify_bitwise_xor_12
        components::verify_bitwise_xor_4::LOG_SIZE, // verify_bitwise_xor_4
        components::verify_bitwise_xor_7::LOG_SIZE, // verify_bitwise_xor_7
        components::verify_bitwise_xor_9::LOG_SIZE, // verify_bitwise_xor_9
        components::range_check_16::LOG_SIZE // range_check_16
    ]
}

/// The hardcoded `COMPONENT_LOG_SIZES` must equal the values derived from the preprocessed
/// column log sizes.
#[test]
fn hardcoded_component_log_sizes_match_derived() {
    let derived = derive_component_log_sizes(PREPROCESSED_COLUMN_LOG_SIZES.span());
    for (expected, actual) in zip_eq(COMPONENT_LOG_SIZES.to_fixed_array().span(), derived.span()) {
        assert!(*expected == *actual);
    }
}
