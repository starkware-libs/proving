use compiled_casm_air::utils::JSONS_OPCODES_DIR;

use super::super::casm_state::*;
use super::super::common::*;
use super::add_opcode::*;

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
    let (_, entry) = AirFnRegistry::new(&AddOpcode {
        is_small: true,
        is_imm: true,
        memory: Felt252IdMemory::default(),
    });
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_OPCODES_DIR, name),
    );

    let (_, entry) = AirFnRegistry::new(&AddOpcode {
        is_small: false,
        is_imm: false,
        memory: Felt252IdMemory::default(),
    });
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_OPCODES_DIR, name),
    );

    let (_, entry) = AirFnRegistry::new(&AddOpcode {
        is_small: true,
        is_imm: false,
        memory: Felt252IdMemory::default(),
    });
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_OPCODES_DIR, name),
    );

    let (_, entry) = AirFnRegistry::new(&AddOpcode {
        is_small: false,
        is_imm: true,
        memory: Felt252IdMemory::default(),
    });
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_OPCODES_DIR, name),
    );
}

// TODO: Support testing with negative dst/op0/op1, and add such test(s)
fn test_add_opcode(
    non_consts_flags: [bool; 7],
    offset_values: [i16; 3],
    dst: Felt252Expr,
    op0: Felt252Expr,
    op1: Felt252Expr,
    expected_state: State,
) {
    // Read the non-constant flags
    let [add_small, flag_dst_base_fp, flag_op0_base_fp, flag_op1_imm, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1] =
        non_consts_flags;

    let [offset_dst_val, offset0_val, mut offset1_val] = offset_values;
    if flag_op1_imm {
        offset1_val = 1;
    }

    // Create the air function
    let mut add_small_opcode = AddOpcode {
        is_small: add_small,
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
                add_small_opcode
                    .get_flags()
                    .non_constants_to_arr(&non_consts_flags),
            ) as u128,
            0
        ),
    )];
    if flag_dst_base_fp {
        memory_values.push((const_expr!((fp_value as i16 + offset_dst_val) as u32), dst));
    } else {
        memory_values.push((const_expr!((ap_value as i16 + offset_dst_val) as u32), dst));
    };
    if flag_op0_base_fp {
        memory_values.push((const_expr!((fp_value as i16 + offset0_val) as u32), op0));
    } else {
        memory_values.push((const_expr!((ap_value as i16 + offset0_val) as u32), op0));
    }
    if flag_op1_imm {
        memory_values.push((const_expr!(pc_value + 1), op1));
    } else if flag_op1_base_fp {
        memory_values.push((const_expr!((fp_value as i16 + offset1_val) as u32), op1));
    } else {
        memory_values.push((const_expr!((ap_value as i16 + offset1_val) as u32), op1));
    };
    add_small_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function

    let (registry, _) = AirFnRegistry::new(&add_small_opcode);
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

    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
fn test_add_small_not_imm() {
    test_add_opcode(
        [true, true, false, false, false, true, false],
        [3, 5, 7],
        const_felt252_expr!(90125677),
        const_felt252_expr!(77779999),
        const_felt252_expr!(12345678),
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
            (0, "msb"),
            (0, "mid_limbs_set"),
            (365, "limb_0"),
            (410, "limb_1"),
            (343, "limb_2"),
            (2, "id"),
            (0, "msb"),
            (0, "mid_limbs_set"),
            (31, "limb_0"),
            (362, "limb_1"),
            (296, "limb_2"),
            (3, "id"),
            (0, "msb"),
            (0, "mid_limbs_set"),
            (334, "limb_0"),
            (48, "limb_1"),
            (47, "limb_2"),
        ]
        .into(),
    );
}

#[test]
#[should_panic]
fn test_add_small_not_equal() {
    test_add_opcode(
        [true, false, true, true, false, false, true],
        [3, 5, 7],
        const_felt252_expr!(90124653),
        const_felt252_expr!(77779999),
        const_felt252_expr!(12345678),
        vec![].into(),
    );
}

