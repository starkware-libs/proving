use super::bitwise::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::utils::test_utils::*;

//Macros
use crate::const_expr;
use crate::const_felt252_expr;

#[test]
fn simple_test_bitwise_builtin() {
    let instance_number = const_expr!(10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(1, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn simple_failed_test_bitwise_builtin() {
    let instance_number = const_expr!(10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(1, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
fn test_bitwise_builtin() {
    let instance_number = const_expr!(10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_or_bitwise_builtin() {
    let instance_number = const_expr!(27);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(567332252375527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 27);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_xor_bitwise_builtin() {
    let instance_number = const_expr!(10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(567257601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_and_bitwise_builtin() {
    let instance_number = const_expr!(10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(34650915127, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
fn test_big_felt252_bitwise_builtin() {
    let instance_number = const_expr!(10);
    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(467968798486, 18694984798),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(3468798969565, 4869486468496),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(157370615316, 18253796368),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(3622026537419, 4851673860558),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(3779397152735, 4869927656926),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);

    // Check entry
    compare_json(
        &registry.get_air_fn_entry(&bitwise.name()),
        &(TEST_JSONS_BUILTINS_DIR.to_owned() + "bitwise.json"),
    );
}
