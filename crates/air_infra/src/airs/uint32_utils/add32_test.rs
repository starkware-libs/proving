use super::add32::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::variables::*;

// Macros
use crate::const_u32_expr;

#[test]
fn test_add32() {
    let air_fn = Add32 {};
    let registry = AirFnRegistry::new(&air_fn);
    let lists = registry.get_compiled_air_fn(&air_fn.name());

    let constraints = [
        "RangeCheck16([state[0]]) == []",
        "RangeCheck16([state[1]]) == []",
        "tmp_3 = ((Add32_cb314bd22a8fc165_input[0].low().as_m31() + Add32_cb314bd22a8fc165_input[1].low().as_m31()) - state[0])",
        "(tmp_3 * (tmp_3 - const_65536))",
        "tmp_4 = (((Add32_cb314bd22a8fc165_input[0].high().as_m31() + Add32_cb314bd22a8fc165_input[1].high().as_m31()) - state[1]) + (tmp_3 * const_32768))",
        "(tmp_4 * (tmp_4 - const_65536))"
    ];

    let deductions = [
        "tmp_0 = (Add32_cb314bd22a8fc165_input[0] + Add32_cb314bd22a8fc165_input[1])",
        "tmp_0.low().as_m31()",
        "tmp_0.high().as_m31()",
        "tmp_1 = RangeCheck16([state[0]])",
        "tmp_2 = RangeCheck16([state[1]])",
    ];

    assert_eq!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints
    );
    assert_eq!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deductions
    );

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
}
