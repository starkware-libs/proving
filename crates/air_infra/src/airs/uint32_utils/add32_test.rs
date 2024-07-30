use super::add32::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::prover_types::*;
// Macros
use crate::u32_expr;

#[test]
fn test_add32() {
    let air_fn = Add32 {};
    let registry = AirFnRegistry::new(&air_fn);
    let lists = registry.get_compiled_air_fn(&air_fn);

    let constraints = [
        "RangeCheck16([state[0]]) == []",
        "RangeCheck16([state[1]]) == []",
        "constraint_tmp_3 = ((Add32_cb314bd22a8fc165_input[0].low().as_m31() + Add32_cb314bd22a8fc165_input[1].low().as_m31()) - state[0])",
        "(constraint_tmp_3 * (constraint_tmp_3 - const_65536))",
        "((((Add32_cb314bd22a8fc165_input[0].high().as_m31() + Add32_cb314bd22a8fc165_input[1].high().as_m31()) - state[1]) * const_65536) + constraint_tmp_3)",
    ];

    let deductions = [
        "deduction_tmp_0 = (Add32_cb314bd22a8fc165_input[0] + Add32_cb314bd22a8fc165_input[1])",
        "deduction_tmp_0.low().as_m31()",
        "deduction_tmp_0.high().as_m31()",
        "deduction_tmp_1 = RangeCheck16(state[0])",
        "deduction_tmp_2 = RangeCheck16(state[1])",
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

    let (state, output) = registry.run_air(
        &air_fn,
        [u32_expr!("x", 1_u32, true), u32_expr!("y", 1_u32, true)],
    );
    assert!(output.calc() == "2");
    assert!(state.calc() == ["2", "0"]);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            u32_expr!("x", 2_u32.pow(15), true),
            u32_expr!("y", 2_u32.pow(15), true),
        ],
    );
    assert!(output.calc() == "65536");
    assert!(state.calc() == ["0", "1"]);
}
