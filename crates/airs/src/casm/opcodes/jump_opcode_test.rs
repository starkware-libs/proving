use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::state::State;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::jump_opcode::*;
use crate::casm::common::*;

fn test_jump_opcode(
    non_consts_flags: [bool; 6],
    op0: i64,
    op1: i64,
    offsets_value: [Option<i16>; 2],
) -> State {
    let [rel, imm, double_deref, op0_base_fp, op1_base_fp, ap_update_add_1] = non_consts_flags;
    // Create the air function
    let mut jump_opcode = JumpOpcode { rel, imm, double_deref, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let pc = 3;
    let ap = 11;
    let fp = 6;

    // Create the non-constant imm_jump
    let non_consts_flags = if imm {
        vec![ap_update_add_1]
    } else if double_deref {
        vec![op0_base_fp, ap_update_add_1]
    } else {
        vec![op1_base_fp, !op1_base_fp, ap_update_add_1]
    };

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_jump(
                offsets_value[0],
                offsets_value[1],
                jump_opcode.get_flags().non_constants_to_arr(&non_consts_flags),
            ),
            0
        ),
    )];
    if imm {
        memory_values.push((const_expr!(pc + 1), const_felt252_expr!(op1)));
    } else if double_deref {
        memory_values.push((
            const_expr!((op0 as i32 + offsets_value[1].unwrap() as i32) as u32),
            const_felt252_expr!(op1),
        ));
        if op0_base_fp {
            memory_values.push((
                const_expr!((fp as i16 + offsets_value[0].unwrap()) as u32),
                const_felt252_expr!(op0),
            ));
        } else {
            memory_values.push((
                const_expr!((ap as i16 + offsets_value[0].unwrap()) as u32),
                const_felt252_expr!(op0),
            ));
        }
    } else if op1_base_fp {
        memory_values.push((
            const_expr!((fp as i16 + offsets_value[1].unwrap()) as u32),
            const_felt252_expr!(op1),
        ));
    } else {
        memory_values.push((
            const_expr!((ap as i16 + offsets_value[1].unwrap()) as u32),
            const_felt252_expr!(op1),
        ));
    }
    jump_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&jump_opcode);
    let (state, next_state) = registry.run_air(
        &jump_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    if rel {
        assert_eq!(next_state.pc().calc(), (pc as i64 + op1).to_string());
    } else {
        assert_eq!(next_state.pc().calc(), op1.to_string());
    }
    assert_eq!(next_state.fp().calc(), fp.to_string());
    if ap_update_add_1 {
        assert_eq!(next_state.ap().calc(), (ap + 1).to_string());
    } else {
        assert_eq!(next_state.ap().calc(), ap.to_string());
    }

    state
}

