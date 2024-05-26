use super::fib::Fib;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;
use crate::expr;

#[test]
fn test_fibonacci() {
    let air_fn = Fib { claim_index: 6 };
    let registry = AirFnRegistry::new(&air_fn);
    let lists = registry.get_codegen_air_fn(&air_fn);

    let constraints = [
        "(state[1] - ((const_1 * const_1) + (state[0] * state[0])))",
        "(state[2] - ((state[0] * state[0]) + (state[1] * state[1])))",
        "(state[3] - ((state[1] * state[1]) + (state[2] * state[2])))",
        "(state[4] - ((state[2] * state[2]) + (state[3] * state[3])))",
    ];

    let deductions = [
        "Fib__6_input",
        "((const_1 * const_1) + (state[0] * state[0]))",
        "((state[0] * state[0]) + (state[1] * state[1]))",
        "((state[1] * state[1]) + (state[2] * state[2]))",
        "((state[2] * state[2]) + (state[3] * state[3]))",
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

    let (state, output) = registry.run_air(&air_fn, expr!("secret", 1));
    assert!(output.calc() == "866");
    assert!(state.calc() == ["1", "2", "5", "29", "866"]);
}
