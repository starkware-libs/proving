use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::add_ap_opcode::*;
use crate::casm::common::*;

#[test]
fn test_add_ap_negative_imm() {
    // Build the air function
    let mut add_ap_opcode = AddApOpcode { memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let pc = 30;
    let ap = 11;
    let fp = 6;

    // Create the non-constant flags
    let non_consts_flags = vec![true, false, false];

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    -1,
                    -1,
                    1,
                    add_ap_opcode.get_flags().non_constants_to_arr(&non_consts_flags),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(-1i128)),
    ];
    add_ap_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&add_ap_opcode);
    let (state, next_state) = registry.run_air(
        &add_ap_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check the output
    assert_eq!(next_state.pc().calc(), (pc + 2).to_string());
    assert_eq!(next_state.fp().calc(), (fp).to_string());
    assert_eq!(next_state.ap().calc(), (ap - 1).to_string());

    // Check the state
    expect![[r#"
        (1, "enabler"),
        (30, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32769, "offset2"),
        (1, "op1_imm"),
        (0, "op1_base_fp"),
        (30, "mem1_base"),
        (1, "op1_id"),
        (1, "msb"),
        (0, "mid_limbs_set"),
        (0, "op1_limb_0"),
        (0, "op1_limb_1"),
        (0, "op1_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
        (10, "range_check_29_bot11bits"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_add_ap_deref_base_fp() {
    // Build the air function
    let mut add_ap_opcode = AddApOpcode { memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let pc = 30;
    let ap = 11;
    let fp = 6;
    let op1 = 299;
    let offset2 = 400;

    // Create the non-constant flags
    let non_consts_flags = vec![false, true, false];

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    -1,
                    -1,
                    offset2,
                    add_ap_opcode.get_flags().non_constants_to_arr(&non_consts_flags),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!((fp as i16 + offset2) as u32), const_felt252_expr!(op1)),
    ];
    add_ap_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&add_ap_opcode);
    let (state, next_state) = registry.run_air(
        &add_ap_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check the output
    assert_eq!(next_state.pc().calc(), (pc + 1).to_string());
    assert_eq!(next_state.fp().calc(), (fp).to_string());
    assert_eq!(next_state.ap().calc(), (ap + op1 as u32).to_string());

    // Check the state
    expect![[r#"
        (1, "enabler"),
        (30, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (33168, "offset2"),
        (0, "op1_imm"),
        (1, "op1_base_fp"),
        (6, "mem1_base"),
        (1, "op1_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (299, "op1_limb_0"),
        (0, "op1_limb_1"),
        (0, "op1_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
        (310, "range_check_29_bot11bits"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_failed_op1_src() {
    // Build the air function
    let mut add_ap_opcode = AddApOpcode { memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let pc = 30;
    let ap = 11;
    let fp = 6;
    let op1 = 299;
    let offset2 = 400;

    // Create the non-constant flags
    let non_consts_flags = vec![true, true, false];

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    -1,
                    -1,
                    offset2,
                    add_ap_opcode.get_flags().non_constants_to_arr(&non_consts_flags),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!((fp as i16 + offset2) as u32), const_felt252_expr!(op1)),
    ];
    add_ap_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&add_ap_opcode);
    registry.run_air(
        &add_ap_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck18 on input 262144")]
fn test_add_ap_too_big() {
    // Build the air function
    let mut add_ap_opcode = AddApOpcode { memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let pc = 30;
    let ap = (1 << 29) - 5;
    let fp = 6;

    // Create the non-constant flags
    let non_consts_flags = vec![true, false, false];

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    -1,
                    -1,
                    1,
                    add_ap_opcode.get_flags().non_constants_to_arr(&non_consts_flags),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(10i128)),
    ];
    add_ap_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&add_ap_opcode);
    registry.run_air(
        &add_ap_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}
