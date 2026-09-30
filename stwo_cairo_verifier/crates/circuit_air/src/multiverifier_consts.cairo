//! Hardcoded constants for the multiverifier circuit.

use stwo_verifier_core::fri::FriParams;
use crate::per_component::PerComponent;

/// Number of public output values of the multiverifier circuit.
///
/// The multiverifier outputs the full unreduced Blake2s digest of its two verified inputs as
/// `N_RESERVED` = 8 QM31 words. (The logup anchor `u` is appended internally by the verifier and
/// is not part of the public outputs.)
pub const N_OUTPUTS: u32 = 8;

// === BEGIN GENERATED (see cairo_consts_test.rs; running it with FIX=1 regenerates) ===

/// Expected FRI params of the multiverifier circuit's proof.
///
/// Pinned to the production registry's proof config, so the verifier accepts
/// only proofs produced with that canonical configuration. This pins
/// every FRI security parameter (a weaker config — fewer queries, smaller
/// blowup, or less proof-of-work — is rejected, independently of stwo's
/// `security_bits >= SECURITY_BITS` floor).
/// Note `pow_bits + log_blowup_factor * n_queries = 26 + 1 * 70 = 96 = SECURITY_BITS`.
pub const CIRCUIT_FRI_PARAMS: FriParams = FriParams {
    pow_bits: 26, log_blowup_factor: 1, log_last_layer_degree_bound: 0, n_queries: 70, fold_step: 4,
};

/// Each component's log size.
pub const COMPONENT_LOG_SIZES: PerComponent<u32> = PerComponent {
    eq: 20,
    qm_31_ops: 23,
    triple_xor: 19,
    m_31_to_u_32: 20,
    blake_g_gate: 23,
    verify_bitwise_xor_8: 16,
    verify_bitwise_xor_12: 20,
    verify_bitwise_xor_4: 8,
    verify_bitwise_xor_7: 14,
    verify_bitwise_xor_9: 18,
    range_check_16: 16,
};

/// Per-column log sizes of the multiverifier circuit's preprocessed trace,
/// in size-sorted column order — the same order as the index constants in
/// `crate::preprocessed_columns`. Every column of a component shares that
/// component's log size, so each entry references the owning component's
/// `COMPONENT_LOG_SIZES` field.
pub const PREPROCESSED_COLUMN_LOG_SIZES: [u32; 45] = [
    COMPONENT_LOG_SIZES.verify_bitwise_xor_4, // bitwise_xor_4_0
    COMPONENT_LOG_SIZES.verify_bitwise_xor_4, // bitwise_xor_4_1
    COMPONENT_LOG_SIZES.verify_bitwise_xor_4, // bitwise_xor_4_2
    COMPONENT_LOG_SIZES.verify_bitwise_xor_7, // bitwise_xor_7_0
    COMPONENT_LOG_SIZES.verify_bitwise_xor_7, // bitwise_xor_7_1
    COMPONENT_LOG_SIZES.verify_bitwise_xor_7, // bitwise_xor_7_2
    COMPONENT_LOG_SIZES.range_check_16, // seq_16
    COMPONENT_LOG_SIZES.verify_bitwise_xor_8, // bitwise_xor_8_0
    COMPONENT_LOG_SIZES.verify_bitwise_xor_8, // bitwise_xor_8_1
    COMPONENT_LOG_SIZES.verify_bitwise_xor_8, // bitwise_xor_8_2
    COMPONENT_LOG_SIZES.verify_bitwise_xor_9, // bitwise_xor_9_0
    COMPONENT_LOG_SIZES.verify_bitwise_xor_9, // bitwise_xor_9_1
    COMPONENT_LOG_SIZES.verify_bitwise_xor_9, // bitwise_xor_9_2
    COMPONENT_LOG_SIZES.triple_xor, // triple_xor_input_addr_0
    COMPONENT_LOG_SIZES.triple_xor, // triple_xor_input_addr_1
    COMPONENT_LOG_SIZES.triple_xor, // triple_xor_input_addr_2
    COMPONENT_LOG_SIZES.triple_xor, // triple_xor_output_addr
    COMPONENT_LOG_SIZES.triple_xor, // triple_xor_multiplicity
    COMPONENT_LOG_SIZES.eq, // eq_in0_address
    COMPONENT_LOG_SIZES.eq, // eq_in1_address
    COMPONENT_LOG_SIZES.m_31_to_u_32, // m31_to_u32_input_addr
    COMPONENT_LOG_SIZES.m_31_to_u_32, // m31_to_u32_output_addr
    COMPONENT_LOG_SIZES.m_31_to_u_32, // m31_to_u32_multiplicity
    COMPONENT_LOG_SIZES.verify_bitwise_xor_12, // bitwise_xor_10_0
    COMPONENT_LOG_SIZES.verify_bitwise_xor_12, // bitwise_xor_10_1
    COMPONENT_LOG_SIZES.verify_bitwise_xor_12, // bitwise_xor_10_2
    COMPONENT_LOG_SIZES.qm_31_ops, // qm_31_ops_add_flag
    COMPONENT_LOG_SIZES.qm_31_ops, // qm_31_ops_sub_flag
    COMPONENT_LOG_SIZES.qm_31_ops, // qm_31_ops_mul_flag
    COMPONENT_LOG_SIZES.qm_31_ops, // qm_31_ops_pointwise_mul_flag
    COMPONENT_LOG_SIZES.qm_31_ops, // qm_31_ops_in_0_address
    COMPONENT_LOG_SIZES.qm_31_ops, // qm_31_ops_in_1_address
    COMPONENT_LOG_SIZES.qm_31_ops, // qm_31_ops_out_address
    COMPONENT_LOG_SIZES.qm_31_ops, // qm_31_ops_mults
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_input_addr_a
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_input_addr_b
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_input_addr_c
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_input_addr_d
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_input_addr_f0
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_input_addr_f1
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_output_addr_a
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_output_addr_b
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_output_addr_c
    COMPONENT_LOG_SIZES.blake_g_gate, // blake_g_gate_output_addr_d
    COMPONENT_LOG_SIZES.blake_g_gate // blake_g_gate_multiplicity
];

/// Log degree bound of the circuit's trace, equal to the largest preprocessed column log size.
pub const TRACE_LOG_DEGREE_BOUND: u32 = 23;
// === END GENERATED ===


