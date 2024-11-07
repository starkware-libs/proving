use compiled_casm_air::utils::JSONS_LOOKUPS_DIR;

use super::super::casm::casm_state::*;
use super::memory::*;
use super::read_positive::*;
use super::read_small::*;
use crate::airs::felt252_id_memory::address_to_id::*;
use crate::airs::felt252_id_memory::id_to_big::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;
use crate::{const_expr, const_felt252_expr};

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
    let (registry, _) = AirFnRegistry::new(&read_small);

    let (state, output) = registry.run_air(&read_small, CasmAddress::new(const_expr!(1), ""));
    assert_eq!(output.0.calc(), "7".to_string());
    let expected_state = vec![
        (0, "id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (7, "limb_0"),
        (0, "limb_1"),
        (0, "limb_2"),
    ]
    .into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );

    let (state, output) = registry.run_air(&read_small, CasmAddress::new(const_expr!(2), ""));
    assert_eq!(output.0.calc(), "7".to_string());
    let expected_state = vec![
        (0, "id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (7, "limb_0"),
        (0, "limb_1"),
        (0, "limb_2"),
    ]
    .into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );

    let (state, output) = registry.run_air(&read_small, CasmAddress::new(const_expr!(3), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - 2).to_string());
    let expected_state = vec![
        (1, "id"),
        (1, "msb"),
        (0, "mid_limbs_set"),
        (0, "limb_0"),
        (0, "limb_1"),
        (0, "limb_2"),
    ]
    .into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );

    let (state, output) = registry.run_air(&read_small, CasmAddress::new(const_expr!(4), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - 3).to_string());
    let expected_state = vec![
        (2, "id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (511, "limb_0"),
        (511, "limb_1"),
        (511, "limb_2"),
    ]
    .into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );

    let (state, output) = registry.run_air(&read_small, CasmAddress::new(const_expr!(5), ""));
    assert_eq!(output.0.calc(), "0".to_string());
    let expected_state = vec![
        (3, "id"),
        (1, "msb"),
        (0, "mid_limbs_set"),
        (1, "limb_0"),
        (0, "limb_1"),
        (0, "limb_2"),
    ]
    .into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );

    let (state, output) = registry.run_air(&read_small, CasmAddress::new(const_expr!(6), ""));
    assert_eq!(output.0.calc(), "1".to_string());
    let expected_state = vec![
        (4, "id"),
        (1, "msb"),
        (0, "mid_limbs_set"),
        (2, "limb_0"),
        (0, "limb_1"),
        (0, "limb_2"),
    ]
    .into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
fn test_read_small_entry_json() {
    let (_, entry) = AirFnRegistry::new(&ReadSmall::default());
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_MEMORY_DIR,
            entry.name.to_lowercase()
        ),
    );
}

fn test_read_positive(value: Felt252Expr, num_bits: usize) {
    let memory = Felt252IdMemory::new_with_data(vec![(const_expr!(0), value.clone())]);

    let read_positive = ReadPositive { memory, num_bits };

    let (registry, _) = AirFnRegistry::new(&read_positive);
    let (_state, output) = registry.run_air(&read_positive, CasmAddress::new(const_expr!(0), ""));

    assert_eq!(output.0.calc(), value.calc());
}

#[test]
fn test_read_positive_entry_json() {
    let (_, entry) = AirFnRegistry::new(&ReadPositive {
        num_bits: 16,
        memory: Felt252IdMemory::default(),
    });
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_MEMORY_DIR,
            entry.name.to_lowercase()
        ),
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

#[test]
fn test_memory_constraints() {
    let (_, entry) = AirFnRegistry::new(&MemoryIdToBig::default());
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_LOOKUPS_DIR, name),
    );
}

#[test]
fn test_address_to_id_json() {
    let (_, entry) = AirFnRegistry::new(&MemoryAddressToId::default());
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_LOOKUPS_DIR, name),
    );
}
