use super::add32::*;
// Macros
use crate::const_u32_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

#[test]
fn test_add32() {
    let air_fn = Add32 {};
    let (registry, entry) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(&air_fn, [const_u32_expr!(1), const_u32_expr!(1)]);
    assert_eq!(output.calc(), "2");
    assert!(
        state == vec![(2, "add_res_limb_0"), (0, "add_res_limb_1")].into(),
        "State {} does not match [(2, 'add_res_limb_0'), (0, 'add_res_limb_1')]",
        state
    );

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_u32_expr!(2_u32.pow(15)),
            const_u32_expr!(2_u32.pow(15)),
        ],
    );
    assert_eq!(output.calc(), "65536");
    assert!(
        state == vec![(0, "add_res_limb_0"), (1, "add_res_limb_1")].into(),
        "State {} does not match [(0, 'add_res_limb_0'), (1, 'add_res_limb_1')]",
        state
    );

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_u32_expr!(2_u32.pow(31)),
            const_u32_expr!(2_u32.pow(31)),
        ],
    );
    assert_eq!(output.calc(), "0");
    assert!(
        state == vec![(0, "add_res_limb_0"), (0, "add_res_limb_1")].into(),
        "State {} does not match [(0, 'add_res_limb_0'), (0, 'add_res_limb_1')]",
        state
    );

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
