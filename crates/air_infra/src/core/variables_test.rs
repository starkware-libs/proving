use super::air_fn_registry::*;
use super::expressions::bool_expr::*;
use super::expressions::expr::*;
use super::expressions::felt_expr::*;
use super::prover_types::*;
use super::variables::*;

// Macros
use crate::{bool_expr, expr};

#[test]
fn test_expr_array() {
    let mut array = [expr!("x", 8), expr!("y", 8)];

    // Let for deduction should change the element's names.
    assert_eq!(&array[0].name(), "x");
    assert_eq!(&array[1].name(), "y");
    let prefix = format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0);
    array = array.let_for_deduction(prefix.clone());
    assert_eq!(array[0].name(), format!("{}{}", prefix.clone(), "[0]"));
    assert_eq!(array[1].name(), format!("{}{}", prefix, "[1]"));

    // Expressions should be marked as "in state" only if *all* of its elements changed to state.
    assert!(!array.in_state());
    array[0].to_state(0);
    assert!(!array.in_state());
    array[1].to_state(1);
    assert!(array.in_state());

    // As felts should return the same expression elements as felts.
    let val0 = array[0].calc();
    let val1 = array[1].calc();
    let felts_vec = array.as_felts();
    assert!(felts_vec[0].calc() == val0);
    assert!(felts_vec[1].calc() == val1);
}

#[test]
fn test_expr_tuple() {
    // Tuples should be marked as "in state" only if *all* of its elements changed to state.
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    assert!(!tup.in_state());
    tup.0.as_felt_mut().to_state(0);
    assert!(!tup.in_state());
    tup.1.to_state(1);
    assert!(tup.in_state());

    // Assert let for deduction changes the element's names.
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    assert!(tup.0.name() == "y");
    assert!(tup.1.name() == "x");
    let prefix = format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0);
    tup = tup.let_for_deduction(prefix.clone());
    assert!(tup.0.name() == format!("{}{}", prefix.clone(), ".0"));
    assert!(tup.1.name() == format!("{}{}", prefix, ".1"));

    // Assert as felts return the vector elements as felts.
    let tup = (bool_expr!("y", true), expr!("x", 5));
    let val1 = tup.1.calc();
    let felts_vec = tup.as_felts();
    assert!(felts_vec[0].calc() == "1");
    assert!(felts_vec[1].calc() == val1);
}
