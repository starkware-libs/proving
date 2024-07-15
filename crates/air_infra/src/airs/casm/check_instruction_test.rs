use std::u16;

use super::check_instruction::*;
use super::common::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::FeltExpr;
use crate::core::memory::*;
use crate::core::prover_types::*;

//Macros
use crate::const_expr;
use crate::felt252_expr;

fn test_with_matching_memory(
    const_flags: Flags,
    offsets: [i16; 3],
    is_offset_const: [bool; 3],
    expected_constraints: Vec<&str>,
    expected_deductions: Vec<&str>,
) {
    let offsets_u16: Vec<u16> = offsets.into_iter().map(offset_as_u16).collect();
    let const_offsets = offsets_u16
        .iter()
        .enumerate()
        .map(|(i, &off)| if is_offset_const[i] { Some(off) } else { None })
        .collect::<Vec<Option<u16>>>()
        .try_into()
        .unwrap();

    // Compute expected state.
    let offsets_parts_arr = [
        offsets_u16[0] & 0xFFF,
        offsets_u16[0] >> 12,
        offsets_u16[1] & 0xFF,
        offsets_u16[1] >> 8,
        offsets_u16[2] & 0xF,
        offsets_u16[2] >> 4,
    ];
    let expected_state: Vec<String> = is_offset_const
        .into_iter()
        .enumerate()
        .filter(|(_, is_const)| !is_const)
        .flat_map(|(i, _)| {
            [
                offsets_parts_arr[i * 2].to_string(),
                offsets_parts_arr[i * 2 + 1].to_string(),
            ]
        })
        .collect();

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Memory::<FeltExpr, Felt252Expr>::new_with_data(vec![(
        pc.clone(),
        felt252_expr!(
            "instruction",
            assemble_instruction(
                offsets[0],
                offsets[1],
                offsets[2],
                const_flags.clone().into()
            ) as u128,
            0
        ),
    )]);

    // Run and check output
    let air_fn = CheckInstruction {
        const_offsets,
        const_flags,
        memory,
    };

    let registry = AirFnRegistry::new(&air_fn);
    let lists = registry.get_compiled_air_fn(&air_fn);
    let (state, output) = registry.run_air(&air_fn, pc);

    assert!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == expected_constraints
    );

    assert!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == expected_deductions
    );

    assert!(state.calc() == expected_state);

    for (i, &offset) in offsets_u16.iter().enumerate() {
        assert!(output[i].calc() == offset.to_string());
    }
}

