use compiled_casm_air::utils::JSONS_LOOKUPS_DIR;

use super::verify_bitwise_xor::*;
// Macros
use crate::const_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::utils::test_utils::*;

#[test]
fn test_entry_json() {
    let (_, entry) = AirFnRegistry::new(&VerifyBitwiseXor { num_bits: 9 });
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_LOOKUPS_DIR, name),
    );
}

#[test]
fn test_bitwise_xor() {
    let bitwise_xor = VerifyBitwiseXor { num_bits: 4 };
    let (registry, _) = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [
            const_expr!(0b1100),
            const_expr!(0b1010),
            const_expr!(0b0110),
        ],
    );
}

#[test]
#[should_panic(expected = "The bitwise XOR of 1111 and 1101 is not 101")]
fn test_falied_bitwise_xor() {
    let bitwise_xor = VerifyBitwiseXor { num_bits: 4 };
    let (registry, _) = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [
            const_expr!(0b1111),
            const_expr!(0b01101),
            const_expr!(0b101),
        ],
    );
}

#[test]
#[should_panic(expected = "RangeCheck4 failed")]
fn test_falied_big_input_xor() {
    let bitwise_xor = VerifyBitwiseXor { num_bits: 4 };
    let (registry, _) = AirFnRegistry::new(&bitwise_xor);
    registry.run_air(
        &bitwise_xor,
        [
            const_expr!(0b11001),
            const_expr!(0b1010),
            const_expr!(0b101),
        ],
    );
}
