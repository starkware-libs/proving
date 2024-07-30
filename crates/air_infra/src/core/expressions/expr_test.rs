use super::super::air_fn_registry::*;
use super::super::compiled_structs::*;
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
use crate::{
    bool_expr, const_expr, const_felt252_expr, const_u32_expr, const_u64_expr, expr, felt252_expr,
    u32_expr, u64_expr,
};

#[test]
fn test_add_sub() {
    let a = const_expr!(1);
    let b = const_expr!(2);
    let c = a.clone() + b.clone();
    assert_eq!(c.calc(), 3.to_string());
    let d = c - b;
    assert_eq!(d.calc(), a.calc());
}

#[test]
fn test_mul_div() {
    let a = const_expr!(2);
    let b = const_expr!(3);
    let c = a.clone() * b.clone();
    assert_eq!(c.calc(), 6.to_string());
    let d = c / b;
    assert_eq!(d.calc(), a.calc());
}

#[test]
fn test_mod_sub() {
    let a = const_expr!(5);
    let b = const_expr!(3);
    let c = b - a;
    let res = 3 + PRIME - 5;
    assert_eq!(c.calc(), res.to_string());
}

#[test]
fn test_bool_not() {
    let a = bool_expr!("a".to_string(), true);
    let b = !a;
    assert_eq!(b.calc(), "false");
}

#[test]
fn test_uint32() {
    let a = const_u32_expr!(0xFFFF);
    let b = const_u32_expr!(1);
    let c = a + b;
    assert_eq!(c.calc(), (0xFFFFu32 + 1).to_string());

    let mut res = u32_expr!("c".to_string(), 0xFFFF + 1);
    assert_eq!(res.low_mut().calc(), 0.to_string());
    assert_eq!(res.high_mut().calc(), 1.to_string());
}

#[test]
fn test_uint64() {
    let a = const_u64_expr!(0xFFFFFFFF);
    let b = const_u64_expr!(1);
    let c = a + b;
    assert_eq!(c.calc(), (0xFFFFFFFFu64 + 1).to_string());

    let mut res = u64_expr!("c".to_string(), 0xFFFFFFFF + 1);
    assert_eq!(res.low_mut().calc(), 0.to_string());
    assert_eq!(res.high_mut().calc(), 1.to_string());
}

#[test]
fn test_felt252() {
    let mut v: Felt252Expr = felt252_expr!("v".to_string(), 0xFFF, 0xFFF);

    let felts = v.as_felts_mut();
    assert_eq!(
        felts.iter().map(|f| f.calc()).collect::<Vec<String>>(),
        [
            "4095", "0", "0", "0", "0", "0", "0", "0", "0", "0", "3840", "255", "0", "0", "0", "0",
            "0", "0", "0", "0", "0"
        ]
    );
}

