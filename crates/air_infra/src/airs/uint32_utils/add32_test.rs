use super::add32::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_u32_expr;

#[test]
fn test_add32() {
    let air_fn = Add32 {};
    let (registry, entry) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(&air_fn, [const_u32_expr!(1), const_u32_expr!(1)]);
    assert_eq!(output.calc(), "2");
    assert_eq!(state.calc(), ["2", "0"]);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_u32_expr!(2_u32.pow(15)),
            const_u32_expr!(2_u32.pow(15)),
        ],
    );
    assert_eq!(output.calc(), "65536");
    assert_eq!(state.calc(), ["0", "1"]);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_u32_expr!(2_u32.pow(31)),
            const_u32_expr!(2_u32.pow(31)),
        ],
    );
    assert_eq!(output.calc(), "0");
    assert_eq!(state.calc(), ["0", "0"]);

    // Check entry
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_UINT32_DIR,
            entry.name.to_lowercase()
        ),
    );
}
