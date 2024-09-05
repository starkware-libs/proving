use super::felt252_id_memory::*;

use crate::airs::memory::felt252_id_memory_read_positive::*;
use crate::airs::memory::felt252_id_memory_read_small::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

use crate::const_expr;
use crate::const_felt252_expr;

#[test]
fn test_read_small() {
    let mem_data = vec![
        // Small positive
        (const_expr!(1), const_felt252_expr!(7i128)),
        (
            // Small positive duplicate
            const_expr!(2),
            const_felt252_expr!(7i128),
        ),
        (
            // Minus one
            const_expr!(3),
            const_felt252_expr!(-1i128),
        ),
        (
            // Minus two
            const_expr!(4),
            const_felt252_expr!(-2i128),
        ),
        (
            // P
            const_expr!(5),
            const_felt252_expr!(1, 10633823966279327296825105735305134080),
        ),
        (
            // P + 1
            const_expr!(6),
            const_felt252_expr!(2, 10633823966279327296825105735305134080),
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
    let memory = Felt252IdMemory::default();
    let read_small = ReadSmall { memory };
    let registry = AirFnRegistry::new(&read_small);

    // Check entry
    compare_test_json(
        registry,
        &read_small.name(),
        &(TEST_JSONS_MEMORY_DIR.to_owned() + "read_small.json"),
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
    let memory = Felt252IdMemory::default();
    let read_positive = ReadPositive {
        memory,
        num_bits: 16,
    };
    let registry = AirFnRegistry::new(&read_positive);
    // Check entry
    compare_test_json(
        registry,
        &read_positive.name(),
        &(TEST_JSONS_MEMORY_DIR.to_owned() + "read_positive.json"),
    );
}

#[test]
fn test_read_positive_whole_limbs() {
    test_read_positive(const_felt252_expr!(1u128 << 35, 0), 36);
}

#[test]
fn test_read_positive_partial_limbs() {
    test_read_positive(const_felt252_expr!(12, 0), 4);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck4 on input 510")]
fn test_read_positive_failure() {
    // Try to read a small negative number using ReadPositive
    test_read_positive(const_felt252_expr!(u128::MAX - 1, u128::MAX), 4);
}
