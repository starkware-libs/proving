use super::super::casm_state::*;
use super::super::common::*;
use super::assert_eq_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
#[cfg(test)]
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;

// [fp + offset] == [ap + offset]
#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_eq_deref() {
    test_assert_equal(
        [true, false, false, false, true, false],
        1,
        4,
        2,
        None,
        vec![],
    );
}

// [ap + offset] == imm
#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_eq_imm() {
    test_assert_equal(
        [false, false, true, false, false, false],
        3,
        4,
        5,
        None,
        vec![],
    );
}

// [ap + offset] == [[fp + offset] + offset]
#[test]
#[should_panic]
fn test_assert_not_eq_double_deref() {
    test_assert_equal(
        [false, true, false, false, false, false],
        15,
        6,
        16,
        None,
        vec![],
    );
}

#[test]
fn test_assert_eq_double_deref_big_op0() {
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        1546487,
        15,
        None,
        vec![3, 11, 6, 32771, 32775, 32770, 1, 0, 0, 2, 247, 460, 5, 1],
    );
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_eq_double_deref_big_op0() {
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        454687,
        78,
        None,
        vec![],
    );
}

// [ap + offset] == [fp + offset]
#[test]
fn test_assert_eq_deref() {
    test_assert_equal(
        [false, false, false, true, false, false],
        15,
        4,
        15,
        Some("assert_eq_deref.json"),
        vec![3, 11, 6, 32771, 32770, 0, 1, 0, 0, 1],
    );
}

// [fp + offset] == imm
#[test]
fn test_assert_eq_imm() {
    test_assert_equal(
        [true, false, true, false, false, false],
        15,
        4,
        15,
        Some("assert_eq_imm.json"),
        vec![3, 11, 6, 32771, 1, 0, 1],
    );
}

// [fp + offset] == [[ap + offset] + offset]
#[test]
fn test_assert_eq_double_deref() {
    let expected_state = vec![3, 11, 6, 32771, 32775, 32770, 1, 0, 0, 2, 4, 0, 0, 1];
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        4,
        15,
        Some("assert_eq_double_deref.json"),
        expected_state,
    );
}

fn test_assert_equal(
    non_consts_flags: [bool; 6],
    dst: u128,
    op0: u128,
    op1: u128,
    entry_file_name: Option<&str>,
    expected_state: Vec<u32>,
) {
    // Read the non-constant flags
    let [flag_dst_base_fp, flag_op0_base_fp, flag_op1_imm, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1] =
        non_consts_flags;

    // Create the air function
    let double_deref = !flag_op1_imm && !flag_op1_base_fp && !flag_op1_base_ap;
    let mut assert_equal_opcode = AssertEqOpcode {
        is_double_deref: double_deref,
        is_imm: flag_op1_imm,
        memory: Felt252IdMemory::default(),
    };

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
        vec![
            flag_dst_base_fp,
            flag_op1_base_fp,
            flag_op1_base_ap,
            flag_ap_update_add_1,
        ]
    };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                offset0_value,
                offset1_value,
                offset2_value,
                assert_equal_opcode
                    .get_flags()
                    .non_constants_to_arr(&non_consts_flags),
            ) as u128,
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

    let registry = AirFnRegistry::new(&assert_equal_opcode);
    let (state, next_state) = registry.run_air(
        &assert_equal_opcode,
        CasmStateVar::new(pc.clone(), ap.clone(), fp.clone()),
    );

    // Check output
    assert_eq!(next_state.fp.calc(), fp.calc());
    if flag_ap_update_add_1 {
        assert_eq!(next_state.ap.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_state.ap.calc(), ap.calc());
    }
    if flag_op1_imm {
        assert_eq!(next_state.pc.calc(), (pc_value + 2).to_string());
    } else {
        assert_eq!(next_state.pc.calc(), (pc_value + 1).to_string());
    };

    // Check state
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check entry
    if let Some(entry_file_name) = entry_file_name {
        compare_json(
            &registry.get_air_fn_entry(&assert_equal_opcode.name()),
            &(TEST_JSONS_OPCODES_DIR.to_owned() + entry_file_name),
        );
    }
}
