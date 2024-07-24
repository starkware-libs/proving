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
    flags: [bool; 15],
    is_flag_const: [bool; 15],
    offsets: [i16; 3],
    is_offset_const: [bool; 3],
    expected_constraints: &[&str],
    expected_deductions: &[&str],
) {
    let offsets_u16: Vec<u16> = offsets.into_iter().map(offset_as_u16).collect();
    let const_offsets = offsets_u16
        .iter()
        .enumerate()
        .map(|(i, &off)| if is_offset_const[i] { Some(off) } else { None })
        .collect::<Vec<Option<u16>>>()
        .try_into()
        .unwrap();
    let const_flags = Flags::from_arr(
        flags
            .iter()
            .enumerate()
            .map(|(i, &flag)| if is_flag_const[i] { Some(flag) } else { None })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );

    // Compute expected state.
    let offsets_parts_arr = [
        offsets_u16[0] & 0xFFF,
        offsets_u16[0] >> 12,
        offsets_u16[1] & 0xFF,
        offsets_u16[1] >> 8,
        offsets_u16[2] & 0xF,
        offsets_u16[2] >> 4,
    ];
    let mut expected_state: Vec<String> = is_offset_const
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
    is_flag_const.iter().enumerate().for_each(|(i, is_const)| {
        if !is_const {
            expected_state.push((flags[i] as u32).to_string())
        }
    });

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Memory::<FeltExpr, Felt252Expr>::new_with_data(vec![(
        pc.clone(),
        felt252_expr!(
            "instruction",
            assemble_instruction(offsets[0], offsets[1], offsets[2], flags,) as u128,
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
    let (state, (offsets_output, flags_output)) = registry.run_air(&air_fn, pc);

    assert_eq!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        expected_constraints
    );

    assert_eq!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        expected_deductions
    );

    assert_eq!(state.calc(), expected_state);

    for (i, &offset) in offsets.iter().enumerate() {
        assert_eq!(
            offsets_output[i].calc(),
            (offset as i64).rem_euclid(PRIME as i64).to_string()
        );
    }
    for (i, flag) in flags.iter().enumerate() {
        assert_eq!(flags_output[i].calc(), flag.to_string());
    }
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
    let is_offset_const = [false; 3];
    let is_flag_const = [false; 15];

    let check_instruction_input = "CheckInstruction__Flags{dst_base_fp:None__op0_base_fp:None__op1_imm:None__op1_base_fp:None__op1_base_ap:None__res_add:None__res_mul:None__pc_update_jump:None__pc_update_jump_rel:None__pc_update_jnz:None__ap_update_add:None__ap_update_add_1:None__opcode_call:None__opcode_ret:None__opcode_assert_eq:None}__[None__None__None]_input";
    let expected_constraints = [
        "RangeCheck4([state[1]]) == []",
        "RangeCheck8([state[2]]) == []",
        "RangeCheck8([state[3]]) == []",
        "RangeCheck4([state[4]]) == []",
        "(state[6] * (const_1 - state[6]))",
        "(state[7] * (const_1 - state[7]))",
        "(state[8] * (const_1 - state[8]))",
        "(state[9] * (const_1 - state[9]))",
        "(state[10] * (const_1 - state[10]))",
        "(state[11] * (const_1 - state[11]))",
        "(state[12] * (const_1 - state[12]))",
        "(state[13] * (const_1 - state[13]))",
        "(state[14] * (const_1 - state[14]))",
        "(state[15] * (const_1 - state[15]))",
        "(state[16] * (const_1 - state[16]))",
        "(state[17] * (const_1 - state[17]))",
        "(state[18] * (const_1 - state[18]))",
        "(state[19] * (const_1 - state[19]))",
        "(state[20] * (const_1 - state[20]))",
        &format!("Memory__FeltExpr__Felt252Expr([{}]) == zero_extend([{}, {}, {}, {}, {}, {}])",
            check_instruction_input,
            "state[0]",
            "(state[1] + (state[2] * const_16))",
            "(state[3] + (state[4] * const_256))",
            "state[5]",
            "((((((((((((const_0 + (state[6] * const_1)) + (state[7] * const_2)) + (state[8] * const_4)) + (state[9] * const_8)) + (state[10] * const_16)) + (state[11] * const_32)) + (state[12] * const_64)) + (state[13] * const_128)) + (state[14] * const_256)) + (state[15] * const_512)) + (state[16] * const_1024)) + (state[17] * const_2048))",
            "(((const_0 + (state[18] * const_1)) + (state[19] * const_2)) + (state[20] * const_4))",
        ),
    ];
    let expected_deductions = [
        &format!("deduction_tmp_0 = Memory__FeltExpr__Felt252Expr({})",check_instruction_input),
        "deduction_tmp_0.get_m31(const_0)",
        "deduction_tmp_1 = (UInt32::from_m31(deduction_tmp_0.get_m31(const_1)) & const_15)",
        "deduction_tmp_1.low().as_m31()",
        "deduction_tmp_2 = RangeCheck4(state[1])",
        "deduction_tmp_3 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_1)) >> const_4) & const_255)",
        "deduction_tmp_3.low().as_m31()",
        "deduction_tmp_4 = RangeCheck8(state[2])",
        "deduction_tmp_5 = (UInt32::from_m31(deduction_tmp_0.get_m31(const_2)) & const_255)",
        "deduction_tmp_5.low().as_m31()",
        "deduction_tmp_6 = RangeCheck8(state[3])",
        "deduction_tmp_7 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_2)) >> const_8) & const_15)",
        "deduction_tmp_7.low().as_m31()",
        "deduction_tmp_8 = RangeCheck4(state[4])",
        "deduction_tmp_0.get_m31(const_3)",
        "deduction_tmp_9 = (UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) & const_1)",
        "deduction_tmp_9.low().as_m31()",
        "deduction_tmp_10 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_1) & const_1)",
        "deduction_tmp_10.low().as_m31()",
        "deduction_tmp_11 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_2) & const_1)",
        "deduction_tmp_11.low().as_m31()",
        "deduction_tmp_12 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_3) & const_1)",
        "deduction_tmp_12.low().as_m31()",
        "deduction_tmp_13 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_4) & const_1)",
        "deduction_tmp_13.low().as_m31()",
        "deduction_tmp_14 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_5) & const_1)",
        "deduction_tmp_14.low().as_m31()",
        "deduction_tmp_15 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_6) & const_1)",
        "deduction_tmp_15.low().as_m31()",
        "deduction_tmp_16 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_7) & const_1)",
        "deduction_tmp_16.low().as_m31()",
        "deduction_tmp_17 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_8) & const_1)",
        "deduction_tmp_17.low().as_m31()",
        "deduction_tmp_18 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_9) & const_1)",
        "deduction_tmp_18.low().as_m31()",
        "deduction_tmp_19 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_10) & const_1)",
        "deduction_tmp_19.low().as_m31()",
        "deduction_tmp_20 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_11) & const_1)",
        "deduction_tmp_20.low().as_m31()",
        "deduction_tmp_21 = (UInt32::from_m31(deduction_tmp_0.get_m31(const_5)) & const_1)",
        "deduction_tmp_21.low().as_m31()",
        "deduction_tmp_22 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_5)) >> const_1) & const_1)",
        "deduction_tmp_22.low().as_m31()",
        "deduction_tmp_23 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_5)) >> const_2) & const_1)",
        "deduction_tmp_23.low().as_m31()"
    ];

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        &expected_constraints,
        &expected_deductions,
    );
}

#[test]
fn test_all_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [true; 3];
    let is_flag_const = [true; 15];

    let check_instruction_input = "CheckInstruction__Flags{dst_base_fp:Some(false)__op0_base_fp:Some(true)__op1_imm:Some(false)__op1_base_fp:Some(true)__op1_base_ap:Some(false)__res_add:Some(false)__res_mul:Some(false)__pc_update_jump:Some(true)__pc_update_jump_rel:Some(false)__pc_update_jnz:Some(true)__ap_update_add:Some(true)__ap_update_add_1:Some(false)__opcode_call:Some(false)__opcode_ret:Some(false)__opcode_assert_eq:Some(true)}__[Some(49953)__Some(30875)__Some(36026)]_input";
    let expected_constraints: [&str; 1] = [&format!(
        "Memory__FeltExpr__Felt252Expr([{}]) == zero_extend([{}, {}, {}, {}, {}, {}])",
        check_instruction_input,
        "const_801",
        "const_2492",
        "const_2680",
        "const_2251",
        "const_1674",
        "const_4",
    )];
    let expected_deductions: [&str; 1] = [&format!(
        "deduction_tmp_0 = Memory__FeltExpr__Felt252Expr({})",
        check_instruction_input
    )];

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        &expected_constraints,
        &expected_deductions,
    );
}

