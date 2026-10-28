use circuit_common::finalize::{
    BLAKE_G_GATE_BLINDING_FACTOR, M31_TO_U32_BLINDING_FACTOR, TRIPLE_XOR_BLINDING_FACTOR,
};
use stwo::core::fields::m31::MODULUS_BITS;

use crate::circuit_air::components::{blake_g_gate, m_31_to_u_32, triple_xor};

/// The number of random u32 inputs that `triple_xor_zk_blinding` draws per row: a, b, c.
const TRIPLE_XOR_N_RANDOM_U32S: u32 = 3;
/// The number of random M31 inputs that `m31_to_u32_zk_blinding` draws per row: input.
const M31_TO_U32_N_RANDOM_M31S: u32 = 1;
/// The number of random u32 inputs that `blake_g_gate_zk_blinding` draws per row: a, b, c, d, f0,
/// f1.
const BLAKE_G_GATE_N_RANDOM_U32S: u32 = 6;

/// Checks that each blinding factor is `ceil(n_columns * 31 / input_bits)` (see the
/// `*_BLINDING_FACTOR` constants), where `n_columns` counts the component's trace and interaction
/// columns (in M31 columns) and `input_bits` are the random bits that one blinding row draws in its
/// `*_zk_blinding` function.
#[test]
fn test_zk_blinding_factors() {
    // The column log sizes per tree: [trace, interaction]. The log size is arbitrary.
    let triple_xor_log_sizes = triple_xor::Claim { log_size: 0 }.log_sizes();
    let m31_to_u32_log_sizes = m_31_to_u_32::Claim { log_size: 0 }.log_sizes();
    let blake_g_gate_log_sizes = blake_g_gate::Claim { log_size: 0 }.log_sizes();
    for (name, factor, log_sizes, input_bits) in [
        (
            "triple_xor",
            TRIPLE_XOR_BLINDING_FACTOR,
            triple_xor_log_sizes,
            TRIPLE_XOR_N_RANDOM_U32S * u32::BITS,
        ),
        (
            "m31_to_u32",
            M31_TO_U32_BLINDING_FACTOR,
            m31_to_u32_log_sizes,
            M31_TO_U32_N_RANDOM_M31S * MODULUS_BITS,
        ),
        (
            "blake_g_gate",
            BLAKE_G_GATE_BLINDING_FACTOR,
            blake_g_gate_log_sizes,
            BLAKE_G_GATE_N_RANDOM_U32S * u32::BITS,
        ),
    ] {
        let (n_trace_columns, n_interaction_columns) = (log_sizes[0].len(), log_sizes[1].len());
        let n_columns = u32::try_from(n_trace_columns + n_interaction_columns).unwrap();
        assert_eq!(
            u32::try_from(factor).unwrap(),
            (n_columns * MODULUS_BITS).div_ceil(input_bits),
            "{name}: {n_trace_columns} trace + {n_interaction_columns} interaction columns, \
             {input_bits} random input bits per row"
        );
    }
}
