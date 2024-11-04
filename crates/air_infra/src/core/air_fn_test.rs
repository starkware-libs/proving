use inst_def::InstDef;

use super::air_fn::*;
use super::air_fn_registry::*;
use super::expressions::felt252_expr::*;
use super::expressions::felt_expr::*;
use super::expressions::uint32_expr::*;
use super::variables::*;
use crate::{const_expr, const_felt252_expr, const_u32_expr};

#[derive(Debug, InstDef)]
struct AirFnWithIncorrectConstraint {}

impl AirFn for AirFnWithIncorrectConstraint {
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, mut input: Self::In) -> Self::Out {
        // Add the input twice to the state
        let x0 = air_builder.deduce(&mut input, "");
        let x1 = air_builder.deduce(&mut input, "");

        // Add incorrect constraint
        air_builder.constrain((x0 - x1) - const_expr!(1), "");

        input
    }
}

#[test]
#[should_panic(expected = "incorrect constraint")]
fn test_incompleteness() {
    let func = AirFnWithIncorrectConstraint {};
    let (registry, _) = AirFnRegistry::new(&func);
    registry.run_air(&func, const_expr!(1234));
}

#[derive(Debug, InstDef)]
struct AirFnWithUInt32 {}

impl AirFn for AirFnWithUInt32 {
    type In = UInt32Expr;
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        let mut x = air_builder.let_for_deduction(input + const_u32_expr!(4), "");

        let x0 = air_builder.deduce(x.low_mut().as_felt_mut(), "");
        let x1 = air_builder.deduce(x.high_mut().as_felt_mut(), "");

        air_builder.constrain(
            (x0 + (x1 * const_expr!(2_u32.pow(16)))) - const_expr!(9),
            "",
        );

        x
    }
}

#[test]
fn test_uint32_deduce() {
    let func = AirFnWithUInt32 {};
    let (registry, _) = AirFnRegistry::new(&func);

    let (_, out) = registry.run_air(&func, const_u32_expr!(5));
    assert!(out.in_state());
    assert!(out.calc() == "9");
}

#[derive(Debug, InstDef)]
struct AirFnWithFelt252 {}

impl AirFn for AirFnWithFelt252 {
    type In = Felt252Expr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        let mut x = air_builder.let_for_deduction(input, "");

        for felt in x.as_felts_mut() {
            air_builder.deduce(felt, "");
        }

        x.get_felt(0)
    }
}

#[test]
fn test_felt252_deduce() {
    let func = AirFnWithFelt252 {};
    let (registry, _) = AirFnRegistry::new(&func);

    let (_, out) = registry.run_air(&func, const_felt252_expr!(5, 0));
    assert!(out.in_state());
    assert!(out.calc() == "5");
}
