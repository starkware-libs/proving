use crate::airs::casm::casm_state::*;
use crate::airs::casm::opcodes::blake::read_blake_word::*;
use crate::const_expr;
// Macros
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

#[test]
fn test_read_blake_word() {
    let mut air_fn = ReadBlakeWord::default();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // Fill memory
    let memory_values = vec![
        (const_expr!(13), const_felt252_expr!(2896667555_i64)),
        (const_expr!(14), const_felt252_expr!(1899217055_i64)),
    ];

    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);

    let (_, output) = registry.run_air(&air_fn, (), CasmAddress::new(const_expr!(13), "word0"));
    // Check output.
    assert_eq!(output.calc(), "2896667555");

    let (state, output) = registry.run_air(&air_fn, (), CasmAddress::new(const_expr!(14), "word1"));
    // Check output.
    assert_eq!(output.calc(), "1899217055");

    // Check state.
    let expected_state = vec![
        (49311, "low_16_bits"),
        (28979, "high_16_bits"),
        (96, "low_7_ms_bits"),
        (7244, "high_14_ms_bits"),
        (14, "high_5_ms_bits"),
        (1, "word1_id"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 1: RangeCheck2 on input 65538")]
fn test_fail_read_blake_word() {
    let mut air_fn = ReadBlakeWord::default();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // Fill memory
    let memory_values = vec![
        // 33 bits exceeds the 32 bits limit.
        (const_expr!(678), const_felt252_expr!(5569403492_i64)),
    ];

    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);

    registry.run_air(&air_fn, (), CasmAddress::new(const_expr!(678), "word0"));
}
