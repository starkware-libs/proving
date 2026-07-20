use stwo_cairo_common::prover_types::cpu::{BigUInt, Bool, Felt252, PRIME, UInt32, UInt64};

use super::biguint_expr::*;
use super::bool_expr::*;
use super::felt_expr::*;
use super::felt252_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
use crate::core::air_body::*;
use crate::core::air_fn::*;
use crate::core::variables::*;
// Macros
use crate::{
    bool_expr, const_bigu384_expr, const_bigu768_expr, const_expr, const_felt252_expr,
    const_u32_expr, const_u64_expr,
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
fn test_invert() {
    let exp = 13;
    let a = const_expr!(1 << exp);
    let inv = a.inverse();
    let res = 1u32 << (31 - exp);
    assert_eq!(inv.calc(), res.to_string());
}

#[test]
fn test_invert_arbitrary_number() {
    let a = const_expr!(2098765432);
    let inv = a.clone().inverse();
    let res = const_expr!(1) / a;
    assert_eq!(inv.calc(), res.calc());
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_invert_zero() {
    let a = const_expr!(0);
    let _inv = a.inverse();
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
    assert_eq!(&res.low().to_string(), "c.low()");
    assert_eq!(res.high_mut().calc(), 1.to_string());
    assert_eq!(&res.high().to_string(), "c.high()");

    let d = UInt32Expr::from(const_expr!(0xFFFFFF));
    assert_eq!(d.calc(), 0xFFFFFF.to_string());

    let d = UInt32Expr::from(vec![const_expr!(0), const_expr!(0x1)]);
    assert_eq!(d.calc(), 65536.to_string());
    assert_eq!(d.as_felts()[1].calc(), 1.to_string());
    assert_eq!(&d.as_felts()[1].clone().to_string(), "1");
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
            "511", "7", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "508", "31",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
        ]
    );
}

