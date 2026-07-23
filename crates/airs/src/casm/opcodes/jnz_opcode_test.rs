use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::state::State;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::jnz_opcode::*;
use crate::casm::common::*;

fn build_and_test(
    [taken, dst_base_fp, ap_update_add_1]: [bool; 3],
    offset_dst: i16,
    dst_value: Felt252Expr,
    op1_value: i64,
) -> State {
    let [pc_value, ap_value, fp_value] = [50, 200, 150];
    let [pc, ap, fp] = [const_expr!(pc_value), const_expr!(ap_value), const_expr!(fp_value)];

    let mut jnz_opcode = JnzOpcode { taken, memory: Felt252IdMemory::default() };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                offset_dst,
                -1,
                1,
                jnz_opcode.get_flags().non_constants_to_arr(&[dst_base_fp, ap_update_add_1]),
                OpcodeExtension::Stone
            ),
            0
        ),
    )];

    memory_values.push((const_expr!(pc_value + 1), const_felt252_expr!(op1_value)));

    if dst_base_fp {
        memory_values.push((const_expr!((fp_value as i16 + offset_dst) as u32), dst_value.clone()));
    } else {
        memory_values.push((const_expr!((ap_value as i16 + offset_dst) as u32), dst_value.clone()));
    }

    jnz_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&jnz_opcode);
    let (state, next_state) =
        registry.run_air(&jnz_opcode, (), CasmStateVar::new(pc, ap.clone(), fp.clone()));

    // Check output
    if taken {
        assert_eq!(next_state.pc().calc(), (pc_value as i128 + op1_value as i128).to_string());
    } else {
        assert_eq!(next_state.pc().calc(), (pc_value + 2).to_string());
    }

    if ap_update_add_1 {
        assert_eq!(next_state.ap().calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_state.ap().calc(), ap_value.to_string());
    }

    assert_eq!(next_state.fp().calc(), fp_value.to_string());

    state
}

#[test]
fn test_jnz_not_taken_base_ap() {
    let state = build_and_test([false, false, false], -13, const_felt252_expr!(0, 0), 15);

    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32755, "offset0"),
        (0, "dst_base_fp"),
        (0, "ap_update_add_1"),
        (200, "mem_dst_base"),
        (2, "dst_id"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_jnz_taken_base_ap() {
    let state = build_and_test([true, false, false], -13, const_felt252_expr!(123, 456), 15);

    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32755, "offset0"),
        (0, "dst_base_fp"),
        (0, "ap_update_add_1"),
        (200, "mem_dst_base"),
        (2, "dst_id"),
        (123, "dst_limb_0"),
        (0, "dst_limb_1"),
        (0, "dst_limb_2"),
        (0, "dst_limb_3"),
        (0, "dst_limb_4"),
        (0, "dst_limb_5"),
        (0, "dst_limb_6"),
        (0, "dst_limb_7"),
        (0, "dst_limb_8"),
        (0, "dst_limb_9"),
        (0, "dst_limb_10"),
        (0, "dst_limb_11"),
        (0, "dst_limb_12"),
        (0, "dst_limb_13"),
        (288, "dst_limb_14"),
        (3, "dst_limb_15"),
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
        (1955558780, "dst_sum_inv"),
        (500077285, "dst_sum_squares_inv"),
        (1, "next_pc_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (15, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_taken_zero_mismatch_base_ap() {
    build_and_test([true, false, false], -13, const_felt252_expr!(0, 0), 15);
}

#[test]
#[should_panic(expected = "assertion `left == right` failed: given value != value in memory")]
fn test_not_taken_mismatch_base_ap() {
    build_and_test([false, false, false], -13, const_felt252_expr!(123, 4567), 15);
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_taken_p_mismatch_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        const_felt252_expr!(1, 17 * u128::pow(2, 64) + u128::pow(2, 123)),
        15,
    );
}

#[test]
fn test_jnz_taken_negative_op1() {
    let state = build_and_test([true, true, false], -13, const_felt252_expr!(123, 456), -22);

    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32755, "offset0"),
        (1, "dst_base_fp"),
        (0, "ap_update_add_1"),
        (150, "mem_dst_base"),
        (2, "dst_id"),
        (123, "dst_limb_0"),
        (0, "dst_limb_1"),
        (0, "dst_limb_2"),
        (0, "dst_limb_3"),
        (0, "dst_limb_4"),
        (0, "dst_limb_5"),
        (0, "dst_limb_6"),
        (0, "dst_limb_7"),
        (0, "dst_limb_8"),
        (0, "dst_limb_9"),
        (0, "dst_limb_10"),
        (0, "dst_limb_11"),
        (0, "dst_limb_12"),
        (0, "dst_limb_13"),
        (288, "dst_limb_14"),
        (3, "dst_limb_15"),
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
        (1955558780, "dst_sum_inv"),
        (500077285, "dst_sum_squares_inv"),
        (1, "next_pc_id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (491, "next_pc_limb_0"),
        (511, "next_pc_limb_1"),
        (511, "next_pc_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}
