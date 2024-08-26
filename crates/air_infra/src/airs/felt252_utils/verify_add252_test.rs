use super::verify_add252::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
// Macros
use crate::const_felt252_expr;

#[test]
fn test_verify_add252_air_body() {
    let air_fn = VerifyAdd252 {};
    let registry = AirFnRegistry::new(&air_fn);
    let entry = registry.get_air_fn_entry(&air_fn.name());

    let expected_air_body = [
        "tmp_0 = \
            (const_1 & ((UInt16::from_m31(VerifyAdd252_4afb134610550b92_input[0].get_m31(const_0)) \
            ^ UInt16::from_m31(VerifyAdd252_4afb134610550b92_input[1].get_m31(const_0))) \
            ^ UInt16::from_m31(VerifyAdd252_4afb134610550b92_input[2].get_m31(const_0))))",
        "Deduction: tmp_0.as_m31()",
        "Constraint: (state[0] * (state[0] - const_1))",
        "tmp_1 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_0) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_0)) + const_0) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_0)) - (const_1 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_1 * ((tmp_1 * tmp_1) - const_1))",
        "tmp_2 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_1) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_1)) + tmp_1) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_1)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_2 * ((tmp_2 * tmp_2) - const_1))",
        "tmp_3 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_2) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_2)) + tmp_2) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_2)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_3 * ((tmp_3 * tmp_3) - const_1))",
        "tmp_4 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_3) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_3)) + tmp_3) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_3)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_4 * ((tmp_4 * tmp_4) - const_1))",
        "tmp_5 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_4) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_4)) + tmp_4) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_4)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_5 * ((tmp_5 * tmp_5) - const_1))",
        "tmp_6 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_5) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_5)) + tmp_5) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_5)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_6 * ((tmp_6 * tmp_6) - const_1))",
        "tmp_7 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_6) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_6)) + tmp_6) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_6)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_7 * ((tmp_7 * tmp_7) - const_1))",
        "tmp_8 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_7) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_7)) + tmp_7) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_7)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_8 * ((tmp_8 * tmp_8) - const_1))",
        "tmp_9 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_8) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_8)) + tmp_8) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_8)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_9 * ((tmp_9 * tmp_9) - const_1))",
        "tmp_10 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_9) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_9)) + tmp_9) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_9)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_10 * ((tmp_10 * tmp_10) - const_1))",
        "tmp_11 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_10) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_10)) + tmp_10) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_10)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_11 * ((tmp_11 * tmp_11) - const_1))",
        "tmp_12 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_11) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_11)) + tmp_11) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_11)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_12 * ((tmp_12 * tmp_12) - const_1))",
        "tmp_13 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_12) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_12)) + tmp_12) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_12)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_13 * ((tmp_13 * tmp_13) - const_1))",
        "tmp_14 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_13) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_13)) + tmp_13) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_13)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_14 * ((tmp_14 * tmp_14) - const_1))",
        "tmp_15 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_14) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_14)) + tmp_14) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_14)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_15 * ((tmp_15 * tmp_15) - const_1))",
        "tmp_16 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_15) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_15)) + tmp_15) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_15)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_16 * ((tmp_16 * tmp_16) - const_1))",
        "tmp_17 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_16) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_16)) + tmp_16) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_16)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_17 * ((tmp_17 * tmp_17) - const_1))",
        "tmp_18 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_17) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_17)) + tmp_17) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_17)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_18 * ((tmp_18 * tmp_18) - const_1))",
        "tmp_19 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_18) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_18)) + tmp_18) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_18)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_19 * ((tmp_19 * tmp_19) - const_1))",
        "tmp_20 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_19) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_19)) + tmp_19) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_19)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_20 * ((tmp_20 * tmp_20) - const_1))",
        "tmp_21 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_20) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_20)) + tmp_20) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_20)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_21 * ((tmp_21 * tmp_21) - const_1))",
        "tmp_22 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_21) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_21)) + tmp_21) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_21)) - (const_136 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_22 * ((tmp_22 * tmp_22) - const_1))",
        "tmp_23 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_22) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_22)) + tmp_22) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_22)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_23 * ((tmp_23 * tmp_23) - const_1))",
        "tmp_24 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_23) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_23)) + tmp_23) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_23)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_24 * ((tmp_24 * tmp_24) - const_1))",
        "tmp_25 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_24) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_24)) + tmp_24) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_24)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_25 * ((tmp_25 * tmp_25) - const_1))",
        "tmp_26 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_25) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_25)) + tmp_25) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_25)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_26 * ((tmp_26 * tmp_26) - const_1))",
        "tmp_27 = (((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_26) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_26)) + tmp_26) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_26)) - (const_0 * state[0])) * \
            const_4194304)",
        "Constraint: (tmp_27 * ((tmp_27 * tmp_27) - const_1))",
        "Constraint: ((((VerifyAdd252_4afb134610550b92_input[0].get_m31(const_27) + \
            VerifyAdd252_4afb134610550b92_input[1].get_m31(const_27)) + tmp_27) - \
            VerifyAdd252_4afb134610550b92_input[2].get_m31(const_27)) - (const_256 * state[0]))",
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
fn test_verify_add252_no_overflow() {
    let air_fn = VerifyAdd252 {};
    let registry = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
            const_felt252_expr!(0x3000040002u128, 0u128),
        ],
    );
    let expected_state = ["0"];
    assert_eq!(state.calc(), expected_state);
}

#[test]
fn test_verify_add252_with_overflow() {
    let air_fn = VerifyAdd252 {};
    let registry = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x7ffffffffffffeeffffffffffffffffu128
            ),
        ],
    );
    let expected_state = ["1"];
    assert_eq!(state.calc(), expected_state);
}
