use super::felt252_id_memory::*;
use crate::airs::memory::felt252_id_memory_read_positive::*;
use crate::airs::memory::felt252_id_memory_read_small::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

use crate::const_expr;
use crate::felt252_expr;

#[test]
fn test_read_small() {
    let mem_data = vec![
        (const_expr!(1), felt252_expr!("small_positive", 7, 0)),
        (
            const_expr!(2),
            felt252_expr!("small_positive_duplicate", 7, 0),
        ),
        (
            const_expr!(3),
            felt252_expr!("minus_one", 0, 10633823966279327296825105735305134080),
        ),
        (
            const_expr!(4),
            felt252_expr!(
                "minus_two",
                340282366920938463463374607431768211455,
                10633823966279327296825105735305134079
            ),
        ),
        (
            const_expr!(5),
            felt252_expr!("p", 1, 10633823966279327296825105735305134080),
        ),
        (
            const_expr!(6),
            felt252_expr!("p_plus_one", 2, 10633823966279327296825105735305134080),
        ),
    ];
    let memory = Felt252IdMemory::new_with_data(mem_data);

    let read_small = ReadSmall { memory };
    let registry = AirFnRegistry::new(&read_small);

    let (state, output) = registry.run_air(&read_small, const_expr!(1));
    assert_eq!(output.calc(), "7".to_string());
    assert_eq!(state.calc(), ["0", "0", "0", "7", "0", "0"]);

    let (state, output) = registry.run_air(&read_small, const_expr!(2));
    assert_eq!(output.calc(), "7".to_string());
    assert_eq!(state.calc(), ["0", "0", "0", "7", "0", "0"]);

    let (state, output) = registry.run_air(&read_small, const_expr!(3));
    assert_eq!(output.calc(), ((1i64 << 31) - 2).to_string());
    assert_eq!(state.calc(), ["1", "1", "0", "0", "0", "0"]);

    let (state, output) = registry.run_air(&read_small, const_expr!(4));
    assert_eq!(output.calc(), ((1i64 << 31) - 3).to_string());
    assert_eq!(state.calc(), ["2", "1", "1", "511", "511", "511"]);

    let (state, output) = registry.run_air(&read_small, const_expr!(5));
    assert_eq!(output.calc(), "0".to_string());
    assert_eq!(state.calc(), ["3", "1", "0", "1", "0", "0"]);

    let (state, output) = registry.run_air(&read_small, const_expr!(6));
    assert_eq!(output.calc(), "1".to_string());
    assert_eq!(state.calc(), ["4", "1", "0", "2", "0", "0"]);
}

#[test]
fn test_read_small_air_body() {
    let expected_air_body = [
        "tmp_0 = Memory_bee9eb79348d853b(ReadSmall_cda8d80eab0abe94_input)",
        "Deduction: tmp_0",
        "Memory_bee9eb79348d853b([ReadSmall_cda8d80eab0abe94_input]) == [state[0]]",
        "tmp_1 = Memory_7419fa4c3aacb251(state[0])",
        "tmp_2 = tmp_1.get_m31(const_27).eq(const_256)",
        "Deduction: tmp_2.as_m31()",
        "tmp_3 = tmp_1.get_m31(const_20).eq(const_511)",
        "Deduction: tmp_3.as_m31()",
        "Constraint: (state[1] * (state[1] - const_1))",
        "Constraint: (state[2] * (state[2] - const_1))",
        "Constraint: (state[2] * (state[1] - const_1))",
        "Deduction: tmp_1.get_m31(const_0)",
        "Deduction: tmp_1.get_m31(const_1)",
        "Deduction: tmp_1.get_m31(const_2)",
        "Memory_7419fa4c3aacb251([state[0]]) == [\
            state[3], state[4], state[5], \
            (state[2] * const_511), (state[2] * const_511), (state[2] * const_511), \
            (state[2] * const_511), (state[2] * const_511), (state[2] * const_511), \
            (state[2] * const_511), (state[2] * const_511), (state[2] * const_511), \
            (state[2] * const_511), (state[2] * const_511), (state[2] * const_511), \
            (state[2] * const_511), (state[2] * const_511), (state[2] * const_511), \
            (state[2] * const_511), (state[2] * const_511), (state[2] * const_511), \
            ((const_136 * state[1]) - state[2]), \
            const_0, const_0, const_0, const_0, const_0, \
            (state[1] * const_256)\
        ]",
    ];
    let memory = Felt252IdMemory::default();
    let read_small = ReadSmall { memory };
    let registry = AirFnRegistry::new(&read_small);
    let entry = registry.get_air_fn_entry(&read_small.name());
    assert_eq!(
        entry
            .air_body
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>(),
        expected_air_body
    );
}

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
        "tmp_0 = Memory_bee9eb79348d853b(ReadPositive_6812050f65145f40_input)",
        "Deduction: tmp_0",
        "Memory_bee9eb79348d853b([ReadPositive_6812050f65145f40_input]) == [state[0]]",
        "tmp_1 = Memory_7419fa4c3aacb251(state[0])",
        "Deduction: tmp_1.get_m31(const_0)",
        "Deduction: tmp_1.get_m31(const_1)",
        "tmp_2 = RangeCheck7([state[2]])",
        "RangeCheck7([state[2]]) == []",
        "Memory_7419fa4c3aacb251([state[0]]) == zero_extend([state[1], state[2]])",
    ];
    let memory = Felt252IdMemory::default();
    let read_positive = ReadPositive {
        memory,
        num_bits: 16,
    };
    let registry = AirFnRegistry::new(&read_positive);
    let entry = registry.get_air_fn_entry(&read_positive.name());
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
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck4 on input 510")]
fn test_read_positive_failure() {
    // Try to read a small negative number using ReadPositive
    test_read_positive(felt252_expr!("value", u128::MAX - 1, u128::MAX), 4);
}
