use circuit_common::finalize::{ComponentSizes, raw_component_sizes};
use circuits::context::TraceContext;

use super::*;

/// Checks the number of rows `add_zk_blinding` adds to each component.
#[test]
fn test_add_zk_blinding_row_counts() {
    const N_PADDING: usize = 3;

    let mut context = TraceContext::default().finalize(false);
    let before = raw_component_sizes(&context);
    add_zk_blinding(&mut context, [7; 32], N_PADDING);
    let after = raw_component_sizes(&context);

    // Every `add_into` call in the `*_zk_blinding` functions adds a qm31_ops row. Per call of:
    let qm31_ops_rows_per_unit = 3 * QM31_OPS_BLINDING_FACTOR // qm31_zk_blinding.
        + EQ_BLINDING_FACTOR // eq_zk_blinding.
        + 3 * TRIPLE_XOR_BLINDING_FACTOR // triple_xor_zk_blinding.
        + M31_TO_U32_BLINDING_FACTOR // m31_to_u32_zk_blinding.
        + 6 * BLAKE_G_GATE_BLINDING_FACTOR; // blake_g_gate_zk_blinding.
    assert_eq!(
        after.zip(&before, |after, before| after - before),
        ComponentSizes {
            eq: N_PADDING * EQ_BLINDING_FACTOR,
            qm31_ops: N_PADDING * qm31_ops_rows_per_unit,
            m31_to_u32: N_PADDING * M31_TO_U32_BLINDING_FACTOR,
            triple_xor: N_PADDING * TRIPLE_XOR_BLINDING_FACTOR,
            blake_g_gate: N_PADDING * BLAKE_G_GATE_BLINDING_FACTOR,
        }
    );
    context.validate_circuit();
}
