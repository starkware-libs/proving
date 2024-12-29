use compiled_casm_air::public_params::PublicParam;

use super::range_check::*;
// Macros
use crate::const_expr;
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::*;

fn run_range_check(value: Felt252Expr, bits: usize) {
    let segment_start = 100;
    let memory = Felt252IdMemory::new_with_data(vec![(const_expr!(segment_start), value)]);

    let rc = RangeCheckBuiltin {
        bits,
        memory: memory.clone(),
    };

    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(
        PublicParam::RangeCheckBuiltinSegmentStart,
        Felt::from_u32_unchecked(segment_start),
    );
    registry.add_entry(&rc);

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
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck4 on input 16")]
fn test_range_check_partial_limbs_fail() {
    run_range_check(const_felt252_expr!(0, 4), 130);
}
