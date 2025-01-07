use compiled_casm_air::compiled_structs::CompiledAirVar;
use compiled_casm_air::utils::INTERMEDIATE_VAR_SUFFIX;
use prover_types::cpu::Bool;

use super::expressions::bool_expr::*;
use super::expressions::felt_expr::*;
use super::variables::*;
// Macros
use crate::bool_expr;

#[test]
fn test_expr_array() {
    let mut array = [expr!("x", 8), expr!("y", 8)];

    // Let for deduction should change the element's names.
    assert_eq!(CompiledAirVar::from(array[0].clone()).to_string(), "x");
    assert_eq!(CompiledAirVar::from(array[1].clone()).to_string(), "y");
    let prefix = format!("{}{}", INTERMEDIATE_VAR_SUFFIX, 0);
    array = array.let_(prefix.clone(), Visibility::default());
    assert_eq!(
        CompiledAirVar::from(array[0].clone()).to_string(),
        format!("{}{}", prefix.clone(), "[0]")
    );
    assert_eq!(
        CompiledAirVar::from(array[1].clone()).to_string(),
        format!("{}{}", prefix, "[1]")
    );

    // Expressions should be marked as "in state" only if *all* of its elements changed to state.
    assert!(!array.in_state());
    array[0].to_state(StateInfo::StateIndex(0, None));
    assert!(!array.in_state());
    array[1].to_state(StateInfo::StateIndex(1, None));
    assert!(array.in_state());

    // As felts should return the same expression elements as felts.
    let val0 = array[0].calc();
    let val1 = array[1].calc();
    let felts_vec = array.as_felts();
    assert_eq!(felts_vec[0].calc(), val0);
    assert_eq!(felts_vec[1].calc(), val1);
}

#[test]
fn test_expr_tuple() {
    // Tuples should be marked as "in state" only if *all* of its elements changed to state.
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    assert!(!tup.in_state());
    tup.0.as_felt_mut().to_state(StateInfo::StateIndex(0, None));
    assert!(!tup.in_state());
    tup.1.to_state(StateInfo::StateIndex(1, None));
    assert!(tup.in_state());

    // Assert let for deduction changes the element's names.
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    assert_eq!(CompiledAirVar::from(tup.0.clone()).to_string(), "y");
    assert_eq!(CompiledAirVar::from(tup.1.clone()).to_string(), "x");
    let prefix = format!("{}{}", INTERMEDIATE_VAR_SUFFIX, 0);
    tup = tup.let_(prefix.clone(), Visibility::default());
    assert_eq!(
        CompiledAirVar::from(tup.0).to_string(),
        format!("{}{}", prefix.clone(), ".0")
    );
    assert_eq!(
        CompiledAirVar::from(tup.1).to_string(),
        format!("{}{}", prefix, ".1")
    );

    // Assert as felts return the vector elements as felts.
    let tup = (bool_expr!("y", true), expr!("x", 5));
    let val1 = tup.1.calc();
    let felts_vec = tup.as_felts();
    assert_eq!(felts_vec[0].calc(), "1");
    assert_eq!(felts_vec[1].calc(), val1);
}
