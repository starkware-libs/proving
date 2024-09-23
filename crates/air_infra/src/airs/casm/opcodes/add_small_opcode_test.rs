use super::super::casm_state::*;
use super::super::common::*;
use super::add_small_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;
use crate::expr;

// TODO: Support testing with negative dst/op0/op1, and add such test(s)
fn test_add_small(
    non_consts_flags: [bool; 6],
    offset_values: [i16; 3],
    dst: u32,
    op0: u32,
    op1: u32,
    entry_file_name: Option<&str>,
    expected_state: Vec<u32>,
) {
    // Read the non-constant flags
    let [flag_dst_base_fp, flag_op0_base_fp, flag_op1_imm, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1] =
        non_consts_flags;

    let [offset_dst_val, offset0_val, mut offset1_val] = offset_values;
    if flag_op1_imm {
        offset1_val = 1;
    }

    // Create the air function
    let mut add_small_opcode = AddSmallOpcode {
        is_imm: flag_op1_imm,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let pc_value = 10;
    let ap_value = 50;
    let fp_value = 100;

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);

    // Cretae the non-constant flags
    let non_consts_flags = if flag_op1_imm {
        vec![flag_dst_base_fp, flag_op0_base_fp, flag_ap_update_add_1]
    } else {
        vec![
            flag_dst_base_fp,
            flag_op0_base_fp,
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
                offset_dst_val,
                offset0_val,
                offset1_val,
                add_small_opcode
                    .get_flags()
                    .non_constants_to_arr(&non_consts_flags),
            ) as u128,
            0
        ),
    )];
    if flag_dst_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset_dst_val) as u32),
            const_felt252_expr!(dst as u128, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset_dst_val) as u32),
            const_felt252_expr!(dst as u128, 0),
        ));
    };
    if flag_op0_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset0_val) as u32),
            const_felt252_expr!(op0 as u128, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset0_val) as u32),
            const_felt252_expr!(op0 as u128, 0),
        ));
    }
    if flag_op1_imm {
        memory_values.push((
            const_expr!(pc_value + 1),
            const_felt252_expr!(op1 as u128, 0),
        ));
    } else if flag_op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset1_val) as u32),
            const_felt252_expr!(op1 as u128, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset1_val) as u32),
            const_felt252_expr!(op1 as u128, 0),
        ));
    };
    add_small_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function

    let registry = AirFnRegistry::new(&add_small_opcode);
    let (state, next_state) = registry.run_air(
        &add_small_opcode,
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

    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check entry
    // Check entry
    if let Some(entry_file_name) = entry_file_name {
        compare_test_json(
            &registry,
            &add_small_opcode.name(),
            &(TEST_JSONS_OPCODES_DIR.to_owned() + entry_file_name),
        );
    }
}

#[test]
fn test_add_small_not_imm() {
    test_add_small(
        [true, false, false, false, true, false],
        [3, 5, 7],
        90125677,
        77779999,
        12345678,
        Some("add_small_not_imm.json"),
        vec![
            10, 50, 100, 3, 64, 1, 1, 16, 7, 0, 4, 1, 0, 0, 1, 0, 0, 1, 0, 0, 365, 410, 343, 2, 0,
            0, 31, 362, 296, 3, 0, 0, 334, 48, 47,
        ],
    );
}

#[test]
#[should_panic]
fn test_add_mod_not_equal() {
    test_add_small(
        [false, true, true, false, false, true],
        [3, 5, 7],
        90124653,
        77779999,
        12345678,
        None,
        vec![],
    );
}

#[test]
#[should_panic]
fn test_add_mod_over_27bit() {
    test_add_small(
        [false, true, true, false, false, true],
        [3, 5, 7],
        134217728,
        134217727,
        1,
        None,
        vec![],
    );
}

#[test]
fn test_add_small_imm() {
    test_add_small(
        [true, false, true, false, true, false],
        [-3, -5, 1],
        90125677,
        77779999,
        12345678,
        Some("add_small_imm.json"),
        vec![
            10, 50, 100, 509, 63, 3, 510, 15, 1, 0, 0, 0, 1, 0, 0, 365, 410, 343, 2, 0, 0, 31, 362,
            296, 3, 0, 0, 334, 48, 47,
        ],
    );
}