#[test]
fn test_some_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [true, false, true];
    let mut is_flag_const = [true; 15];
    is_flag_const[0] = false;
    is_flag_const[2] = false;

    let check_instruction_input = "CheckInstruction__Flags{dst_base_fp:None__op0_base_fp:Some(true)__op1_imm:None__op1_base_fp:Some(true)__op1_base_ap:Some(false)__res_add:Some(false)__res_mul:Some(false)__pc_update_jump:Some(true)__pc_update_jump_rel:Some(false)__pc_update_jnz:Some(true)__ap_update_add:Some(true)__ap_update_add_1:Some(false)__opcode_call:Some(false)__opcode_ret:Some(false)__opcode_assert_eq:Some(true)}__[Some(49953)__None__Some(36026)]_input";
    let expected_constraints = [
        "RangeCheck8([state[0]]) == []",
        "RangeCheck8([state[1]]) == []",
        "(state[2] * (const_1 - state[2]))",
        "(state[3] * (const_1 - state[3]))",
        &format!(
            "Memory__FeltExpr__Felt252Expr([{}]) == zero_extend([{}, {}, {}, {}, {}, {}])",
            check_instruction_input,
            "const_801",
            "(const_12 + (state[0] * const_16))",
            "(state[1] + const_2560)",
            "const_2251",
            "((const_1674 + (state[2] * const_1)) + (state[3] * const_4))",
            "const_4",
        ),
    ];
    let expected_deductions = [
        &format!("deduction_tmp_0 = Memory__FeltExpr__Felt252Expr({})",check_instruction_input),
        "deduction_tmp_1 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_1)) >> const_4) & const_255)",
        "deduction_tmp_1.low().as_m31()",
        "deduction_tmp_2 = RangeCheck8(state[0])",
        "deduction_tmp_3 = (UInt32::from_m31(deduction_tmp_0.get_m31(const_2)) & const_255)",
        "deduction_tmp_3.low().as_m31()",
        "deduction_tmp_4 = RangeCheck8(state[1])",
        "deduction_tmp_5 = (UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) & const_1)",
        "deduction_tmp_5.low().as_m31()",
        "deduction_tmp_6 = ((UInt32::from_m31(deduction_tmp_0.get_m31(const_4)) >> const_2) & const_1)",
        "deduction_tmp_6.low().as_m31()"
    ];

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        &expected_constraints,
        &expected_deductions,
    );
}