fn init_flags_and_offsets() -> (Flags, [i16; 3]) {
    let const_flags = Flags {
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
    (const_flags, offsets)
}

#[test]
fn test_no_consts() {
    let (const_flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [false, false, false];
    let expected_constraints = vec![
        "RangeCheck4([state[1]]) == []",
        "RangeCheck8([state[2]]) == []",
        "RangeCheck8([state[3]]) == []",
        "RangeCheck4([state[4]]) == []",
        concat!("Memory__FeltExpr__Felt252Expr([CheckInstruction__Flags{",
        "0:false__1:true__2:false__3:true__4:false__5:false__6:false__7:true__8:false__9:true__",
        "10:true__11:false__12:false__13:false__14:true}__[None__None__None]_input]) == ",
        "[state[0], (state[1] + (state[2] * const_16)), (state[3] + (state[4] * const_256)), ",
        "state[5], const_1674, const_4]"),
    ];
    let expected_deductions = vec![
        concat!("deduction_tmp_0 = Memory__FeltExpr__Felt252Expr(CheckInstruction__Flags{",
        "0:false__1:true__2:false__3:true__4:false__5:false__6:false__7:true__8:false__9:true__",
        "10:true__11:false__12:false__13:false__14:true}__[None__None__None]_input)"),
        "deduction_tmp_0.get_felt(const_0)",
        "deduction_tmp_1 = (UInt32::from_felt(deduction_tmp_0.get_felt(const_1)) & const_15)",
        "deduction_tmp_1.low().as_felt()",
        "deduction_tmp_2 = RangeCheck4(state[1])",
        "deduction_tmp_3 = ((UInt32::from_felt(deduction_tmp_0.get_felt(const_1)) >> const_4) & const_255)",
        "deduction_tmp_3.low().as_felt()",
        "deduction_tmp_4 = RangeCheck8(state[2])",
        "deduction_tmp_5 = (UInt32::from_felt(deduction_tmp_0.get_felt(const_2)) & const_255)",
        "deduction_tmp_5.low().as_felt()",
        "deduction_tmp_6 = RangeCheck8(state[3])",
        "deduction_tmp_7 = ((UInt32::from_felt(deduction_tmp_0.get_felt(const_2)) >> const_8) & const_15)",
        "deduction_tmp_7.low().as_felt()",
        "deduction_tmp_8 = RangeCheck4(state[4])",
        "deduction_tmp_0.get_felt(const_3)",
    ];
    test_with_matching_memory(
        const_flags,
        offsets,
        is_offset_const,
        expected_constraints,
        expected_deductions,
    );
}

#[test]
fn test_all_consts() {
    let (const_flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [true, true, true];
    let expected_constraints = vec![
        concat!("Memory__FeltExpr__Felt252Expr([CheckInstruction__Flags{",
        "0:false__1:true__2:false__3:true__4:false__5:false__6:false__7:true__8:false__9:true__10:true__11:false__",
        "12:false__13:false__14:true}__[Some(49953)__Some(30875)__Some(36026)]_input]) == ",
        "[const_801, const_2492, const_2680, const_2251, const_1674, const_4]")];
    let expected_deductions = vec![
        concat!("deduction_tmp_0 = Memory__FeltExpr__Felt252Expr(",
    "CheckInstruction__Flags{0:false__1:true__2:false__3:true__4:false__5:false__6:false__7:true__8:false__",
    "9:true__10:true__11:false__12:false__13:false__14:true}__[Some(49953)__Some(30875)__Some(36026)]_input)")
    ];
    test_with_matching_memory(
        const_flags,
        offsets,
        is_offset_const,
        expected_constraints,
        expected_deductions,
    );
}

#[test]
fn test_two_consts() {
    let (const_flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [true, false, true];
    let expected_constraints = vec![
    "RangeCheck8([state[0]]) == []",
    "RangeCheck8([state[1]]) == []",
    concat!("Memory__FeltExpr__Felt252Expr([CheckInstruction__Flags{",
    "0:false__1:true__2:false__3:true__4:false__5:false__6:false__7:true__8:false__9:true__",
    "10:true__11:false__12:false__13:false__14:true}__[Some(49953)__None__Some(36026)]_input]) == ",
    "[const_801, (const_12 + (state[0] * const_16)), (state[1] + const_2560), const_2251, const_1674, const_4]")
    ];
    let expected_deductions = vec![
        concat!("deduction_tmp_0 = Memory__FeltExpr__Felt252Expr(CheckInstruction__Flags{",
        "0:false__1:true__2:false__3:true__4:false__5:false__6:false__7:true__8:false__9:true__",
        "10:true__11:false__12:false__13:false__14:true}__[Some(49953)__None__Some(36026)]_input)"),
        "deduction_tmp_1 = ((UInt32::from_felt(deduction_tmp_0.get_felt(const_1)) >> const_4) & const_255)",
        "deduction_tmp_1.low().as_felt()",
        "deduction_tmp_2 = RangeCheck8(state[0])",
        "deduction_tmp_3 = (UInt32::from_felt(deduction_tmp_0.get_felt(const_2)) & const_255)",
        "deduction_tmp_3.low().as_felt()",
        "deduction_tmp_4 = RangeCheck8(state[1])",
    ];
    test_with_matching_memory(
        const_flags,
        offsets,
        is_offset_const,
        expected_constraints,
        expected_deductions,
    );
}
