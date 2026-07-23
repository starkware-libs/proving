use std::collections::BTreeSet;

use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::state::State;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;
use stwo_cairo_common::prover_types::cpu::PRIME;

use super::decode_inst::*;
use crate::casm::common::*;

fn test_with_matching_memory(
    flags: [bool; 15],
    flag_const: [bool; 15],
    offsets: [i16; 3],
    offset_const: [bool; 3],
    flag_sets_of_sum_1: BTreeSet<BTreeSet<usize>>,
) -> State {
    let const_offsets = offsets
        .iter()
        .enumerate()
        .map(|(i, &off)| if offset_const[i] { Some(off) } else { None })
        .collect::<Vec<Option<i16>>>()
        .try_into()
        .unwrap();
    let const_flags = Flags::from_arr(
        flags
            .iter()
            .enumerate()
            .map(|(i, &flag)| if flag_const[i] { Some(flag) } else { None })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Felt252IdMemory::new_with_data(vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(offsets[0], offsets[1], offsets[2], flags, OpcodeExtension::Stone),
            0
        ),
    )]);

    // Run and check output
    let air_fn = DecodeInstruction {
        const_offsets,
        const_flags,
        const_opcode_extension: Some(OpcodeExtension::Stone),
        flag_sets_of_sum_1,
        memory,
    };

    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, (offsets_output, flags_output, _)) =
        registry.run_air(&air_fn, (), CasmAddress::new(pc, "pc"));

    for (i, &offset) in offsets.iter().enumerate() {
        assert_eq!(offsets_output[i].calc(), (offset as i64).rem_euclid(PRIME as i64).to_string());
    }
    for (i, flag) in flags.iter().enumerate() {
        assert_eq!(flags_output[i].calc(), (*flag as u32).to_string());
    }

    state
}

fn init_flags_and_offsets() -> ([bool; 15], [i16; 3]) {
    let flags = Flags {
        dst_base_fp: Some(false),
        op0_base_fp: Some(true),
        op1_imm: Some(false),
        op1_base_fp: Some(true),
        op1_base_ap: Some(false),
        res_add: Some(false),
        res_mul: Some(false),
        pc_update_jump: Some(true),
        pc_update_jump_rel: Some(false),
        pc_update_jnz: Some(true),
        ap_update_add: Some(true),
        ap_update_add_1: Some(false),
        opcode_call: Some(false),
        opcode_ret: Some(false),
        opcode_assert_eq: Some(true),
    };
    let offsets = [0x4321, -0x0765, 0xcba];
    (flags.into(), offsets)
}

