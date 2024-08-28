use super::super::common::*;
use super::decode_instruction::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

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
    expected_state: Vec<u32>,
) {
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

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Felt252IdMemory::new_with_data(vec![(
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
    let lists = registry.get_compiled_air_fn(&air_fn.name());
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
        "Memory([DecodeInstruction_64fce74ff258858e_input]) == [state[23]]",
        "Memory([state[23]]) == zero_extend([\
            state[0], \
            (state[1] + (state[2] * const_128)), \
            state[3], \
            (state[4] + (state[5] * const_32)), \
            state[6], \
            (\
                ((((((state[7] + const_0) + \
                (state[8] * const_8)) + \
                (state[9] * const_16)) + \
                (state[10] * const_32)) + \
                (state[11] * const_64)) + \
                (state[12] * const_128)) + \
                (state[13] * const_256)\
            ), (\
                ((((((((const_0 + (state[14] * const_1)) + \
                (state[15] * const_2)) + \
                (state[16] * const_4)) + \
                (state[17] * const_8)) + \
                (state[18] * const_16)) + \
                (state[19] * const_32)) + \
                (state[20] * const_64)) + \
                (state[21] * const_128)) + \
                (state[22] * const_256)\
            )\
        ])",
    ];
    let expected_deductions = [
        "tmp_0 = Memory(DecodeInstruction_64fce74ff258858e_input)",
        "tmp_1 = Memory(tmp_0)",
        "tmp_1.get_m31(const_0)",
        "tmp_2 = (UInt32::from_m31(tmp_1.get_m31(const_1)) & const_127)",
        "tmp_2.low().as_m31()",
        "tmp_3 = RangeCheck7([state[1]])",
        "tmp_4 = ((UInt32::from_m31(tmp_1.get_m31(const_1)) >> const_7) & const_3)",
        "tmp_4.low().as_m31()",
        "tmp_5 = RangeCheck2([state[2]])",
        "tmp_1.get_m31(const_2)",
        "tmp_6 = (UInt32::from_m31(tmp_1.get_m31(const_3)) & const_31)",
        "tmp_6.low().as_m31()",
        "tmp_7 = RangeCheck5([state[4]])",
        "tmp_8 = ((UInt32::from_m31(tmp_1.get_m31(const_3)) >> const_5) & const_15)",
        "tmp_8.low().as_m31()",
        "tmp_9 = RangeCheck4([state[5]])",
        "tmp_1.get_m31(const_4)",
        "tmp_10 = (UInt32::from_m31(tmp_1.get_m31(const_5)) & const_7)",
        "tmp_10.low().as_m31()",
        "tmp_11 = RangeCheck3([state[7]])",
        "tmp_12 = ((UInt32::from_m31(tmp_1.get_m31(const_5)) >> const_3) & const_1)",
        "tmp_12.low().as_m31()",
        "tmp_13 = ((UInt32::from_m31(tmp_1.get_m31(const_5)) >> const_4) & const_1)",
        "tmp_13.low().as_m31()",
        "tmp_14 = ((UInt32::from_m31(tmp_1.get_m31(const_5)) >> const_5) & const_1)",
        "tmp_14.low().as_m31()",
        "tmp_15 = ((UInt32::from_m31(tmp_1.get_m31(const_5)) >> const_6) & const_1)",
        "tmp_15.low().as_m31()",
        "tmp_16 = ((UInt32::from_m31(tmp_1.get_m31(const_5)) >> const_7) & const_1)",
        "tmp_16.low().as_m31()",
        "tmp_17 = ((UInt32::from_m31(tmp_1.get_m31(const_5)) >> const_8) & const_1)",
        "tmp_17.low().as_m31()",
        "tmp_18 = (UInt32::from_m31(tmp_1.get_m31(const_6)) & const_1)",
        "tmp_18.low().as_m31()",
        "tmp_19 = ((UInt32::from_m31(tmp_1.get_m31(const_6)) >> const_1) & const_1)",
        "tmp_19.low().as_m31()",
        "tmp_20 = ((UInt32::from_m31(tmp_1.get_m31(const_6)) >> const_2) & const_1)",
        "tmp_20.low().as_m31()",
        "tmp_21 = ((UInt32::from_m31(tmp_1.get_m31(const_6)) >> const_3) & const_1)",
        "tmp_21.low().as_m31()",
        "tmp_22 = ((UInt32::from_m31(tmp_1.get_m31(const_6)) >> const_4) & const_1)",
        "tmp_22.low().as_m31()",
        "tmp_23 = ((UInt32::from_m31(tmp_1.get_m31(const_6)) >> const_5) & const_1)",
        "tmp_23.low().as_m31()",
        "tmp_24 = ((UInt32::from_m31(tmp_1.get_m31(const_6)) >> const_6) & const_1)",
        "tmp_24.low().as_m31()",
        "tmp_25 = ((UInt32::from_m31(tmp_1.get_m31(const_6)) >> const_7) & const_1)",
        "tmp_25.low().as_m31()",
        "tmp_26 = ((UInt32::from_m31(tmp_1.get_m31(const_6)) >> const_8) & const_1)",
        "tmp_26.low().as_m31()",
        "tmp_0",
    ];

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        &expected_constraints,
        &expected_deductions,
        vec![
            289, 97, 3, 38, 15, 10, 203, 4, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 1, 0,
        ],
    );
}

#[test]
fn test_all_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [true; 3];
    let is_flag_const = [true; 15];

    let expected_constraints: [&str; 2] = [
        "Memory([DecodeInstruction_bdf8c7acbfd48726_input]) == [state[0]]",
        "Memory([state[0]]) == zero_extend([\
            const_289, \
            const_481, \
            const_38, \
            const_335, \
            const_203, \
            const_84, \
            const_282\
        ])",
    ];
    let expected_deductions: [&str; 3] = [
        "tmp_0 = Memory(DecodeInstruction_bdf8c7acbfd48726_input)",
        "tmp_1 = Memory(tmp_0)",
        "tmp_0",
    ];

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        &expected_constraints,
        &expected_deductions,
        vec![0],
    );
}

