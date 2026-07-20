use air_infra::const_u32_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use expect_test::expect;

use super::triple_xor::TripleXor;

#[test]
fn test_triple_xor() {
    let a: u32 = 145646949;
    let b: u32 = 52416546;
    let c: u32 = 856468484;
    let d = a ^ b ^ c;

    let triple_xor = TripleXor {};
    let (registry, _) = AirFnRegistry::new(&triple_xor);
    let (state, _) = registry.run_air(
        &triple_xor,
        (),
        [const_u32_expr!(a), const_u32_expr!(b), const_u32_expr!(c), const_u32_expr!(d)],
    );

    expect![[r#"
        (25957, "input_a_limb_0"),
        (2222, "input_a_limb_1"),
        (53282, "input_b_limb_0"),
        (799, "input_b_limb_1"),
        (44036, "input_c_limb_0"),
        (13068, "input_c_limb_1"),
        (6467, "input_a_xor_b_xor_c_limb_0"),
        (14525, "input_a_xor_b_xor_c_limb_1"),
        (101, "ms_8_bits"),
        (8, "ms_8_bits"),
        (208, "ms_8_bits"),
        (3, "ms_8_bits"),
        (172, "ms_8_bits"),
        (51, "ms_8_bits"),
        (25, "ms_8_bits"),
        (56, "ms_8_bits"),
        (71, "xor"),
        (181, "xor"),
        (177, "xor"),
        (11, "xor"),
    "#]]
    .assert_eq(&state.to_string());
}