#[test]
fn test_felt252_ops() {
    let a = const_felt252_expr!(1, 2);
    let b = const_felt252_expr!(3, 4);
    assert_eq!(
        (a.clone() + b.clone()).calc(),
        const_felt252_expr!(4, 6).calc()
    );
    assert_eq!(
        (a.clone() - b.clone()).calc(),
        const_felt252_expr!(
            340282366920938463463374607431768211455u128,
            10633823966279327296825105735305134077u128
        )
        .calc()
    );
    assert_eq!(
        (a.clone() * b.clone()).calc(),
        const_felt252_expr!(
            340282366920938463463374607431768211204u128,
            10633823966279247016594896951336501257u128
        )
        .calc()
    );
    assert_eq!(
        (a.clone() / b.clone()).calc(),
        const_felt252_expr!(
            301589100446816170481308260342883819822u128,
            4410038550152422379901404080731837379u128
        )
        .calc()
    );
    let c = const_felt252_expr!(1, 0);
    let d = const_felt252_expr!(0, 10633823966279327296825105735305134080u128);
    assert_eq!((c + d).calc(), const_felt252_expr!(0, 0).calc());
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_felt252_division_by_zero() {
    let _div = const_felt252_expr!(3, 4) / const_felt252_expr!(0, 0);
}

#[test]
fn test_conversion_felt_to_bool() {
    let mut f = expr!("x", 1, true);
    let b: BoolExpr = f.clone().into();
    assert_eq!(b.calc(), "true");
    assert!(b.in_state());
    let compiled_felt: CompiledAirVar = b.as_felt().into();
    assert_eq!(&compiled_felt.to_string(), "state[0]");
    let compiled_bool: CompiledAirVar = b.into();
    assert_eq!(&compiled_bool.to_string(), "Bool::from_m31(state[0])");

    f = f.let_for_constraint(format!("{}0", CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
    let b: BoolExpr = f.into();
    assert_eq!(b.calc(), "true");
    assert!(b.in_state());
    let compiled_felt: CompiledAirVar = b.as_felt().into();
    assert_eq!(&compiled_felt.to_string(), "constraint_tmp_0");
    let compiled_bool: CompiledAirVar = b.into();
    assert_eq!(
        &compiled_bool.to_string(),
        "Bool::from_m31(constraint_tmp_0)"
    );
}

#[test]
#[should_panic(expected = "M31 value is not a bool")]
fn test_bad_felt_to_bool() {
    let f = expr!("x", 2);
    let _b: BoolExpr = f.into();
}

#[test]
fn test_conversion_bool_to_uint16() {
    let mut b: BoolExpr = bool_expr!("x", true);
    b = b.let_for_deduction(format!("{}0", DEDUCTION_INTERMEDIATE_VAR_PREFIX));
    let i: UInt16Expr = b.clone().into();
    assert_eq!(i.calc(), "1");
    let compiled_felt: CompiledAirVar = i.as_felt().into();
    assert_eq!(&compiled_felt.to_string(), "deduction_tmp_0.as_m31()");

    b.as_felt_mut().to_state(0);
    let mut i: UInt16Expr = b.clone().into();
    assert!(i.in_state());
    let compiled_felt: CompiledAirVar = i.as_felt().into();
    assert_eq!(&compiled_felt.to_string(), "state[0]");
    let compiled_i: CompiledAirVar = i.into();
    assert_eq!(
        &compiled_i.to_string(),
        "UInt16::from_bool(deduction_tmp_0)"
    );

    let f = b
        .as_felt()
        .let_for_constraint(format!("{}0", CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
    i = Into::<BoolExpr>::into(f).into();
    let compiled_felt: CompiledAirVar = i.as_felt().into();
    assert_eq!(&compiled_felt.to_string(), "constraint_tmp_0");
}

#[test]
fn test_conversion_felt_to_uint16() {
    let mut f = expr!("x", 0xFF);
    f = f.let_for_deduction(format!("{}0", DEDUCTION_INTERMEDIATE_VAR_PREFIX));
    let i: UInt16Expr = f.clone().into();
    assert_eq!(i.calc(), "255");
    let compiled_felt: CompiledAirVar = i.as_felt().clone().into();
    assert_eq!(&compiled_felt.to_string(), "deduction_tmp_0");

    f.to_state(0);
    let mut i: UInt16Expr = f.clone().into();
    assert!(i.in_state());
    let compiled_felt: CompiledAirVar = i.as_felt().into();
    assert_eq!(&compiled_felt.to_string(), "state[0]");
    let compiled_i: CompiledAirVar = i.into();
    assert_eq!(&compiled_i.to_string(), "UInt16::from_m31(state[0])");

    let f = f.let_for_constraint(format!("{}0", CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
    i = f.into();
    let compiled_felt: CompiledAirVar = i.as_felt().into();
    assert_eq!(&compiled_felt.to_string(), "constraint_tmp_0");
    let compiled_i: CompiledAirVar = i.into();
    assert_eq!(
        &compiled_i.to_string(),
        "UInt16::from_m31(constraint_tmp_0)"
    );
}

#[test]
#[should_panic(expected = "M31 value is not a u16")]
fn test_bad_felt_to_uint16() {
    let f = expr!("x", 0xFFFF1);
    let _i: UInt16Expr = f.into();
}

#[test]
fn test_conversion_felts_to_felt252() {
    let mut f1 = expr!("x1", 1, true);
    let mut f2 = expr!("x2", 2, false);
    let mut e = Felt252Expr::from(vec![f1.clone(), f2.clone()]);
    assert_eq!(e.calc(), "(8193, 0)");
    assert_eq!(e.as_felts()[0].calc(), f1.calc());
    assert_eq!(e.as_felts()[1].calc(), f2.calc());
    assert!(!e.in_state());
    let compiled_felt1: CompiledAirVar = e.as_felts_mut()[0].clone().into();
    assert_eq!(&compiled_felt1.to_string(), "state[0]");
    let compiled_felt2: CompiledAirVar = e.as_felts_mut()[1].clone().into();
    assert_eq!(&compiled_felt2.to_string(), "x2");
    let compiled_expr: CompiledAirVar = e.into();
    assert_eq!(
        &compiled_expr.to_string(),
        "Felt252::from_m31_(zero_extend([state[0], x2]))"
    );

    f2 = expr!("x2", 2, true);
    f1 = f1.let_for_constraint(format!("{}0", CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
    let mut e = Felt252Expr::from(vec![f1.clone(), f2.clone()]);
    assert!(e.in_state());
    let compiled_felt1: CompiledAirVar = e.as_felts_mut()[0].clone().into();
    assert_eq!(&compiled_felt1.to_string(), "constraint_tmp_0");
    let compiled_felt2: CompiledAirVar = e.as_felts_mut()[0].clone().into();
    assert_eq!(&compiled_felt2.to_string(), "constraint_tmp_0");
    let compiled_expr: CompiledAirVar = e.into();
    assert_eq!(
        &compiled_expr.to_string(),
        "Felt252::from_m31_(zero_extend([constraint_tmp_0, state[0]]))"
    );

    let mut v: Felt252Expr = felt252_expr!("v".to_string(), 0xFFF, 0xFFF);
    let felts = v.as_felts();
    let mut e = Felt252Expr::from(felts);
    for (i, f) in e.as_felts_mut().iter().enumerate() {
        assert_eq!(f.calc(), v.as_felts_mut()[i].calc());
    }
}
