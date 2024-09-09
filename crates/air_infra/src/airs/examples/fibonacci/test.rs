use super::fib::*;

use crate::airs::examples::fibonacci::wide_fib::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

use crate::expr;

#[test]
fn test_wide_fibonacci() {
    let air_fn = WideFib {
        num_narrow: 2,
        narrow_size: 2,
    };
    let registry = AirFnRegistry::new(&air_fn);
    let (_, output) = registry.run_air(&air_fn, expr!("secret", 1));
    assert!(output.calc() == *"866");

    // Check entry
    compare_test_json(
        registry,
        &air_fn.name(),
        &(TEST_JSONS_EXAMPLES_DIR.to_owned() + "wide_fibonacci.json"),
    );
}

#[test]
fn test_fibonacci() {
    let air_fn = Fib { claim_index: 6 };
    let registry = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(&air_fn, expr!("secret", 1));
    assert_eq!(output.calc(), "866");
    assert_eq!(state.calc(), ["1", "2", "5", "29", "866"]);

    // Check entry
    compare_test_json(
        registry,
        &air_fn.name(),
        &(TEST_JSONS_EXAMPLES_DIR.to_owned() + "fibonacci.json"),
    );
}