#[test]
#[should_panic]
fn test_add_small_over_27bit() {
    test_add_opcode(
        [true, false, true, true, false, false, true],
        [3, 5, 7],
        const_felt252_expr!(134217728),
        const_felt252_expr!(134217727),
        const_felt252_expr!(1),
        vec![].into(),
    );
}

#[test]
fn test_add_small_imm() {
    test_add_opcode(
        [true, true, false, true, false, true, false],
        [-3, -5, 1],
        const_felt252_expr!(90125677),
        const_felt252_expr!(77779999),
        const_felt252_expr!(12345678),
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
            (0, "msb"),
            (0, "mid_limbs_set"),
            (365, "limb_0"),
            (410, "limb_1"),
            (343, "limb_2"),
            (2, "id"),
            (0, "msb"),
            (0, "mid_limbs_set"),
            (31, "limb_0"),
            (362, "limb_1"),
            (296, "limb_2"),
            (3, "id"),
            (0, "msb"),
            (0, "mid_limbs_set"),
            (334, "limb_0"),
            (48, "limb_1"),
            (47, "limb_2"),
        ]
        .into(),
    );
}

#[test]
fn test_add_big_not_imm() {
    test_add_opcode(
        [false, false, true, false, false, true, false],
        [3, 5, 7],
        const_felt252_expr!(0x3000040002i128),
        const_felt252_expr!(0x1008020001i128),
        const_felt252_expr!(0x1ff8020001i128),
        vec![
            (10, ""),
            (50, ""),
            (100, ""),
            (32771, "offset_0"),
            (32773, "offset_1"),
            (32775, "offset_2"),
            (0, "dst_base_fp"),
            (1, "op0_base_fp"),
            (0, "op1_base_fp"),
            (1, "op1_base_ap"),
            (0, "ap_update_add_1"),
            (1, "id"),
            (2, "limb_0"),
            (0, "limb_1"),
            (1, "limb_2"),
            (0, "limb_3"),
            (3, "limb_4"),
            (0, "limb_5"),
            (0, "limb_6"),
            (0, "limb_7"),
            (0, "limb_8"),
            (0, "limb_9"),
            (0, "limb_10"),
            (0, "limb_11"),
            (0, "limb_12"),
            (0, "limb_13"),
            (0, "limb_14"),
            (0, "limb_15"),
            (0, "limb_16"),
            (0, "limb_17"),
            (0, "limb_18"),
            (0, "limb_19"),
            (0, "limb_20"),
            (0, "limb_21"),
            (0, "limb_22"),
            (0, "limb_23"),
            (0, "limb_24"),
            (0, "limb_25"),
            (0, "limb_26"),
            (0, "limb_27"),
            (2, "id"),
            (1, "limb_0"),
            (256, "limb_1"),
            (0, "limb_2"),
            (1, "limb_3"),
            (1, "limb_4"),
            (0, "limb_5"),
            (0, "limb_6"),
            (0, "limb_7"),
            (0, "limb_8"),
            (0, "limb_9"),
            (0, "limb_10"),
            (0, "limb_11"),
            (0, "limb_12"),
            (0, "limb_13"),
            (0, "limb_14"),
            (0, "limb_15"),
            (0, "limb_16"),
            (0, "limb_17"),
            (0, "limb_18"),
            (0, "limb_19"),
            (0, "limb_20"),
            (0, "limb_21"),
            (0, "limb_22"),
            (0, "limb_23"),
            (0, "limb_24"),
            (0, "limb_25"),
            (0, "limb_26"),
            (0, "limb_27"),
            (3, "id"),
            (1, "limb_0"),
            (256, "limb_1"),
            (0, "limb_2"),
            (511, "limb_3"),
            (1, "limb_4"),
            (0, "limb_5"),
            (0, "limb_6"),
            (0, "limb_7"),
            (0, "limb_8"),
            (0, "limb_9"),
            (0, "limb_10"),
            (0, "limb_11"),
            (0, "limb_12"),
            (0, "limb_13"),
            (0, "limb_14"),
            (0, "limb_15"),
            (0, "limb_16"),
            (0, "limb_17"),
            (0, "limb_18"),
            (0, "limb_19"),
            (0, "limb_20"),
            (0, "limb_21"),
            (0, "limb_22"),
            (0, "limb_23"),
            (0, "limb_24"),
            (0, "limb_25"),
            (0, "limb_26"),
            (0, "limb_27"),
            (0, "sub_p_bit"),
        ]
        .into(),
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evalutate to 0)")]
fn test_add_big_mod_not_equal() {
    test_add_opcode(
        [false, false, true, false, false, true, false],
        [3, 5, 7],
        const_felt252_expr!(0x3000040002u128, 1u128),
        const_felt252_expr!(0x1008020001i128),
        const_felt252_expr!(0x1ff8020001i128),
        vec![].into(),
    );
}

#[test]
fn test_add_big_imm() {
    test_add_opcode(
        [false, false, true, true, false, false, true],
        [3, 5, 1],
        const_felt252_expr!(0x3000040002i128),
        const_felt252_expr!(0x1008020001i128),
        const_felt252_expr!(0x1ff8020001i128),
        vec![
            (10, ""),
            (50, ""),
            (100, ""),
            (32771, "offset_0"),
            (32773, "offset_1"),
            (0, "dst_base_fp"),
            (1, "op0_base_fp"),
            (1, "ap_update_add_1"),
            (1, "id"),
            (2, "limb_0"),
            (0, "limb_1"),
            (1, "limb_2"),
            (0, "limb_3"),
            (3, "limb_4"),
            (0, "limb_5"),
            (0, "limb_6"),
            (0, "limb_7"),
            (0, "limb_8"),
            (0, "limb_9"),
            (0, "limb_10"),
            (0, "limb_11"),
            (0, "limb_12"),
            (0, "limb_13"),
            (0, "limb_14"),
            (0, "limb_15"),
            (0, "limb_16"),
            (0, "limb_17"),
            (0, "limb_18"),
            (0, "limb_19"),
            (0, "limb_20"),
            (0, "limb_21"),
            (0, "limb_22"),
            (0, "limb_23"),
            (0, "limb_24"),
            (0, "limb_25"),
            (0, "limb_26"),
            (0, "limb_27"),
            (2, "id"),
            (1, "limb_0"),
            (256, "limb_1"),
            (0, "limb_2"),
            (1, "limb_3"),
            (1, "limb_4"),
            (0, "limb_5"),
            (0, "limb_6"),
            (0, "limb_7"),
            (0, "limb_8"),
            (0, "limb_9"),
            (0, "limb_10"),
            (0, "limb_11"),
            (0, "limb_12"),
            (0, "limb_13"),
            (0, "limb_14"),
            (0, "limb_15"),
            (0, "limb_16"),
            (0, "limb_17"),
            (0, "limb_18"),
            (0, "limb_19"),
            (0, "limb_20"),
            (0, "limb_21"),
            (0, "limb_22"),
            (0, "limb_23"),
            (0, "limb_24"),
            (0, "limb_25"),
            (0, "limb_26"),
            (0, "limb_27"),
            (3, "id"),
            (1, "limb_0"),
            (256, "limb_1"),
            (0, "limb_2"),
            (511, "limb_3"),
            (1, "limb_4"),
            (0, "limb_5"),
            (0, "limb_6"),
            (0, "limb_7"),
            (0, "limb_8"),
            (0, "limb_9"),
            (0, "limb_10"),
            (0, "limb_11"),
            (0, "limb_12"),
            (0, "limb_13"),
            (0, "limb_14"),
            (0, "limb_15"),
            (0, "limb_16"),
            (0, "limb_17"),
            (0, "limb_18"),
            (0, "limb_19"),
            (0, "limb_20"),
            (0, "limb_21"),
            (0, "limb_22"),
            (0, "limb_23"),
            (0, "limb_24"),
            (0, "limb_25"),
            (0, "limb_26"),
            (0, "limb_27"),
            (0, "sub_p_bit"),
        ]
        .into(),
    );
}

#[test]
fn test_add_big_with_overflow() {
    test_add_opcode(
        [false, false, true, true, false, false, true],
        [3, 5, 1],
        const_felt252_expr!(
            0xffffffffffffffffffffffffffffffffu128,
            0x7ffffffffffffeeffffffffffffffffu128
        ),
        const_felt252_expr!(0, 1u128 << (251 - 128)),
        const_felt252_expr!(0, 1u128 << (251 - 128)),
        vec![
            (10, ""),
            (50, ""),
            (100, ""),
            (32771, "offset_0"),
            (32773, "offset_1"),
            (0, "dst_base_fp"),
            (1, "op0_base_fp"),
            (1, "ap_update_add_1"),
            (1, "id"),
            (511, "limb_0"),
            (511, "limb_1"),
            (511, "limb_2"),
            (511, "limb_3"),
            (511, "limb_4"),
            (511, "limb_5"),
            (511, "limb_6"),
            (511, "limb_7"),
            (511, "limb_8"),
            (511, "limb_9"),
            (511, "limb_10"),
            (511, "limb_11"),
            (511, "limb_12"),
            (511, "limb_13"),
            (511, "limb_14"),
            (511, "limb_15"),
            (511, "limb_16"),
            (511, "limb_17"),
            (511, "limb_18"),
            (511, "limb_19"),
            (511, "limb_20"),
            (375, "limb_21"),
            (511, "limb_22"),
            (511, "limb_23"),
            (511, "limb_24"),
            (511, "limb_25"),
            (511, "limb_26"),
            (255, "limb_27"),
            (2, "id"),
            (0, "limb_0"),
            (0, "limb_1"),
            (0, "limb_2"),
            (0, "limb_3"),
            (0, "limb_4"),
            (0, "limb_5"),
            (0, "limb_6"),
            (0, "limb_7"),
            (0, "limb_8"),
            (0, "limb_9"),
            (0, "limb_10"),
            (0, "limb_11"),
            (0, "limb_12"),
            (0, "limb_13"),
            (0, "limb_14"),
            (0, "limb_15"),
            (0, "limb_16"),
            (0, "limb_17"),
            (0, "limb_18"),
            (0, "limb_19"),
            (0, "limb_20"),
            (0, "limb_21"),
            (0, "limb_22"),
            (0, "limb_23"),
            (0, "limb_24"),
            (0, "limb_25"),
            (0, "limb_26"),
            (256, "limb_27"),
            (2, "id"),
            (0, "limb_0"),
            (0, "limb_1"),
            (0, "limb_2"),
            (0, "limb_3"),
            (0, "limb_4"),
            (0, "limb_5"),
            (0, "limb_6"),
            (0, "limb_7"),
            (0, "limb_8"),
            (0, "limb_9"),
            (0, "limb_10"),
            (0, "limb_11"),
            (0, "limb_12"),
            (0, "limb_13"),
            (0, "limb_14"),
            (0, "limb_15"),
            (0, "limb_16"),
            (0, "limb_17"),
            (0, "limb_18"),
            (0, "limb_19"),
            (0, "limb_20"),
            (0, "limb_21"),
            (0, "limb_22"),
            (0, "limb_23"),
            (0, "limb_24"),
            (0, "limb_25"),
            (0, "limb_26"),
            (256, "limb_27"),
            (1, "sub_p_bit"),
        ]
        .into(),
    );
}
