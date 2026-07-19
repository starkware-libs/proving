use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::state::State;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::call_opcode::*;
use crate::casm::common::*;

fn build_and_test(
    op1_base_fp: bool,
    offset2_option: Option<i16>,
    op1_value: i64,
    [pc_value, ap_value, fp_value]: [u32; 3],
) -> State {
    let [pc, ap, fp] = [const_expr!(pc_value), const_expr!(ap_value), const_expr!(fp_value)];

    let rel_imm = offset2_option.is_none();
    let offset2 = offset2_option.unwrap_or(1);

    assert!((!rel_imm) || (!op1_base_fp), "Flag op1_base_fp cannot be set for relative calls.");

    let mut call_opcode = CallOpcode { rel_imm, memory: Felt252IdMemory::default() };

    // Fill memory
    let mut memory_values = vec![
        (
            pc.clone(),
            const_felt252_expr!(assemble_call(offset2, &call_opcode.get_flags(), op1_base_fp), 0),
        ),
        (const_expr!(ap_value), const_felt252_expr!(fp_value as u128, 0)),
        (
            const_expr!(ap_value + 1),
            const_felt252_expr!((pc_value + (if rel_imm { 2 } else { 1 })) as u128, 0),
        ),
    ];

    let op1_value_252 = const_felt252_expr!(op1_value);
    if rel_imm {
        memory_values.push((const_expr!(pc_value + 1), op1_value_252));
    } else if op1_base_fp {
        memory_values.push((const_expr!((fp_value as i16 + offset2) as u32), op1_value_252));
    } else {
        memory_values.push((const_expr!((ap_value as i16 + offset2) as u32), op1_value_252));
    }

    call_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&call_opcode);
    let (state, next_state) =
        registry.run_air(&call_opcode, (), CasmStateVar::new(pc, ap.clone(), fp.clone()));

    // Check output
    if rel_imm {
        assert_eq!(next_state.pc().calc(), (pc_value as i128 + op1_value as i128).to_string());
    } else {
        assert_eq!(next_state.pc().calc(), op1_value.to_string());
    }
    assert_eq!(next_state.ap().calc(), (ap_value + 2).to_string());
    assert_eq!(next_state.fp().calc(), (ap_value + 2).to_string());

    state
}

