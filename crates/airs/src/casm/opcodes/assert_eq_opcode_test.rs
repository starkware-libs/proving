use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::state::State;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::assert_eq_opcode::*;
use crate::casm::common::*;

// [fp + offset] == [ap + offset]
#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_eq_deref() {
    test_assert_equal([true, false, false, false, true, false], 1, 4, 2);
}

// [ap + offset] == imm
#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_eq_imm() {
    test_assert_equal([false, false, true, false, false, false], 3, 4, 5);
}

// [ap + offset] == [[fp + offset] + offset]
#[test]
#[should_panic]
fn test_assert_not_eq_double_deref() {
    test_assert_equal([false, true, false, false, false, false], 15, 6, 16);
}

#[test]
fn test_assert_eq_double_deref_big_op0() {
    let state = test_assert_equal([true, false, false, false, false, false], 15, 1546487, 15);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32771, "offset0"),
        (32775, "offset1"),
        (32770, "offset2"),
        (1, "dst_base_fp"),
        (0, "op0_base_fp"),
        (0, "ap_update_add_1"),
        (6, "mem_dst_base"),
        (11, "mem0_base"),
        (2, "mem1_base_id"),
        (247, "mem1_base_limb_0"),
        (460, "mem1_base_limb_1"),
        (5, "mem1_base_limb_2"),
        (0, "mem1_base_limb_3"),
        (0, "partial_limb_msb"),
        (1, "dst_id"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_eq_double_deref_big_op0() {
    test_assert_equal([true, false, false, false, false, false], 15, 454687, 78);
}

// [ap + offset] == [fp + offset]
#[test]
fn test_assert_eq_deref() {
    let state = test_assert_equal([false, false, false, true, false, false], 15, 4, 15);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32771, "offset0"),
        (32770, "offset2"),
        (0, "dst_base_fp"),
        (1, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (11, "mem_dst_base"),
        (6, "mem1_base"),
        (1, "dst_id"),
    "#]]
    .assert_eq(&state.to_string());
}

// [fp + offset] == imm
#[test]
fn test_assert_eq_imm() {
    let state = test_assert_equal([true, false, true, false, false, false], 15, 4, 15);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32771, "offset0"),
        (1, "dst_base_fp"),
        (0, "ap_update_add_1"),
        (6, "mem_dst_base"),
        (1, "dst_id"),
    "#]]
    .assert_eq(&state.to_string());
}

// [fp + offset] == [[ap + offset] + offset]
#[test]
fn test_assert_eq_double_deref() {
    let state = test_assert_equal([true, false, false, false, false, false], 15, 4, 15);

    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32771, "offset0"),
        (32775, "offset1"),
        (32770, "offset2"),
        (1, "dst_base_fp"),
        (0, "op0_base_fp"),
        (0, "ap_update_add_1"),
        (6, "mem_dst_base"),
        (11, "mem0_base"),
        (2, "mem1_base_id"),
        (4, "mem1_base_limb_0"),
        (0, "mem1_base_limb_1"),
        (0, "mem1_base_limb_2"),
        (0, "mem1_base_limb_3"),
        (0, "partial_limb_msb"),
        (1, "dst_id"),
    "#]]
    .assert_eq(&state.to_string());
}

fn test_assert_equal(non_consts_flags: [bool; 6], dst: u128, op0: u128, op1: u128) -> State {
    // Read the non-constant flags
    let [
        flag_dst_base_fp,
        flag_op0_base_fp,
        flag_op1_imm,
        flag_op1_base_fp,
        flag_op1_base_ap,
        flag_ap_update_add_1,
    ] = non_consts_flags;

    // Create the air function
    let double_deref = !flag_op1_imm && !flag_op1_base_fp && !flag_op1_base_ap;
    let mut assert_equal_opcode =
        AssertEqOpcode { double_deref, imm: flag_op1_imm, memory: Felt252IdMemory::default() };

    let offset0_value = 3;
    let offset1_value = if double_deref { 7 } else { -1 };
    let offset2_value = if flag_op1_imm { 1 } else { 2 };

    // Register values at opcode start
    let pc_value = 3;
    let ap_value = 11;
    let fp_value = 6;
    let pc = const_expr!(pc_value);
    let ap = const_expr!(ap_value);
    let fp = const_expr!(fp_value);

    // Create the non-constant flags
    let non_consts_flags = if flag_op1_imm {
        vec![flag_dst_base_fp, flag_ap_update_add_1]
    } else if double_deref {
        vec![flag_dst_base_fp, flag_op0_base_fp, flag_ap_update_add_1]
    } else {
        vec![flag_dst_base_fp, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1]
    };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                offset0_value,
                offset1_value,
                offset2_value,
                assert_equal_opcode.get_flags().non_constants_to_arr(&non_consts_flags),
                OpcodeExtension::Stone
            ),
            0
        ),
    )];
    if flag_dst_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset0_value) as u32),
            const_felt252_expr!(dst, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset0_value) as u32),
            const_felt252_expr!(dst, 0),
        ));
    };
    if flag_op1_imm {
        memory_values.push((const_expr!(pc_value + 1), const_felt252_expr!(op1, 0)));
    } else if double_deref {
        memory_values.push((
            const_expr!((op0 as i32 + offset2_value as i32) as u32),
            const_felt252_expr!(op1, 0),
        ));
        if flag_op0_base_fp {
            memory_values.push((
                const_expr!((fp_value as i16 + offset1_value) as u32),
                const_felt252_expr!(op0, 0),
            ));
        } else {
            memory_values.push((
                const_expr!((ap_value as i16 + offset1_value) as u32),
                const_felt252_expr!(op0, 0),
            ));
        }
    } else if flag_op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset2_value) as u32),
            const_felt252_expr!(op1, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset2_value) as u32),
            const_felt252_expr!(op1, 0),
        ));
    };
    assert_equal_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function

    let (registry, _) = AirFnRegistry::new(&assert_equal_opcode);
    let (state, next_state) = registry.run_air(
        &assert_equal_opcode,
        (),
        CasmStateVar::new(pc.clone(), ap.clone(), fp.clone()),
    );

    // Check output
    assert_eq!(next_state.fp().var.calc(), fp.calc());
    if flag_ap_update_add_1 {
        assert_eq!(next_state.ap().var.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_state.ap().var.calc(), ap.calc());
    }
    if flag_op1_imm {
        assert_eq!(next_state.pc().var.calc(), (pc_value + 2).to_string());
    } else {
        assert_eq!(next_state.pc().var.calc(), (pc_value + 1).to_string());
    };

    state
}
