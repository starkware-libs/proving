use crate::const_expr;
use crate::core::air_fn_registry::AirFnRegistry;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::expr;
use crate::felt252_expr;

use super::range_check::RangeCheckBuiltin;

#[test]
fn test_range_check() {
    let deductions = [
        "deduction_tmp_0 = RangeCheckBuiltin__32_input",
        "deduction_tmp_0",
        "deduction_tmp_2 = Memory__FeltExpr__Felt252Expr(state[0])",
        "deduction_tmp_2.get_m31(const_0)",
        "deduction_tmp_2.get_m31(const_1)",
        "deduction_tmp_2.get_m31(const_2)",
        "deduction_tmp_3 = RangeCheck8(state[3])",
    ];

    let constraints = [
        "Memory__FeltExpr__Felt252Expr([state[0]]) == zero_extend([state[1], state[2], state[3]])",
        "RangeCheck8([state[3]]) == []",
    ];

    let memory = Memory::new_with_data(vec![(
        const_expr!(0),
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
    let address = 0;
    let memory = Memory::new_with_data(vec![(const_expr!(address), value)]);

    let rc = RangeCheckBuiltin {
        bits,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);

    registry.run_air(&rc, expr!("address", address));
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
