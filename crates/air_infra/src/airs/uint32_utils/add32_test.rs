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
    let (registry, _, lists) = AirFnRegistry::new(&air_fn);

    let constraints = [
        "constraint_tmp_1 = ((state[0] + state[2]) - state[4])",
        "(constraint_tmp_1 * (constraint_tmp_1 - const_65536))",
        "((((state[1] + state[3]) - state[5]) * const_65536) + constraint_tmp_1)",
    ];

    let deductions = [
        "deduction_tmp_0 = (Add32_input_0 + Add32_input_1)",
        "deduction_tmp_0.low().as_felt()",
        "deduction_tmp_0.high().as_felt()",
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
