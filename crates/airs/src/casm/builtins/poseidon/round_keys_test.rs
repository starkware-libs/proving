use std::process::Command;

use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use air_infra::{const_expr, const_felt252_expr};

use super::round_keys::*;

#[test]
fn test_round_keys() {
    let air_fn = PoseidonRoundKeys {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(&air_fn, [const_expr!(7)], ());
    let expected_output = [
        const_felt252_expr!(
            256874565931396631624738152782893432232,
            4352086033451412036946223669213047930
        ),
        const_felt252_expr!(
            163793251516503563982125799053952319746,
            6212478756318084830005273194119433888
        ),
        const_felt252_expr!(
            110539381072939588940455818857317321380,
            1145475375772662886444657706229960339
        ),
    ];
    for (out, exp_out) in output.into_iter().zip(expected_output) {
        assert_eq!(out.calc(), exp_out.calc());
    }

    assert_eq!(state.get_felts().len(), 1);
}

#[test]
fn test_key_generation_python_utils() {
    let py_test_filename = "src/casm/builtins/poseidon/poseidon_simulator.py";
    let status = Command::new("python3").arg(py_test_filename).status().unwrap();
    assert_eq!(status.code(), Some(0));
}