#[test]
fn test_no_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let offset_const = [false; 3];
    let flag_const = [false; 15];

    let state =
        test_with_matching_memory(flags, flag_const, offsets, offset_const, BTreeSet::new());
    expect![[r#"
        (49953, "offset0"),
        (30875, "offset1"),
        (36026, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (0, "op1_imm"),
        (1, "op1_base_fp"),
        (0, "op1_base_ap"),
        (0, "res_add"),
        (0, "res_mul"),
        (1, "pc_update_jump"),
        (0, "pc_update_jump_rel"),
        (1, "pc_update_jnz"),
        (1, "ap_update_add"),
        (0, "ap_update_add_1"),
        (0, "opcode_call"),
        (0, "opcode_ret"),
        (1, "opcode_assert_eq"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_all_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let offset_const = [true; 3];
    let flag_const = [true; 15];

    let state =
        test_with_matching_memory(flags, flag_const, offsets, offset_const, BTreeSet::new());

    assert!(state.is_empty());
}

#[test]
fn test_some_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let offset_const = [true, false, true];
    let mut flag_const = [true; 15];
    flag_const[0] = false;
    flag_const[2] = false;

    let state =
        test_with_matching_memory(flags, flag_const, offsets, offset_const, BTreeSet::new());

    expect![[r#"
        (30875, "offset1"),
        (0, "dst_base_fp"),
        (0, "op1_imm"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_flag_sets_of_sum_1() {
    let (flags, offsets) = init_flags_and_offsets();
    let offset_const = [true, false, true];
    let mut flag_const = [true; 15];
    flag_const[0] = false;
    flag_const[FLAG_OP1_IMM_INDEX] = false;
    flag_const[FLAG_OP1_BASE_FP_INDEX] = false;
    flag_const[FLAG_OP1_BASE_AP_INDEX] = false;
    flag_const[FLAG_PC_UPDATE_JUMP_INDEX] = false;
    flag_const[FLAG_OPCODE_CALL_INDEX] = false;
    flag_const[FLAG_OPCODE_RET_INDEX] = false;
    flag_const[FLAG_OPCODE_ASSERT_EQ_INDEX] = false;

    let state = test_with_matching_memory(
        flags,
        flag_const,
        offsets,
        offset_const,
        BTreeSet::from([
            BTreeSet::from([FLAG_OP1_IMM_INDEX, FLAG_OP1_BASE_FP_INDEX, FLAG_OP1_BASE_AP_INDEX]),
            BTreeSet::from([
                FLAG_OPCODE_CALL_INDEX,
                FLAG_OPCODE_RET_INDEX,
                FLAG_OPCODE_ASSERT_EQ_INDEX,
            ]),
        ]),
    );

    expect![[r#"
        (30875, "offset1"),
        (0, "dst_base_fp"),
        (0, "op1_imm"),
        (1, "op1_base_fp"),
        (1, "pc_update_jump"),
        (0, "opcode_call"),
        (0, "opcode_ret"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_opcode_extension_const() {
    let const_offsets = [0x4321, -0x0765, 0xcba];

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Felt252IdMemory::new_with_data(vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                const_offsets[0],
                const_offsets[1],
                const_offsets[2],
                [true; 15],
                OpcodeExtension::Blake
            ),
            0
        ),
    )]);

    // Run and check output
    let air_fn = DecodeInstruction {
        const_offsets: [Some(const_offsets[0]), Some(const_offsets[1]), Some(const_offsets[2])],
        const_flags: Flags::from_arr([Some(true); 15]),
        const_opcode_extension: Some(OpcodeExtension::Blake),
        flag_sets_of_sum_1: BTreeSet::new(),
        memory,
    };

    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, (_, _, opcode_extension)) = registry.run_air(&air_fn, (), CasmAddress::new(pc, "pc"));
    assert_eq!(opcode_extension.calc(), "1");
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_fail_opcode_extension_const() {
    let const_offsets = [0x4321, -0x0765, 0xcba];

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Felt252IdMemory::new_with_data(vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                const_offsets[0],
                const_offsets[1],
                const_offsets[2],
                [true; 15],
                OpcodeExtension::BlakeFinalize
            ),
            0
        ),
    )]);

    // Run and check output
    let air_fn = DecodeInstruction {
        const_offsets: [Some(const_offsets[0]), Some(const_offsets[1]), Some(const_offsets[2])],
        const_flags: Flags::from_arr([Some(true); 15]),
        const_opcode_extension: Some(OpcodeExtension::Blake),
        flag_sets_of_sum_1: BTreeSet::new(),
        memory,
    };

    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, (_, _, opcode_extension)) = registry.run_air(&air_fn, (), CasmAddress::new(pc, "pc"));
    assert_eq!(opcode_extension.calc(), "1");
}

#[test]
fn test_opcode_extension() {
    let const_offsets = [0x4321, -0x0765, 0xcba];

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Felt252IdMemory::new_with_data(vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                const_offsets[0],
                const_offsets[1],
                const_offsets[2],
                [true; 15],
                OpcodeExtension::BlakeFinalize
            ),
            0
        ),
    )]);

    // Run and check output
    let air_fn = DecodeInstruction {
        const_offsets: [Some(const_offsets[0]), Some(const_offsets[1]), Some(const_offsets[2])],
        const_flags: Flags::from_arr([Some(true); 15]),
        const_opcode_extension: None,
        flag_sets_of_sum_1: BTreeSet::new(),
        memory,
    };

    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, (_, _, opcode_extension)) = registry.run_air(&air_fn, (), CasmAddress::new(pc, "pc"));
    assert_eq!(opcode_extension.calc(), "2");
}