#[test]
fn test_felt252_ops() {
    let a = const_felt252_expr!(1, 2);
    let b = const_felt252_expr!(3, 4);
    assert_eq!((a.clone() + b.clone()).calc(), const_felt252_expr!(4, 6).calc());
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
    let e = const_felt252_expr!(
        0xffffffff_ffffffff_ffffffff_ffffffffu128,
        0xffffffff_ffffffff_ffffffff_ffffffffu128
    );
    assert_eq!(
        (e.clone() + e).calc(),
        const_felt252_expr!(
            0xffffffff_ffffffff_ffffffff_fffffffbu128,
            0x37ffffff_ffffffcc_ffffffff_ffffffffu128
        )
        .calc()
    );
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_felt252_division_by_zero() {
    let _div = const_felt252_expr!(3, 4) / const_felt252_expr!(0, 0);
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_felt252_division_by_p() {
    let _div = const_felt252_expr!(3, 4)
        / const_felt252_expr!(1, 0x08000000_00000011_00000000_00000000u128);
}

#[test]
fn test_conversion_felt_to_bool() {
    let f = const_expr!(1);
    let b: BoolExpr = f.into();
    assert_eq!(b.calc(), "true");
    assert!(b.as_felt().in_state());

    assert_eq!(&b.as_felt().to_string(), "1");
    assert_eq!(&b.to_string(), "true");

    let f = expr!("x", 1);
    let b: BoolExpr = f.clone().into();
    assert_eq!(&b.as_felt().to_string(), "x");
    assert_eq!(&b.to_string(), "Bool::from_m31(x)");

    let f = f.let_for_deduction(format!("{INTERMEDIATE_VAR_SUFFIX}0")).0;
    let b: BoolExpr = f.into();
    assert_eq!(b.calc(), "true");
    assert_eq!(&b.as_felt().to_string(), "tmp0");
    assert_eq!(&b.to_string(), "Bool::from_m31(tmp0)");
}

#[test]
#[should_panic(expected = "M31 value is not a bool")]
fn test_bad_felt_to_bool() {
    let f = expr!("x", 2);
    let _b: BoolExpr = f.into();
}

#[test]
#[should_panic(expected = "Operation Eq does not allow extracting felts")]
fn test_bad_bool_as_felt() {
    let b = const_expr!(1).eq(const_expr!(2));
    let _f = b.as_felt();
}

#[test]
fn test_conversion_bool_to_uint16() {
    let mut b: BoolExpr = bool_expr!("x", true);
    b = b.let_for_deduction(format!("{INTERMEDIATE_VAR_SUFFIX}0")).0;
    let i: UInt16Expr = b.clone().into();
    assert_eq!(i.calc(), "1");
    assert_eq!(&i.to_string(), "UInt16::from_bool(tmp0)");

    b.as_felt_mut().set_value(ValueInfo::StateIndex(0, None));
    let mut i: UInt16Expr = b.clone().into();
    i = i.let_for_deduction("tmp0".to_string()).0;
    assert!(!i.as_felt().in_state());
}

#[test]
fn test_conversion_felt_to_uint16() {
    let mut f = expr!("x", 0xFF);
    f = f.let_for_deduction(format!("{INTERMEDIATE_VAR_SUFFIX}0")).0;
    let i: UInt16Expr = f.clone().into();
    assert_eq!(i.calc(), "255");
    assert_eq!(&i.as_felt().to_string(), "tmp0");

    f.set_value(ValueInfo::StateIndex(0, None));
    let mut i: UInt16Expr = f.clone().into();
    assert!(i.as_felt().in_state());
    assert_eq!(&i.as_felt().to_string(), "col0");
    assert_eq!(&i.to_string(), "UInt16::from_m31(col0)");

    let f = f.let_for_deduction(format!("{INTERMEDIATE_VAR_SUFFIX}0")).0;
    i = f.into();
    assert_eq!(&i.as_felt().to_string(), "col0");
    assert_eq!(&i.to_string(), "UInt16::from_m31(col0)");
}

#[test]
fn test_in_state() {
    let mut x = u32_expr!("x".to_string(), 0xFFFF);
    assert!(!x.low().as_felt().in_state());

    x.low_mut().as_felt_mut().set_value(ValueInfo::StateIndex(0, None));
    assert!(x.low().as_felt().in_state());
    assert!(!x.high().as_felt().in_state());

    x = x.let_for_deduction("new_x".to_string()).0;
    assert!(x.low().as_felt().in_state());
    assert!(!x.high().as_felt().in_state());

    x.high_mut().as_felt_mut().set_value(ValueInfo::StateIndex(1, None));
    assert!(x.low().as_felt().in_state());
    assert!(x.high().as_felt().in_state());

    x = x.let_for_deduction("newer_x".to_string()).0;
    assert!(x.low().as_felt().in_state());
    assert!(x.high().as_felt().in_state());
}

#[test]
fn test_is_const() {
    let mut x: UInt32Expr = vec![const_expr!(0), expr!("x", 0xFFFF)].into();
    assert!(!x.is_const());
    assert!(x.get_felt(0).is_const());

    x = x.let_for_deduction("new_x".to_string()).0;
    assert!(!x.is_const());
    assert!(x.get_felt(0).is_const());
}

#[test]
fn test_degree_in_state() {
    let mut f = expr!("x", 17);
    f = f.let_for_deduction(format!("{INTERMEDIATE_VAR_SUFFIX}0")).0;
    assert!(!f.in_state());
    assert!(f.deg_in_state().is_none());

    f.set_value(ValueInfo::StateIndex(0, None));
    assert!(f.in_state());
    assert_eq!(f.deg_in_state(), Some(1));

    let f2 = f.clone() * f.clone();
    assert!(f2.in_state());
    assert_eq!(f2.deg_in_state(), Some(2));

    let c = const_expr!(32);
    assert!(c.in_state());
    assert_eq!(c.deg_in_state(), Some(0));

    let f3 = f.clone() * f.clone() * f.clone();
    assert_eq!(f3.deg_in_state(), Some(3));

    let f4 = f3.clone() - f2.clone() + f.clone() - c.clone();
    assert!(f4.in_state());
    assert_eq!(f4.deg_in_state(), Some(3));

    let x = c.clone() * f.clone();
    assert_eq!(x.deg_in_state(), Some(1));

    let x = f.clone() / c.clone();
    assert_eq!(x.deg_in_state(), Some(1));

    let x = c.clone() / f.clone();
    assert_eq!(x.deg_in_state(), None);
}

#[test]
#[should_panic(expected = "M31 value is not a u16")]
fn test_bad_felt_to_uint16() {
    let f = expr!("x", 0xFFFF1);
    let _i: UInt16Expr = f.into();
}

#[test]
fn test_conversion_felt_to_felt252() {
    let f = const_expr!(0xFFFFFFF);
    let i: Felt252Expr = f.into();
    assert_eq!(i.calc(), format!("[{}, 0, 0, 0]", 0xFFFFFFF));
}

#[test]
fn test_conversion_felts_to_felt252() {
    let f1 = const_expr!(1);
    let mut f2 = expr!("x2", 2);
    let mut e = Felt252Expr::from(vec![f1.clone(), f2.clone()]);
    assert_eq!(e.calc(), "[1025, 0, 0, 0]");
    assert_eq!(e.as_felts()[0].calc(), f1.calc());
    assert_eq!(e.as_felts()[1].calc(), f2.calc());
    assert!(!e.as_felts().iter().all(|f| f.in_state()));
    assert_eq!(&e.as_felts_mut()[0].to_string(), "1");
    assert_eq!(&e.get_felt_mut(1).to_string(), "x2");
    assert_eq!(&e.to_string(), "Felt252::from_limbs(zero_extend([1, x2]))");

    f2 = f2.let_for_deduction(format!("{INTERMEDIATE_VAR_SUFFIX}0")).0;
    let mut e = Felt252Expr::from(vec![f1.clone(), f2.clone()]);
    assert_eq!(&e.as_felts_mut()[0].to_string(), "1");
    assert_eq!(&e.get_felt_mut(1).to_string(), "tmp0");
    assert_eq!(&e.to_string(), "Felt252::from_limbs(zero_extend([1, tmp0]))");

    f2.set_value(ValueInfo::StateIndex(1, None));
    let e = Felt252Expr::from(vec![f1.clone(), f2.clone()]);
    assert!(e.as_felts().iter().all(|f| f.in_state()));

    let mut v: Felt252Expr = felt252_expr!("v".to_string(), 0xFFF, 0xFFF);
    let felts = v.as_felts();
    let mut e = Felt252Expr::from(felts);
    for (i, f) in e.as_felts_mut().into_iter().enumerate() {
        assert_eq!(f.calc(), v.as_felts_mut()[i].calc());
        assert_eq!(f.calc(), v.get_felt_mut(i).calc());
    }
}

#[test]
fn test_biguint384() {
    let a = const_bigu384_expr!(1, 1, 0, 1, 0, 0);
    let b = const_bigu384_expr!(0, 1, 0, 1, 0, 0);
    let a_768: BigUInt768Expr = a.clone().into();
    let b_768: BigUInt768Expr = b.clone().into();

    assert_eq!(
        (BigUInt384Expr::from(a_768.clone() + b_768.clone())).calc(),
        "[1, 2, 0, 2, 0, 0]".to_string()
    );

    assert_eq!((a.clone() - b.clone()).calc(), "[1, 0, 0, 0, 0, 0]".to_string());

    assert_eq!(
        (a_768.clone() * b_768.clone()).calc(),
        "[0, 1, 1, 1, 2, 0, 1, 0, 0, 0, 0, 0]".to_string()
    );
    let c_768: BigUInt768Expr = a.clone().widening_mul(b.clone());
    assert_eq!(c_768.calc(), "[0, 1, 1, 1, 2, 0, 1, 0, 0, 0, 0, 0]".to_string());

    assert_eq!(
        ((a_768.clone() * b_768.clone())
            / BigUInt768Expr::from(const_bigu384_expr!(0, 1, 0, 0, 0, 0)))
        .calc(),
        "[1, 1, 1, 2, 0, 1, 0, 0, 0, 0, 0, 0]".to_string()
    );

    let f = const_felt252_expr!(1, 1);
    assert_eq!((BigUInt384Expr::from(f.clone())).calc(), "[1, 0, 1, 0, 0, 0]".to_string());
    assert_eq!(
        BigUInt768Expr::from(f.clone()).calc(),
        "[1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]".to_string()
    );

    assert_eq!(&BigUInt384Expr::from(f).to_string(), "[1, 0, 1, 0, 0, 0]");

    let f = felt252_expr!("x", 1, 1);
    assert_eq!(&BigUInt384Expr::from(f).to_string(), "BigUInt::<384, 6, 32>::from_felt252(x)");

    assert_eq!(
        format!(
            "{:?}",
            bigu384_expr!("v".to_string(), 1, 0, 1, 0, 0, 0).compile(CompileFor::Deductions)
        ),
        "Var(\"BigUInt<384, 6, 32>\", \"v\")"
    );

    let h = const_felt252_expr!(1, 1);
    let g = BigUInt384Expr::from(h.clone());
    assert_eq!(g.calc(), "[1, 0, 1, 0, 0, 0]".to_string());
    assert_eq!(g.clone().eq(const_bigu384_expr!(1, 0, 1, 0, 0, 0)).calc(), "true".to_string());
    assert_eq!(g.eq(const_bigu384_expr!(1, 0, 1, 0, 0, 1)).calc(), "false".to_string());

    let felt1 = bigu384_expr!("x", 1, 0, 1, 0, 0, 1).as_felts_mut()[0].clone();
    assert_eq!(&felt1.to_string(), "x.get_m31(0)");
    let felt2 = const_bigu384_expr!(1, 0, 1, 0, 0, 1).get_felt(0);
    assert_eq!(felt1.value(), felt2.value());
}

#[test]
#[should_panic(expected = "BigUInt is too big")]
fn test_bad_bigu768_to_bigu384() {
    let e = const_bigu768_expr!(0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0);
    let _: BigUInt384Expr = e.into();
}

#[test]
fn test_from252_vec() {
    let x0 = const_felt252_expr!(1, 0);
    let x1 = const_felt252_expr!(2, 0);
    let x2 = const_felt252_expr!(1u128 << 64, 0);
    let x3 = const_felt252_expr!(1u128 << 32, 0);
    let x: BigUInt384Expr = vec![x0.clone(), x1.clone(), x2.clone(), x3.clone()].into();
    assert_eq!(x.calc(), "[1, 8589934592, 0, 0, 1, 1]".to_string());
}

#[test]
fn test_felt_expr_binary_ops_with_const0_and_const1() {
    let x = expr!("x", 7);

    assert_eq!("(x + 1)", &(x.clone() + const_expr!(1)).to_string());
    assert_eq!("(1 + x)", &(const_expr!(1) + x.clone()).to_string());
    assert_eq!("x", &(x.clone() + const_expr!(0)).to_string());
    assert_eq!("x", &(const_expr!(0) + x.clone()).to_string());

    assert_eq!("(x - 1)", &(x.clone() - const_expr!(1)).to_string());
    assert_eq!("(1 - x)", &(const_expr!(1) - x.clone()).to_string());
    assert_eq!("x", &(x.clone() - const_expr!(0)).to_string());
    assert_eq!("(0 - x)", &(const_expr!(0) - x.clone()).to_string());

    assert_eq!("(x * 2)", &(x.clone() * const_expr!(2)).to_string());
    assert_eq!("(2 * x)", &(const_expr!(2) * x.clone()).to_string());
    assert_eq!("x", &(x.clone() * const_expr!(1)).to_string());
    assert_eq!("x", &(const_expr!(1) * x.clone()).to_string());
    assert_eq!("0", &(x.clone() * const_expr!(0)).to_string());
    assert_eq!("0", &(const_expr!(0) * x.clone()).to_string());
}
