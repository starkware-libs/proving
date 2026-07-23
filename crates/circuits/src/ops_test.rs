use expect_test::expect;
use num_traits::Zero;
use rstest::rstest;
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::P;

use crate::context::TraceContext;
use crate::eval;
use crate::ivalue::qm31_from_u32s;
use crate::ops::{
    Guess, add, cond_flip, conj, eq, from_partial_evals, guess, im, inv, mul, pointwise_mul,
};
use crate::stats::Stats;

#[test]
fn test_basic_ops() {
    let x = qm31_from_u32s(1, 2, 3, 4);
    let y = qm31_from_u32s(0, 5, 8, 20);

    let mut context = TraceContext::default();
    let a = context.new_var(x);
    let b = context.new_var(y);
    let c = eval!(&mut context, ((a) + (b)) * ((a) - (b)));
    assert_eq!(context.get(c), (x + y) * (x - y));

    let u = qm31_from_u32s(0, 0, 1, 0);
    assert_eq!(
        context.values(),
        &vec![0.into(), 1.into(), u, x, y, x + y, x - y, (x + y) * (x - y)]
    );

    context.validate_circuit();
}

#[test]
fn test_eq() {
    let x = qm31_from_u32s(1, 2, 3, 4);

    let mut context = TraceContext::default();
    let a = context.new_var(x);
    let b = context.new_var(x + x);
    // The following equality is wrong, it'll be caught by the circuit checker.
    eq(&mut context, a, b);
    context.circuit.check(context.values()).unwrap_err();
    let u = qm31_from_u32s(0, 0, 1, 0);
    context.circuit.check(&[0.into(), 1.into(), u, x, x]).unwrap();
}

#[test]
fn test_eval_macro() {
    let mut context = TraceContext::default();
    let a = context.new_var(10.into());
    let res = eval!(&mut context, (((a) * (20)) - ((2) * (3))) - (10));
    assert_eq!(context.get(res), 184.into());
}

#[test]
fn test_eval_neg() {
    let mut context = TraceContext::default();
    let a = context.new_var(10.into());
    let res = eval!(&mut context, (-(a)) + (13));
    assert_eq!(context.get(res), 3.into());
}

