use std::array::from_fn;

use super::air_fn::*;
use super::air_fn_registry::*;
use super::expressions::expr::*;
use super::expressions::felt252_expr::*;
use super::expressions::felt_expr::*;
use super::expressions::uint32_expr::*;
use super::prover_types::*;
use super::variables::*;
use crate::{const_expr, const_u32_expr, expr, felt252_expr, u32_expr};

#[derive(Debug)]
struct AirFnWithIncorrectConstraint {}

impl AirFn for AirFnWithIncorrectConstraint {
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, mut input: Self::In) -> Self::Out {
        // Add the input twice to the state
        let x0 = air_builder.deduce(&mut input);
        let x1 = air_builder.deduce(&mut input);

        // Add incorrect constraint
        air_builder.constrain(&(&x0 - &x1) - &const_expr!(1));

        input
    }
}

#[test]
#[should_panic(expected = "incorrect constraint")]
fn test_incompleteness() {
    let func = AirFnWithIncorrectConstraint {};
    let registry = AirFnRegistry::new(&func);
    registry.run_air(&func, expr!("x", 1234, true));
}

#[derive(Debug)]
struct AirFnWithUInt32 {}

impl AirFn for AirFnWithUInt32 {
    type In = UInt32Expr;
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        let mut x = air_builder.let_for_deduction(&input + &const_u32_expr!(4));

        let x0 = air_builder.deduce(x.low().as_felt());
        let x1 = air_builder.deduce(x.high().as_felt());

        air_builder.constrain(&(&x0 + &(&x1 * &const_expr!(2_u32.pow(16)))) - &const_expr!(9));

        x
    }
}

#[test]
fn test_uint32_deduce() {
    let func = AirFnWithUInt32 {};
    let registry = AirFnRegistry::new(&func);

    let (_, out) = registry.run_air(&func, u32_expr!("x", 5, true));
    assert!(out.in_state());
    assert!(out.calc() == "9");
}

#[derive(Debug)]
struct AirFnWithArray {}

impl AirFn for AirFnWithArray {
    type In = [FeltExpr; 2];
    type Out = [FeltExpr; 2];

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        let mut x = air_builder.let_for_deduction(input);

        let _x0 = air_builder.deduce(&mut x[0]);
        let _x1 = air_builder.deduce(&mut x[1]);
        x
    }
}
// Should delete?
// #[test]
// fn test_array_deduce() {
//     let func = AirFnWithArray {};
//     let registry = AirFnRegistry::new(&func);

//     let (_, out) = registry.run_air(&func, [expr!("x", 5, true), expr!("y", 5, true)]);
//     assert!(out.in_state());
//     assert!(out[0].name() == "state[0]");
// }

#[derive(Debug)]
struct AirFnWithFelt252 {}

impl AirFn for AirFnWithFelt252 {
    type In = Felt252Expr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        let mut x = air_builder.let_for_deduction(input);

        for felt in x.as_felts() {
            air_builder.deduce(felt);
        }

        x.as_felts()[0].clone()
    }
}

#[test]
fn test_felt252_deduce() {
    let func = AirFnWithFelt252 {};
    let registry = AirFnRegistry::new(&func);

    let (_, out) = registry.run_air(&func, felt252_expr!("x", 5, 0, true));
    assert!(out.in_state());
    assert!(out.calc() == "5");

    let lists = registry.get_codegen_air_fn(&func);
    assert_eq!(
        "deduction_tmp_0.get_felt(const_0)",
        lists.deductions[1].to_string()
    );
}
