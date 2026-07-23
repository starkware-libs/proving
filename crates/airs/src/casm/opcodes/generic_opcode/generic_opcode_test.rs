use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::generic_opcode::*;
use crate::casm::common::*;
use crate::casm::opcodes::add_ap_opcode::*;
use crate::casm::opcodes::assert_eq_opcode::*;
use crate::casm::opcodes::call_opcode::*;
use crate::casm::opcodes::jnz_opcode::*;
use crate::casm::opcodes::jump_opcode::*;
use crate::casm::opcodes::jump_opcode_test::*;
use crate::casm::opcodes::ret_opcode::*;
use crate::casm::opcodes::ret_opcode_test::*;

#[test]
fn test_generic_consistency_rel_call() {
    let mut generic_opcode = GenericOpcode::default();
    let mut call_opcode = CallOpcode { rel_imm: true, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let immediate = 299;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    0,
                    1,
                    1,
                    call_opcode.get_flags().into(),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(immediate)),
        (const_expr!(ap), const_felt252_expr!(fp as i64)),
        (const_expr!(ap + 1), const_felt252_expr!(pc as i64 + 2)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    call_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&call_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &call_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32768, "offset0"),
        (32769, "offset1"),
        (32769, "offset2"),
        (0, "dst_base_fp"),
        (0, "op0_base_fp"),
        (1, "op1_imm"),
        (0, "op1_base_fp"),
        (0, "op1_base_ap"),
        (0, "res_add"),
        (0, "res_mul"),
        (0, "pc_update_jump"),
        (1, "pc_update_jump_rel"),
        (0, "pc_update_jnz"),
        (0, "ap_update_add"),
        (0, "ap_update_add_1"),
        (1, "opcode_call"),
        (0, "opcode_ret"),
        (0, "opcode_assert_eq"),
        (200, "dst_src"),
        (2, "dst_id"),
        (150, "dst_limb_0"),
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
        (200, "op0_src"),
        (3, "op0_id"),
        (52, "op0_limb_0"),
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
        (0, "op0_limb_27"),
        (0, "partial_limb_msb"),
        (50, "op1_src"),
        (1, "op1_id"),
        (299, "op1_limb_0"),
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
        (0, "op1_limb_27"),
        (351, "add_res_limb_0"),
        (0, "add_res_limb_1"),
        (0, "add_res_limb_2"),
        (0, "add_res_limb_3"),
        (0, "add_res_limb_4"),
        (0, "add_res_limb_5"),
        (0, "add_res_limb_6"),
        (0, "add_res_limb_7"),
        (0, "add_res_limb_8"),
        (0, "add_res_limb_9"),
        (0, "add_res_limb_10"),
        (0, "add_res_limb_11"),
        (0, "add_res_limb_12"),
        (0, "add_res_limb_13"),
        (0, "add_res_limb_14"),
        (0, "add_res_limb_15"),
        (0, "add_res_limb_16"),
        (0, "add_res_limb_17"),
        (0, "add_res_limb_18"),
        (0, "add_res_limb_19"),
        (0, "add_res_limb_20"),
        (0, "add_res_limb_21"),
        (0, "add_res_limb_22"),
        (0, "add_res_limb_23"),
        (0, "add_res_limb_24"),
        (0, "add_res_limb_25"),
        (0, "add_res_limb_26"),
        (0, "add_res_limb_27"),
        (0, "sub_p_bit"),
        (188, "mul_res_limb_0"),
        (30, "mul_res_limb_1"),
        (0, "mul_res_limb_2"),
        (0, "mul_res_limb_3"),
        (0, "mul_res_limb_4"),
        (0, "mul_res_limb_5"),
        (0, "mul_res_limb_6"),
        (0, "mul_res_limb_7"),
        (0, "mul_res_limb_8"),
        (0, "mul_res_limb_9"),
        (0, "mul_res_limb_10"),
        (0, "mul_res_limb_11"),
        (0, "mul_res_limb_12"),
        (0, "mul_res_limb_13"),
        (0, "mul_res_limb_14"),
        (0, "mul_res_limb_15"),
        (0, "mul_res_limb_16"),
        (0, "mul_res_limb_17"),
        (0, "mul_res_limb_18"),
        (0, "mul_res_limb_19"),
        (0, "mul_res_limb_20"),
        (0, "mul_res_limb_21"),
        (0, "mul_res_limb_22"),
        (0, "mul_res_limb_23"),
        (0, "mul_res_limb_24"),
        (0, "mul_res_limb_25"),
        (0, "mul_res_limb_26"),
        (0, "mul_res_limb_27"),
        (0, "k"),
        (960, "carry_0"),
        (30, "carry_1"),
        (0, "carry_2"),
        (0, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (60, "carry_7"),
        (0, "carry_8"),
        (0, "carry_9"),
        (0, "carry_10"),
        (0, "carry_11"),
        (0, "carry_12"),
        (0, "carry_13"),
        (0, "carry_14"),
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
        (299, "res_limb_0"),
        (0, "res_limb_1"),
        (0, "res_limb_2"),
        (0, "res_limb_3"),
        (0, "res_limb_4"),
        (0, "res_limb_5"),
        (0, "res_limb_6"),
        (0, "res_limb_7"),
        (0, "res_limb_8"),
        (0, "res_limb_9"),
        (0, "res_limb_10"),
        (0, "res_limb_11"),
        (0, "res_limb_12"),
        (0, "res_limb_13"),
        (0, "res_limb_14"),
        (0, "res_limb_15"),
        (0, "res_limb_16"),
        (0, "res_limb_17"),
        (0, "res_limb_18"),
        (0, "res_limb_19"),
        (0, "res_limb_20"),
        (0, "res_limb_21"),
        (0, "res_limb_22"),
        (0, "res_limb_23"),
        (0, "res_limb_24"),
        (0, "res_limb_25"),
        (0, "res_limb_26"),
        (0, "res_limb_27"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (1285643938, "dst_sum_squares_inv"),
        (243381480, "dst_sum_inv"),
        (0, "op1_as_rel_imm_cond"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (349, "next_pc_jnz"),
        (349, "next_pc"),
        (202, "next_ap"),
        (202, "range_check_29_bot11bits"),
        (202, "next_fp"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_generic_call_abs_imm() {
    let mut generic_opcode = GenericOpcode::default();
    let call_opcode = CallOpcode { rel_imm: false, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let immediate = 2346;

    // Create flags
    let mut flags = call_opcode.get_flags();
    flags.op1_imm = Some(true);
    flags.op1_base_fp = Some(false);
    flags.op1_base_ap = Some(false);

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(0, 1, 1, flags.clone().into(), OpcodeExtension::Stone),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(immediate)),
        (const_expr!(ap), const_felt252_expr!(fp as i64)),
        (const_expr!(ap + 1), const_felt252_expr!(pc as i64 + 2)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&call_opcode);
    let (_, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(output.pc().calc(), (immediate).to_string());
    assert_eq!(output.ap().calc(), (ap + 2).to_string());
    assert_eq!(output.fp().calc(), (ap + 2).to_string());
}

#[test]
fn test_generic_call_rel_deref() {
    let mut generic_opcode = GenericOpcode::default();
    let call_opcode = CallOpcode { rel_imm: true, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let op1 = 34698498;
    let offset2 = 3545;

    // Create flags
    let mut flags = call_opcode.get_flags();
    flags.op1_base_ap = Some(true);
    flags.op1_imm = Some(false);

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(0, 1, offset2, flags.clone().into(), OpcodeExtension::Stone),
                0
            ),
        ),
        (const_expr!(ap + offset2 as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!(ap), const_felt252_expr!(fp as i64)),
        (const_expr!(ap + 1), const_felt252_expr!(pc as i64 + 1)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&call_opcode);
    let (_, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(output.pc().calc(), (pc + op1).to_string());
    assert_eq!(output.ap().calc(), (ap + 2).to_string());
    assert_eq!(output.fp().calc(), (ap + 2).to_string());
}

#[test]
fn test_generic_consistency_ret() {
    let mut generic_opcode = GenericOpcode::default();
    let mut ret_opcode = RetOpcode { memory: Felt252IdMemory::default() };
    // Register values at opcode start
    let [pc, ap, fp] = [3, 11, 6];

    // Old values of pc, fp saved by the last call opcode
    let saved_fp = 4;
    let saved_pc = 1;

    // Fill memory
    let memory_values = vec![
        (const_expr!(pc), const_felt252_expr!(assemble_ret(), 0)),
        (const_expr!(fp - 1), const_felt252_expr!(saved_pc)),
        (const_expr!(fp - 2), const_felt252_expr!(saved_fp)),
    ];
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    ret_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&ret_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &ret_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32766, "offset0"),
        (32767, "offset1"),
        (32767, "offset2"),
        (1, "dst_base_fp"),
        (1, "op0_base_fp"),
        (0, "op1_imm"),
        (1, "op1_base_fp"),
        (0, "op1_base_ap"),
        (0, "res_add"),
        (0, "res_mul"),
        (1, "pc_update_jump"),
        (0, "pc_update_jump_rel"),
        (0, "pc_update_jnz"),
        (0, "ap_update_add"),
        (0, "ap_update_add_1"),
        (0, "opcode_call"),
        (1, "opcode_ret"),
        (0, "opcode_assert_eq"),
        (6, "dst_src"),
        (2, "dst_id"),
        (4, "dst_limb_0"),
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
        (6, "op0_src"),
        (1, "op0_id"),
        (1, "op0_limb_0"),
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
        (0, "op0_limb_27"),
        (0, "partial_limb_msb"),
        (6, "op1_src"),
        (1, "op1_id"),
        (1, "op1_limb_0"),
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
        (0, "op1_limb_27"),
        (2, "add_res_limb_0"),
        (0, "add_res_limb_1"),
        (0, "add_res_limb_2"),
        (0, "add_res_limb_3"),
        (0, "add_res_limb_4"),
        (0, "add_res_limb_5"),
        (0, "add_res_limb_6"),
        (0, "add_res_limb_7"),
        (0, "add_res_limb_8"),
        (0, "add_res_limb_9"),
        (0, "add_res_limb_10"),
        (0, "add_res_limb_11"),
        (0, "add_res_limb_12"),
        (0, "add_res_limb_13"),
        (0, "add_res_limb_14"),
        (0, "add_res_limb_15"),
        (0, "add_res_limb_16"),
        (0, "add_res_limb_17"),
        (0, "add_res_limb_18"),
        (0, "add_res_limb_19"),
        (0, "add_res_limb_20"),
        (0, "add_res_limb_21"),
        (0, "add_res_limb_22"),
        (0, "add_res_limb_23"),
        (0, "add_res_limb_24"),
        (0, "add_res_limb_25"),
        (0, "add_res_limb_26"),
        (0, "add_res_limb_27"),
        (0, "sub_p_bit"),
        (1, "mul_res_limb_0"),
        (0, "mul_res_limb_1"),
        (0, "mul_res_limb_2"),
        (0, "mul_res_limb_3"),
        (0, "mul_res_limb_4"),
        (0, "mul_res_limb_5"),
        (0, "mul_res_limb_6"),
        (0, "mul_res_limb_7"),
        (0, "mul_res_limb_8"),
        (0, "mul_res_limb_9"),
        (0, "mul_res_limb_10"),
        (0, "mul_res_limb_11"),
        (0, "mul_res_limb_12"),
        (0, "mul_res_limb_13"),
        (0, "mul_res_limb_14"),
        (0, "mul_res_limb_15"),
        (0, "mul_res_limb_16"),
        (0, "mul_res_limb_17"),
        (0, "mul_res_limb_18"),
        (0, "mul_res_limb_19"),
        (0, "mul_res_limb_20"),
        (0, "mul_res_limb_21"),
        (0, "mul_res_limb_22"),
        (0, "mul_res_limb_23"),
        (0, "mul_res_limb_24"),
        (0, "mul_res_limb_25"),
        (0, "mul_res_limb_26"),
        (0, "mul_res_limb_27"),
        (0, "k"),
        (0, "carry_0"),
        (0, "carry_1"),
        (0, "carry_2"),
        (0, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (0, "carry_7"),
        (0, "carry_8"),
        (0, "carry_9"),
        (0, "carry_10"),
        (0, "carry_11"),
        (0, "carry_12"),
        (0, "carry_13"),
        (0, "carry_14"),
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
        (1, "res_limb_0"),
        (0, "res_limb_1"),
        (0, "res_limb_2"),
        (0, "res_limb_3"),
        (0, "res_limb_4"),
        (0, "res_limb_5"),
        (0, "res_limb_6"),
        (0, "res_limb_7"),
        (0, "res_limb_8"),
        (0, "res_limb_9"),
        (0, "res_limb_10"),
        (0, "res_limb_11"),
        (0, "res_limb_12"),
        (0, "res_limb_13"),
        (0, "res_limb_14"),
        (0, "res_limb_15"),
        (0, "res_limb_16"),
        (0, "res_limb_17"),
        (0, "res_limb_18"),
        (0, "res_limb_19"),
        (0, "res_limb_20"),
        (0, "res_limb_21"),
        (0, "res_limb_22"),
        (0, "res_limb_23"),
        (0, "res_limb_24"),
        (0, "res_limb_25"),
        (0, "res_limb_26"),
        (0, "res_limb_27"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (2029429668, "dst_sum_squares_inv"),
        (536870912, "dst_sum_inv"),
        (0, "op1_as_rel_imm_cond"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (4, "next_pc_jnz"),
        (1, "next_pc"),
        (11, "next_ap"),
        (11, "range_check_29_bot11bits"),
        (4, "next_fp"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_generic_consistency_assert_equal() {
    let mut generic_opcode = GenericOpcode::default();
    let mut assert_equal_opcode =
        AssertEqOpcode { double_deref: false, imm: false, memory: Felt252IdMemory::default() };
    let [offset0, offset1, offset2] = [3, -1, 2];
    let dst = 3;
    let op1 = 3;

    // Register values at opcode start
    let [pc, ap, fp] = [3, 11, 6];

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    offset0,
                    offset1,
                    offset2,
                    assert_equal_opcode
                        .get_flags()
                        .non_constants_to_arr(&[false, true, false, false]),
                    OpcodeExtension::Stone,
                ),
                0
            ),
        ),
        (const_expr!((ap as i16 + offset0) as u32), const_felt252_expr!(dst as i128)),
        (const_expr!((fp as i16 + offset2) as u32), const_felt252_expr!(op1 as i128)),
        // Not in use
        (const_expr!((fp as i16 + offset1) as u32), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    assert_equal_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&assert_equal_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &assert_equal_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32771, "offset0"),
        (32767, "offset1"),
        (32770, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (0, "op1_imm"),
        (1, "op1_base_fp"),
        (0, "op1_base_ap"),
        (0, "res_add"),
        (0, "res_mul"),
        (0, "pc_update_jump"),
        (0, "pc_update_jump_rel"),
        (0, "pc_update_jnz"),
        (0, "ap_update_add"),
        (0, "ap_update_add_1"),
        (0, "opcode_call"),
        (0, "opcode_ret"),
        (1, "opcode_assert_eq"),
        (11, "dst_src"),
        (1, "dst_id"),
        (3, "dst_limb_0"),
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
        (6, "op0_src"),
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
        (0, "op0_limb_27"),
        (0, "partial_limb_msb"),
        (6, "op1_src"),
        (1, "op1_id"),
        (3, "op1_limb_0"),
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
        (0, "op1_limb_27"),
        (3, "add_res_limb_0"),
        (0, "add_res_limb_1"),
        (0, "add_res_limb_2"),
        (0, "add_res_limb_3"),
        (0, "add_res_limb_4"),
        (0, "add_res_limb_5"),
        (0, "add_res_limb_6"),
        (0, "add_res_limb_7"),
        (0, "add_res_limb_8"),
        (0, "add_res_limb_9"),
        (0, "add_res_limb_10"),
        (0, "add_res_limb_11"),
        (0, "add_res_limb_12"),
        (0, "add_res_limb_13"),
        (0, "add_res_limb_14"),
        (0, "add_res_limb_15"),
        (0, "add_res_limb_16"),
        (0, "add_res_limb_17"),
        (0, "add_res_limb_18"),
        (0, "add_res_limb_19"),
        (0, "add_res_limb_20"),
        (0, "add_res_limb_21"),
        (0, "add_res_limb_22"),
        (0, "add_res_limb_23"),
        (0, "add_res_limb_24"),
        (0, "add_res_limb_25"),
        (0, "add_res_limb_26"),
        (0, "add_res_limb_27"),
        (0, "sub_p_bit"),
        (0, "mul_res_limb_0"),
        (0, "mul_res_limb_1"),
        (0, "mul_res_limb_2"),
        (0, "mul_res_limb_3"),
        (0, "mul_res_limb_4"),
        (0, "mul_res_limb_5"),
        (0, "mul_res_limb_6"),
        (0, "mul_res_limb_7"),
        (0, "mul_res_limb_8"),
        (0, "mul_res_limb_9"),
        (0, "mul_res_limb_10"),
        (0, "mul_res_limb_11"),
        (0, "mul_res_limb_12"),
        (0, "mul_res_limb_13"),
        (0, "mul_res_limb_14"),
        (0, "mul_res_limb_15"),
        (0, "mul_res_limb_16"),
        (0, "mul_res_limb_17"),
        (0, "mul_res_limb_18"),
        (0, "mul_res_limb_19"),
        (0, "mul_res_limb_20"),
        (0, "mul_res_limb_21"),
        (0, "mul_res_limb_22"),
        (0, "mul_res_limb_23"),
        (0, "mul_res_limb_24"),
        (0, "mul_res_limb_25"),
        (0, "mul_res_limb_26"),
        (0, "mul_res_limb_27"),
        (0, "k"),
        (0, "carry_0"),
        (0, "carry_1"),
        (0, "carry_2"),
        (0, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (0, "carry_7"),
        (0, "carry_8"),
        (0, "carry_9"),
        (0, "carry_10"),
        (0, "carry_11"),
        (0, "carry_12"),
        (0, "carry_13"),
        (0, "carry_14"),
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
        (3, "res_limb_0"),
        (0, "res_limb_1"),
        (0, "res_limb_2"),
        (0, "res_limb_3"),
        (0, "res_limb_4"),
        (0, "res_limb_5"),
        (0, "res_limb_6"),
        (0, "res_limb_7"),
        (0, "res_limb_8"),
        (0, "res_limb_9"),
        (0, "res_limb_10"),
        (0, "res_limb_11"),
        (0, "res_limb_12"),
        (0, "res_limb_13"),
        (0, "res_limb_14"),
        (0, "res_limb_15"),
        (0, "res_limb_16"),
        (0, "res_limb_17"),
        (0, "res_limb_18"),
        (0, "res_limb_19"),
        (0, "res_limb_20"),
        (0, "res_limb_21"),
        (0, "res_limb_22"),
        (0, "res_limb_23"),
        (0, "res_limb_24"),
        (0, "res_limb_25"),
        (0, "res_limb_26"),
        (0, "res_limb_27"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (60231555, "dst_sum_squares_inv"),
        (1431655765, "dst_sum_inv"),
        (0, "op1_as_rel_imm_cond"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (6, "next_pc_jnz"),
        (4, "next_pc"),
        (11, "next_ap"),
        (11, "range_check_29_bot11bits"),
        (6, "next_fp"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_generic_consistency_jump() {
    let mut generic_opcode = GenericOpcode::default();
    let mut jump_opcode = JumpOpcode {
        rel: false,
        imm: false,
        double_deref: false,
        memory: Felt252IdMemory::default(),
    };

    let offset_value = 10;
    let op1 = 5;

    // Register values at opcode start
    let [pc, ap, fp] = [3, 11, 6];

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_jump(
                    None,
                    Some(offset_value),
                    jump_opcode.get_flags().non_constants_to_arr(&[true, false, false]),
                ),
                0
            ),
        ),
        (const_expr!((fp as i16 + offset_value) as u32), const_felt252_expr!(op1 as i128)),
        // Not in use
        (const_expr!((fp as i64 - 1) as u32), const_felt252_expr!(0, 0)),
    ];
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    jump_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&jump_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &jump_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32767, "offset0"),
        (32767, "offset1"),
        (32778, "offset2"),
        (1, "dst_base_fp"),
        (1, "op0_base_fp"),
        (0, "op1_imm"),
        (1, "op1_base_fp"),
        (0, "op1_base_ap"),
        (0, "res_add"),
        (0, "res_mul"),
        (1, "pc_update_jump"),
        (0, "pc_update_jump_rel"),
        (0, "pc_update_jnz"),
        (0, "ap_update_add"),
        (0, "ap_update_add_1"),
        (0, "opcode_call"),
        (0, "opcode_ret"),
        (0, "opcode_assert_eq"),
        (6, "dst_src"),
        (2, "dst_id"),
        (0, "dst_limb_0"),
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
        (6, "op0_src"),
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
        (0, "op0_limb_27"),
        (0, "partial_limb_msb"),
        (6, "op1_src"),
        (1, "op1_id"),
        (5, "op1_limb_0"),
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
        (0, "op1_limb_27"),
        (5, "add_res_limb_0"),
        (0, "add_res_limb_1"),
        (0, "add_res_limb_2"),
        (0, "add_res_limb_3"),
        (0, "add_res_limb_4"),
        (0, "add_res_limb_5"),
        (0, "add_res_limb_6"),
        (0, "add_res_limb_7"),
        (0, "add_res_limb_8"),
        (0, "add_res_limb_9"),
        (0, "add_res_limb_10"),
        (0, "add_res_limb_11"),
        (0, "add_res_limb_12"),
        (0, "add_res_limb_13"),
        (0, "add_res_limb_14"),
        (0, "add_res_limb_15"),
        (0, "add_res_limb_16"),
        (0, "add_res_limb_17"),
        (0, "add_res_limb_18"),
        (0, "add_res_limb_19"),
        (0, "add_res_limb_20"),
        (0, "add_res_limb_21"),
        (0, "add_res_limb_22"),
        (0, "add_res_limb_23"),
        (0, "add_res_limb_24"),
        (0, "add_res_limb_25"),
        (0, "add_res_limb_26"),
        (0, "add_res_limb_27"),
        (0, "sub_p_bit"),
        (0, "mul_res_limb_0"),
        (0, "mul_res_limb_1"),
        (0, "mul_res_limb_2"),
        (0, "mul_res_limb_3"),
        (0, "mul_res_limb_4"),
        (0, "mul_res_limb_5"),
        (0, "mul_res_limb_6"),
        (0, "mul_res_limb_7"),
        (0, "mul_res_limb_8"),
        (0, "mul_res_limb_9"),
        (0, "mul_res_limb_10"),
        (0, "mul_res_limb_11"),
        (0, "mul_res_limb_12"),
        (0, "mul_res_limb_13"),
        (0, "mul_res_limb_14"),
        (0, "mul_res_limb_15"),
        (0, "mul_res_limb_16"),
        (0, "mul_res_limb_17"),
        (0, "mul_res_limb_18"),
        (0, "mul_res_limb_19"),
        (0, "mul_res_limb_20"),
        (0, "mul_res_limb_21"),
        (0, "mul_res_limb_22"),
        (0, "mul_res_limb_23"),
        (0, "mul_res_limb_24"),
        (0, "mul_res_limb_25"),
        (0, "mul_res_limb_26"),
        (0, "mul_res_limb_27"),
        (0, "k"),
        (0, "carry_0"),
        (0, "carry_1"),
        (0, "carry_2"),
        (0, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (0, "carry_7"),
        (0, "carry_8"),
        (0, "carry_9"),
        (0, "carry_10"),
        (0, "carry_11"),
        (0, "carry_12"),
        (0, "carry_13"),
        (0, "carry_14"),
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
        (5, "res_limb_0"),
        (0, "res_limb_1"),
        (0, "res_limb_2"),
        (0, "res_limb_3"),
        (0, "res_limb_4"),
        (0, "res_limb_5"),
        (0, "res_limb_6"),
        (0, "res_limb_7"),
        (0, "res_limb_8"),
        (0, "res_limb_9"),
        (0, "res_limb_10"),
        (0, "res_limb_11"),
        (0, "res_limb_12"),
        (0, "res_limb_13"),
        (0, "res_limb_14"),
        (0, "res_limb_15"),
        (0, "res_limb_16"),
        (0, "res_limb_17"),
        (0, "res_limb_18"),
        (0, "res_limb_19"),
        (0, "res_limb_20"),
        (0, "res_limb_21"),
        (0, "res_limb_22"),
        (0, "res_limb_23"),
        (0, "res_limb_24"),
        (0, "res_limb_25"),
        (0, "res_limb_26"),
        (0, "res_limb_27"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (1351207863, "dst_sum_squares_inv"),
        (1, "dst_sum_inv"),
        (0, "op1_as_rel_imm_cond"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (4, "next_pc_jnz"),
        (5, "next_pc"),
        (11, "next_ap"),
        (11, "range_check_29_bot11bits"),
        (6, "next_fp"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_generic_jump_abs_imm() {
    let mut generic_opcode = GenericOpcode::default();
    let jump_opcode = JumpOpcode {
        rel: false,
        imm: false,
        double_deref: false,
        memory: Felt252IdMemory::default(),
    };

    // Create flags
    let mut flags = jump_opcode.get_flags();
    flags.op1_imm = Some(true);
    flags.op1_base_ap = Some(false);
    flags.op1_base_fp = Some(false);
    flags.ap_update_add_1 = Some(false);
    let imm = 5;

    // Register values at opcode start
    let [pc, ap, fp] = [3, 11, 6];

    // Fill memory
    let memory_values = vec![
        (const_expr!(pc), const_felt252_expr!(assemble_jump(None, None, flags.clone().into(),), 0)),
        (const_expr!(pc + 1), const_felt252_expr!(imm as i128)),
        // Not in use
        (const_expr!((fp as i64 - 1) as u32), const_felt252_expr!(0, 0)),
    ];
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    let (_, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(output.pc().calc(), imm.to_string());
    assert_eq!(output.fp().calc(), fp.to_string());
    assert_eq!(output.ap().calc(), ap.to_string());
}

#[test]
fn test_generic_jump_rel_double_deref() {
    let mut generic_opcode = GenericOpcode::default();
    let jump_opcode = JumpOpcode {
        rel: false,
        imm: false,
        double_deref: true,
        memory: Felt252IdMemory::default(),
    };

    // Create flags
    let mut flags = jump_opcode.get_flags();
    flags.pc_update_jump = Some(false);
    flags.pc_update_jump_rel = Some(true);
    flags.op1_imm = Some(false);
    flags.op0_base_fp = Some(true);
    flags.ap_update_add_1 = Some(false);

    // Register values at opcode start
    let [pc, ap, fp] = [3, 11, 12345];
    let op0 = 5465446;
    let op1 = 46867;
    let offset1 = -1265;
    let offset2 = 125;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_jump(Some(offset1), Some(offset2), flags.clone().into()),
                0
            ),
        ),
        (const_expr!((op0 + offset2 as i32) as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!((fp + offset1 as i32) as u32), const_felt252_expr!(op0 as i128)),
        // Not in use
        (const_expr!((fp as i64 - 1) as u32), const_felt252_expr!(0, 0)),
    ];
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    let (_, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(output.pc().calc(), (pc as i64 + op1).to_string());
    assert_eq!(output.fp().calc(), fp.to_string());
    assert_eq!(output.ap().calc(), ap.to_string());
}

#[test]
fn test_generic_consistency_jnz_taken() {
    let mut generic_opcode = GenericOpcode::default();
    let mut jnz_opcode = JnzOpcode { taken: true, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let offset_dst = -13;
    let op1 = 15;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    offset_dst,
                    -1,
                    1,
                    jnz_opcode.get_flags().non_constants_to_arr(&[false, false]),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(op1 as i128)),
        (const_expr!((ap as i16 + offset_dst) as u32), const_felt252_expr!(123, 456)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    jnz_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&jnz_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &jnz_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32755, "offset0"),
        (32767, "offset1"),
        (32769, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (1, "op1_imm"),
        (0, "op1_base_fp"),
        (0, "op1_base_ap"),
        (0, "res_add"),
        (0, "res_mul"),
        (0, "pc_update_jump"),
        (0, "pc_update_jump_rel"),
        (1, "pc_update_jnz"),
        (0, "ap_update_add"),
        (0, "ap_update_add_1"),
        (0, "opcode_call"),
        (0, "opcode_ret"),
        (0, "opcode_assert_eq"),
        (200, "dst_src"),
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
        (150, "op0_src"),
        (3, "op0_id"),
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
        (0, "op0_limb_27"),
        (0, "partial_limb_msb"),
        (50, "op1_src"),
        (1, "op1_id"),
        (15, "op1_limb_0"),
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
        (0, "op1_limb_27"),
        (15, "add_res_limb_0"),
        (0, "add_res_limb_1"),
        (0, "add_res_limb_2"),
        (0, "add_res_limb_3"),
        (0, "add_res_limb_4"),
        (0, "add_res_limb_5"),
        (0, "add_res_limb_6"),
        (0, "add_res_limb_7"),
        (0, "add_res_limb_8"),
        (0, "add_res_limb_9"),
        (0, "add_res_limb_10"),
        (0, "add_res_limb_11"),
        (0, "add_res_limb_12"),
        (0, "add_res_limb_13"),
        (0, "add_res_limb_14"),
        (0, "add_res_limb_15"),
        (0, "add_res_limb_16"),
        (0, "add_res_limb_17"),
        (0, "add_res_limb_18"),
        (0, "add_res_limb_19"),
        (0, "add_res_limb_20"),
        (0, "add_res_limb_21"),
        (0, "add_res_limb_22"),
        (0, "add_res_limb_23"),
        (0, "add_res_limb_24"),
        (0, "add_res_limb_25"),
        (0, "add_res_limb_26"),
        (0, "add_res_limb_27"),
        (0, "sub_p_bit"),
        (0, "mul_res_limb_0"),
        (0, "mul_res_limb_1"),
        (0, "mul_res_limb_2"),
        (0, "mul_res_limb_3"),
        (0, "mul_res_limb_4"),
        (0, "mul_res_limb_5"),
        (0, "mul_res_limb_6"),
        (0, "mul_res_limb_7"),
        (0, "mul_res_limb_8"),
        (0, "mul_res_limb_9"),
        (0, "mul_res_limb_10"),
        (0, "mul_res_limb_11"),
        (0, "mul_res_limb_12"),
        (0, "mul_res_limb_13"),
        (0, "mul_res_limb_14"),
        (0, "mul_res_limb_15"),
        (0, "mul_res_limb_16"),
        (0, "mul_res_limb_17"),
        (0, "mul_res_limb_18"),
        (0, "mul_res_limb_19"),
        (0, "mul_res_limb_20"),
        (0, "mul_res_limb_21"),
        (0, "mul_res_limb_22"),
        (0, "mul_res_limb_23"),
        (0, "mul_res_limb_24"),
        (0, "mul_res_limb_25"),
        (0, "mul_res_limb_26"),
        (0, "mul_res_limb_27"),
        (0, "k"),
        (0, "carry_0"),
        (0, "carry_1"),
        (0, "carry_2"),
        (0, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (0, "carry_7"),
        (0, "carry_8"),
        (0, "carry_9"),
        (0, "carry_10"),
        (0, "carry_11"),
        (0, "carry_12"),
        (0, "carry_13"),
        (0, "carry_14"),
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
        (0, "res_limb_0"),
        (0, "res_limb_1"),
        (0, "res_limb_2"),
        (0, "res_limb_3"),
        (0, "res_limb_4"),
        (0, "res_limb_5"),
        (0, "res_limb_6"),
        (0, "res_limb_7"),
        (0, "res_limb_8"),
        (0, "res_limb_9"),
        (0, "res_limb_10"),
        (0, "res_limb_11"),
        (0, "res_limb_12"),
        (0, "res_limb_13"),
        (0, "res_limb_14"),
        (0, "res_limb_15"),
        (0, "res_limb_16"),
        (0, "res_limb_17"),
        (0, "res_limb_18"),
        (0, "res_limb_19"),
        (0, "res_limb_20"),
        (0, "res_limb_21"),
        (0, "res_limb_22"),
        (0, "res_limb_23"),
        (0, "res_limb_24"),
        (0, "res_limb_25"),
        (0, "res_limb_26"),
        (0, "res_limb_27"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (500077285, "dst_sum_squares_inv"),
        (1955558780, "dst_sum_inv"),
        (414, "op1_as_rel_imm_cond"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (65, "next_pc_jnz"),
        (65, "next_pc"),
        (200, "next_ap"),
        (200, "range_check_29_bot11bits"),
        (150, "next_fp"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_generic_consistency_jnz_not_taken() {
    let mut generic_opcode = GenericOpcode::default();
    let mut jnz_opcode = JnzOpcode { taken: false, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let offset_dst = -13;
    let op1 = 15;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    offset_dst,
                    -1,
                    1,
                    jnz_opcode.get_flags().non_constants_to_arr(&[false, false]),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(op1 as i128)),
        (const_expr!((ap as i16 + offset_dst) as u32), const_felt252_expr!(0, 0)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    jnz_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&jnz_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &jnz_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    expect![[r#"
        (1, "enabler"),
        (50, "input_pc"),
        (200, "input_ap"),
        (150, "input_fp"),
        (32755, "offset0"),
        (32767, "offset1"),
        (32769, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (1, "op1_imm"),
        (0, "op1_base_fp"),
        (0, "op1_base_ap"),
        (0, "res_add"),
        (0, "res_mul"),
        (0, "pc_update_jump"),
        (0, "pc_update_jump_rel"),
        (1, "pc_update_jnz"),
        (0, "ap_update_add"),
        (0, "ap_update_add_1"),
        (0, "opcode_call"),
        (0, "opcode_ret"),
        (0, "opcode_assert_eq"),
        (200, "dst_src"),
        (2, "dst_id"),
        (0, "dst_limb_0"),
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
        (150, "op0_src"),
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
        (0, "op0_limb_27"),
        (0, "partial_limb_msb"),
        (50, "op1_src"),
        (1, "op1_id"),
        (15, "op1_limb_0"),
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
        (0, "op1_limb_27"),
        (15, "add_res_limb_0"),
        (0, "add_res_limb_1"),
        (0, "add_res_limb_2"),
        (0, "add_res_limb_3"),
        (0, "add_res_limb_4"),
        (0, "add_res_limb_5"),
        (0, "add_res_limb_6"),
        (0, "add_res_limb_7"),
        (0, "add_res_limb_8"),
        (0, "add_res_limb_9"),
        (0, "add_res_limb_10"),
        (0, "add_res_limb_11"),
        (0, "add_res_limb_12"),
        (0, "add_res_limb_13"),
        (0, "add_res_limb_14"),
        (0, "add_res_limb_15"),
        (0, "add_res_limb_16"),
        (0, "add_res_limb_17"),
        (0, "add_res_limb_18"),
        (0, "add_res_limb_19"),
        (0, "add_res_limb_20"),
        (0, "add_res_limb_21"),
        (0, "add_res_limb_22"),
        (0, "add_res_limb_23"),
        (0, "add_res_limb_24"),
        (0, "add_res_limb_25"),
        (0, "add_res_limb_26"),
        (0, "add_res_limb_27"),
        (0, "sub_p_bit"),
        (0, "mul_res_limb_0"),
        (0, "mul_res_limb_1"),
        (0, "mul_res_limb_2"),
        (0, "mul_res_limb_3"),
        (0, "mul_res_limb_4"),
        (0, "mul_res_limb_5"),
        (0, "mul_res_limb_6"),
        (0, "mul_res_limb_7"),
        (0, "mul_res_limb_8"),
        (0, "mul_res_limb_9"),
        (0, "mul_res_limb_10"),
        (0, "mul_res_limb_11"),
        (0, "mul_res_limb_12"),
        (0, "mul_res_limb_13"),
        (0, "mul_res_limb_14"),
        (0, "mul_res_limb_15"),
        (0, "mul_res_limb_16"),
        (0, "mul_res_limb_17"),
        (0, "mul_res_limb_18"),
        (0, "mul_res_limb_19"),
        (0, "mul_res_limb_20"),
        (0, "mul_res_limb_21"),
        (0, "mul_res_limb_22"),
        (0, "mul_res_limb_23"),
        (0, "mul_res_limb_24"),
        (0, "mul_res_limb_25"),
        (0, "mul_res_limb_26"),
        (0, "mul_res_limb_27"),
        (0, "k"),
        (0, "carry_0"),
        (0, "carry_1"),
        (0, "carry_2"),
        (0, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (0, "carry_7"),
        (0, "carry_8"),
        (0, "carry_9"),
        (0, "carry_10"),
        (0, "carry_11"),
        (0, "carry_12"),
        (0, "carry_13"),
        (0, "carry_14"),
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
        (0, "res_limb_0"),
        (0, "res_limb_1"),
        (0, "res_limb_2"),
        (0, "res_limb_3"),
        (0, "res_limb_4"),
        (0, "res_limb_5"),
        (0, "res_limb_6"),
        (0, "res_limb_7"),
        (0, "res_limb_8"),
        (0, "res_limb_9"),
        (0, "res_limb_10"),
        (0, "res_limb_11"),
        (0, "res_limb_12"),
        (0, "res_limb_13"),
        (0, "res_limb_14"),
        (0, "res_limb_15"),
        (0, "res_limb_16"),
        (0, "res_limb_17"),
        (0, "res_limb_18"),
        (0, "res_limb_19"),
        (0, "res_limb_20"),
        (0, "res_limb_21"),
        (0, "res_limb_22"),
        (0, "res_limb_23"),
        (0, "res_limb_24"),
        (0, "res_limb_25"),
        (0, "res_limb_26"),
        (0, "res_limb_27"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "partial_limb_msb"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (1351207863, "dst_sum_squares_inv"),
        (1, "dst_sum_inv"),
        (0, "op1_as_rel_imm_cond"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "partial_limb_msb"),
        (52, "next_pc_jnz"),
        (52, "next_pc"),
        (200, "next_ap"),
        (200, "range_check_29_bot11bits"),
        (150, "next_fp"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_generic_jnz_deref_taken() {
    let mut generic_opcode = GenericOpcode::default();
    let jnz_opcode = JnzOpcode { taken: true, memory: Felt252IdMemory::default() };

    // Create flags
    let mut flags = jnz_opcode.get_flags();
    flags.dst_base_fp = Some(false);
    flags.op1_imm = Some(false);
    flags.op1_base_ap = Some(true);
    flags.ap_update_add_1 = Some(true);

    // Register values at opcode start
    let [pc, ap, fp] = [50, 458, 150];
    let offset_dst = -150;
    let offset2 = 3244;
    let op1 = 5456;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    offset_dst,
                    -1,
                    offset2,
                    flags.clone().into(),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(ap + offset2 as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!((ap as i16 + offset_dst) as u32), const_felt252_expr!(123, 456)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    let (_, next_state) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(next_state.pc().calc(), (pc as i128 + op1 as i128).to_string());
    assert_eq!(next_state.ap().calc(), (ap + 1).to_string());
    assert_eq!(next_state.fp().calc(), fp.to_string());
}

#[test]
fn test_generic_jnz_deref_not_taken() {
    let mut generic_opcode = GenericOpcode::default();
    let jnz_opcode = JnzOpcode { taken: true, memory: Felt252IdMemory::default() };

    // Create flags
    let mut flags = jnz_opcode.get_flags();
    flags.dst_base_fp = Some(true);
    flags.op1_imm = Some(false);
    flags.op1_base_fp = Some(true);
    flags.ap_update_add_1 = Some(false);

    // Register values at opcode start
    let [pc, ap, fp] = [50, 458, 150];
    let offset_dst = -150;
    let offset2 = 3244;
    let op1 = 5456;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    offset_dst,
                    -1,
                    offset2,
                    flags.clone().into(),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(fp + offset2 as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!((fp as i16 + offset_dst) as u32), const_felt252_expr!(0, 0)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    let (_, next_state) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(next_state.pc().calc(), (pc + 1).to_string());
    assert_eq!(next_state.ap().calc(), ap.to_string());
    assert_eq!(next_state.fp().calc(), fp.to_string());
}

#[test]
fn test_generic_add_ap_double_deref() {
    let mut generic_opcode = GenericOpcode::default();
    let add_ap = AddApOpcode { memory: Felt252IdMemory::default() };

    // Create flags
    let non_consts_flags = vec![false, false, false];
    let flags = add_ap.get_flags().non_constants_to_arr(&non_consts_flags);

    // Register values at opcode start
    let [pc, ap, fp] = [50, 5458, 150];
    let offset1 = -123;
    let offset2 = 3244;
    let op0 = 789;
    let op1 = 5456;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(-1, offset1, offset2, flags, OpcodeExtension::Stone),
                0
            ),
        ),
        (const_expr!((op0 + offset2 as i32) as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!((fp + offset1 as i32) as u32), const_felt252_expr!(op0 as i128)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    let (_, next_state) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check the output
    assert_eq!(next_state.pc().calc(), (pc + 1).to_string());
    assert_eq!(next_state.fp().calc(), fp.to_string());
    assert_eq!(next_state.ap().calc(), (ap + op1).to_string());
}

#[test]
fn test_generic_add_ap_res_mul() {
    let mut generic_opcode = GenericOpcode::default();
    let add_ap = AddApOpcode { memory: Felt252IdMemory::default() };

    // Create flags
    let non_consts_flags = vec![false, false, true];
    let mut flags = add_ap.get_flags().non_constants_to_arr(&non_consts_flags);
    flags[FLAG_RES_MUL_INDEX] = true;

    // Register values at opcode start
    let [pc, ap, fp] = [50, 5458, 150];
    let offset1 = -123;
    let offset2 = 3244;
    let op0 = 789;
    let op1 = 5456;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(-1, offset1, offset2, flags, OpcodeExtension::Stone),
                0
            ),
        ),
        (const_expr!(ap + offset2 as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!((fp as i32 + offset1 as i32) as u32), const_felt252_expr!(op0 as i128)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    let (_, next_state) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check the output
    assert_eq!(next_state.pc().calc(), (pc + 1).to_string());
    assert_eq!(next_state.fp().calc(), fp.to_string());
    assert_eq!(next_state.ap().calc(), (ap + op1 * op0).to_string());
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck18 on input 262144")]
fn test_generic_add_ap_res_mul_too_big() {
    let mut generic_opcode = GenericOpcode::default();
    let add_ap = AddApOpcode { memory: Felt252IdMemory::default() };

    // Create flags
    let non_consts_flags = vec![false, false, true];
    let mut flags = add_ap.get_flags().non_constants_to_arr(&non_consts_flags);
    flags[FLAG_RES_MUL_INDEX] = true;

    // Register values at opcode start
    let [pc, ap, fp] = [50, 1 << 28, 150];
    let offset1 = -123;
    let offset2 = 3244;
    let op0 = 1_u32 << 14;
    let op1 = 1_u32 << 14;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(-1, offset1, offset2, flags, OpcodeExtension::Stone),
                0
            ),
        ),
        (const_expr!(ap + offset2 as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!((fp as i32 + offset1 as i32) as u32), const_felt252_expr!(op0 as i128)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck18 on input 1048575")]
fn test_generic_add_ap_res_mul_negative() {
    let mut generic_opcode = GenericOpcode::default();
    let add_ap = AddApOpcode { memory: Felt252IdMemory::default() };

    // Create flags
    let non_consts_flags = vec![false, false, true];
    let mut flags = add_ap.get_flags().non_constants_to_arr(&non_consts_flags);
    flags[FLAG_RES_MUL_INDEX] = true;

    // Register values at opcode start
    let [pc, ap, fp] = [50, 1234, 150];
    let offset1 = -123;
    let offset2 = 3244;
    let op0 = ap + 1;
    let op1 = -1;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(-1, offset1, offset2, flags, OpcodeExtension::Stone),
                0
            ),
        ),
        (const_expr!(ap + offset2 as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!((fp as i32 + offset1 as i32) as u32), const_felt252_expr!(op0 as i128)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

#[test]
fn test_generic_add_ap_res_add() {
    let mut generic_opcode = GenericOpcode::default();
    let add_ap = AddApOpcode { memory: Felt252IdMemory::default() };

    // Create flags
    let non_consts_flags = vec![false, true, false];
    let mut flags = add_ap.get_flags().non_constants_to_arr(&non_consts_flags);
    flags[FLAG_OP0_BASE_FP_INDEX] = false;
    flags[FLAG_RES_ADD_INDEX] = true;

    // Register values at opcode start
    let [pc, ap, fp] = [454, 7888, 5656];
    let offset1 = -45;
    let offset2 = 1255;
    let op0: i32 = -465;
    let op1: i32 = 5456;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(-1, offset1, offset2, flags, OpcodeExtension::Stone),
                0
            ),
        ),
        (const_expr!(fp + offset2 as u32), const_felt252_expr!(op1 as i128)),
        (const_expr!((ap as i32 + offset1 as i32) as u32), const_felt252_expr!(op0 as i128)),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    let (_, next_state) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check the output
    assert_eq!(next_state.pc().calc(), (pc + 1).to_string());
    assert_eq!(next_state.fp().calc(), fp.to_string());
    assert_eq!(next_state.ap().calc(), (ap as i32 + op0 + op1).to_string());
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_generic_soundness_call_wrong_offset() {
    let mut generic_opcode = GenericOpcode::default();
    let call_opcode = CallOpcode { rel_imm: true, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let immediate = 2346;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                // Use invalid value for offset dst
                assemble_instruction(
                    1,
                    1,
                    1,
                    call_opcode.get_flags().into(),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(immediate)),
        (const_expr!(ap), const_felt252_expr!(fp as i64)),
        (const_expr!(ap + 1), const_felt252_expr!(pc as i64 + 2)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&call_opcode);
    let (..) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_generic_soundness_call_fp_not_pushed() {
    let mut generic_opcode = GenericOpcode::default();
    let call_opcode = CallOpcode { rel_imm: true, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let immediate = 2346;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    0,
                    1,
                    1,
                    call_opcode.get_flags().into(),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(immediate)),
        // save ap instead of fp
        (const_expr!(ap), const_felt252_expr!(ap as i64)),
        (const_expr!(ap + 1), const_felt252_expr!(pc as i64 + 2)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&call_opcode);
    let (..) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_generic_soundness_call_wrong_next_pc() {
    let mut generic_opcode = GenericOpcode::default();
    let call_opcode = CallOpcode { rel_imm: false, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let offset2 = -5;
    let op1 = 400;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    0,
                    1,
                    offset2,
                    call_opcode.get_flags().non_constants_to_arr(&[true, false]),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!((fp as i16 + offset2) as u32), const_felt252_expr!(op1)),
        (const_expr!(ap), const_felt252_expr!(fp as i64)),
        // Set next pc to wrong value
        (const_expr!(ap + 1), const_felt252_expr!(pc as i64 + 2)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    let (..) = registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_generic_soundness_jnz_dst_p() {
    let mut generic_opcode = GenericOpcode::default();
    let jnz_opcode = JnzOpcode { taken: false, memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let offset_dst = -13;
    let op1 = 15;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    offset_dst,
                    -1,
                    1,
                    jnz_opcode.get_flags().non_constants_to_arr(&[false, false]),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!(pc + 1), const_felt252_expr!(op1 as i128)),
        (
            const_expr!((ap as i16 + offset_dst) as u32),
            const_felt252_expr!(1, 10633823966279327296825105735305134080),
        ),
        // Not in use
        (const_expr!(fp - 1), const_felt252_expr!(0, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_generic_soundness_assert_eq() {
    let mut generic_opcode = GenericOpcode::default();
    let assert_eq =
        AssertEqOpcode { double_deref: false, imm: false, memory: Felt252IdMemory::default() };

    // Create flags
    let mut flags = assert_eq.get_flags();
    flags.dst_base_fp = Some(false);
    flags.op1_base_ap = Some(true);
    flags.op1_base_fp = Some(false);
    flags.res_mul = Some(true);
    flags.ap_update_add_1 = Some(true);

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let offset_dst = -13;
    let offset1 = 45;
    let offset2 = 3244;
    // Wrong value for the multiplication
    let dst = 359;
    let op0 = 24;
    let op1 = 15;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(
                assemble_instruction(
                    offset_dst,
                    offset1,
                    offset2,
                    flags.clone().into(),
                    OpcodeExtension::Stone
                ),
                0
            ),
        ),
        (const_expr!((ap as i16 + offset_dst) as u32), const_felt252_expr!(dst)),
        (const_expr!((fp as i16 + offset1) as u32), const_felt252_expr!(op0, 0)),
        (const_expr!((ap as i16 + offset2) as u32), const_felt252_expr!(op1, 0)),
    ];

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.run_air(
        &generic_opcode,
        (),
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}
