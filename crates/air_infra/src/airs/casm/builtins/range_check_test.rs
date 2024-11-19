use super::range_check::*;
// Macros
use crate::const_expr;
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;

fn run_range_check(value: Felt252Expr, bits: usize) {
    let address = DUMMY_SEGMENT_START;
    let memory = Felt252IdMemory::new_with_data(vec![(const_expr!(address), value)]);

    let rc = RangeCheckBuiltin {
        bits,
        memory: memory.clone(),
    };

    let (registry, _) = AirFnRegistry::new(&rc);

    registry.run_air_with_row_number(&rc, (), 0);
}

#[test]
fn test_range_check_whole_limbs() {
    run_range_check(const_felt252_expr!(1u128 << 70, 0), 72);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_range_check_whole_limbs_fail() {
    run_range_check(const_felt252_expr!(1u128 << 74, 0), 72);
}

// Tests where <bits> is not divisible by 12
#[test]
fn test_range_check_partial_limbs() {
    run_range_check(const_felt252_expr!(1u128 << 127, 0), 128);
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evalutate to 0)")]
fn test_range_check_2_bit_msl_fail() {
    run_range_check(const_felt252_expr!(0, 1), 128);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck3 on input 8")]
fn test_range_check_partial_limbs_fail() {
    run_range_check(const_felt252_expr!(0, 2), 129);
}
