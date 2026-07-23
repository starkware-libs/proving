use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::state::State;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::add_opcode::*;
use crate::casm::common::*;

fn test_add_opcode(
    non_consts_flags: [bool; 7],
    offset_values: [i16; 3],
    dst: Felt252Expr,
    op0: Felt252Expr,
    op1: Felt252Expr,
) -> State {
    // Read the non-constant flags
    let [
        add_small,
        flag_dst_base_fp,
        flag_op0_base_fp,
        flag_op1_imm,
        flag_op1_base_fp,
        flag_op1_base_ap,
        flag_ap_update_add_1,
    ] = non_consts_flags;

    let [offset_dst_val, offset0_val, mut offset1_val] = offset_values;
    if flag_op1_imm {
        offset1_val = 1;
    }

    // Create the air function
    let mut add_small_opcode = AddOpcode { small: add_small, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let pc_value = 10;
    let ap_value = 50;
    let fp_value = 100;

    let pc = const_expr!(pc_value);
    let ap = const_expr!(ap_value);
    let fp = const_expr!(fp_value);

    // Create the non-constant flags
    let non_consts_flags = vec![
        flag_dst_base_fp,
        flag_op0_base_fp,
        flag_op1_imm,
        flag_op1_base_fp,
        flag_op1_base_ap,
        flag_ap_update_add_1,
    ];

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                offset_dst_val,
                offset0_val,
                offset1_val,
                add_small_opcode.get_flags().non_constants_to_arr(&non_consts_flags),
                OpcodeExtension::Stone,
            ),
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

    state
}

