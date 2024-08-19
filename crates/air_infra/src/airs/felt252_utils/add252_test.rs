use super::add252::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
// Macros
use crate::const_felt252_expr;

#[test]
fn test_add252_air_body() {
    let air_fn = Add252 {};
    let registry = AirFnRegistry::new(&air_fn);
    let entry = registry.get_air_fn_entry(&air_fn);

    let expected_air_body = [
        "tmp_0 = (Add252_dd24ce7e828639e8_input[0] + Add252_dd24ce7e828639e8_input[1])",
        "Deduction: tmp_0.get_m31(const_0)",
        "tmp_1 = RangeCheck9([state[0]])",
        "RangeCheck9([state[0]]) == []",
        "Deduction: tmp_0.get_m31(const_1)",
        "tmp_2 = RangeCheck9([state[1]])",
        "RangeCheck9([state[1]]) == []",
        "Deduction: tmp_0.get_m31(const_2)",
        "tmp_3 = RangeCheck9([state[2]])",
        "RangeCheck9([state[2]]) == []",
        "Deduction: tmp_0.get_m31(const_3)",
        "tmp_4 = RangeCheck9([state[3]])",
        "RangeCheck9([state[3]]) == []",
        "Deduction: tmp_0.get_m31(const_4)",
        "tmp_5 = RangeCheck9([state[4]])",
        "RangeCheck9([state[4]]) == []",
        "Deduction: tmp_0.get_m31(const_5)",
        "tmp_6 = RangeCheck9([state[5]])",
        "RangeCheck9([state[5]]) == []",
        "Deduction: tmp_0.get_m31(const_6)",
        "tmp_7 = RangeCheck9([state[6]])",
        "RangeCheck9([state[6]]) == []",
        "Deduction: tmp_0.get_m31(const_7)",
        "tmp_8 = RangeCheck9([state[7]])",
        "RangeCheck9([state[7]]) == []",
        "Deduction: tmp_0.get_m31(const_8)",
        "tmp_9 = RangeCheck9([state[8]])",
        "RangeCheck9([state[8]]) == []",
        "Deduction: tmp_0.get_m31(const_9)",
        "tmp_10 = RangeCheck9([state[9]])",
        "RangeCheck9([state[9]]) == []",
        "Deduction: tmp_0.get_m31(const_10)",
        "tmp_11 = RangeCheck9([state[10]])",
        "RangeCheck9([state[10]]) == []",
        "Deduction: tmp_0.get_m31(const_11)",
        "tmp_12 = RangeCheck9([state[11]])",
        "RangeCheck9([state[11]]) == []",
        "Deduction: tmp_0.get_m31(const_12)",
        "tmp_13 = RangeCheck9([state[12]])",
        "RangeCheck9([state[12]]) == []",
        "Deduction: tmp_0.get_m31(const_13)",
        "tmp_14 = RangeCheck9([state[13]])",
        "RangeCheck9([state[13]]) == []",
        "Deduction: tmp_0.get_m31(const_14)",
        "tmp_15 = RangeCheck9([state[14]])",
        "RangeCheck9([state[14]]) == []",
        "Deduction: tmp_0.get_m31(const_15)",
        "tmp_16 = RangeCheck9([state[15]])",
        "RangeCheck9([state[15]]) == []",
        "Deduction: tmp_0.get_m31(const_16)",
        "tmp_17 = RangeCheck9([state[16]])",
        "RangeCheck9([state[16]]) == []",
        "Deduction: tmp_0.get_m31(const_17)",
        "tmp_18 = RangeCheck9([state[17]])",
        "RangeCheck9([state[17]]) == []",
        "Deduction: tmp_0.get_m31(const_18)",
        "tmp_19 = RangeCheck9([state[18]])",
        "RangeCheck9([state[18]]) == []",
        "Deduction: tmp_0.get_m31(const_19)",
        "tmp_20 = RangeCheck9([state[19]])",
        "RangeCheck9([state[19]]) == []",
        "Deduction: tmp_0.get_m31(const_20)",
        "tmp_21 = RangeCheck9([state[20]])",
        "RangeCheck9([state[20]]) == []",
        "Deduction: tmp_0.get_m31(const_21)",
        "tmp_22 = RangeCheck9([state[21]])",
        "RangeCheck9([state[21]]) == []",
        "Deduction: tmp_0.get_m31(const_22)",
        "tmp_23 = RangeCheck9([state[22]])",
        "RangeCheck9([state[22]]) == []",
        "Deduction: tmp_0.get_m31(const_23)",
        "tmp_24 = RangeCheck9([state[23]])",
        "RangeCheck9([state[23]]) == []",
        "Deduction: tmp_0.get_m31(const_24)",
        "tmp_25 = RangeCheck9([state[24]])",
        "RangeCheck9([state[24]]) == []",
        "Deduction: tmp_0.get_m31(const_25)",
        "tmp_26 = RangeCheck9([state[25]])",
        "RangeCheck9([state[25]]) == []",
        "Deduction: tmp_0.get_m31(const_26)",
        "tmp_27 = RangeCheck9([state[26]])",
        "RangeCheck9([state[26]]) == []",
        "Deduction: tmp_0.get_m31(const_27)",
        "tmp_28 = RangeCheck9([state[27]])",
        "RangeCheck9([state[27]]) == []",
        "() = VerifyAdd252_4afb134610550b92([Add252_dd24ce7e828639e8_input[0], Add252_dd24ce7e828639e8_input[1], tmp_0])",
    ];
    assert_eq!(
        entry
            .air_body
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>(),
        expected_air_body
    );
}

#[test]
fn test_add252_no_overflow() {
    let air_fn = Add252 {};
    let registry = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(0x3000040002u128, 0u128).calc()
    );
    let expected_state = [
        "2", "0", "1", "0", "3", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
        "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
    ];
    assert_eq!(state.calc(), expected_state);
}

#[test]
fn test_add252_with_overflow() {
    let air_fn = Add252 {};
    let registry = AirFnRegistry::new(&air_fn);
    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0xffffffffffffffffffffffffffffffffu128,
            0x7ffffffffffffeeffffffffffffffffu128
        )
        .calc()
    );
    let expected_state = [
        "511", "511", "511", "511", "511", "511", "511", "511", "511", "511", "511", "511", "511",
        "511", "511", "511", "511", "511", "511", "511", "511", "375", "511", "511", "511", "511",
        "511", "255", "1",
    ];
    assert_eq!(state.calc(), expected_state);
}
