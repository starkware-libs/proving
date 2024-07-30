use super::range_check::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;

// Macros
use crate::{const_expr, expr, felt252_expr};

#[test]
fn test_range_check() {
    let deductions = [
        "deduction_tmp_0 = RangeCheckBuiltin_2398e5a71b8b7a99_input",
        "deduction_tmp_0",
        "deduction_tmp_3 = Memory_81f75475e4cf34d6((const_100 + state[0]))",
        "deduction_tmp_3.get_m31(const_0)",
        "deduction_tmp_3.get_m31(const_1)",
        "deduction_tmp_3.get_m31(const_2)",
        "deduction_tmp_4 = RangeCheck8(state[3])",
    ];

    let constraints = [
        "Memory_81f75475e4cf34d6([(const_100 + state[0])]) == zero_extend([state[1], state[2], state[3]])",
        "RangeCheck8([state[3]]) == []",
    ];

    let memory = Memory::new_with_data(vec![(
        const_expr!(DUMMY_SEGMENT_START),
        felt252_expr!("value_to_check", (1 << 17), 0),
    )]);

    let rc = RangeCheckBuiltin {
        bits: 32,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);
    let lists = registry.get_compiled_air_fn(&rc);

    assert_eq!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints
    );

    assert_eq!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deductions
    );
}

fn run_range_check(value: Felt252Expr, bits: usize) {
    let address = DUMMY_SEGMENT_START;
    let memory = Memory::new_with_data(vec![(const_expr!(address), value)]);

    let rc = RangeCheckBuiltin {
        bits,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);

    registry.run_air(&rc, expr!("address", 0));
}

#[test]
fn test_range_check_whole_limbs() {
    run_range_check(felt252_expr!("value_to_check", 1 << 94, 0), 96);
}

#[test]
#[should_panic(expected = "Memory::set() failed")]
fn test_range_check_whole_limbs_fail() {
    run_range_check(felt252_expr!("value_to_check", 1 << 98, 0), 96);
}

// Tests where <bits> is not divisible by 12
#[test]
fn test_range_check_partial_limbs() {
    run_range_check(felt252_expr!("value_to_check", 1 << 127, 0), 128);
}

#[test]
#[should_panic(expected = "RangeCheck8 failed")]
fn test_range_check_partial_limbs_fail() {
    run_range_check(felt252_expr!("value_to_check", 0, 1), 128);
}
