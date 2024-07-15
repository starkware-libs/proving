use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::core::Felt;

use crate::expr;

use super::range_check::*;

#[derive(Debug)]
struct SmallAdd {}

// A simple AirFn, just for tests: add two felts, asserting that both the
// inputs and the output are < 2**16
impl AirFn for SmallAdd {
    type In = [FeltExpr; 2];

    type Out = FeltExpr;

    fn trace_type(&self) -> crate::core::air_fn::TraceType {
        TraceType::Inline
    }

    fn call(
        &self,
        air_builder: &mut crate::core::air_fn::AirBuilder,
        [mut a, mut b]: Self::In,
    ) -> Self::Out {
        let rc_air_fn = RangeCheck { bits: 16 };

        air_builder.deduce(&mut a);
        air_builder.deduce(&mut b);

        air_builder.lookup_call(&rc_air_fn, a.clone());
        air_builder.lookup_call(&rc_air_fn, b.clone());

        let result = a + b;

        air_builder.lookup_call(&rc_air_fn, result.clone());

        result
    }
}

#[test]
fn test_range_check() {
    let air_fn = SmallAdd {};
    let registry = AirFnRegistry::new(&air_fn);
    let lists = registry.get_compiled_air_fn(&air_fn);

    let constraints = [
        "RangeCheck16([state[0]]) == []",
        "RangeCheck16([state[1]]) == []",
        "RangeCheck16([(state[0] + state[1])]) == []",
    ];

    let deductions = [
        "SmallAdd_input[0]",
        "SmallAdd_input[1]",
        "deduction_tmp_0 = RangeCheck16(state[0])",
        "deduction_tmp_1 = RangeCheck16(state[1])",
        "deduction_tmp_2 = RangeCheck16((state[0] + state[1]))",
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
}

#[test]
#[should_panic(expected = "RangeCheck16 failed")]
fn test_range_check_runtime_failure() {
    let air_fn = SmallAdd {};
    let registry = AirFnRegistry::new(&air_fn);
    let a = expr!("a", 40000, true);
    let b = expr!("b", 40000, true);
    registry.run_air(&air_fn, [a, b]);
}

#[test]
fn test_range_check_runtime_success() {
    let air_fn = SmallAdd {};
    let registry = AirFnRegistry::new(&air_fn);
    let a = expr!("a", 20000, true);
    let b = expr!("b", 20000, true);
    registry.run_air(&air_fn, [a, b]);
}
