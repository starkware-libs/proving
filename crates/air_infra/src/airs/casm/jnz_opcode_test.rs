use super::common::*;
use super::jnz_opcode::*;

use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::core::variables::*;
use crate::core::Felt;

use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

fn build_and_test(
    [is_taken, flag_dst_base_fp, flag_ap_update_add_1]: [bool; 3],
    offset_dst: i16,
    dst_value: Felt252Expr,
    op1_value: u32,
    check_instruction_name: &str,
    rest_of_air_body: Vec<&str>,
) {
    let [pc_value, ap_value, fp_value] = [50, 200, 150];
    let [pc, ap, fp] = [
        expr!("pc", pc_value),
        expr!("ap", ap_value),
        expr!("fp", fp_value),
    ];

    let mut jnz_opcode = JnzOpcode {
        is_taken,
        flag_dst_base_fp,
        flag_ap_update_add_1,
        memory: Memory::default(),
    };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!(
            "op",
            assemble_jnz(offset_dst, &jnz_opcode.get_flags()) as u128,
            0
        ),
    )];

    memory_values.push((
        const_expr!(pc_value + 1),
        felt252_expr!("op1_imm", op1_value as u128, 0),
    ));

    if flag_dst_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset_dst) as u32),
            dst_value.clone(),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset_dst) as u32),
            dst_value.clone(),
        ));
    }
    let memory = Memory::new_with_data(memory_values);

    jnz_opcode.init_memory(&memory);

    // Run air function
    let registry = AirFnRegistry::new(&jnz_opcode);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&jnz_opcode, [pc, ap.clone(), fp.clone()]);

    // Check output
    if is_taken {
        assert_eq!(next_pc.calc(), (pc_value + op1_value).to_string());
    } else {
        assert_eq!(next_pc.calc(), (pc_value + 2).to_string());
    }

    if flag_ap_update_add_1 {
        assert_eq!(next_ap.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_ap.calc(), ap_value.to_string());
    }

    assert_eq!(next_fp.calc(), fp_value.to_string());

    // Construct expected_state
    let mut expected_state = vec![pc_value, ap_value, fp_value];
    expected_state.push((offset_as_u16(offset_dst) & 0x1FF) as u32);
    expected_state.push((offset_as_u16(offset_dst) >> 9) as u32);

    let dst_vec = dst_value
        .to_values()
        .iter()
        .map(|x| x.0)
        .collect::<Vec<_>>();
    expected_state.append(dst_vec.clone().as_mut());

    if is_taken {
        let sum_dst = dst_vec.iter().sum::<u32>();

        let dst_minus_p_sos = dst_vec
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                if P_FELTS[i] == 0 {
                    x
                } else {
                    ((x as i64 - P_FELTS[i] as i64) * (x as i64 - P_FELTS[i] as i64)) as u32
                }
            })
            .sum::<u32>();

        expected_state.extend([
            (Felt::from_u32_unchecked(1) / Felt::from_u32_unchecked(sum_dst)).0,
            (Felt::from_u32_unchecked(1) / Felt::from_u32_unchecked(dst_minus_p_sos)).0,
            op1_value,
        ]);
    }

    // Check state
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    // Construct expected_air_body
    let check_instruction_call = format!(
        "([(((state[3] + (state[4] * const_512)) + const_0) - const_32768), const_2147483646, const_1], {}) = {}(state[0])",
        jnz_opcode.get_flags(),
        check_instruction_name,
    );

    let expected_air_body_array = [
        &format!(
            "deduction_tmp_0 = [{name}_input[0], {name}_input[1], {name}_input[2]]",
            name = jnz_opcode.name()
        ),
        "Deduction: deduction_tmp_0[0]", // state[0] = pc
        "Deduction: deduction_tmp_0[1]", // state[1] = ap
        "Deduction: deduction_tmp_0[2]", // state[2] = fp
        &check_instruction_call,
    ];
    let mut expected_air_body = expected_air_body_array.to_vec();
    expected_air_body.extend(rest_of_air_body.iter());

    // Check air_body
    let air_body = registry.get_air_fn_entry(&jnz_opcode).air_body;

    assert_eq!(
        air_body
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        expected_air_body
    );
}

#[test]
fn test_not_taken_zero_match_base_ap() {
    build_and_test(
        [false, false, false],
        -13,
        felt252_expr!("dst", 0, 0),
        15,
        "CheckInstruction_30add879b650f090",
        vec![
            &format!("{} = {}({})",
                "Felt252::from_m31_([state[5], state[6], state[7], state[8], state[9], state[10], \
                    state[11], state[12], state[13], state[14], state[15], state[16], state[17], \
                    state[18], state[19], state[20], state[21], state[22], state[23], state[24], \
                    state[25], state[26], state[27], state[28], state[29], state[30], state[31], state[32]])",
                "ReadSmallFelt252_bb908db6c9837328",
                "(state[1] + (((state[3] + (state[4] * const_512)) + const_0) - const_32768))"),

            "Constraint: ((((((((((((((((((((((((((((const_0 + state[5]) + state[6]) + state[7]) + \
            state[8]) + state[9]) + state[10]) + state[11]) + state[12]) + state[13]) + state[14]) + \
            state[15]) + state[16]) + state[17]) + state[18]) + state[19]) + state[20]) + state[21]) + \
            state[22]) + state[23]) + state[24]) + state[25]) + state[26]) + state[27]) + state[28]) + \
            state[29]) + state[30]) + state[31]) + state[32])"
            ],
    );
}

