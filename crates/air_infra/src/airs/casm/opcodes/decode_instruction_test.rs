use super::super::common::*;
use super::decode_instruction::*;

use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
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
    let const_offsets = offsets
        .iter()
        .enumerate()
        .map(|(i, &off)| if is_offset_const[i] { Some(off) } else { None })
        .collect::<Vec<Option<i16>>>()
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
    let mut expected_state: Vec<_> = is_offset_const
        .into_iter()
        .enumerate()
        .filter(|(_, is_const)| !is_const)
        .flat_map(|(i, _)| match i {
            0 => vec![offsets_u16[0] & 0x1FF, offsets_u16[0] >> 9],
            1 => vec![
                (offsets_u16[1] & 0x3),
                (offsets_u16[1] >> 2) & 0x1FF,
                offsets_u16[1] >> 11,
            ],
            2 => vec![
                (offsets_u16[2] & 0xF),
                (offsets_u16[2] >> 4) & 0x1FF,
                offsets_u16[2] >> 13,
            ],
            _ => unreachable!(),
        })
        .collect();

    is_flag_const.iter().enumerate().for_each(|(i, is_const)| {
        if !is_const {
            expected_state.push(flags[i] as u16)
        }
    });

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Memory::<FeltExpr, Felt252Expr>::new_with_data(vec![(
        pc.clone(),
        felt252_expr!(
            "instruction",
            assemble_instruction(offsets[0], offsets[1], offsets[2], flags) as u128,
            0
        ),
    )]);

    // Run and check output
    let air_fn = DecodeInstruction {
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

    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
    );

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

    let check_instruction_input = "DecodeInstruction_64fce74ff258858e_input";
    let expected_constraints = [
        "RangeCheck7([state[1]]) == []",
        "RangeCheck2([state[2]]) == []",
        "RangeCheck5([state[4]]) == []",
        "RangeCheck4([state[5]]) == []",
        "RangeCheck3([state[7]]) == []",
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
        "(state[21] * (const_1 - state[21]))",
        "(state[22] * (const_1 - state[22]))",
        &format!("Memory_59f18133215d0936([{}]) == zero_extend([{}, {}, {}, {}, {}, {}, {}])",
            check_instruction_input,
            "state[0]",
            "(state[1] + (state[2] * const_128))",
            "state[3]",
            "(state[4] + (state[5] * const_32))",
            "state[6]",
            "(((((((state[7] + const_0) + (state[8] * const_8)) + (state[9] * const_16)) + (state[10] * const_32)) + (state[11] * const_64)) + (state[12] * const_128)) + (state[13] * const_256))",
            "(((((((((const_0 + (state[14] * const_1)) + (state[15] * const_2)) + (state[16] * const_4)) + (state[17] * const_8)) + (state[18] * const_16)) + (state[19] * const_32)) + (state[20] * const_64)) + (state[21] * const_128)) + (state[22] * const_256))",
        ),
    ];
    let expected_deductions = [
        &format!(
            "tmp_0 = Memory_59f18133215d0936({})",
            check_instruction_input
        ),
        "tmp_0.get_m31(const_0)",
        "tmp_1 = (UInt32::from_m31(tmp_0.get_m31(const_1)) & const_127)",
        "tmp_1.low().as_m31()",
        "tmp_2 = RangeCheck7([state[1]])",
        "tmp_3 = ((UInt32::from_m31(tmp_0.get_m31(const_1)) >> const_7) & const_3)",
        "tmp_3.low().as_m31()",
        "tmp_4 = RangeCheck2([state[2]])",
        "tmp_0.get_m31(const_2)",
        "tmp_5 = (UInt32::from_m31(tmp_0.get_m31(const_3)) & const_31)",
        "tmp_5.low().as_m31()",
        "tmp_6 = RangeCheck5([state[4]])",
        "tmp_7 = ((UInt32::from_m31(tmp_0.get_m31(const_3)) >> const_5) & const_15)",
        "tmp_7.low().as_m31()",
        "tmp_8 = RangeCheck4([state[5]])",
        "tmp_0.get_m31(const_4)",
        "tmp_9 = (UInt32::from_m31(tmp_0.get_m31(const_5)) & const_7)",
        "tmp_9.low().as_m31()",
        "tmp_10 = RangeCheck3([state[7]])",
        "tmp_11 = ((UInt32::from_m31(tmp_0.get_m31(const_5)) >> const_3) & const_1)",
        "tmp_11.low().as_m31()",
        "tmp_12 = ((UInt32::from_m31(tmp_0.get_m31(const_5)) >> const_4) & const_1)",
        "tmp_12.low().as_m31()",
        "tmp_13 = ((UInt32::from_m31(tmp_0.get_m31(const_5)) >> const_5) & const_1)",
        "tmp_13.low().as_m31()",
        "tmp_14 = ((UInt32::from_m31(tmp_0.get_m31(const_5)) >> const_6) & const_1)",
        "tmp_14.low().as_m31()",
        "tmp_15 = ((UInt32::from_m31(tmp_0.get_m31(const_5)) >> const_7) & const_1)",
        "tmp_15.low().as_m31()",
        "tmp_16 = ((UInt32::from_m31(tmp_0.get_m31(const_5)) >> const_8) & const_1)",
        "tmp_16.low().as_m31()",
        "tmp_17 = (UInt32::from_m31(tmp_0.get_m31(const_6)) & const_1)",
        "tmp_17.low().as_m31()",
        "tmp_18 = ((UInt32::from_m31(tmp_0.get_m31(const_6)) >> const_1) & const_1)",
        "tmp_18.low().as_m31()",
        "tmp_19 = ((UInt32::from_m31(tmp_0.get_m31(const_6)) >> const_2) & const_1)",
        "tmp_19.low().as_m31()",
        "tmp_20 = ((UInt32::from_m31(tmp_0.get_m31(const_6)) >> const_3) & const_1)",
        "tmp_20.low().as_m31()",
        "tmp_21 = ((UInt32::from_m31(tmp_0.get_m31(const_6)) >> const_4) & const_1)",
        "tmp_21.low().as_m31()",
        "tmp_22 = ((UInt32::from_m31(tmp_0.get_m31(const_6)) >> const_5) & const_1)",
        "tmp_22.low().as_m31()",
        "tmp_23 = ((UInt32::from_m31(tmp_0.get_m31(const_6)) >> const_6) & const_1)",
        "tmp_23.low().as_m31()",
        "tmp_24 = ((UInt32::from_m31(tmp_0.get_m31(const_6)) >> const_7) & const_1)",
        "tmp_24.low().as_m31()",
        "tmp_25 = ((UInt32::from_m31(tmp_0.get_m31(const_6)) >> const_8) & const_1)",
        "tmp_25.low().as_m31()",
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

    let check_instruction_input = "DecodeInstruction_bdf8c7acbfd48726_input";
    let expected_constraints: [&str; 1] = [&format!(
        "Memory_59f18133215d0936([{}]) == zero_extend([{}, {}, {}, {}, {}, {}, {}])",
        check_instruction_input,
        "const_289",
        "const_481",
        "const_38",
        "const_335",
        "const_203",
        "const_84",
        "const_282",
    )];
    let expected_deductions: [&str; 1] = [&format!(
        "tmp_0 = Memory_59f18133215d0936({})",
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

    let check_instruction_input = "DecodeInstruction_116be7744be089d8_input";
    let expected_constraints = [
        "RangeCheck2([state[0]]) == []",
        "RangeCheck5([state[2]]) == []",
        "(state[3] * (const_1 - state[3]))",
        "(state[4] * (const_1 - state[4]))",
        &format!(
            "Memory_59f18133215d0936([{}]) == zero_extend([{}, {}, {}, {}, {}, {}, {}])",
            check_instruction_input,
            "const_289",
            "(const_97 + (state[0] * const_128))",
            "state[1]",
            "(state[2] + const_320)",
            "const_203",
            "((const_84 + (state[3] * const_8)) + (state[4] * const_32))",
            "const_282",
        ),
    ];
    let expected_deductions = [
        &format!(
            "tmp_0 = Memory_59f18133215d0936({})",
            check_instruction_input
        ),
        "tmp_1 = ((UInt32::from_m31(tmp_0.get_m31(const_1)) >> const_7) & const_3)",
        "tmp_1.low().as_m31()",
        "tmp_2 = RangeCheck2([state[0]])",
        "tmp_0.get_m31(const_2)",
        "tmp_3 = (UInt32::from_m31(tmp_0.get_m31(const_3)) & const_31)",
        "tmp_3.low().as_m31()",
        "tmp_4 = RangeCheck5([state[2]])",
        "tmp_5 = ((UInt32::from_m31(tmp_0.get_m31(const_5)) >> const_3) & const_1)",
        "tmp_5.low().as_m31()",
        "tmp_6 = ((UInt32::from_m31(tmp_0.get_m31(const_5)) >> const_5) & const_1)",
        "tmp_6.low().as_m31()",
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
