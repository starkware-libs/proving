use std::ops::IndexMut;

use super::air_fn_registry::*;
use super::expressions::bool_expr::*;
use super::expressions::expr::*;
use super::expressions::felt_expr::*;
use super::prover_types::*;
use super::variables::*;

// Macros
use crate::{bool_expr, expr};

#[test]
fn test_expr_array_vec() {
    let array = [expr!("x", 8), expr!("y", 8)];
    test_in_state(array.clone());
    test_let_for_deduction(array.clone());
    test_as_felt(array.clone());

    let vec = vec![expr!("x", 5), expr!("y", 5)];
    test_in_state(vec.clone());
    test_let_for_deduction(vec.clone());
    test_as_felt(vec.clone());
}

// Expressions should be marked as "in state" only if *all* of its elements changed to state.
fn test_in_state<T>(mut expr: T)
where
    T: AirVar + IndexMut<usize, Output = FeltExpr>,
{
    assert!(!expr.in_state());
    expr[0].to_state(0);
    assert!(!expr.in_state());
    expr[1].to_state(1);
    assert!(expr.in_state());
}

// Let for deduction should change the element's names.
fn test_let_for_deduction<T>(mut expr: T)
where
    T: AirVar + IndexMut<usize, Output = FeltExpr>,
{
    assert!(expr[0].name() == "x");
    assert!(expr[1].name() == "y");
    let prefix = format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0);
    expr = expr.let_for_deduction(prefix.clone());
    assert!(expr[0].name() == format!("{}{}", prefix.clone(), "[0]"));
    assert!(expr[1].name() == format!("{}{}", prefix, "[1]"));
}

// As felts should return the same expression elements as felts.
fn test_as_felt<T>(mut expr: T)
where
    T: AirVar + IndexMut<usize, Output = FeltExpr>,
{
    let val0 = expr[0].calc();
    let val1 = expr[1].calc();
    let felts_vec = expr.as_felts();
    assert!(felts_vec[0].calc() == val0);
    assert!(felts_vec[1].calc() == val1);
}

#[test]
fn test_expr_tuple() {
    // Tuples should be marked as "in state" only if *all* of its elements changed to state.
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    assert!(!tup.in_state());
    tup.0.as_felt().to_state(0);
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
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    let val1 = tup.1.calc();
    let felts_vec = tup.as_felts();
    assert!(felts_vec[0].calc() == "1");
    assert!(felts_vec[1].calc() == val1);
}

#[test]
fn test_option() {
    let mut opt = Some(expr!("x", 5));
    assert_eq!(&opt.name(), "Some(x)");
    let felts_vec = opt.as_mut().unwrap().as_felts();
    assert_eq!(&felts_vec[0].calc(), "5");

    let name = format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0);
    opt = opt.let_for_deduction(name.clone());
    assert_eq!(opt.as_ref().unwrap().name(), name);

    assert!(!opt.in_state());
    opt.as_mut().unwrap().to_state(0);
    assert!(opt.in_state());
}