#[test]
fn test_taken_match_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        felt252_expr!("dst", 123, 456),
        15,
        "CheckInstruction_30add879b650f090",
        vec![
            &format!("{} = {}({})",
                "Felt252::from_m31_([state[5], state[6], state[7], state[8], state[9], state[10], \
                    state[11], state[12], state[13], state[14], state[15], state[16], state[17], \
                    state[18], state[19], state[20], state[21], state[22], state[23], state[24], \
                    state[25], state[26], state[27], state[28], state[29], state[30], state[31], state[32]])",
                "ReadSmallFelt252_bb908db6c9837328",
                "(state[1] + (((state[3] + (state[4] * const_512)) + const_0) - const_32768))"),

            "Deduction: (const_1 // ((((((((((((((((((((((((((((const_0 + state[5]) + state[6]) + \
            state[7]) + state[8]) + state[9]) + state[10]) + state[11]) + state[12]) + state[13]) + \
            state[14]) + state[15]) + state[16]) + state[17]) + state[18]) + state[19]) + state[20]) + \
            state[21]) + state[22]) + state[23]) + state[24]) + state[25]) + state[26]) + state[27]) + \
            state[28]) + state[29]) + state[30]) + state[31]) + state[32]))",

            "Constraint: ((((((((((((((((((((((((((((((const_0 + state[5]) + state[6]) + state[7]) + \
            state[8]) + state[9]) + state[10]) + state[11]) + state[12]) + state[13]) + state[14]) + \
            state[15]) + state[16]) + state[17]) + state[18]) + state[19]) + state[20]) + state[21]) + \
            state[22]) + state[23]) + state[24]) + state[25]) + state[26]) + state[27]) + state[28]) + \
            state[29]) + state[30]) + state[31]) + state[32]) * state[33]) - const_1)",

            "constraint_tmp_9 = (state[5] - const_1)",
            "constraint_tmp_10 = (state[26] - const_136)",
            "constraint_tmp_11 = (state[32] - const_256)",

            "Deduction: (const_1 // ((((((((((((((((((((((((((((const_0 + \
                (constraint_tmp_9 * constraint_tmp_9)) + state[6]) + state[7]) + state[8]) + state[9]) + \
                state[10]) + state[11]) + state[12]) + state[13]) + state[14]) + state[15]) + state[16]) + \
                state[17]) + state[18]) + state[19]) + state[20]) + state[21]) + state[22]) + state[23]) + \
                state[24]) + state[25]) + (constraint_tmp_10 * constraint_tmp_10)) + state[27]) + state[28]) + \
                state[29]) + state[30]) + state[31]) + (constraint_tmp_11 * constraint_tmp_11)))",

            "Constraint: ((((((((((((((((((((((((((((((const_0 + (constraint_tmp_9 * constraint_tmp_9)) + \
            state[6]) + state[7]) + state[8]) + state[9]) + state[10]) + state[11]) + state[12]) + \
            state[13]) + state[14]) + state[15]) + state[16]) + state[17]) + state[18]) + state[19]) + \
            state[20]) + state[21]) + state[22]) + state[23]) + state[24]) + state[25]) + \
            (constraint_tmp_10 * constraint_tmp_10)) + state[27]) + state[28]) + state[29]) + state[30]) + \
            state[31]) + (constraint_tmp_11 * constraint_tmp_11)) * state[34]) - const_1)",

            &format!("{} = {}({})",
                "Felt252::from_m31_(zero_extend([state[35]]))",
                "ReadSmallFelt252_cc824bd2f61c6ef6",
                "(state[0] + const_1)")

            ],
    );
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_taken_zero_mismatch_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        felt252_expr!("dst", 0, 0),
        15,
        "CheckInstruction_",
        vec![],
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evalutate to 0)")]
fn test_not_taken_mismatch_base_ap() {
    build_and_test(
        [false, false, false],
        -13,
        felt252_expr!("dst", 123, 4567),
        15,
        "CheckInstruction_",
        vec![],
    );
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_taken_p_mismatch_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        felt252_expr!("dst", 1, 17 * u128::pow(2, 64) + u128::pow(2, 123)),
        15,
        "CheckInstruction_",
        vec![],
    );
}

pub fn assemble_jnz(offset_dst: i16, flags: &Flags) -> u64 {
    assemble_instruction(offset_dst, -1, 1, flags.clone().into())
}
