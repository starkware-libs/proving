use super::super::casm_state::*;
use super::super::common::*;
use super::call_opcode::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::state::*;
use crate::core::variables::*;
use crate::{const_expr, const_felt252_expr};

fn build_and_test(
    op1_base_fp: bool,
    offset2_option: Option<i16>,
    op1_value: i64,
    expected_state: State,
) {
    let [pc_value, ap_value, fp_value] = [50, 200, 150];
    let [pc, ap, fp] = [
        const_expr!(pc_value),
        const_expr!(ap_value),
        const_expr!(fp_value),
    ];

    let is_rel = offset2_option.is_none();
    let offset2 = offset2_option.unwrap_or(1);

    let mut call_opcode = CallOpcode {
        is_rel,
        op1_base_fp,
        memory: Felt252IdMemory::default(),
    };

    // Fill memory
    let mut memory_values = vec![
        (
            pc.clone(),
            const_felt252_expr!(assemble_call(offset2, &call_opcode.get_flags()) as u128, 0),
        ),
        (
            const_expr!(ap_value),
            const_felt252_expr!(fp_value as u128, 0),
        ),
        (
            const_expr!(ap_value + 1),
            const_felt252_expr!((pc_value + (if is_rel { 2 } else { 1 })) as u128, 0),
        ),
    ];

    let op1_value_252 = const_felt252_expr!(op1_value);
    if is_rel {
        memory_values.push((const_expr!(pc_value + 1), op1_value_252));
    } else if op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset2) as u32),
            op1_value_252,
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset2) as u32),
            op1_value_252,
        ));
    }

    call_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&call_opcode);
    let (state, next_state) =
        registry.run_air(&call_opcode, CasmStateVar::new(pc, ap.clone(), fp.clone()));

    // Check output
    if is_rel {
        assert_eq!(
            next_state.pc.calc(),
            (pc_value as i128 + op1_value as i128).to_string()
        );
    } else {
        assert_eq!(next_state.pc.calc(), op1_value.to_string());
    }
    assert_eq!(next_state.ap.calc(), (ap_value + 2).to_string());
    assert_eq!(next_state.fp.calc(), (ap_value + 2).to_string());

    // Check state
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
fn test_relative_call() {
    build_and_test(
        false,
        None,
        500,
        vec![
            (50, "input_pc"),
            (200, "input_ap"),
            (150, "input_fp"),
            (1, "ap_id"),
            (2, "ap_plus_one_id"),
            (3, "next_pc_id"),
            (0, "msb"),
            (0, "mid_limbs_set"),
            (500, "next_pc_limb_0"),
            (0, "next_pc_limb_1"),
            (0, "next_pc_limb_2"),
        ]
        .into(),
    );
}

#[test]
fn test_relative_call_negative() {
    build_and_test(
        false,
        None,
        -17,
        vec![
            (50, "input_pc"),
            (200, "input_ap"),
            (150, "input_fp"),
            (1, "ap_id"),
            (2, "ap_plus_one_id"),
            (3, "next_pc_id"),
            (1, "msb"),
            (1, "mid_limbs_set"),
            (496, "next_pc_limb_0"),
            (511, "next_pc_limb_1"),
            (511, "next_pc_limb_2"),
        ]
        .into(),
    );
}

#[test]
fn test_call_base_fp_positive_offset2() {
    build_and_test(
        true,
        Some(5),
        600,
        vec![
            (50, "input_pc"),
            (200, "input_ap"),
            (150, "input_fp"),
            (32773, "offset2"),
            (1, "ap_id"),
            (2, "ap_plus_one_id"),
            (3, "next_pc_id"),
            (88, "next_pc_limb_0"),
            (1, "next_pc_limb_1"),
            (0, "next_pc_limb_2"),
        ]
        .into(),
    );
}

#[test]
fn test_call_base_fp_negative_offset2() {
    build_and_test(
        true,
        Some(-5),
        400,
        vec![
            (50, "input_pc"),
            (200, "input_ap"),
            (150, "input_fp"),
            (32763, "offset2"),
            (1, "ap_id"),
            (2, "ap_plus_one_id"),
            (3, "next_pc_id"),
            (400, "next_pc_limb_0"),
            (0, "next_pc_limb_1"),
            (0, "next_pc_limb_2"),
        ]
        .into(),
    );
}

#[test]
fn test_call_base_ap_positive_offset2() {
    build_and_test(
        false,
        Some(10),
        1234,
        vec![
            (50, "input_pc"),
            (200, "input_ap"),
            (150, "input_fp"),
            (32778, "offset2"),
            (1, "ap_id"),
            (2, "ap_plus_one_id"),
            (3, "next_pc_id"),
            (210, "next_pc_limb_0"),
            (2, "next_pc_limb_1"),
            (0, "next_pc_limb_2"),
        ]
        .into(),
    );
}

#[test]
fn test_call_base_ap_negative_offset2() {
    build_and_test(
        false,
        Some(-10),
        55,
        vec![
            (50, "input_pc"),
            (200, "input_ap"),
            (150, "input_fp"),
            (32758, "offset2"),
            (1, "ap_id"),
            (2, "ap_plus_one_id"),
            (3, "next_pc_id"),
            (55, "next_pc_limb_0"),
            (0, "next_pc_limb_1"),
            (0, "next_pc_limb_2"),
        ]
        .into(),
    );
}

pub fn assemble_call(offset2: i16, flags: &Flags) -> u64 {
    let call_op1_off = flags
        .pc_update_jump_rel
        .map(|b| if b { 1 } else { offset2 })
        .unwrap();
    assemble_instruction(0, 1, call_op1_off, flags.clone().into())
}
