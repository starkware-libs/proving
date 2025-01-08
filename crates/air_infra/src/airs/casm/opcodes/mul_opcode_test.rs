use super::super::casm_state::*;
use super::super::common::*;
use super::mul_opcode::*;
// Macros
use crate::const_expr;
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::state::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

fn test_mul(
    non_consts_flags: [bool; 7],
    offset_values: [i16; 3],
    dst: Felt252Expr,
    op0: Felt252Expr,
    op1: Felt252Expr,
    expected_state: State,
) {
    // Read the non-constant flags
    let [mul_small, flag_dst_base_fp, flag_op0_base_fp, flag_op1_imm, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1] =
        non_consts_flags;

    let [offset_dst_val, offset0_val, mut offset1_val] = offset_values;
    if flag_op1_imm {
        offset1_val = 1;
    }

    // Create the air function
    let mut mul_small_opcode = MulOpcode {
        small: mul_small,
        imm: flag_op1_imm,
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
    mul_small_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function

    let (registry, _) = AirFnRegistry::new(&mul_small_opcode);
    let (state, next_state) = registry.run_air(
        &mul_small_opcode,
        (),
        CasmStateVar::new(pc.clone(), ap.clone(), fp.clone()),
    );

    // Check output
    assert_eq!(next_state.fp().calc(), fp.calc());
    if flag_ap_update_add_1 {
        assert_eq!(next_state.ap().calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_state.ap().calc(), ap.calc());
    }
    if flag_op1_imm {
        assert_eq!(next_state.pc().calc(), (pc_value + 2).to_string());
    } else {
        assert_eq!(next_state.pc().calc(), (pc_value + 1).to_string());
    };

    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_mul_small_not_imm() {
    test_mul(
        [true, true, false, false, false, true, false],
        [3, 5, 7],
        const_felt252_expr!(4157290412704114895599, 0),
        const_felt252_expr!(67891234567, 0),
        const_felt252_expr!(61234567897, 0),
        vec![
            (10, "input_pc"),
            (50, "input_ap"),
            (100, "input_fp"),
            (32771, "offset0"),
            (32773, "offset1"),
            (32775, "offset2"),
            (1, "dst_base_fp"),
            (0, "op0_base_fp"),
            (0, "op1_base_fp"),
            (1, "op1_base_ap"),
            (0, "ap_update_add_1"),
            (100, "mem_dst_base"),
            (50, "mem0_base"),
            (50, "mem1_base"),
            (1, "dst_id"),
            (239, "dst_limb_0"),
            (437, "dst_limb_1"),
            (374, "dst_limb_2"),
            (190, "dst_limb_3"),
            (65, "dst_limb_4"),
            (500, "dst_limb_5"),
            (375, "dst_limb_6"),
            (450, "dst_limb_7"),
            (2, "op0_id"),
            (263, "op0_limb_0"),
            (259, "op0_limb_1"),
            (424, "op0_limb_2"),
            (505, "op0_limb_3"),
            (3, "op1_id"),
            (217, "op1_limb_0"),
            (173, "op1_limb_1"),
            (119, "op1_limb_2"),
            (456, "op1_limb_3"),
            (198, "carry_1"),
            (652, "carry_3"),
            (495, "carry_5"),
        ]
        .into(),
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck11 on input 1796768395")]
fn test_mul_small_not_equal() {
    test_mul(
        [true, false, true, true, false, false, true],
        [3, 5, 7],
        const_felt252_expr!(4057290412704114895599, 0),
        const_felt252_expr!(67891234567, 0),
        const_felt252_expr!(61234567897, 0),
        vec![].into(),
    );
}

#[test]
#[should_panic(expected = "assertion `left == right` failed: given value != value in memory")]
fn test_mul_small_over_36bit() {
    test_mul(
        [true, false, true, true, false, false, true],
        [3, 5, 7],
        const_felt252_expr!(1i128 << 36),
        const_felt252_expr!(68719476736i128),
        const_felt252_expr!(1),
        vec![].into(),
    );
}

#[test]
fn test_mul_small_imm() {
    test_mul(
        [true, true, false, true, false, false, false],
        [-3, -5, 1],
        const_felt252_expr!(56),
        const_felt252_expr!(7),
        const_felt252_expr!(8),
        vec![
            (10, "input_pc"),
            (50, "input_ap"),
            (100, "input_fp"),
            (32765, "offset0"),
            (32763, "offset1"),
            (1, "dst_base_fp"),
            (0, "op0_base_fp"),
            (0, "ap_update_add_1"),
            (100, "mem_dst_base"),
            (50, "mem0_base"),
            (1, "dst_id"),
            (56, "dst_limb_0"),
            (0, "dst_limb_1"),
            (0, "dst_limb_2"),
            (0, "dst_limb_3"),
            (0, "dst_limb_4"),
            (0, "dst_limb_5"),
            (0, "dst_limb_6"),
            (0, "dst_limb_7"),
            (2, "op0_id"),
            (7, "op0_limb_0"),
            (0, "op0_limb_1"),
            (0, "op0_limb_2"),
            (0, "op0_limb_3"),
            (3, "op1_id"),
            (8, "op1_limb_0"),
            (0, "op1_limb_1"),
            (0, "op1_limb_2"),
            (0, "op1_limb_3"),
            (0, "carry_1"),
            (0, "carry_3"),
            (0, "carry_5"),
        ]
        .into(),
    );
}

#[test]
fn test_mul_big_imm_no_overflow() {
    test_mul(
        [false, true, false, true, false, false, false],
        [-3, -5, 1],
        const_felt252_expr!(0x2008020003400040001u128, 0u128),
        const_felt252_expr!(0x1008020001u128, 0u128),
        const_felt252_expr!(0x1ff8020001u128, 0u128),
        vec![
            (10, "input_pc"),
            (50, "input_ap"),
            (100, "input_fp"),
            (32765, "offset0"),
            (32763, "offset1"),
            (1, "dst_base_fp"),
            (0, "op0_base_fp"),
            (0, "ap_update_add_1"),
            (100, "mem_dst_base"),
            (50, "mem0_base"),
            (1, "dst_id"),
            (1, "dst_limb_0"),
            (0, "dst_limb_1"),
            (1, "dst_limb_2"),
            (128, "dst_limb_3"),
            (3, "dst_limb_4"),
            (256, "dst_limb_5"),
            (0, "dst_limb_6"),
            (1, "dst_limb_7"),
            (2, "dst_limb_8"),
            (0, "dst_limb_9"),
            (0, "dst_limb_10"),
            (0, "dst_limb_11"),
            (0, "dst_limb_12"),
            (0, "dst_limb_13"),
            (0, "dst_limb_14"),
            (0, "dst_limb_15"),
            (0, "dst_limb_16"),
            (0, "dst_limb_17"),
            (0, "dst_limb_18"),
            (0, "dst_limb_19"),
            (0, "dst_limb_20"),
            (0, "dst_limb_21"),
            (0, "dst_limb_22"),
            (0, "dst_limb_23"),
            (0, "dst_limb_24"),
            (0, "dst_limb_25"),
            (0, "dst_limb_26"),
            (0, "dst_limb_27"),
            (2, "op0_id"),
            (1, "op0_limb_0"),
            (256, "op0_limb_1"),
            (0, "op0_limb_2"),
            (1, "op0_limb_3"),
            (1, "op0_limb_4"),
            (0, "op0_limb_5"),
            (0, "op0_limb_6"),
            (0, "op0_limb_7"),
            (0, "op0_limb_8"),
            (0, "op0_limb_9"),
            (0, "op0_limb_10"),
            (0, "op0_limb_11"),
            (0, "op0_limb_12"),
            (0, "op0_limb_13"),
            (0, "op0_limb_14"),
            (0, "op0_limb_15"),
            (0, "op0_limb_16"),
            (0, "op0_limb_17"),
            (0, "op0_limb_18"),
            (0, "op0_limb_19"),
            (0, "op0_limb_20"),
            (0, "op0_limb_21"),
            (0, "op0_limb_22"),
            (0, "op0_limb_23"),
            (0, "op0_limb_24"),
            (0, "op0_limb_25"),
            (0, "op0_limb_26"),
            (0, "op0_limb_27"),
            (3, "op1_id"),
            (1, "op1_limb_0"),
            (256, "op1_limb_1"),
            (0, "op1_limb_2"),
            (511, "op1_limb_3"),
            (1, "op1_limb_4"),
            (0, "op1_limb_5"),
            (0, "op1_limb_6"),
            (0, "op1_limb_7"),
            (0, "op1_limb_8"),
            (0, "op1_limb_9"),
            (0, "op1_limb_10"),
            (0, "op1_limb_11"),
            (0, "op1_limb_12"),
            (0, "op1_limb_13"),
            (0, "op1_limb_14"),
            (0, "op1_limb_15"),
            (0, "op1_limb_16"),
            (0, "op1_limb_17"),
            (0, "op1_limb_18"),
            (0, "op1_limb_19"),
            (0, "op1_limb_20"),
            (0, "op1_limb_21"),
            (0, "op1_limb_22"),
            (0, "op1_limb_23"),
            (0, "op1_limb_24"),
            (0, "op1_limb_25"),
            (0, "op1_limb_26"),
            (0, "op1_limb_27"),
            (0, "k"),
            (0, "carry_0"),
            (32, "carry_1"),
            (4097, "carry_2"),
            (160, "carry_3"),
            (8193, "carry_4"),
            (288, "carry_5"),
            (33, "carry_6"),
            (33, "carry_7"),
            (3, "carry_8"),
            (256, "carry_9"),
            (2, "carry_10"),
            (512, "carry_11"),
            (2, "carry_12"),
            (2, "carry_13"),
            (2, "carry_14"),
            (0, "carry_15"),
            (0, "carry_16"),
            (0, "carry_17"),
            (0, "carry_18"),
            (0, "carry_19"),
            (0, "carry_20"),
            (0, "carry_21"),
            (0, "carry_22"),
            (0, "carry_23"),
            (0, "carry_24"),
            (0, "carry_25"),
            (0, "carry_26"),
        ]
        .into(),
    );
}

#[test]
fn test_mul_big_with_overflow() {
    test_mul(
        [false, false, true, false, false, true, true],
        [-3, -5, 1],
        const_felt252_expr!(
            0x4cc3ffffffffff5cdf8002u128,
            0x7fffff52ad78032ffffffffffffdbe0u128
        ),
        const_felt252_expr!(0, 1u128 << (251 - 128)),
        const_felt252_expr!(0, 1u128 << (251 - 128)),
        vec![
            (10, "input_pc"),
            (50, "input_ap"),
            (100, "input_fp"),
            (32765, "offset0"),
            (32763, "offset1"),
            (32769, "offset2"),
            (0, "dst_base_fp"),
            (1, "op0_base_fp"),
            (0, "op1_base_fp"),
            (1, "op1_base_ap"),
            (1, "ap_update_add_1"),
            (50, "mem_dst_base"),
            (100, "mem0_base"),
            (50, "mem1_base"),
            (1, "dst_id"),
            (2, "dst_limb_0"),
            (448, "dst_limb_1"),
            (311, "dst_limb_2"),
            (491, "dst_limb_3"),
            (511, "dst_limb_4"),
            (511, "dst_limb_5"),
            (511, "dst_limb_6"),
            (511, "dst_limb_7"),
            (195, "dst_limb_8"),
            (38, "dst_limb_9"),
            (0, "dst_limb_10"),
            (0, "dst_limb_11"),
            (0, "dst_limb_12"),
            (0, "dst_limb_13"),
            (384, "dst_limb_14"),
            (439, "dst_limb_15"),
            (511, "dst_limb_16"),
            (511, "dst_limb_17"),
            (511, "dst_limb_18"),
            (511, "dst_limb_19"),
            (511, "dst_limb_20"),
            (407, "dst_limb_21"),
            (0, "dst_limb_22"),
            (431, "dst_limb_23"),
            (298, "dst_limb_24"),
            (506, "dst_limb_25"),
            (511, "dst_limb_26"),
            (255, "dst_limb_27"),
            (2, "op0_id"),
            (0, "op0_limb_0"),
            (0, "op0_limb_1"),
            (0, "op0_limb_2"),
            (0, "op0_limb_3"),
            (0, "op0_limb_4"),
            (0, "op0_limb_5"),
            (0, "op0_limb_6"),
            (0, "op0_limb_7"),
            (0, "op0_limb_8"),
            (0, "op0_limb_9"),
            (0, "op0_limb_10"),
            (0, "op0_limb_11"),
            (0, "op0_limb_12"),
            (0, "op0_limb_13"),
            (0, "op0_limb_14"),
            (0, "op0_limb_15"),
            (0, "op0_limb_16"),
            (0, "op0_limb_17"),
            (0, "op0_limb_18"),
            (0, "op0_limb_19"),
            (0, "op0_limb_20"),
            (0, "op0_limb_21"),
            (0, "op0_limb_22"),
            (0, "op0_limb_23"),
            (0, "op0_limb_24"),
            (0, "op0_limb_25"),
            (0, "op0_limb_26"),
            (256, "op0_limb_27"),
            (2, "op1_id"),
            (0, "op1_limb_0"),
            (0, "op1_limb_1"),
            (0, "op1_limb_2"),
            (0, "op1_limb_3"),
            (0, "op1_limb_4"),
            (0, "op1_limb_5"),
            (0, "op1_limb_6"),
            (0, "op1_limb_7"),
            (0, "op1_limb_8"),
            (0, "op1_limb_9"),
            (0, "op1_limb_10"),
            (0, "op1_limb_11"),
            (0, "op1_limb_12"),
            (0, "op1_limb_13"),
            (0, "op1_limb_14"),
            (0, "op1_limb_15"),
            (0, "op1_limb_16"),
            (0, "op1_limb_17"),
            (0, "op1_limb_18"),
            (0, "op1_limb_19"),
            (0, "op1_limb_20"),
            (0, "op1_limb_21"),
            (0, "op1_limb_22"),
            (0, "op1_limb_23"),
            (0, "op1_limb_24"),
            (0, "op1_limb_25"),
            (0, "op1_limb_26"),
            (256, "op1_limb_27"),
            (540, "k"),
            (2, "carry_0"),
            (2147483619, "carry_1"),
            (2147483630, "carry_2"),
            (2147483618, "carry_3"),
            (2147483618, "carry_4"),
            (995, "carry_5"),
            (2147483618, "carry_6"),
            (2147483614, "carry_7"),
            (2147483632, "carry_8"),
            (2147483643, "carry_9"),
            (2147483645, "carry_10"),
            (2147483645, "carry_11"),
            (2147483645, "carry_12"),
            (2147483645, "carry_13"),
            (2147483621, "carry_14"),
            (2147483618, "carry_15"),
            (2147483614, "carry_16"),
            (2147483614, "carry_17"),
            (2147483614, "carry_18"),
            (2147483614, "carry_19"),
            (2147483614, "carry_20"),
            (2147483501, "carry_21"),
            (2147483645, "carry_22"),
            (2147483645, "carry_23"),
            (2147483645, "carry_24"),
            (2147483645, "carry_25"),
            (8190, "carry_26"),
        ]
        .into(),
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck19 on input 738328558")]
fn test_mul_big_not_equal() {
    test_mul(
        [false, false, true, false, false, true, true],
        [-3, -5, 1],
        const_felt252_expr!(
            0x4cc3ffffffffff5cdf8002u128,
            0x7fffff52ad8022ffffffffffffdae0u128
        ),
        const_felt252_expr!(0, 1u128 << (251 - 128)),
        const_felt252_expr!(0, 1u128 << (251 - 128)),
        vec![].into(),
    );
}
