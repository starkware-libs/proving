use super::bitwise::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;

//Macros
use crate::expr;
use crate::felt252_expr;

#[test]
fn simple_test_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
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
#[should_panic(expected = "given value != value in memory")]
fn simple_failed_test_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
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

    let memory = Felt252IdMemory::new_with_data(vec![
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
#[should_panic(expected = "given value != value in memory")]
fn test_failed_or_bitwise_builtin() {
    let instance_number = expr!("instance_number", 27);

    let memory = Felt252IdMemory::new_with_data(vec![
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
#[should_panic(expected = "given value != value in memory")]
fn test_failed_xor_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
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
#[should_panic(expected = "given value != value in memory")]
fn test_failed_and_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
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
        "tmp_0 = BitwiseBuiltin_efb2afede285371d_input",
        "Deduction: tmp_0",
        &format!(
            "Felt252::from_limbs({}) = {}({})",
            "[state[2], state[3], state[4], state[5], state[6], \
            state[7], state[8], state[9], state[10], state[11], \
            state[12], state[13], state[14], state[15], state[16], \
            state[17], state[18], state[19], state[20], state[21], \
            state[22], state[23], state[24], state[25], state[26], \
            state[27], state[28], state[29]]",
            "ReadPositive_57b5981b206c7ed",
            "((const_500 + (state[0] * const_5)) + const_0)"
        ),
        &format!(
            "Felt252::from_limbs({}) = {}({})",
            "[state[31], state[32], state[33], \
            state[34], state[35], state[36], state[37], state[38], \
            state[39], state[40], state[41], state[42], state[43], \
            state[44], state[45], state[46], state[47], state[48], \
            state[49], state[50], state[51], state[52], state[53], \
            state[54], state[55], state[56], state[57], state[58]]",
            "ReadPositive_57b5981b206c7ed",
            "((const_500 + (state[0] * const_5)) + const_1)",
        ),
        "state[59] = BitwiseXor_36334f1766571820([state[2], state[31]])",
        "tmp_11 = (const_1073741824 * ((state[2] + state[31]) - state[59]))",
        "state[60] = BitwiseXor_36334f1766571820([state[3], state[32]])",
        "tmp_14 = (const_1073741824 * ((state[3] + state[32]) - state[60]))",
        "state[61] = BitwiseXor_36334f1766571820([state[4], state[33]])",
        "tmp_17 = (const_1073741824 * ((state[4] + state[33]) - state[61]))",
        "state[62] = BitwiseXor_36334f1766571820([state[5], state[34]])",
        "tmp_20 = (const_1073741824 * ((state[5] + state[34]) - state[62]))",
        "state[63] = BitwiseXor_36334f1766571820([state[6], state[35]])",
        "tmp_23 = (const_1073741824 * ((state[6] + state[35]) - state[63]))",
        "state[64] = BitwiseXor_36334f1766571820([state[7], state[36]])",
        "tmp_26 = (const_1073741824 * ((state[7] + state[36]) - state[64]))",
        "state[65] = BitwiseXor_36334f1766571820([state[8], state[37]])",
        "tmp_29 = (const_1073741824 * ((state[8] + state[37]) - state[65]))",
        "state[66] = BitwiseXor_36334f1766571820([state[9], state[38]])",
        "tmp_32 = (const_1073741824 * ((state[9] + state[38]) - state[66]))",
        "state[67] = BitwiseXor_36334f1766571820([state[10], state[39]])",
        "tmp_35 = (const_1073741824 * ((state[10] + state[39]) - state[67]))",
        "state[68] = BitwiseXor_36334f1766571820([state[11], state[40]])",
        "tmp_38 = (const_1073741824 * ((state[11] + state[40]) - state[68]))",
        "state[69] = BitwiseXor_36334f1766571820([state[12], state[41]])",
        "tmp_41 = (const_1073741824 * ((state[12] + state[41]) - state[69]))",
        "state[70] = BitwiseXor_36334f1766571820([state[13], state[42]])",
        "tmp_44 = (const_1073741824 * ((state[13] + state[42]) - state[70]))",
        "state[71] = BitwiseXor_36334f1766571820([state[14], state[43]])",
        "tmp_47 = (const_1073741824 * ((state[14] + state[43]) - state[71]))",
        "state[72] = BitwiseXor_36334f1766571820([state[15], state[44]])",
        "tmp_50 = (const_1073741824 * ((state[15] + state[44]) - state[72]))",
        "state[73] = BitwiseXor_36334f1766571820([state[16], state[45]])",
        "tmp_53 = (const_1073741824 * ((state[16] + state[45]) - state[73]))",
        "state[74] = BitwiseXor_36334f1766571820([state[17], state[46]])",
        "tmp_56 = (const_1073741824 * ((state[17] + state[46]) - state[74]))",
        "state[75] = BitwiseXor_36334f1766571820([state[18], state[47]])",
        "tmp_59 = (const_1073741824 * ((state[18] + state[47]) - state[75]))",
        "state[76] = BitwiseXor_36334f1766571820([state[19], state[48]])",
        "tmp_62 = (const_1073741824 * ((state[19] + state[48]) - state[76]))",
        "state[77] = BitwiseXor_36334f1766571820([state[20], state[49]])",
        "tmp_65 = (const_1073741824 * ((state[20] + state[49]) - state[77]))",
        "state[78] = BitwiseXor_36334f1766571820([state[21], state[50]])",
        "tmp_68 = (const_1073741824 * ((state[21] + state[50]) - state[78]))",
        "state[79] = BitwiseXor_36334f1766571820([state[22], state[51]])",
        "tmp_71 = (const_1073741824 * ((state[22] + state[51]) - state[79]))",
        "state[80] = BitwiseXor_36334f1766571820([state[23], state[52]])",
        "tmp_74 = (const_1073741824 * ((state[23] + state[52]) - state[80]))",
        "state[81] = BitwiseXor_36334f1766571820([state[24], state[53]])",
        "tmp_77 = (const_1073741824 * ((state[24] + state[53]) - state[81]))",
        "state[82] = BitwiseXor_36334f1766571820([state[25], state[54]])",
        "tmp_80 = (const_1073741824 * ((state[25] + state[54]) - state[82]))",
        "state[83] = BitwiseXor_36334f1766571820([state[26], state[55]])",
        "tmp_83 = (const_1073741824 * ((state[26] + state[55]) - state[83]))",
        "state[84] = BitwiseXor_36334f1766571820([state[27], state[56]])",
        "tmp_86 = (const_1073741824 * ((state[27] + state[56]) - state[84]))",
        "state[85] = BitwiseXor_36334f1766571820([state[28], state[57]])",
        "tmp_89 = (const_1073741824 * ((state[28] + state[57]) - state[85]))",
        "state[86] = BitwiseXor_36334f1766571820([state[29], state[58]])",
        "tmp_92 = (const_1073741824 * ((state[29] + state[58]) - state[86]))",
        &format!(
            "() = MemVerify_611491d0b573efe1(({}, Felt252::from_limbs({})))",
            "((const_500 + (state[0] * const_5)) + const_2)",
            "[tmp_11, tmp_14, tmp_17, tmp_20, \
            tmp_23, tmp_26, tmp_29, tmp_32, \
            tmp_35, tmp_38, tmp_41, tmp_44, \
            tmp_47, tmp_50, tmp_53, tmp_56, \
            tmp_59, tmp_62, tmp_65, tmp_68, \
            tmp_71, tmp_74, tmp_77, tmp_80, \
            tmp_83, tmp_86, tmp_89, tmp_92]"
        ),
        &format!(
            "() = MemVerify_611491d0b573efe1(({}, Felt252::from_limbs({})))",
            "((const_500 + (state[0] * const_5)) + const_3)",
            "[state[59], state[60], state[61], state[62], state[63], \
            state[64], state[65], state[66], state[67], state[68], \
            state[69], state[70], state[71], state[72], state[73], \
            state[74], state[75], state[76], state[77], state[78], \
            state[79], state[80], state[81], state[82], state[83], \
            state[84], state[85], state[86]]"
        ),
        &format!(
            "() = MemVerify_611491d0b573efe1(({}, Felt252::from_limbs({})))",
            "((const_500 + (state[0] * const_5)) + const_4)",
            "[(tmp_11 + state[59]), (tmp_14 + state[60]), \
            (tmp_17 + state[61]), (tmp_20 + state[62]), \
            (tmp_23 + state[63]), (tmp_26 + state[64]), \
            (tmp_29 + state[65]), (tmp_32 + state[66]), \
            (tmp_35 + state[67]), (tmp_38 + state[68]), \
            (tmp_41 + state[69]), (tmp_44 + state[70]), \
            (tmp_47 + state[71]), (tmp_50 + state[72]), \
            (tmp_53 + state[73]), (tmp_56 + state[74]), \
            (tmp_59 + state[75]), (tmp_62 + state[76]), \
            (tmp_65 + state[77]), (tmp_68 + state[78]), \
            (tmp_71 + state[79]), (tmp_74 + state[80]), \
            (tmp_77 + state[81]), (tmp_80 + state[82]), \
            (tmp_83 + state[83]), (tmp_86 + state[84]), \
            (tmp_89 + state[85]), (tmp_92 + state[86])]"
        ),
    ];
    let memory = Felt252IdMemory::new_with_data(vec![
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
