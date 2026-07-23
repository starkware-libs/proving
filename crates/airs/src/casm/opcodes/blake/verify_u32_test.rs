use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr, const_u32_expr};
use expect_test::expect;

use crate::casm::opcodes::blake::verify_u32::*;

#[test]
fn test_verify_u32() {
    let mut air_fn = VerifyU32::default();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // Fill memory
    let memory_values = vec![
        (const_expr!(127), const_felt252_expr!(1882757439_i64)),
        (const_expr!(302), const_felt252_expr!(231161980_i64)),
    ];

    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);

    let (state, _) = registry.run_air(
        &air_fn,
        (),
        (CasmAddress::new(const_expr!(127), "u32_1_addr"), const_u32_expr!(1882757439)),
    );

    // Check state.
    expect![[r#"
        (76, "low_7_ms_bits"),
        (7182, "high_14_ms_bits"),
        (14, "high_5_ms_bits"),
        (0, "u32_1_addr_id"),
    "#]]
    .assert_eq(&state.to_string());
    registry.run_air(
        &air_fn,
        (),
        (CasmAddress::new(const_expr!(302), "u32_2_addr"), const_u32_expr!(231161980)),
    );
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_fail_verify_u32() {
    let mut air_fn = VerifyU32::default();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // Fill memory
    let memory_values = vec![(const_expr!(0), const_felt252_expr!(1345646847_i64))];

    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);

    registry.run_air(
        &air_fn,
        (),
        (CasmAddress::new(const_expr!(0), "u32_addr"), const_u32_expr!(1344646847)),
    );
}