#[test]
fn test_add_small_not_imm() {
    let state = test_add_opcode(
        [true, true, false, false, false, true, false],
        [3, 5, 7],
        const_felt252_expr!(90125677),
        const_felt252_expr!(77779999),
        const_felt252_expr!(12345678),
    );

    expect![[r#"
        (1, "enabler"),
        (10, "input_pc"),
        (50, "input_ap"),
        (100, "input_fp"),
        (32771, "offset0"),
        (32773, "offset1"),
        (32775, "offset2"),
        (1, "dst_base_fp"),
        (0, "op0_base_fp"),
        (0, "op1_imm"),
        (0, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (100, "mem_dst_base"),
        (50, "mem0_base"),
        (50, "mem1_base"),
        (1, "dst_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (365, "dst_limb_0"),
        (410, "dst_limb_1"),
        (343, "dst_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
        (2, "op0_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (31, "op0_limb_0"),
        (362, "op0_limb_1"),
        (296, "op0_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
        (3, "op1_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (334, "op1_limb_0"),
        (48, "op1_limb_1"),
        (47, "op1_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
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
    );
}

#[test]
fn test_add_small_over_27bit() {
    test_add_opcode(
        [true, false, true, true, false, false, true],
        [3, 5, 7],
        const_felt252_expr!(134217728),
        const_felt252_expr!(134217727),
        const_felt252_expr!(1),
    );
}

#[test]
fn test_add_small_neg_imm() {
    let state = test_add_opcode(
        [true, true, false, true, false, false, false],
        [-3, -5, 1],
        const_felt252_expr!(-2687280i128),
        const_felt252_expr!(-2662632i128),
        const_felt252_expr!(-24648i128),
    );

    expect![[r#"
        (1, "enabler"),
        (10, "input_pc"),
        (50, "input_ap"),
        (100, "input_fp"),
        (32765, "offset0"),
        (32763, "offset1"),
        (32769, "offset2"),
        (1, "dst_base_fp"),
        (0, "op0_base_fp"),
        (1, "op1_imm"),
        (0, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (100, "mem_dst_base"),
        (50, "mem0_base"),
        (10, "mem1_base"),
        (1, "dst_id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (209, "dst_limb_0"),
        (383, "dst_limb_1"),
        (501, "dst_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
        (2, "op0_id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (281, "op0_limb_0"),
        (431, "op0_limb_1"),
        (501, "op0_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
        (3, "op1_id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (441, "op1_limb_0"),
        (463, "op1_limb_1"),
        (511, "op1_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_add_big_not_imm() {
    let state = test_add_opcode(
        [false, false, true, false, false, true, false],
        [3, 5, 7],
        const_felt252_expr!(0x3000040002i128),
        const_felt252_expr!(0x1008020001i128),
        const_felt252_expr!(0x1ff8020001i128),
    );

    expect![[r#"
        (1, "enabler"),
        (10, "input_pc"),
        (50, "input_ap"),
        (100, "input_fp"),
        (32771, "offset0"),
        (32773, "offset1"),
        (32775, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (0, "op1_imm"),
        (0, "op1_base_fp"),
        (0, "ap_update_add_1"),
        (50, "mem_dst_base"),
        (100, "mem0_base"),
        (50, "mem1_base"),
        (1, "dst_id"),
        (2, "dst_limb_0"),
        (0, "dst_limb_1"),
        (1, "dst_limb_2"),
        (0, "dst_limb_3"),
        (3, "dst_limb_4"),
        (0, "dst_limb_5"),
        (0, "dst_limb_6"),
        (0, "dst_limb_7"),
        (0, "dst_limb_8"),
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
        (0, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_add_big_mod_not_equal() {
    test_add_opcode(
        [false, false, true, false, false, true, false],
        [3, 5, 7],
        const_felt252_expr!(0x3000040002u128, 1u128),
        const_felt252_expr!(0x1008020001i128),
        const_felt252_expr!(0x1ff8020001i128),
    );
}

#[test]
fn test_add_big_imm() {
    let state = test_add_opcode(
        [false, false, true, true, false, false, true],
        [3, 5, 1],
        const_felt252_expr!(0x3000040002i128),
        const_felt252_expr!(0x1008020001i128),
        const_felt252_expr!(0x1ff8020001i128),
    );

    expect![[r#"
        (1, "enabler"),
        (10, "input_pc"),
        (50, "input_ap"),
        (100, "input_fp"),
        (32771, "offset0"),
        (32773, "offset1"),
        (32769, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (1, "op1_imm"),
        (0, "op1_base_fp"),
        (1, "ap_update_add_1"),
        (50, "mem_dst_base"),
        (100, "mem0_base"),
        (10, "mem1_base"),
        (1, "dst_id"),
        (2, "dst_limb_0"),
        (0, "dst_limb_1"),
        (1, "dst_limb_2"),
        (0, "dst_limb_3"),
        (3, "dst_limb_4"),
        (0, "dst_limb_5"),
        (0, "dst_limb_6"),
        (0, "dst_limb_7"),
        (0, "dst_limb_8"),
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
        (0, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_add_big_with_overflow() {
    let state = test_add_opcode(
        [false, false, true, true, false, false, true],
        [3, 5, 1],
        const_felt252_expr!(
            0xffffffffffffffffffffffffffffffffu128,
            0x7ffffffffffffeeffffffffffffffffu128
        ),
        const_felt252_expr!(0, 1u128 << (251 - 128)),
        const_felt252_expr!(0, 1u128 << (251 - 128)),
    );

    expect![[r#"
        (1, "enabler"),
        (10, "input_pc"),
        (50, "input_ap"),
        (100, "input_fp"),
        (32771, "offset0"),
        (32773, "offset1"),
        (32769, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (1, "op1_imm"),
        (0, "op1_base_fp"),
        (1, "ap_update_add_1"),
        (50, "mem_dst_base"),
        (100, "mem0_base"),
        (10, "mem1_base"),
        (1, "dst_id"),
        (511, "dst_limb_0"),
        (511, "dst_limb_1"),
        (511, "dst_limb_2"),
        (511, "dst_limb_3"),
        (511, "dst_limb_4"),
        (511, "dst_limb_5"),
        (511, "dst_limb_6"),
        (511, "dst_limb_7"),
        (511, "dst_limb_8"),
        (511, "dst_limb_9"),
        (511, "dst_limb_10"),
        (511, "dst_limb_11"),
        (511, "dst_limb_12"),
        (511, "dst_limb_13"),
        (511, "dst_limb_14"),
        (511, "dst_limb_15"),
        (511, "dst_limb_16"),
        (511, "dst_limb_17"),
        (511, "dst_limb_18"),
        (511, "dst_limb_19"),
        (511, "dst_limb_20"),
        (375, "dst_limb_21"),
        (511, "dst_limb_22"),
        (511, "dst_limb_23"),
        (511, "dst_limb_24"),
        (511, "dst_limb_25"),
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
        (1, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}
