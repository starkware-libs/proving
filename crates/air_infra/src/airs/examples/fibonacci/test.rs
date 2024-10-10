use super::fib::*;

use crate::airs::examples::fibonacci::wide_fib::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

use crate::const_expr;

#[test]
fn test_wide_fibonacci() {
    let air_fn = WideFib {
        num_narrow: 8,
        narrow_size: 20,
    };
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, output) = registry.run_air(&air_fn, const_expr!(1));
    assert_eq!(output.calc(), *"1594392009");

    // Check entries
    for (name, entry) in registry.compile().iter() {
        compare_json(
            &entry,
            &format!("{}{}.json", TEST_JSONS_EXAMPLES_DIR, name.to_lowercase()),
        );
    }
}

#[test]
fn test_fibonacci() {
    let air_fn = Fib { claim_index: 6 };
    let (registry, entry) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(&air_fn, const_expr!(1));
    assert_eq!(output.calc(), "866");
    assert_eq!(state.calc(), ["1", "2", "5", "29", "866"]);

    // Check entry
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_EXAMPLES_DIR,
            entry.name.to_lowercase()
        ),
    );
}
