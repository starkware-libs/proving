use super::super::air_fn_registry::*;
use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::bool_expr::*;
use super::expr::*;
use super::felt252_expr::*;
use super::felt_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
// Macros
use crate::{bool_expr, const_expr, const_u32_expr, const_u64_expr, expr, felt252_expr};
pub const DEDUCTION_INTERMEDIATE_VAR_PREFIX: &str = "deduction_tmp_";

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
fn test_bool_not() {
    let a: &BoolExpr = &bool_expr!("a".to_string(), true);
    let b = !a;
    assert_eq!(b.calc(), "false");
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
    let mut b: BoolExpr = f.clone().into();
    assert_eq!(b.calc(), "true");
    assert!(b.in_state());
    let compiled_felt: ProcessedAirVar = b.as_felt().clone().into();
    assert_eq!(compiled_felt.to_string(), "state[0]".to_string());
    let compiled_bool: ProcessedAirVar = b.into();
    assert_eq!(
        compiled_bool.to_string(),
        "Bool::from(state[0])".to_string()
    );

    f = f.let_for_constraint(format!("{}0", CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
    let mut b: BoolExpr = f.into();
    assert_eq!(b.calc(), "true");
    assert!(b.in_state());
    let compiled_felt: ProcessedAirVar = b.as_felt().clone().into();
    assert_eq!(compiled_felt.to_string(), "constraint_tmp_0".to_string());
    let compiled_bool: ProcessedAirVar = b.into();
    assert_eq!(
        compiled_bool.to_string(),
        "Bool::from(constraint_tmp_0)".to_string()
    );
}

#[test]
fn test_conversion_bool_to_uint16() {
    let mut b: BoolExpr = bool_expr!("x", true);
    b = b.let_for_deduction(format!("{}0", DEDUCTION_INTERMEDIATE_VAR_PREFIX));
    let mut i: UInt16Expr = b.clone().into();
    assert_eq!(i.calc(), "1");
    let compiled_felt: ProcessedAirVar = i.as_felt().clone().into();
    assert_eq!(
        compiled_felt.to_string(),
        "deduction_tmp_0.as_felt()".to_string() // This will be fixed in PR 68
    );

    b.as_felt().to_state(0);
    let mut i: UInt16Expr = b.clone().into();
    assert!(i.in_state());
    let compiled_felt: ProcessedAirVar = i.as_felt().clone().into();
    assert_eq!(compiled_felt.to_string(), "state[0]".to_string());
    let compiled_i: ProcessedAirVar = i.into();
    assert_eq!(
        compiled_i.to_string(),
        "UInt16::from(deduction_tmp_0)".to_string()
    );

    let f = b
        .as_felt()
        .let_for_constraint(format!("{}0", CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
    i = Into::<BoolExpr>::into(f).into();
    let compiled_felt: ProcessedAirVar = i.as_felt().clone().into();
    assert_eq!(compiled_felt.to_string(), "constraint_tmp_0".to_string());
}

#[test]
fn test_conversion_felt_to_felt252() {
    let mut f = expr!("x", 1, true);
    let mut e: Felt252Expr = f.clone().into();
    assert_eq!(e.calc(), "(1, 0)");
    assert!(e.in_state());
    let compiled_felt: ProcessedAirVar = e.as_felts()[0].clone().into();
    assert_eq!(compiled_felt.to_string(), "state[0]".to_string());
    let compiled_expr: ProcessedAirVar = e.into();
    assert_eq!(
        compiled_expr.to_string(),
        "Felt252::from(state[0])".to_string()
    );

    f = f.let_for_constraint(format!("{}0", CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
    let mut e: Felt252Expr = f.into();
    assert_eq!(e.calc(), "(1, 0)");
    assert!(e.in_state());
    let compiled_felt: ProcessedAirVar = e.as_felts()[0].clone().into();
    assert_eq!(compiled_felt.to_string(), "constraint_tmp_0".to_string());
    let compiled_expr: ProcessedAirVar = e.into();
    assert_eq!(
        compiled_expr.to_string(),
        "Felt252::from(constraint_tmp_0)".to_string()
    );
}


#[test]
fn test_expr_array() {

    // Array should be marked as "in state" only if *all* of its elements changed to state.
    let mut array = [expr!("x", 5), expr!("y", 5)]; 
    assert!(!array.in_state());
    array[0].to_state(0);
    assert!(!array.in_state());
    array[1].to_state(1);
    assert!(array.in_state());

    // Assert let for deduction changes the element's names.
    let mut array = [expr!("x", 5), expr!("y", 5)]; 
    assert!(array[0].name() == "x");
    assert!(array[1].name() == "y");
    array = array.let_for_deduction(format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0));
    assert!(array[0].name() == format!("{}{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0, "[0]"));
    assert!(array[1].name() == format!("{}{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0, "[1]"));

    // Assert as felts return the array elements as felts .
    let mut array = [expr!("x", 5), expr!("y", 5)]; 
    let felts_vec =array.as_felts();
    // Cannot compare to array[0].name since as_felts borrow immutable reference
    assert!(felts_vec[0].name() == (expr!("x", 5).name()));
    assert!(felts_vec[1].name() == (expr!("y", 5).name()));

}

#[test]
fn test_expr_vector() {

    // Vec should be marked as "in state" only if *all* of its elements changed to state.
    let mut vec =  Vec::<FeltExpr>::from([expr!("x", 5), expr!("y", 5)]);
    assert!(!vec.in_state());
    vec[0].to_state(0);
    assert!(!vec.in_state());
    vec[1].to_state(1);
    assert!(vec.in_state());

    // // Assert let for deduction changes the element's names.
    let mut vec =  Vec::<FeltExpr>::from([expr!("x", 5), expr!("y", 5)]);
    assert!(vec[0].name() == "x");
    assert!(vec[1].name() == "y");
    vec = vec.let_for_deduction(format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0));
    assert!(vec[0].name() == format!("{}{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0, "[0]"));
    assert!(vec[1].name() == format!("{}{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0, "[1]"));

    // Assert as felts return the vector elements as felts .
    let mut vec =  Vec::<FeltExpr>::from([expr!("x", 5), expr!("y", 5)]);
    let felts_vec =vec.as_felts();
    assert!(felts_vec[0].name() == (expr!("x", 5).name()));
    assert!(felts_vec[1].name() == (expr!("y", 5).name()));
}

#[test]
//problems here
fn test_expr_tuple() {

    // Tupples with bool should not be marked as "in state".
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    assert!(!tup.in_state());
    tup.1.to_state(0);
    assert!(!tup.in_state());

    // // Assert let for deduction changes the element's names.
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    assert!(tup.0.name() == "y");
    assert!(tup.1.name() == "x");
    tup = tup.let_for_deduction(format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0));
    println!("{}", tup.0.name());
    println!("{}", tup.1.name());
    assert!(tup.0.name() == format!("{}{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0, ".1"));
    assert!(tup.1.name() == format!("{}{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, 0, ".2"));

    // Assert as felts return the vector elements as felts .
    let mut tup = (bool_expr!("y", true), expr!("x", 5));
    let felts_vec =tup.as_felts();
    println!("{}", felts_vec[0].name() );
    assert!(felts_vec[0].name() == bool_expr!("y", true).name());
    assert!(felts_vec[1].name() == (expr!("x", 5).name()));
}

