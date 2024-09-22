use super::range_check::*;

use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::utils::test_utils::*;

// Macros
use crate::expr;

#[derive(Debug)]
struct SmallAdd {}

// A simple AirFn, just for tests: add two felts, asserting that both the
// inputs and the output are < 2**16
impl AirFn for SmallAdd {
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn call(
        &self,
        air_builder: &mut crate::core::air_fn::AirBuilder,
        [mut a, mut b]: Self::In,
    ) -> Self::Out {
        let rc_air_fn = RangeCheck { bits: [16] };

        air_builder.deduce(&mut a);
        air_builder.deduce(&mut b);

        air_builder.lookup_call(&rc_air_fn, [a.clone()]);
        air_builder.lookup_call(&rc_air_fn, [b.clone()]);

        let result = a + b;

        air_builder.lookup_call(&rc_air_fn, [result.clone()]);

        result
    }
}

#[test]
fn test_rc_small_add() {
    let air_fn = SmallAdd {};
    let registry = AirFnRegistry::new(&air_fn);
    // Check entry
    compare_test_json(
        registry,
        &air_fn.name(),
        &(TEST_JSONS_CONST_TABLES_DIR.to_owned() + "rc_small_add.json"),
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck16 on input 80000")]
fn test_range_check_runtime_failure() {
    let air_fn = SmallAdd {};
    let registry = AirFnRegistry::new(&air_fn);
    let a = expr!("a", 40000);
    let b = expr!("b", 40000);
    registry.run_air(&air_fn, [a, b]);
}

#[test]
fn test_range_check_runtime_success() {
    let air_fn = SmallAdd {};
    let registry = AirFnRegistry::new(&air_fn);
    let a = expr!("a", 20000);
    let b = expr!("b", 20000);
    registry.run_air(&air_fn, [a, b]);
}

#[test]
fn test_range_check_vector() {
    let range_check_vector = RangeCheck { bits: [2, 5] };
    let registry = AirFnRegistry::new(&range_check_vector);
    registry.run_air(&range_check_vector, [expr!("a", 0b11), expr!("b", 0b11111)]);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck2 on input 4")]
fn test_failed_range_check_first_element() {
    let range_check_vector = RangeCheck { bits: [2, 5] };
    let registry = AirFnRegistry::new(&range_check_vector);
    registry.run_air(
        &range_check_vector,
        [expr!("a", 0b100), expr!("b", 0b11111)],
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 1: RangeCheck5 on input 32")]
fn test_failed_range_check_second_element() {
    let range_check_vector = RangeCheck { bits: [2, 5] };
    let registry = AirFnRegistry::new(&range_check_vector);
    registry.run_air(
        &range_check_vector,
        [expr!("a", 0b11), expr!("b", 0b100000)],
    );
}

#[test]
#[should_panic(expected = "Invalid range check bits [3, 4].")]
fn test_failed_range_check_vector_size() {
    let range_check_vector = RangeCheck { bits: [3, 4] };
    let registry = AirFnRegistry::new(&range_check_vector);
    registry.run_air(
        &range_check_vector,
        [expr!("a", 0b11), expr!("b", 0b100000)],
    );
}
