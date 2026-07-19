use air_infra::const_u32_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::AsProverType;
use expect_test::expect;

use crate::casm::opcodes::blake::triple_xor32::*;

#[test]
fn test_xor() {
    let triple_xor = TripleXor32 {};
    let (registry, _) = AirFnRegistry::new(&triple_xor);
    let (state, new_state) = registry.run_air(
        &triple_xor,
        (),
        [const_u32_expr!(145646949), const_u32_expr!(52416546), const_u32_expr!(856468484)],
    );
    assert_eq!(new_state.calc(), "951916867");

    // Check state
    expect![[r#"
        (1, "enabler"),
        (25957, "input_limb_0"),
        (2222, "input_limb_1"),
        (53282, "input_limb_2"),
        (799, "input_limb_3"),
        (44036, "input_limb_4"),
        (13068, "input_limb_5"),
        (101, "ms_8_bits"),
        (8, "ms_8_bits"),
        (208, "ms_8_bits"),
        (3, "ms_8_bits"),
        (172, "ms_8_bits"),
        (51, "ms_8_bits"),
        (71, "xor"),
        (67, "xor"),
        (181, "xor"),
        (25, "xor"),
        (177, "xor"),
        (189, "xor"),
        (11, "xor"),
        (56, "xor"),
    "#]]
    .assert_eq(&state.to_string());
}
