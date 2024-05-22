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
    let func = BitUnpack { n_bits: 4 };
    let (registry, _, lists) = AirFnRegistry::new(&func);

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
        "BitUnpack__4_input.as_felt()",
        "deduction_tmp_2 = (BitUnpack__4_input >> const_1)",
        "deduction_tmp_2.as_felt()",
        "deduction_tmp_4 = (deduction_tmp_2 >> const_1)",
        "deduction_tmp_4.as_felt()",
        "deduction_tmp_6 = (deduction_tmp_4 >> const_1)",
        "deduction_tmp_6.as_felt()",
        "deduction_tmp_8 = (deduction_tmp_6 >> const_1)",
        "deduction_tmp_8.as_felt()",
    ];

    assert!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == constraints
    );
    assert!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == deductions
    );

    let (state, output) = registry.run_air(&func, u16_expr!("x", 10));
    assert!(state.calc() == ["10", "5", "2", "1", "0"]);
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
        air_builder.deduce(x.as_felt());
        let air_fn = Div2 {};

        let (mut bit, _) = air_builder.call(&air_fn, x.clone());
        air_builder.constrain(
            &(&*bit.as_felt() * &x.as_felt())
                + &(&(&const_expr!(1) - &*bit.as_felt()) * &(&*x.as_felt() - &const_expr!(2))),
        );
        bit
    }

    fn input_in_trace(&self) -> bool {
        false
    }
}

#[test]
fn test_bit_mux() {
    let func = AirFnBitMux {};
    let (registry, _, lists) = AirFnRegistry::new(&func);

    let constraints = [
        "constraint_tmp_3 = (state[0] - (state[1] * const_2))",
        "(constraint_tmp_3 * (constraint_tmp_3 - const_1))",
        "((constraint_tmp_3 * state[0]) + ((const_1 - constraint_tmp_3) * (state[0] - const_2)))",
    ];

    let deductions = [
        "AirFnBitMux_input.as_felt()",
        "deduction_tmp_2 = (AirFnBitMux_input >> const_1)",
        "deduction_tmp_2.as_felt()",
    ];

    assert!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == constraints
    );
    assert!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == deductions
    );

    let (_, out) = registry.run_air(&func, u16_expr!("x", 2));
    assert!(out.calc() == "false");
}
