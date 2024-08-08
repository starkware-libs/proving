use super::bitwise::*;

use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;

//Macros
use crate::expr;
use crate::felt252_expr;

#[test]
fn simple_test_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Memory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            felt252_expr!("input_a", 0, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            felt252_expr!("input_b", 1, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            felt252_expr!("input_c", 0, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            felt252_expr!("input_d", 1, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            felt252_expr!("input_d", 1, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air(&bitwise, instance_number);
}

#[test]
#[should_panic(
    expected = "Memory::set() failed for key [M31(553)]- given value != value in memory"
)]
fn simple_failed_test_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Memory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            felt252_expr!("input_a", 0, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            felt252_expr!("input_b", 1, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            felt252_expr!("input_c", 0, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            felt252_expr!("input_d", 0, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            felt252_expr!("input_d", 1, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air(&bitwise, instance_number);
}

#[test]
fn test_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Memory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            felt252_expr!("input_a", 1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            felt252_expr!("input_b", 565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            felt252_expr!("input_c", 34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            felt252_expr!("input_d", 567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            felt252_expr!("input_e", 567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air(&bitwise, instance_number);
}

#[test]
#[should_panic(
    expected = "Memory::set() failed for key [M31(639)]- given value != value in memory"
)]
fn test_failed_or_bitwise_builtin() {
    let instance_number = expr!("instance_number", 27);

    let memory = Memory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            felt252_expr!("input_a", 1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            felt252_expr!("input_b", 565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            felt252_expr!("input_c", 34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            felt252_expr!("input_d", 567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            felt252_expr!("input_e", 567332252375527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air(&bitwise, instance_number);
}

#[test]
#[should_panic(
    expected = "Memory::set() failed for key [M31(553)]- given value != value in memory"
)]
fn test_failed_xor_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Memory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            felt252_expr!("input_a", 1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            felt252_expr!("input_b", 565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            felt252_expr!("input_c", 34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            felt252_expr!("input_d", 567257601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            felt252_expr!("input_e", 567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air(&bitwise, instance_number);
}

#[test]
#[should_panic(
    expected = "Memory::set() failed for key [M31(552)]- given value != value in memory"
)]
fn test_failed_and_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Memory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            felt252_expr!("input_a", 1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            felt252_expr!("input_b", 565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            felt252_expr!("input_c", 34650915127, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            felt252_expr!("input_d", 567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            felt252_expr!("input_e", 567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air(&bitwise, instance_number);
}

#[test]
fn test_big_felt252_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);
    let air_body_vec = [
        "deduction_tmp_0 = BitwiseBuiltin_f4e479ac821af69e_input",
        "Deduction: deduction_tmp_0",
        &format!(
            "Felt252::from_m31_({}) = {}({})",
            "[state[1], state[2], state[3], state[4], state[5], \
            state[6], state[7], state[8], state[9], state[10], \
            state[11], state[12], state[13], state[14], state[15], \
            state[16], state[17], state[18], state[19], state[20], \
            state[21], state[22], state[23], state[24], state[25], \
            state[26], state[27], state[28]]",
            "ReadSmallFelt252_bb908db6c9837328",
            "((const_500 + (state[0] * const_5)) + const_0)"
        ),
        &format!(
            "Felt252::from_m31_({}) = {}({})",
            "[state[29], state[30], state[31], \
            state[32], state[33], state[34], state[35], state[36], \
            state[37], state[38], state[39], state[40], state[41], \
            state[42], state[43], state[44], state[45], state[46], \
            state[47], state[48], state[49], state[50], state[51], \
            state[52], state[53], state[54], state[55], state[56]]",
            "ReadSmallFelt252_bb908db6c9837328",
            "((const_500 + (state[0] * const_5)) + const_1)",
        ),
        "state[57] = BitwiseXor_e991911c19957b24([state[1], state[29]])",
        "constraint_tmp_8 = (const_1073741824 * ((state[1] + state[29]) - state[57]))",
        "state[58] = BitwiseXor_e991911c19957b24([state[2], state[30]])",
        "constraint_tmp_11 = (const_1073741824 * ((state[2] + state[30]) - state[58]))",
        "state[59] = BitwiseXor_e991911c19957b24([state[3], state[31]])",
        "constraint_tmp_14 = (const_1073741824 * ((state[3] + state[31]) - state[59]))",
        "state[60] = BitwiseXor_e991911c19957b24([state[4], state[32]])",
        "constraint_tmp_17 = (const_1073741824 * ((state[4] + state[32]) - state[60]))",
        "state[61] = BitwiseXor_e991911c19957b24([state[5], state[33]])",
        "constraint_tmp_20 = (const_1073741824 * ((state[5] + state[33]) - state[61]))",
        "state[62] = BitwiseXor_e991911c19957b24([state[6], state[34]])",
        "constraint_tmp_23 = (const_1073741824 * ((state[6] + state[34]) - state[62]))",
        "state[63] = BitwiseXor_e991911c19957b24([state[7], state[35]])",
        "constraint_tmp_26 = (const_1073741824 * ((state[7] + state[35]) - state[63]))",
        "state[64] = BitwiseXor_e991911c19957b24([state[8], state[36]])",
        "constraint_tmp_29 = (const_1073741824 * ((state[8] + state[36]) - state[64]))",
        "state[65] = BitwiseXor_e991911c19957b24([state[9], state[37]])",
        "constraint_tmp_32 = (const_1073741824 * ((state[9] + state[37]) - state[65]))",
        "state[66] = BitwiseXor_e991911c19957b24([state[10], state[38]])",
        "constraint_tmp_35 = (const_1073741824 * ((state[10] + state[38]) - state[66]))",
        "state[67] = BitwiseXor_e991911c19957b24([state[11], state[39]])",
        "constraint_tmp_38 = (const_1073741824 * ((state[11] + state[39]) - state[67]))",
        "state[68] = BitwiseXor_e991911c19957b24([state[12], state[40]])",
        "constraint_tmp_41 = (const_1073741824 * ((state[12] + state[40]) - state[68]))",
        "state[69] = BitwiseXor_e991911c19957b24([state[13], state[41]])",
        "constraint_tmp_44 = (const_1073741824 * ((state[13] + state[41]) - state[69]))",
        "state[70] = BitwiseXor_e991911c19957b24([state[14], state[42]])",
        "constraint_tmp_47 = (const_1073741824 * ((state[14] + state[42]) - state[70]))",
        "state[71] = BitwiseXor_e991911c19957b24([state[15], state[43]])",
        "constraint_tmp_50 = (const_1073741824 * ((state[15] + state[43]) - state[71]))",
        "state[72] = BitwiseXor_e991911c19957b24([state[16], state[44]])",
        "constraint_tmp_53 = (const_1073741824 * ((state[16] + state[44]) - state[72]))",
        "state[73] = BitwiseXor_e991911c19957b24([state[17], state[45]])",
        "constraint_tmp_56 = (const_1073741824 * ((state[17] + state[45]) - state[73]))",
        "state[74] = BitwiseXor_e991911c19957b24([state[18], state[46]])",
        "constraint_tmp_59 = (const_1073741824 * ((state[18] + state[46]) - state[74]))",
        "state[75] = BitwiseXor_e991911c19957b24([state[19], state[47]])",
        "constraint_tmp_62 = (const_1073741824 * ((state[19] + state[47]) - state[75]))",
        "state[76] = BitwiseXor_e991911c19957b24([state[20], state[48]])",
        "constraint_tmp_65 = (const_1073741824 * ((state[20] + state[48]) - state[76]))",
        "state[77] = BitwiseXor_e991911c19957b24([state[21], state[49]])",
        "constraint_tmp_68 = (const_1073741824 * ((state[21] + state[49]) - state[77]))",
        "state[78] = BitwiseXor_e991911c19957b24([state[22], state[50]])",
        "constraint_tmp_71 = (const_1073741824 * ((state[22] + state[50]) - state[78]))",
        "state[79] = BitwiseXor_e991911c19957b24([state[23], state[51]])",
        "constraint_tmp_74 = (const_1073741824 * ((state[23] + state[51]) - state[79]))",
        "state[80] = BitwiseXor_e991911c19957b24([state[24], state[52]])",
        "constraint_tmp_77 = (const_1073741824 * ((state[24] + state[52]) - state[80]))",
        "state[81] = BitwiseXor_e991911c19957b24([state[25], state[53]])",
        "constraint_tmp_80 = (const_1073741824 * ((state[25] + state[53]) - state[81]))",
        "state[82] = BitwiseXor_e991911c19957b24([state[26], state[54]])",
        "constraint_tmp_83 = (const_1073741824 * ((state[26] + state[54]) - state[82]))",
        "state[83] = BitwiseXor_e991911c19957b24([state[27], state[55]])",
        "constraint_tmp_86 = (const_1073741824 * ((state[27] + state[55]) - state[83]))",
        "state[84] = BitwiseXor_e991911c19957b24([state[28], state[56]])",
        "constraint_tmp_89 = (const_1073741824 * ((state[28] + state[56]) - state[84]))",
        &format!(
            "Memory_81f75475e4cf34d6({}) == {}",
            "[((const_500 + (state[0] * const_5)) + const_2)]",
            "[constraint_tmp_8, constraint_tmp_11, constraint_tmp_14, constraint_tmp_17, \
            constraint_tmp_20, constraint_tmp_23, constraint_tmp_26, constraint_tmp_29, \
            constraint_tmp_32, constraint_tmp_35, constraint_tmp_38, constraint_tmp_41, \
            constraint_tmp_44, constraint_tmp_47, constraint_tmp_50, constraint_tmp_53, \
            constraint_tmp_56, constraint_tmp_59, constraint_tmp_62, constraint_tmp_65, \
            constraint_tmp_68, constraint_tmp_71, constraint_tmp_74, constraint_tmp_77, \
            constraint_tmp_80, constraint_tmp_83, constraint_tmp_86, constraint_tmp_89]"
        ),
        &format!(
            "Memory_81f75475e4cf34d6({}) == {}",
            "[((const_500 + (state[0] * const_5)) + const_3)]",
            "[state[57], state[58], state[59], state[60], state[61], \
            state[62], state[63], state[64], state[65], state[66], \
            state[67], state[68], state[69], state[70], state[71], \
            state[72], state[73], state[74], state[75], state[76], \
            state[77], state[78], state[79], state[80], state[81], \
            state[82], state[83], state[84]]"
        ),
        &format!(
            "Memory_81f75475e4cf34d6({}) == {}",
            "[((const_500 + (state[0] * const_5)) + const_4)]",
            "[(constraint_tmp_8 + state[57]), (constraint_tmp_11 + state[58]), \
            (constraint_tmp_14 + state[59]), (constraint_tmp_17 + state[60]), \
            (constraint_tmp_20 + state[61]), (constraint_tmp_23 + state[62]), \
            (constraint_tmp_26 + state[63]), (constraint_tmp_29 + state[64]), \
            (constraint_tmp_32 + state[65]), (constraint_tmp_35 + state[66]), \
            (constraint_tmp_38 + state[67]), (constraint_tmp_41 + state[68]), \
            (constraint_tmp_44 + state[69]), (constraint_tmp_47 + state[70]), \
            (constraint_tmp_50 + state[71]), (constraint_tmp_53 + state[72]), \
            (constraint_tmp_56 + state[73]), (constraint_tmp_59 + state[74]), \
            (constraint_tmp_62 + state[75]), (constraint_tmp_65 + state[76]), \
            (constraint_tmp_68 + state[77]), (constraint_tmp_71 + state[78]), \
            (constraint_tmp_74 + state[79]), (constraint_tmp_77 + state[80]), \
            (constraint_tmp_80 + state[81]), (constraint_tmp_83 + state[82]), \
            (constraint_tmp_86 + state[83]), (constraint_tmp_89 + state[84])]"
        ),
    ];
    let memory = Memory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            felt252_expr!("input_a", 467968798486, 18694984798),
        ),
        (
            get_addr(instance_number.clone(), 1),
            felt252_expr!("input_b", 3468798969565, 4869486468496),
        ),
        (
            get_addr(instance_number.clone(), 2),
            felt252_expr!("input_c", 157370615316, 18253796368),
        ),
        (
            get_addr(instance_number.clone(), 3),
            felt252_expr!("input_d", 3622026537419, 4851673860558),
        ),
        (
            get_addr(instance_number.clone(), 4),
            felt252_expr!("input_e", 3779397152735, 4869927656926),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air(&bitwise, instance_number);

    // Check air body
    let entry = registry.get_air_fn_entry(&bitwise);
    assert_eq!(
        entry
            .air_body
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        air_body_vec
    );
}
