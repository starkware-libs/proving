use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::AsProverType;
use air_infra::{const_expr, const_u32_expr};
use expect_test::expect;
use stwo_cairo_common::prover_types::cpu::QM31;

use super::blake::blake_gate::*;
use super::blake::blake_output::*;
use super::blake::create_blake_round_input::IV;
use super::qm31_ops::*;

#[test]
fn test_add_mul_gate() {
    let op0 = [1, 2, 3, 4];
    let op1 = [0x12345678, 0x7fedcba9, 0x1033c4d6, 0x0fedcba9];
    let dst = (QM31::from_u32_unchecked(op0[0], op0[1], op0[2], op0[3])
        * QM31::from_u32_unchecked(op1[0], op1[1], op1[2], op1[3]))
    .to_m31_array();

    let func = Qm31Ops {};
    let (registry, _) = AirFnRegistry::new(&func);
    let (state, _) = registry.run_air(
        &func,
        (),
        [
            [const_expr!(op0[0]), const_expr!(op0[1]), const_expr!(op0[2]), const_expr!(op0[3])],
            [const_expr!(op1[0]), const_expr!(op1[1]), const_expr!(op1[2]), const_expr!(op1[3])],
            [
                const_expr!(dst[0].0),
                const_expr!(dst[1].0),
                const_expr!(dst[2].0),
                const_expr!(dst[3].0),
            ],
        ],
    );

    expect![[r#"
        (1, "input_op0_limb0"),
        (2, "input_op0_limb1"),
        (3, "input_op0_limb2"),
        (4, "input_op0_limb3"),
        (305419896, "input_op1_limb0"),
        (2146290601, "input_op1_limb1"),
        (271828182, "input_op1_limb2"),
        (267242409, "input_op1_limb3"),
        (59279500, "input_dst_limb0"),
        (1986757919, "input_dst_limb1"),
        (658375236, "input_dst_limb2"),
        (2028999219, "input_dst_limb3"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_blake_gate() {
    let message: [u32; 16] = [
        876098866, 6998120, 13470920, 26317257, 58954531, 212272678, 420334798, 840571289,
        1684353892, 13158510, 25742556, 26845577, 51783203, 116180710, 215583885, 429404441,
    ];
    let state_before: [u32; 8] =
        [IV[0] ^ 0x01010020, IV[1], IV[2], IV[3], IV[4], IV[5], IV[6], IV[7]];
    let state_after: [u32; 8] = [
        3425822922, 1886818505, 958016992, 1751539680, 2591581574, 923412807, 4068093030,
        1030609454,
    ];

    let func = BlakeGate {};
    let (mut registry, _entry) = AirFnRegistry::new(&func);
    registry.run_air(
        &func,
        (),
        (
            [state_before.map(|f| const_u32_expr!(f)), state_after.map(|f| const_u32_expr!(f))],
            message.map(|f| const_expr!(f)),
        ),
    );

    let func = BlakeOutput {};
    let _ = registry.add_entry(&func);
    let (_, output) = registry.run_air(&func, (), state_after.map(|f| const_u32_expr!(f)));

    let expected_output = [
        const_u32_expr!(1278339275),
        const_u32_expr!(1886818505),
        const_u32_expr!(958016992),
        const_u32_expr!(1751539680),
        const_u32_expr!(444097927),
        const_u32_expr!(923412807),
        const_u32_expr!(1920609383),
        const_u32_expr!(1030609454),
    ];
    for (out, exp_out) in output.into_iter().zip(expected_output) {
        assert_eq!(out.calc(), exp_out.calc());
    }
}