#[test]
fn test_some_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [true, false, true];
    let mut is_flag_const = [true; 15];
    is_flag_const[0] = false;
    is_flag_const[2] = false;

    let expected_constraints = [
        "RangeCheck2([state[0]]) == []",
        "RangeCheck5([state[2]]) == []",
        "(state[3] * (const_1 - state[3]))",
        "(state[4] * (const_1 - state[4]))",
        "Memory([DecodeInstruction_116be7744be089d8_input]) == [state[5]]",
        "Memory([state[5]]) == \
            zero_extend([\
                const_289, \
                (const_97 + (state[0] * const_128)), \
                state[1], \
                (state[2] + const_320), \
                const_203, \
                ((const_84 + (state[3] * const_8)) + (state[4] * const_32)), \
                const_282\
            ])",
    ];
    let expected_deductions = [
        "tmp_0 = Memory(DecodeInstruction_116be7744be089d8_input)",
        "tmp_1 = Memory(tmp_0)",
        "tmp_2 = ((UInt32::from_m31(tmp_1.get_m31(const_1)) >> const_7) & const_3)",
        "tmp_2.low().as_m31()",
        "tmp_3 = RangeCheck2([state[0]])",
        "tmp_1.get_m31(const_2)",
        "tmp_4 = (UInt32::from_m31(tmp_1.get_m31(const_3)) & const_31)",
        "tmp_4.low().as_m31()",
        "tmp_5 = RangeCheck5([state[2]])",
        "tmp_6 = ((UInt32::from_m31(tmp_1.get_m31(const_5)) >> const_3) & const_1)",
        "tmp_6.low().as_m31()",
        "tmp_7 = ((UInt32::from_m31(tmp_1.get_m31(const_5)) >> const_5) & const_1)",
        "tmp_7.low().as_m31()",
        "tmp_0",
    ];

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        &expected_constraints,
        &expected_deductions,
        vec![3, 38, 15, 0, 0, 0],
    );
}
