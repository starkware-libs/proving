use super::super::prover_types::*;
use super::expr::*;
use super::felt_expr::*;
// Macros
use crate::const_expr;

#[test]
fn test_add_sub() {
    let a: &FeltExpr = &const_expr!(1);
    let b = &const_expr!(2);
    let c = a + b;
    assert_eq!(c.calc(), 3.to_string());
    let d = &c - b;
    assert_eq!(d.calc(), a.calc());
}

#[test]
fn test_mul_div() {
    let a: &FeltExpr = &const_expr!(2);
    let b = &const_expr!(3);
    let c = a * b;
    assert_eq!(c.calc(), 6.to_string());
    let d = &c / b;
    assert_eq!(d.calc(), a.calc());
}

#[test]
fn test_mod_sub() {
    let a: &FeltExpr = &const_expr!(5);
    let b = &const_expr!(3);
    let c = b - a;
    let res = 3 + PRIME - 5;
    assert_eq!(c.calc(), res.to_string());
}
