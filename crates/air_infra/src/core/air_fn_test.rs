use air_common::TraceType;
use serde::Serialize;

use super::air_fn::*;
use super::air_fn_registry::*;
use super::expressions::felt_expr::*;
use super::expressions::felt252_expr::*;
use super::expressions::uint32_expr::*;
use super::variables::*;
use crate::range_check::*;
use crate::{const_expr, const_felt252_expr, const_u32_expr};

#[test]
#[should_panic(expected = "incorrect constraint")]
fn test_incompleteness() {
    #[derive(Debug, Serialize)]
    struct AirFnWithIncorrectConstraint {}

    impl AirFn for AirFnWithIncorrectConstraint {
        type ExtIn = ();
        type In = FeltExpr;
        type Out = FeltExpr;

        fn call(&self, air_builder: &mut AirBuilder, _: (), mut input: Self::In) -> Self::Out {
            // Add the input twice to the state
            let x0 = air_builder.deduce(&mut input, "");
            let x1 = air_builder.deduce(&mut input, "");

            // Add incorrect constraint
            air_builder.constrain((x0 - x1) - const_expr!(1), "");

            input
        }
    }

    let func = AirFnWithIncorrectConstraint {};
    let (registry, _) = AirFnRegistry::new(&func);
    registry.run_air(&func, (), const_expr!(1234));
}

#[test]
#[should_panic(expected = "constraint must have degree <= 3")]
fn test_high_degree_constraint_validation() {
    #[derive(Debug, Serialize)]
    struct AirFnWithHighDegreeConstraint {}

    impl AirFn for AirFnWithHighDegreeConstraint {
        type ExtIn = ();
        type In = FeltExpr;
        type Out = ();

        fn call(&self, air_builder: &mut AirBuilder, _: (), input: Self::In) -> Self::Out {
            // Add high degree constraint
            let prod = input.clone()
                * (input.clone() - const_expr!(1))
                * (input.clone() - const_expr!(2))
                * (input - const_expr!(3));
            air_builder.constrain(prod, "range_check_2_on_input");
        }
    }

    let func = AirFnWithHighDegreeConstraint {};
    let (registry, _) = AirFnRegistry::new(&func);
    registry.run_air(&func, (), const_expr!(2));
}

#[test]
#[should_panic(expected = "lookup term must have degree <= 1")]
fn test_high_degree_lookup_validation() {
    #[derive(Debug, Serialize)]
    struct AirFnWithHighDegreeLookup {}

    impl AirFn for AirFnWithHighDegreeLookup {
        type ExtIn = ();
        type In = FeltExpr;
        type Out = ();

        fn call(&self, air_builder: &mut AirBuilder, _: (), input: Self::In) -> Self::Out {
            // Add high degree constraint
            let prod = input.clone() * (input.clone() - const_expr!(1));
            range_check(air_builder, &[12], &[prod]);
        }
    }

    let func = AirFnWithHighDegreeLookup {};
    let (registry, _) = AirFnRegistry::new(&func);
    registry.run_air(&func, (), const_expr!(17));
}

#[derive(Debug, Serialize)]
struct AirFnWithUInt32 {}

impl AirFn for AirFnWithUInt32 {
    type ExtIn = ();
    type In = UInt32Expr;
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), input: Self::In) -> Self::Out {
        let mut x = air_builder.let_for_deduction(input + const_u32_expr!(4), "");

        let x0 = air_builder.deduce(x.low_mut().as_felt_mut(), "");
        let x1 = air_builder.deduce(x.high_mut().as_felt_mut(), "");

        air_builder.constrain((x0 + (x1 * const_expr!(2_u32.pow(16)))) - const_expr!(9), "");

        x
    }
}

#[test]
fn test_uint32_deduce() {
    let func = AirFnWithUInt32 {};
    let (registry, _) = AirFnRegistry::new(&func);

    let (_, out) = registry.run_air(&func, (), const_u32_expr!(5));
    assert!(out.as_felts().iter().all(|f| f.in_state()));
    assert!(out.calc() == "9");
}

#[derive(Debug, Serialize)]
struct AirFnWithFelt252 {}

impl AirFn for AirFnWithFelt252 {
    type ExtIn = ();
    type In = Felt252Expr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), input: Self::In) -> Self::Out {
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

    let (_, out) = registry.run_air(&func, (), const_felt252_expr!(5, 0));
    assert!(out.in_state());
    assert!(out.calc() == "5");
}

type TestState = FeltExpr;

#[derive(Debug, Serialize, Default)]
struct TestChainRound {}
impl AirFn for TestChainRound {
    type ExtIn = ();
    type In = (ChainIdVar, RoundNumVar, TestState);
    type Out = (ChainIdVar, RoundNumVar, TestState);

    fn call(
        &self,
        _air_builder: &mut AirBuilder,
        _: (),
        (chain, rnd, state): Self::In,
    ) -> Self::Out {
        let new_state = rnd.clone() + state.clone();
        (chain, rnd + const_expr!(1), new_state)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::ChainRound
    }
}

impl ChainRoundAirFn<TestState> for TestChainRound {
    fn number_of_chains(&self) -> usize {
        2
    }
}

#[derive(Debug, Serialize, Default)]
struct TestChainLookupCall {}
impl AirFn for TestChainLookupCall {
    type ExtIn = ();
    type In = TestState;
    type Out = TestState;

    fn call(&self, air_builder: &mut AirBuilder, _: (), state: Self::In) -> Self::Out {
        let new_state = air_builder.chain_lookup_call(&TestChainRound {}, state, 0, 3);

        air_builder.chain_lookup_call(&TestChainRound {}, new_state, 5, 2)
    }
}

#[test]
fn test_chain_lookup_call() {
    let func = TestChainLookupCall {};
    let (registry, _) = AirFnRegistry::new(&func);
    let (_, out) = registry.run_air(&func, (), const_expr!(2));
    assert_eq!(out.calc(), "16");
}
