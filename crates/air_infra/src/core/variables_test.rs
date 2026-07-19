use stwo_cairo_common::prover_types::cpu::Bool;

use super::air_fn::*;
use super::expressions::bool_expr::*;
use super::expressions::felt_expr::*;
use super::variables::*;
// Macros
use crate::bool_expr;

#[test]
fn test_expr_array() {
    let mut array = [expr!("x", 8), expr!("y", 8)];

    // Let for deduction should change the element's names.
    assert_eq!(array[0].clone().to_string(), "x");
    assert_eq!(array[1].clone().to_string(), "y");
    let prefix = format!("{}{}", INTERMEDIATE_VAR_SUFFIX, 0);
    array = array.let_for_deduction(prefix.clone()).0;
    assert_eq!(array[0].clone().to_string(), format!("{}{}", prefix.clone(), "[0]"));
    assert_eq!(array[1].clone().to_string(), format!("{}{}", prefix, "[1]"));

    // Expressions should be marked as "in state" only if *all* of its elements changed to state.
    assert!(!AirVarImpl::from(array.clone()).as_felts().iter().all(|f| f.in_state()));
    array[0].set_value(ValueInfo::StateIndex(0, None));
    assert!(!AirVarImpl::from(array.clone()).as_felts().iter().all(|f| f.in_state()));
    array[1].set_value(ValueInfo::StateIndex(1, None));
    assert!(AirVarImpl::from(array.clone()).as_felts().iter().all(|f| f.in_state()));

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
    assert!(!(AirVarImpl::from(tup.clone()).as_felts().iter().all(|f| f.in_state())));
    tup.0.as_felt_mut().set_value(ValueInfo::StateIndex(0, None));
    assert!(!AirVarImpl::from(tup.clone()).as_felts().iter().all(|f| f.in_state()));
    tup.1.set_value(ValueInfo::StateIndex(1, None));
    assert!(AirVarImpl::from(tup.clone()).as_felts().iter().all(|f| f.in_state()));

    // Assert let for deduction changes the element's names.
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    assert_eq!(tup.0.clone().to_string(), "y");
    assert_eq!(tup.1.clone().to_string(), "x");
    let prefix = format!("{}{}", INTERMEDIATE_VAR_SUFFIX, 0);
    tup = tup.let_for_deduction(prefix.clone()).0;
    assert_eq!(tup.0.to_string(), format!("{}{}", prefix.clone(), ".0"));
    assert_eq!(tup.1.to_string(), format!("{}{}", prefix, ".1"));

    // Assert as felts return the vector elements as felts.
    let tup = (bool_expr!("y", true), expr!("x", 5));
    let val1 = tup.1.calc();
    let felts_vec = tup.as_felts();
    assert_eq!(felts_vec[0].calc(), "1");
    assert_eq!(felts_vec[1].calc(), val1);
}
