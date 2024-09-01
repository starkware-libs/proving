use super::bitwise::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;

//Macros
use crate::const_felt252_expr;
use crate::expr;

#[test]
fn simple_test_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(1, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn simple_failed_test_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(1, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(1, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
fn test_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_or_bitwise_builtin() {
    let instance_number = expr!("instance_number", 27);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(567332252375527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 27);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_xor_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(34650915137, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(567257601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_failed_and_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(1546546796877, 0),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(565820356494787, 0),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(34650915127, 0),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(567297601461390, 0),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(567332252376527, 0),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);
}

#[test]
fn test_big_felt252_bitwise_builtin() {
    let instance_number = expr!("instance_number", 10);
    let air_body_vec = [
        "tmp_0 = ()",
        "tmp_1 = external(Seq)",
        "Felt252::from_limbs([\
            state[1], state[2], state[3], state[4], state[5], state[6], state[7], state[8], \
            state[9], state[10], state[11], state[12], state[13], state[14], state[15], \
            state[16], state[17], state[18], state[19], state[20], state[21], state[22], \
            state[23], state[24], state[25], state[26], state[27], state[28]\
        ]) = ReadPositive_num_bits_252(((const_500 + (tmp_1 * const_5)) + const_0))",
        "Felt252::from_limbs([\
            state[30], state[31], state[32], state[33], state[34], state[35], state[36], \
            state[37], state[38], state[39], state[40], state[41], state[42], state[43], \
            state[44], state[45], state[46], state[47], state[48], state[49], state[50], \
            state[51], state[52], state[53], state[54], state[55], state[56], state[57]\
        ]) = ReadPositive_num_bits_252(((const_500 + (tmp_1 * const_5)) + const_1))",
        "state[58] = BitwiseXor([state[1], state[30]])",
        "tmp_12 = (const_1073741824 * ((state[1] + state[30]) - state[58]))",
        "state[59] = BitwiseXor([state[2], state[31]])",
        "tmp_15 = (const_1073741824 * ((state[2] + state[31]) - state[59]))",
        "state[60] = BitwiseXor([state[3], state[32]])",
        "tmp_18 = (const_1073741824 * ((state[3] + state[32]) - state[60]))",
        "state[61] = BitwiseXor([state[4], state[33]])",
        "tmp_21 = (const_1073741824 * ((state[4] + state[33]) - state[61]))",
        "state[62] = BitwiseXor([state[5], state[34]])",
        "tmp_24 = (const_1073741824 * ((state[5] + state[34]) - state[62]))",
        "state[63] = BitwiseXor([state[6], state[35]])",
        "tmp_27 = (const_1073741824 * ((state[6] + state[35]) - state[63]))",
        "state[64] = BitwiseXor([state[7], state[36]])",
        "tmp_30 = (const_1073741824 * ((state[7] + state[36]) - state[64]))",
        "state[65] = BitwiseXor([state[8], state[37]])",
        "tmp_33 = (const_1073741824 * ((state[8] + state[37]) - state[65]))",
        "state[66] = BitwiseXor([state[9], state[38]])",
        "tmp_36 = (const_1073741824 * ((state[9] + state[38]) - state[66]))",
        "state[67] = BitwiseXor([state[10], state[39]])",
        "tmp_39 = (const_1073741824 * ((state[10] + state[39]) - state[67]))",
        "state[68] = BitwiseXor([state[11], state[40]])",
        "tmp_42 = (const_1073741824 * ((state[11] + state[40]) - state[68]))",
        "state[69] = BitwiseXor([state[12], state[41]])",
        "tmp_45 = (const_1073741824 * ((state[12] + state[41]) - state[69]))",
        "state[70] = BitwiseXor([state[13], state[42]])",
        "tmp_48 = (const_1073741824 * ((state[13] + state[42]) - state[70]))",
        "state[71] = BitwiseXor([state[14], state[43]])",
        "tmp_51 = (const_1073741824 * ((state[14] + state[43]) - state[71]))",
        "state[72] = BitwiseXor([state[15], state[44]])",
        "tmp_54 = (const_1073741824 * ((state[15] + state[44]) - state[72]))",
        "state[73] = BitwiseXor([state[16], state[45]])",
        "tmp_57 = (const_1073741824 * ((state[16] + state[45]) - state[73]))",
        "state[74] = BitwiseXor([state[17], state[46]])",
        "tmp_60 = (const_1073741824 * ((state[17] + state[46]) - state[74]))",
        "state[75] = BitwiseXor([state[18], state[47]])",
        "tmp_63 = (const_1073741824 * ((state[18] + state[47]) - state[75]))",
        "state[76] = BitwiseXor([state[19], state[48]])",
        "tmp_66 = (const_1073741824 * ((state[19] + state[48]) - state[76]))",
        "state[77] = BitwiseXor([state[20], state[49]])",
        "tmp_69 = (const_1073741824 * ((state[20] + state[49]) - state[77]))",
        "state[78] = BitwiseXor([state[21], state[50]])",
        "tmp_72 = (const_1073741824 * ((state[21] + state[50]) - state[78]))",
        "state[79] = BitwiseXor([state[22], state[51]])",
        "tmp_75 = (const_1073741824 * ((state[22] + state[51]) - state[79]))",
        "state[80] = BitwiseXor([state[23], state[52]])",
        "tmp_78 = (const_1073741824 * ((state[23] + state[52]) - state[80]))",
        "state[81] = BitwiseXor([state[24], state[53]])",
        "tmp_81 = (const_1073741824 * ((state[24] + state[53]) - state[81]))",
        "state[82] = BitwiseXor([state[25], state[54]])",
        "tmp_84 = (const_1073741824 * ((state[25] + state[54]) - state[82]))",
        "state[83] = BitwiseXor([state[26], state[55]])",
        "tmp_87 = (const_1073741824 * ((state[26] + state[55]) - state[83]))",
        "state[84] = BitwiseXor([state[27], state[56]])",
        "tmp_90 = (const_1073741824 * ((state[27] + state[56]) - state[84]))",
        "state[85] = BitwiseXor([state[28], state[57]])",
        "tmp_93 = (const_1073741824 * ((state[28] + state[57]) - state[85]))",
        "() = MemVerify(\
            (((const_500 + (tmp_1 * const_5)) + const_2), \
            Felt252::from_limbs([\
                tmp_12, tmp_15, tmp_18, tmp_21, tmp_24, tmp_27, tmp_30, tmp_33, tmp_36, tmp_39, \
                tmp_42, tmp_45, tmp_48, tmp_51, tmp_54, tmp_57, tmp_60, tmp_63, tmp_66, tmp_69, \
                tmp_72, tmp_75, tmp_78, tmp_81, tmp_84, tmp_87, tmp_90, tmp_93\
            ]))\
        )",
        "() = MemVerify(\
            (((const_500 + (tmp_1 * const_5)) + const_3), \
            Felt252::from_limbs([\
                state[58], state[59], state[60], state[61], state[62], state[63], state[64], \
                state[65], state[66], state[67], state[68], state[69], state[70], state[71], \
                state[72], state[73], state[74], state[75], state[76], state[77], state[78], \
                state[79], state[80], state[81], state[82], state[83], state[84], state[85]\
            ]))\
        )",
        "() = MemVerify(\
            (((const_500 + (tmp_1 * const_5)) + const_4), \
            Felt252::from_limbs([\
                (tmp_12 + state[58]), (tmp_15 + state[59]), (tmp_18 + state[60]), \
                (tmp_21 + state[61]), (tmp_24 + state[62]), (tmp_27 + state[63]), \
                (tmp_30 + state[64]), (tmp_33 + state[65]), (tmp_36 + state[66]), \
                (tmp_39 + state[67]), (tmp_42 + state[68]), (tmp_45 + state[69]), \
                (tmp_48 + state[70]), (tmp_51 + state[71]), (tmp_54 + state[72]), \
                (tmp_57 + state[73]), (tmp_60 + state[74]), (tmp_63 + state[75]), \
                (tmp_66 + state[76]), (tmp_69 + state[77]), (tmp_72 + state[78]), \
                (tmp_75 + state[79]), (tmp_78 + state[80]), (tmp_81 + state[81]), \
                (tmp_84 + state[82]), (tmp_87 + state[83]), (tmp_90 + state[84]), \
                (tmp_93 + state[85])\
            ]))\
        )",
    ];
    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(instance_number.clone(), 0),
            const_felt252_expr!(467968798486, 18694984798),
        ),
        (
            get_addr(instance_number.clone(), 1),
            const_felt252_expr!(3468798969565, 4869486468496),
        ),
        (
            get_addr(instance_number.clone(), 2),
            const_felt252_expr!(157370615316, 18253796368),
        ),
        (
            get_addr(instance_number.clone(), 3),
            const_felt252_expr!(3622026537419, 4851673860558),
        ),
        (
            get_addr(instance_number.clone(), 4),
            const_felt252_expr!(3779397152735, 4869927656926),
        ),
    ]);

    let bitwise = BitwiseBuiltin {
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&bitwise);

    registry.run_air_with_row_number(&bitwise, (), 10);

    // Check air body
    let entry = registry.get_air_fn_entry(&bitwise.name());
    assert_eq!(
        entry
            .air_body
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        air_body_vec
    );
}
