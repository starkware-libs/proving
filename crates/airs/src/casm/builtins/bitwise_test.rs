use air_infra::core::Felt;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::public_params::PublicParam;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};

use super::bitwise::*;

#[test]
fn simple_test_bitwise_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 500;

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(1, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::BitwiseBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn simple_failed_test_bitwise_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 500;

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(1, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::BitwiseBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), (), 10);
}

#[test]
fn test_bitwise_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 500;

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::BitwiseBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_or_bitwise_builtin() {
    let instance_number = const_expr!(27);
    let segment_start = 500;

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(567332252375527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::BitwiseBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), (), 27);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_xor_bitwise_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 500;

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(567257601461390, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::BitwiseBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_and_bitwise_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 500;

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(34650915127, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::BitwiseBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), (), 10);
}

#[test]
fn test_big_felt252_bitwise_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 500;
    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(467968798486, 18694984798),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(3468798969565, 4869486468496),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(157370615316, 18253796368),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(3622026537419, 4851673860558),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(3779397152735, 4869927656926),
        ),
    ]);

    let bitwise = BitwiseBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::BitwiseBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), (), 10);
}

#[test]
#[should_panic(expected = "RangeCheck8 failed (input 256)")]
fn test_too_big_felt252_bitwise_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 500;
    // p_upper = p >> 128 where p is the prime of the big field.
    // If we allow entries over 2^251 - 1 then we may have cases like
    // (0, 1) = (p, 1) -> (1, p - 1, p) instead of (0, 1, 1)
    let p_upper = (1_u128 << (251 - 128)) + (17 << (192 - 128));
    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(1, p_upper),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(0, p_upper),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(1, p_upper),
        ),
    ]);

    let bitwise = BitwiseBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::BitwiseBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), (), 10);
}