#[test]
fn test_abs_jump_base_ap() {
    let state =
        test_jump_opcode([false, false, false, false, false, false], 125, 8, [None, Some(2)]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32770, "offset2"),
        (0, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (11, "mem1_base"),
        (1, "next_pc_id"),
        (8, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_abs_jump_base_fp() {
    let state =
        test_jump_opcode([false, false, false, false, true, false], 125, 5, [None, Some(10)]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32778, "offset2"),
        (1, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (6, "mem1_base"),
        (1, "next_pc_id"),
        (5, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_abs_jump_base_ap_inc_ap() {
    let state =
        test_jump_opcode([false, false, false, false, false, true], 125, 8, [None, Some(2)]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32770, "offset2"),
        (0, "op1_base_fp"),
        (1, "ap_update_add_1"),
        (11, "mem1_base"),
        (1, "next_pc_id"),
        (8, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_abs_jump_base_fp_inc_ap() {
    let state =
        test_jump_opcode([false, false, false, false, true, true], 125, 5, [None, Some(10)]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32778, "offset2"),
        (1, "op1_base_fp"),
        (1, "ap_update_add_1"),
        (6, "mem1_base"),
        (1, "next_pc_id"),
        (5, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_abs_big_op1() {
    let state = test_jump_opcode(
        [false, false, false, false, false, false],
        125,
        1684685,
        [None, Some(402)],
    );

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (33170, "offset2"),
        (0, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (11, "mem1_base"),
        (1, "next_pc_id"),
        (205, "next_pc_limb_0"),
        (218, "next_pc_limb_1"),
        (6, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_abs_jump_negative_offset() {
    let state =
        test_jump_opcode([false, false, false, false, false, false], 125, 9, [None, Some(-9)]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32759, "offset2"),
        (0, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (11, "mem1_base"),
        (1, "next_pc_id"),
        (9, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_rel_jump() {
    let state = test_jump_opcode([true, true, false, false, false, false], 125, 100, [None, None]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (0, "ap_update_add_1"),
        (1, "next_pc_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (100, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_rel_jump_inc_ap() {
    let state = test_jump_opcode([true, true, false, false, false, true], 125, 3, [None, None]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (1, "ap_update_add_1"),
        (1, "next_pc_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (3, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_rel_big_op1() {
    let state =
        test_jump_opcode([true, true, false, false, false, false], 125, 54687687, [None, None]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (0, "ap_update_add_1"),
        (1, "next_pc_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (455, "next_pc_limb_0"),
        (315, "next_pc_limb_1"),
        (208, "next_pc_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_rel_negative_imm() {
    let state = test_jump_opcode([true, true, false, false, false, false], 125, -2, [None, None]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (0, "ap_update_add_1"),
        (1, "next_pc_id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (511, "next_pc_limb_0"),
        (511, "next_pc_limb_1"),
        (511, "next_pc_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_rel_negative_op1() {
    let state =
        test_jump_opcode([true, false, false, false, false, false], 125, -2, [None, Some(333)]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (33101, "offset2"),
        (0, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (11, "mem1_base"),
        (1, "next_pc_id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (511, "next_pc_limb_0"),
        (511, "next_pc_limb_1"),
        (511, "next_pc_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_rel_deref_base_fp() {
    let state =
        test_jump_opcode([true, false, false, false, true, true], 125, 16584, [None, Some(12345)]);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (45113, "offset2"),
        (1, "op1_base_fp"),
        (1, "ap_update_add_1"),
        (6, "mem1_base"),
        (1, "next_pc_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (200, "next_pc_limb_0"),
        (32, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_abs_double_deref() {
    let state = test_jump_opcode(
        [false, false, true, true, true, true],
        125,
        16584,
        [Some(4654), Some(12345)],
    );

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (37422, "offset1"),
        (45113, "offset2"),
        (1, "op0_base_fp"),
        (1, "ap_update_add_1"),
        (6, "mem0_base"),
        (2, "mem1_base_id"),
        (125, "mem1_base_limb_0"),
        (0, "mem1_base_limb_1"),
        (0, "mem1_base_limb_2"),
        (0, "mem1_base_limb_3"),
        (0, "partial_limb_msb"),
        (1, "next_pc_id"),
        (200, "next_pc_limb_0"),
        (32, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "Immediate jump must be relative")]
fn test_abs_immediate() {
    test_jump_opcode(
        [false, true, false, false, false, true],
        125,
        16584,
        [Some(4654), Some(12345)],
    );
}

#[test]
#[should_panic(expected = "Double deref jump must be absolute")]
fn test_rel_double_deref() {
    test_jump_opcode(
        [true, false, true, true, false, false],
        125,
        16584,
        [Some(4654), Some(12345)],
    );
}

pub fn assemble_jump(op0_off: Option<i16>, op1_off: Option<i16>, flags: [bool; 15]) -> u128 {
    let off0 = op0_off.map_or(-1, |v| v);
    let off1 = op1_off.map_or(1, |v| v);
    assemble_instruction(-1, off0, off1, flags, OpcodeExtension::Stone)
}
