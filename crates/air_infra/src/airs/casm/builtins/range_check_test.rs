use super::range_check::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;

#[test]
fn test_range_check() {
    let memory = Felt252IdMemory::new_with_data(vec![(
        const_expr!(DUMMY_SEGMENT_START),
        const_felt252_expr!((1 << 17), 0),
    )]);

    let rc = RangeCheckBuiltin {
        bits: 32,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);
    // Check entry
    compare_test_json(
        registry,
        &rc.name(),
        &(TEST_JSONS_BUILTINS_DIR.to_owned() + "range_check.json"),
    );
}

fn run_range_check(value: Felt252Expr, bits: usize) {
    let address = DUMMY_SEGMENT_START;
    let memory = Felt252IdMemory::new_with_data(vec![(const_expr!(address), value)]);

    let rc = RangeCheckBuiltin {
        bits,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);

    registry.run_air_with_row_number(&rc, (), 0);
}

#[test]
fn test_range_check_whole_limbs() {
    run_range_check(const_felt252_expr!(1u128 << 70, 0), 72);
}

#[test]
#[should_panic(expected = "Memory::set() failed")]
fn test_range_check_whole_limbs_fail() {
    run_range_check(const_felt252_expr!(1u128 << 74, 0), 72);
}

// Tests where <bits> is not divisible by 12
#[test]
fn test_range_check_partial_limbs() {
    run_range_check(const_felt252_expr!(1u128 << 127, 0), 128);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck2 on input 4")]
fn test_range_check_partial_limbs_fail() {
    run_range_check(const_felt252_expr!(0, 1), 128);
}
