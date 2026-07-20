use air_infra::core::Felt;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::public_params::PublicParam;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};

use super::range_check::*;

fn run_range_check(value: Felt252Expr, bits: usize) {
    let segment_start = 100;
    let memory = Felt252IdMemory::new_with_data(vec![(const_expr!(segment_start), value)]);

    let rc = RangeCheckBuiltin {
        bits,
        memory: memory.clone(),
        segment_start: PublicParam::RangeCheckBuiltinSegmentStart,
    };

    let mut registry = AirFnRegistry::new_empty();
    registry
        .public_params
        .set(PublicParam::RangeCheckBuiltinSegmentStart, Felt::from_u32_unchecked(segment_start));
    registry.add_entry(&rc);

    registry.run_air_with_row_number(&rc, (), (), 0);
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
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_range_check_2_bit_msl_fail() {
    run_range_check(const_felt252_expr!(0, 1), 128);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck6 on input 260")]
fn test_range_check_partial_limbs_fail() {
    run_range_check(const_felt252_expr!(0, 65), 132);
}
