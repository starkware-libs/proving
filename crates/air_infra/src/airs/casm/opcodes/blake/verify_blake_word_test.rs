use expect_test::expect;

use crate::airs::casm::casm_state::*;
use crate::airs::casm::opcodes::blake::verify_blake_word::*;
// Macros
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::{const_expr, const_u32_expr};

#[test]
fn test_verify_blake_word() {
    let mut air_fn = VerifyBlakeWord::default();
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
        (
            CasmAddress::new(const_expr!(127), "blake_word1_addr"),
            const_u32_expr!(1882757439),
        ),
    );

    // Check state.
    expect![[r#"
        (76, "low_7_ms_bits"),
        (7182, "high_14_ms_bits"),
        (14, "high_5_ms_bits"),
        (0, "blake_word1_addr_id"),
    "#]]
    .assert_eq(&state.to_string());
    registry.run_air(
        &air_fn,
        (),
        (
            CasmAddress::new(const_expr!(302), "blake_word2_addr"),
            const_u32_expr!(231161980),
        ),
    );
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_fail_verify_blake_word() {
    let mut air_fn = VerifyBlakeWord::default();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // Fill memory
    let memory_values = vec![(const_expr!(0), const_felt252_expr!(1345646847_i64))];

    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);

    registry.run_air(
        &air_fn,
        (),
        (
            CasmAddress::new(const_expr!(0), "blake_word2_addr"),
            const_u32_expr!(1344646847),
        ),
    );
}
