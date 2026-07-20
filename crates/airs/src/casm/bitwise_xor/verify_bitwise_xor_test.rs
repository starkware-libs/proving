use air_infra::const_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;

use super::verify_bitwise_xor::*;

#[test]
fn test_bitwise_xor() {
    let bitwise_xor = VerifyBitwiseXor::<VerifyBitwiseXor_4_Const>::default();
    let (registry, _) = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [const_expr!(0b1100), const_expr!(0b1010), const_expr!(0b0110)],
        (),
    );
}

#[test]
#[should_panic(expected = "The bitwise XOR of 1111 and 1101 is not 101")]
fn test_failed_bitwise_xor() {
    let bitwise_xor = VerifyBitwiseXor::<VerifyBitwiseXor_4_Const>::default();
    let (registry, _) = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [const_expr!(0b1111), const_expr!(0b01101), const_expr!(0b101)],
        (),
    );
}

#[test]
#[should_panic(expected = "RangeCheck4 failed")]
fn test_failed_big_input_xor() {
    let bitwise_xor = VerifyBitwiseXor::<VerifyBitwiseXor_4_Const>::default();
    let (registry, _) = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [const_expr!(0b11001), const_expr!(0b1010), const_expr!(0b101)],
        (),
    );
}
