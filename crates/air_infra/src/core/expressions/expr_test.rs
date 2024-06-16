use super::super::air_fn_registry::*;
use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt252_expr::*;
use super::felt_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
// Macros
use crate::{const_expr, const_u32_expr, const_u64_expr, expr, felt252_expr};

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

#[test]
fn test_uint32() {
    let a: &UInt32Expr = &const_u32_expr!(0xFFFF);
    let b = &const_u32_expr!(1);
    let c = a + b;
    assert_eq!(c.calc(), (0xFFFFu32 + 1).to_string());

    let mut res = UInt32Expr::new_var("c".to_string(), Some(UInt32::from(0xFFFF + 1)), None, None);
    assert_eq!(res.low().calc(), 0.to_string());
    assert_eq!(res.high().calc(), 1.to_string());
}

#[test]
fn test_uint64() {
    let a: &UInt64Expr = &const_u64_expr!(0xFFFFFFFF);
    let b = &const_u64_expr!(1);
    let c = a + b;
    assert_eq!(c.calc(), (0xFFFFFFFFu64 + 1).to_string());

    let mut res = UInt64Expr::new_var(
        "c".to_string(),
        Some(UInt64::from(0xFFFFFFFF + 1)),
        None,
        None,
        None,
        None,
    );
    assert_eq!(res.low().calc(), 0.to_string());
    assert_eq!(res.high().calc(), 1.to_string());
}

#[test]
fn test_felt252() {
    let mut v: Felt252Expr = felt252_expr!("v".to_string(), 0xFFF, 0xFFF);

    let felts = v.as_felts();
    assert_eq!(
        felts.iter().map(|f| f.calc()).collect::<Vec<String>>(),
        [
            "4095", "0", "0", "0", "0", "0", "0", "0", "0", "0", "3840", "255", "0", "0", "0", "0",
            "0", "0", "0", "0", "0"
        ]
    );
}

#[test]
fn test_conversion_felt_to_bool() {
    let mut f = expr!("x", 1, true);
    let mut b = f.clone().as_bool();
    assert_eq!(b.calc(), "true");
    assert!(b.in_state());
    let compiled_felt: ProcessedAirVar = b.as_felt().clone().into();
    assert_eq!(compiled_felt.to_string(), "state[0]".to_string());
    let compiled_bool: ProcessedAirVar = b.into();
    assert_eq!(
        compiled_bool.to_string(),
        "felt_as_bool(state[0])".to_string()
    );

    f = f.let_for_constraint(format!("{}0", CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
    let mut b = f.as_bool();
    assert_eq!(b.calc(), "true");
    assert!(b.in_state());
    let compiled_felt: ProcessedAirVar = b.as_felt().clone().into();
    assert_eq!(compiled_felt.to_string(), "constraint_tmp_0".to_string());
    let compiled_bool: ProcessedAirVar = b.into();
    assert_eq!(
        compiled_bool.to_string(),
        "felt_as_bool(constraint_tmp_0)".to_string()
    );
}