#[test]
fn test_inv() {
    let mut context = TraceContext::default();
    let x = guess(&mut context, 10.into());
    let y = guess(&mut context, 2.into());
    let y_inv = inv(&mut context, y);
    let res = mul(&mut context, x, y_inv);
    assert_eq!(context.get(res), 5.into());

    expect![[r#"
        [6] = [4] * [5]
        [7] = [3] * [5]
        [6] = [1]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);

    let context = context.finalize(false);
    context.validate_circuit();
    context.circuit().check_yields();
}

#[test]
fn test_pointwise_mul() {
    let mut context = TraceContext::default();
    let x = guess(&mut context, qm31_from_u32s(1, 2, 3, 4));
    let y = guess(&mut context, qm31_from_u32s(5, 6, 7, 8));
    let res = pointwise_mul(&mut context, x, y);
    assert_eq!(context.get(res), qm31_from_u32s(5, 12, 21, 32));
    context.validate_circuit();
}

#[rstest]
#[case::zero(0, (10, 20))]
#[case::one(1, (20, 10))]
fn test_select(#[case] selector: u32, #[case] result: (u32, u32)) {
    let mut context = TraceContext::default();
    let selector = guess(&mut context, selector.into());
    let a = guess(&mut context, 10.into());
    let b = guess(&mut context, 20.into());
    let res = cond_flip(&mut context, selector, a, b);
    assert_eq!(context.get(res.0), result.0.into());
    assert_eq!(context.get(res.1), result.1.into());
    context.validate_circuit();
}

#[test]
fn test_conj() {
    let mut context = TraceContext::default();
    let a = qm31_from_u32s(1, 2, 3, 4).guess(&mut context);
    let b = conj(&mut context, a);
    assert_eq!(context.get(b), qm31_from_u32s(1, 2, P - 3, P - 4));

    // Multiplying by the conjugate should result in a value in `CM31`.
    let c = eval!(&mut context, (a) * (b));
    assert_eq!(context.get(c).1, CM31::zero());

    expect!["[5], [6]"].assert_eq(&format!("{b:?}, {c:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (1 + 1i) + (2147483646 + 2147483646i)u: [4],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [6] = [3] * [5]
        [5] = [3] x [4]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);

    context.validate_circuit();
}

#[test]
fn test_im() {
    let mut context = TraceContext::default();
    let a = qm31_from_u32s(1, 2, 3, 4).guess(&mut context);
    let b = im(&mut context, a);
    assert_eq!(context.get(b), qm31_from_u32s(0, 0, 3, 4));

    expect!["[5]"].assert_eq(&format!("{b:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (0 + 0i) + (1 + 1i)u: [4],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [5] = [3] x [4]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);

    context.validate_circuit();
}

#[test]
fn test_from_partial_evals() {
    let mut context = TraceContext::default();
    let values = [
        guess(&mut context, qm31_from_u32s(1, 10, 100, 1000)),
        guess(&mut context, 2.into()),
        guess(&mut context, 3.into()),
        guess(&mut context, 4.into()),
    ];
    let res = from_partial_evals(&mut context, values);
    assert_eq!(context.get(res), qm31_from_u32s(1, 12, 103, 1004));
    context.validate_circuit();
}

#[test]
fn test_stats() {
    let mut context = TraceContext::default();

    // A fresh context marks `u` as an output in its constructor.
    let stats = Stats { outputs: 1, ..Stats::default() };
    assert_eq!(context.stats, stats);

    let x = guess(&mut context, 5.into());
    let y = context.constant(25.into());

    let x_sqr = eval!(&mut context, (x) * (x));
    let stats = Stats { mul: 1, guess: 1, ..stats };
    assert_eq!(context.stats, stats);

    let x_sqr_minus_y = eval!(&mut context, (x_sqr) - (y));
    let stats = Stats { sub: 1, ..stats };
    assert_eq!(context.stats, stats);

    let zero = context.zero();
    eq(&mut context, x_sqr_minus_y, zero);
    let stats = Stats { equals: 1, ..stats };
    assert_eq!(context.stats, stats);

    // `0 + 0` is elided by the add-zero optimization, so it creates no gate.
    eval!(&mut context, (0) + (0));
    assert_eq!(context.stats, stats);

    // A non-trivial add creates a gate and bumps the stat.
    eval!(&mut context, (1) + (1));
    let stats = Stats { add: 1, ..stats };
    assert_eq!(context.stats, stats);

    let y_inv = inv(&mut context, y);
    mul(&mut context, x, y_inv);
    let stats = Stats {
        inv: 1,
        mul: stats.mul + 2,
        guess: stats.guess + 1,
        equals: stats.equals + 1,
        ..stats
    };
    assert_eq!(context.stats, stats);

    pointwise_mul(&mut context, x, y);
    let stats = Stats { pointwise_mul: 1, ..stats };
    assert_eq!(context.stats, stats);

    context.validate_circuit();
}

#[test]
fn test_no_gate_optimizations() {
    let mut context = TraceContext::default();
    let zero = context.zero();
    let one = context.one();
    let x = guess(&mut context, 42.into());

    // None of the optimized ops below should add a gate, so stats stay fixed.
    let expected = Stats { outputs: 1, guess: 1, ..Stats::default() };
    assert_eq!(context.stats, expected);

    // `add`: zero is the identity, so the other operand is returned and no gate is created.
    assert_eq!(add(&mut context, zero, x).idx, x.idx);
    assert_eq!(add(&mut context, x, zero).idx, x.idx);
    assert_eq!(add(&mut context, zero, zero).idx, zero.idx);

    // `mul`: multiplying by zero yields zero, and one is the identity.
    assert_eq!(mul(&mut context, zero, x).idx, zero.idx);
    assert_eq!(mul(&mut context, x, zero).idx, zero.idx);
    assert_eq!(mul(&mut context, zero, zero).idx, zero.idx);
    assert_eq!(mul(&mut context, one, x).idx, x.idx);
    assert_eq!(mul(&mut context, x, one).idx, x.idx);
    assert_eq!(mul(&mut context, one, one).idx, one.idx);

    assert_eq!(context.stats, expected);

    context.validate_circuit();
}