#[test]
fn test_relative_call_large_state() {
    let state = build_and_test(false, None, 500, [1000000, 2000000, 4000000]);

    expect![[r#"
        (1, "enabler"),
        (1000000, "input_pc"),
        (2000000, "input_ap"),
        (4000000, "input_fp"),
        (1, "stored_fp_id"),
        (256, "stored_fp_limb_0"),
        (132, "stored_fp_limb_1"),
        (15, "stored_fp_limb_2"),
        (0, "stored_fp_limb_3"),
        (0, "partial_limb_msb"),
        (2, "stored_ret_pc_id"),
        (66, "stored_ret_pc_limb_0"),
        (417, "stored_ret_pc_limb_1"),
        (3, "stored_ret_pc_limb_2"),
        (0, "stored_ret_pc_limb_3"),
        (0, "partial_limb_msb"),
        (3, "distance_to_next_pc_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (500, "distance_to_next_pc_limb_0"),
        (0, "distance_to_next_pc_limb_1"),
        (0, "distance_to_next_pc_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_relative_call_negative() {
    let state = build_and_test(false, None, -17, [50, 200, 150]);

    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (1, "stored_fp_id"),
        (150, "stored_fp_limb_0"),
        (0, "stored_fp_limb_1"),
        (0, "stored_fp_limb_2"),
        (0, "stored_fp_limb_3"),
        (0, "partial_limb_msb"),
        (2, "stored_ret_pc_id"),
        (52, "stored_ret_pc_limb_0"),
        (0, "stored_ret_pc_limb_1"),
        (0, "stored_ret_pc_limb_2"),
        (0, "stored_ret_pc_limb_3"),
        (0, "partial_limb_msb"),
        (3, "distance_to_next_pc_id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (496, "distance_to_next_pc_limb_0"),
        (511, "distance_to_next_pc_limb_1"),
        (511, "distance_to_next_pc_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_call_base_fp_positive_offset2() {
    let state = build_and_test(true, Some(5), 600, [50, 200, 150]);

    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32773, "offset2"),
        (1, "op1_base_fp"),
        (1, "stored_fp_id"),
        (150, "stored_fp_limb_0"),
        (0, "stored_fp_limb_1"),
        (0, "stored_fp_limb_2"),
        (0, "stored_fp_limb_3"),
        (0, "partial_limb_msb"),
        (2, "stored_ret_pc_id"),
        (51, "stored_ret_pc_limb_0"),
        (0, "stored_ret_pc_limb_1"),
        (0, "stored_ret_pc_limb_2"),
        (0, "stored_ret_pc_limb_3"),
        (0, "partial_limb_msb"),
        (150, "mem1_base"),
        (3, "next_pc_id"),
        (88, "next_pc_limb_0"),
        (1, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_call_base_fp_negative_offset2() {
    let state = build_and_test(true, Some(-5), 400, [50, 200, 150]);

    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32763, "offset2"),
        (1, "op1_base_fp"),
        (1, "stored_fp_id"),
        (150, "stored_fp_limb_0"),
        (0, "stored_fp_limb_1"),
        (0, "stored_fp_limb_2"),
        (0, "stored_fp_limb_3"),
        (0, "partial_limb_msb"),
        (2, "stored_ret_pc_id"),
        (51, "stored_ret_pc_limb_0"),
        (0, "stored_ret_pc_limb_1"),
        (0, "stored_ret_pc_limb_2"),
        (0, "stored_ret_pc_limb_3"),
        (0, "partial_limb_msb"),
        (150, "mem1_base"),
        (3, "next_pc_id"),
        (400, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_call_base_ap_positive_offset2() {
    let state = build_and_test(false, Some(10), 1234, [50, 200, 150]);

    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32778, "offset2"),
        (0, "op1_base_fp"),
        (1, "stored_fp_id"),
        (150, "stored_fp_limb_0"),
        (0, "stored_fp_limb_1"),
        (0, "stored_fp_limb_2"),
        (0, "stored_fp_limb_3"),
        (0, "partial_limb_msb"),
        (2, "stored_ret_pc_id"),
        (51, "stored_ret_pc_limb_0"),
        (0, "stored_ret_pc_limb_1"),
        (0, "stored_ret_pc_limb_2"),
        (0, "stored_ret_pc_limb_3"),
        (0, "partial_limb_msb"),
        (200, "mem1_base"),
        (3, "next_pc_id"),
        (210, "next_pc_limb_0"),
        (2, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_call_base_ap_negative_offset2() {
    let state = build_and_test(false, Some(-10), 55, [50, 200, 150]);

    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32758, "offset2"),
        (0, "op1_base_fp"),
        (1, "stored_fp_id"),
        (150, "stored_fp_limb_0"),
        (0, "stored_fp_limb_1"),
        (0, "stored_fp_limb_2"),
        (0, "stored_fp_limb_3"),
        (0, "partial_limb_msb"),
        (2, "stored_ret_pc_id"),
        (51, "stored_ret_pc_limb_0"),
        (0, "stored_ret_pc_limb_1"),
        (0, "stored_ret_pc_limb_2"),
        (0, "stored_ret_pc_limb_3"),
        (0, "partial_limb_msb"),
        (200, "mem1_base"),
        (3, "next_pc_id"),
        (55, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

pub fn assemble_call(offset2: i16, flags: &Flags, op1_base_fp: bool) -> u128 {
    let call_op1_off = flags.pc_update_jump_rel.map(|b| if b { 1 } else { offset2 }).unwrap();
    assemble_instruction(
        0,
        1,
        call_op1_off,
        flags.clone().non_constants_to_arr(&[op1_base_fp, !op1_base_fp]),
        OpcodeExtension::Stone,
    )
}
