use super::bit_unpack::BitUnpack;
use super::div2::Div2;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::bool_expr::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::prover_types::*;

// Macros
use crate::const_expr;
use crate::u16_expr;

#[test]
fn test_bit_unpacking() {
    let func = BitUnpack::<4> {};
    let registry = AirFnRegistry::new(&func);
    let lists = registry.get_compiled_air_fn(&func);

    let constraints = [
        "constraint_tmp_3 = (state[0] - (state[1] * const_2))",
        "(constraint_tmp_3 * (constraint_tmp_3 - const_1))",
        "constraint_tmp_5 = (state[1] - (state[2] * const_2))",
        "(constraint_tmp_5 * (constraint_tmp_5 - const_1))",
        "constraint_tmp_7 = (state[2] - (state[3] * const_2))",
        "(constraint_tmp_7 * (constraint_tmp_7 - const_1))",
        "constraint_tmp_9 = (state[3] - (state[4] * const_2))",
        "(constraint_tmp_9 * (constraint_tmp_9 - const_1))",
        "state[4]",
    ];

    let deductions = [
        "BitUnpack_3cfd160c00d5343f_input.as_m31()",
        "deduction_tmp_2 = (BitUnpack_3cfd160c00d5343f_input >> const_1)",
        "deduction_tmp_2.as_m31()",
        "deduction_tmp_4 = (deduction_tmp_2 >> const_1)",
        "deduction_tmp_4.as_m31()",
        "deduction_tmp_6 = (deduction_tmp_4 >> const_1)",
        "deduction_tmp_6.as_m31()",
        "deduction_tmp_8 = (deduction_tmp_6 >> const_1)",
        "deduction_tmp_8.as_m31()",
    ];

    assert_eq!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints
    );
    assert_eq!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deductions
    );

    let (state, output) = registry.run_air(&func, u16_expr!("x", 10));
    assert_eq!(state.calc(), ["10", "5", "2", "1", "0"]);
    assert!(
        output.iter().map(|x| x.calc()).collect::<Vec<String>>()
            == ["false", "true", "false", "true"]
    );
}

#[derive(Debug)]
struct AirFnBitMux {}

impl AirFn for AirFnBitMux {
    type In = UInt16Expr;
    type Out = BoolExpr;

    fn call(&self, air_builder: &mut AirBuilder, mut x: Self::In) -> Self::Out {
        let x_f = air_builder.deduce(x.as_felt_mut());
        let air_fn = Div2 {};

        let (bit, _) = air_builder.call(&air_fn, x.clone());
        air_builder.constrain(
            (bit.as_felt() * x_f.clone())
                + (const_expr!(1) - bit.as_felt()) * (x_f - const_expr!(2)),
        );
        bit
    }
}

#[test]
fn test_bit_mux() {
    let func = AirFnBitMux {};
    let registry = AirFnRegistry::new(&func);
    let lists = registry.get_compiled_air_fn(&func);

    let constraints = [
        "constraint_tmp_3 = (state[0] - (state[1] * const_2))",
        "(constraint_tmp_3 * (constraint_tmp_3 - const_1))",
        "((constraint_tmp_3 * state[0]) + ((const_1 - constraint_tmp_3) * (state[0] - const_2)))",
    ];

    let deductions = [
        "AirFnBitMux_6ffde77494a1d1e8_input.as_m31()",
        "deduction_tmp_2 = (AirFnBitMux_6ffde77494a1d1e8_input >> const_1)",
        "deduction_tmp_2.as_m31()",
    ];

    assert_eq!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints
    );
    assert_eq!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deductions
    );

    let (_, out) = registry.run_air(&func, u16_expr!("x", 2));
    assert!(out.calc() == "false");
}
