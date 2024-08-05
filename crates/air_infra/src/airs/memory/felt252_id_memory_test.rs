use super::felt252_id_memory::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;

use crate::const_expr;
use crate::felt252_expr;

fn test_read_positive(value: Felt252Expr, num_bits: usize) {
    let memory = Felt252IdMemory::new_with_data(vec![(const_expr!(0), value.clone())]);

    let read_positive = ReadPositive { memory, num_bits };

    let registry = AirFnRegistry::new(&read_positive);
    let (_state, output) = registry.run_air(&read_positive, const_expr!(0));

    assert_eq!(output.calc(), value.calc());
}

#[test]
fn test_read_positive_air_body() {
    let expected_air_body = [
        "deduction_tmp_0 = Memory_5458bf3d74919439(ReadPositive_22ebb6fbe9ff7280_input)",
        "Deduction: deduction_tmp_0",
        "Memory_5458bf3d74919439([ReadPositive_22ebb6fbe9ff7280_input]) == [state[0]]",
        "deduction_tmp_1 = Memory_81f75475e4cf34d6(state[0])",
        "Deduction: deduction_tmp_1.get_m31(const_0)",
        "Deduction: deduction_tmp_1.get_m31(const_1)",
        "deduction_tmp_2 = RangeCheck7(state[2])",
        "RangeCheck7([state[2]]) == []",
        "Memory_81f75475e4cf34d6([state[0]]) == zero_extend([state[1], state[2]])",
    ];
    let memory = Felt252IdMemory::default();
    let read_positive = ReadPositive {
        memory,
        num_bits: 16,
    };
    let registry = AirFnRegistry::new(&read_positive);
    let entry = registry.get_air_fn_entry(&read_positive);
    assert_eq!(
        entry
            .air_body
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>(),
        expected_air_body
    );
}

#[test]
fn test_read_positive_whole_limbs() {
    test_read_positive(felt252_expr!("value", 1 << 35, 0), 36);
}

#[test]
fn test_read_positive_partial_limbs() {
    test_read_positive(felt252_expr!("value", 12, 0), 4);
}

#[test]
#[should_panic(expected = "RangeCheck4 failed")]
fn test_read_positive_failure() {
    // Try to read a small negative number using ReadPositive
    test_read_positive(felt252_expr!("value", u128::MAX - 1, u128::MAX), 4);
}
