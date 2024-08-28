use super::fib::*;
use crate::airs::examples::fibonacci::wide_fib::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

use crate::expr;

#[test]
fn test_wide_fibonacci() {
    let air_fn = WideFib {
        num_narrow: 2,
        narrow_size: 2,
    };
    let registry = AirFnRegistry::new(&air_fn);
    let lists = registry.get_compiled_air_fn(&air_fn.name());
    let (_, output) = registry.run_air(&air_fn, expr!("secret", 1));

    let constraints = [
        "NarrowFib_num_steps_2([const_1, state[0]]) == [state[1], state[2]]",
        "NarrowFib_num_steps_2([state[1], state[2]]) == [state[3], state[4]]",
    ];

    let deductions = [
        "WideFib_num_narrow_2_narrow_size_2_input",
        "tmp_1 = NarrowFib_num_steps_2([const_1, state[0]])",
        "tmp_1[0]",
        "tmp_1[1]",
        "tmp_2 = NarrowFib_num_steps_2([state[1], state[2]])",
        "tmp_2[0]",
        "tmp_2[1]",
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
    assert!(output.calc() == *"866");
}

#[test]
fn test_fibonacci() {
    let air_fn = Fib { claim_index: 6 };
    let registry = AirFnRegistry::new(&air_fn);
    let lists = registry.get_compiled_air_fn(&air_fn.name());

    let constraints = [
        "(state[1] - (const_1 + (state[0] * state[0])))",
        "(state[2] - ((state[0] * state[0]) + (state[1] * state[1])))",
        "(state[3] - ((state[1] * state[1]) + (state[2] * state[2])))",
        "(state[4] - ((state[2] * state[2]) + (state[3] * state[3])))",
    ];

    let deductions = [
        "Fib_claim_index_6_input",
        "(const_1 + (state[0] * state[0]))",
        "((state[0] * state[0]) + (state[1] * state[1]))",
        "((state[1] * state[1]) + (state[2] * state[2]))",
        "((state[2] * state[2]) + (state[3] * state[3]))",
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

    let (state, output) = registry.run_air(&air_fn, expr!("secret", 1));
    assert_eq!(output.calc(), "866");
    assert_eq!(state.calc(), ["1", "2", "5", "29", "866"]);
}
