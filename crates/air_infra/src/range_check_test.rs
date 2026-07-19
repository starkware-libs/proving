use std::slice::from_ref;

use serde::Serialize;

use super::range_check::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;

#[derive(Debug, Serialize)]
struct SmallAdd {}

// A simple AirFn, just for tests: add two felts, asserting that both the
// inputs and the output are < 2**20
impl AirFn for SmallAdd {
    type ExtIn = ();
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [mut a, mut b]: Self::In) -> Self::Out {
        air_builder.deduce(&mut a, "");
        air_builder.deduce(&mut b, "");

        range_check(air_builder, &[20], &[a.clone()]);
        range_check(air_builder, &[20], &[b.clone()]);

        let result = a + b;

        range_check(air_builder, &[20], from_ref(&result));

        result
    }
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck20 on input 2000000")]
fn test_range_check_runtime_failure() {
    let air_fn = SmallAdd {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let a = const_expr!(1000000);
    let b = const_expr!(1000000);
    registry.run_air(&air_fn, (), [a, b]);
}

#[test]
fn test_range_check_runtime_success() {
    let air_fn = SmallAdd {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let a = const_expr!(20000);
    let b = const_expr!(20000);
    registry.run_air(&air_fn, (), [a, b]);
}

#[test]
fn test_range_check_vector() {
    let range_check_vector = RangeCheck::<RangeCheck_4_3_Const>::default();
    let (registry, _) = AirFnRegistry::new(&range_check_vector);
    registry.run_air(&range_check_vector, [const_expr!(0b11), const_expr!(0b111)], ());
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck4 on input 31")]
fn test_failed_range_check_first_element() {
    let range_check_vector = RangeCheck::<RangeCheck_4_3_Const>::default();
    let (registry, _) = AirFnRegistry::new(&range_check_vector);
    registry.run_air(&range_check_vector, [const_expr!(0b11111), const_expr!(0b111)], ());
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 1: RangeCheck3 on input 32")]
fn test_failed_range_check_second_element() {
    let range_check_vector = RangeCheck::<RangeCheck_4_3_Const>::default();
    let (registry, _) = AirFnRegistry::new(&range_check_vector);
    registry.run_air(&range_check_vector, [const_expr!(0b11), const_expr!(0b100000)], ());
}
