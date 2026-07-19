use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use crate::casm::opcodes::blake::read_u32::*;

#[test]
fn test_read_u32() {
    let mut air_fn = ReadU32::default();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // Fill memory
    let memory_values = vec![
        (const_expr!(13), const_felt252_expr!(2896667555_i64)),
        (const_expr!(14), const_felt252_expr!(1899217055_i64)),
    ];

    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);

    let (_, output) = registry.run_air(&air_fn, (), CasmAddress::new(const_expr!(13), "u32_0"));
    // Check output.
    assert_eq!(output.calc(), "2896667555");

    let (state, output) = registry.run_air(&air_fn, (), CasmAddress::new(const_expr!(14), "u32_1"));
    // Check output.
    assert_eq!(output.calc(), "1899217055");

    // Check state.
    expect![[r#"
        (49311, "low_16_bits"),
        (28979, "high_16_bits"),
        (96, "low_7_ms_bits"),
        (7244, "high_14_ms_bits"),
        (14, "high_5_ms_bits"),
        (1, "u32_1_id"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 1: RangeCheck2 on input 65538")]
fn test_fail_read_u32() {
    let mut air_fn = ReadU32::default();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // Fill memory
    let memory_values = vec![
        // 33 bits exceeds the 32 bits limit.
        (const_expr!(678), const_felt252_expr!(5569403492_i64)),
    ];

    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);

    registry.run_air(&air_fn, (), CasmAddress::new(const_expr!(678), "u32"));
}
