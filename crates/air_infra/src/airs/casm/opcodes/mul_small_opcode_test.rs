use compiled_casm_air::utils::JSONS_OPCODES_DIR;

use super::super::casm_state::*;
use super::super::common::*;
use super::mul_small_opcode::*;

use crate::airs::felt252_id_memory::memory::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::state::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;

#[test]
fn test_entry_json() {
    let (_, entry) = AirFnRegistry::new(&MulSmallOpcode {
        is_imm: true,
        memory: Felt252IdMemory::default(),
    });
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_OPCODES_DIR, name),
    );

    let (_, entry) = AirFnRegistry::new(&MulSmallOpcode {
        is_imm: false,
        memory: Felt252IdMemory::default(),
    });
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_OPCODES_DIR, name),
    );
}

fn test_mul_small(
    non_consts_flags: [bool; 6],
    offset_values: [i16; 3],
    dst: u32,
    op0: u32,
    op1: u32,
    expected_state: State,
) {
    // Read the non-constant flags
    let [flag_dst_base_fp, flag_op0_base_fp, flag_op1_imm, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1] =
        non_consts_flags;

    let [offset_dst_val, offset0_val, mut offset1_val] = offset_values;
    if flag_op1_imm {
        offset1_val = 1;
    }

    // Create the air function
    let mut mul_small_opcode = MulSmallOpcode {
        is_imm: flag_op1_imm,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let pc_value = 10;
    let ap_value = 50;
    let fp_value = 100;

    let pc = const_expr!(pc_value);
    let ap = const_expr!(ap_value);
    let fp = const_expr!(fp_value);

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
                mul_small_opcode
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
    mul_small_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function

    let (registry, _) = AirFnRegistry::new(&mul_small_opcode);
    let (state, next_state) = registry.run_air(
        &mul_small_opcode,
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

    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
fn test_mul_small_not_imm() {
    test_mul_small(
        [true, false, false, false, true, false],
        [3, 5, 7],
        1042584088,
        32123,
        32456,
        vec![
            (10, ""),
            (50, ""),
            (100, ""),
            (32771, "offset_0"),
            (32773, "offset_1"),
            (32775, "offset_2"),
            (1, "dst_base_fp"),
            (0, "op0_base_fp"),
            (0, "op1_base_fp"),
            (1, "op1_base_ap"),
            (0, "ap_update_add_1"),
            (1, "id"),
            (24, "limb_0"),
            (73, "limb_1"),
            (393, "limb_2"),
            (7, "limb_3"),
            (2, "id"),
            (379, "limb_0"),
            (62, "limb_1"),
            (3, "id"),
            (200, "limb_0"),
            (63, "limb_1"),
        ]
        .into(),
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evalutate to 0)")]
fn test_mul_small_not_equal() {
    test_mul_small(
        [false, true, true, false, false, true],
        [3, 5, 7],
        1042584088,
        32123,
        32457,
        vec![].into(),
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck6 on input 64")]
fn test_mul_small_over_15bit() {
    test_mul_small(
        [false, true, true, false, false, true],
        [3, 5, 7],
        32768,
        32768,
        1,
        vec![].into(),
    );
}

#[test]
fn test_mul_small_imm() {
    test_mul_small(
        [true, false, true, false, true, false],
        [-3, -5, 1],
        56,
        7,
        8,
        vec![
            (10, ""),
            (50, ""),
            (100, ""),
            (32765, "offset_0"),
            (32763, "offset_1"),
            (1, "dst_base_fp"),
            (0, "op0_base_fp"),
            (0, "ap_update_add_1"),
            (1, "id"),
            (56, "limb_0"),
            (0, "limb_1"),
            (0, "limb_2"),
            (0, "limb_3"),
            (2, "id"),
            (7, "limb_0"),
            (0, "limb_1"),
            (3, "id"),
            (8, "limb_0"),
            (0, "limb_1"),
        ]
        .into(),
    );
}
