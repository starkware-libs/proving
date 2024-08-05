use super::bitwise_xor::*;

use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::expr;

#[test]
fn test_bitwise_xor() {
    let bitwise_xor = BitwiseXor { num_bits: 4 };
    let registry = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [expr!("a", 0b1100), expr!("b", 0b1010), expr!("c", 0b0110)],
    );
}

#[test]
#[should_panic(expected = "assertion `left == right")]
fn test_falied_bitwise_xor() {
    let bitwise_xor = BitwiseXor { num_bits: 4 };
    let registry = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [expr!("a", 0b1111), expr!("b", 0b01101), expr!("c", 0b101)],
    );
}

#[test]
#[should_panic(expected = "RangeCheck4 failed")]
fn test_falied_big_input_xor() {
    let bitwise_xor = BitwiseXor { num_bits: 4 };
    let registry = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [expr!("a", 0b11001), expr!("b", 0b1010), expr!("c", 0b101)],
    );
}
