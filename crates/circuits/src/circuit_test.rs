use crate::circuit::{Add, Circuit, Eq, Mul, Output, Permutation, Sub};

#[test]
fn circuit_add() {
    let mut c = Circuit { n_vars: 3, ..Circuit::default() };
    c.add.push(Add { in0: 0, in1: 1, out: 2 });
    c.check(&[1.into(), 2.into(), 3.into()]).unwrap();
    assert_eq!(
        c.check(&[1.into(), 2.into(), 4.into()]).unwrap_err(),
        "(3 + 0i) + (0 + 0i)u != (4 + 0i) + (0 + 0i)u"
    );
    assert_eq!(c.compute_multiplicities(), (vec![1, 1, 0], vec![0, 0, 1]));
}

#[test]
fn circuit_sub() {
    let mut c = Circuit::default();
    c.sub.push(Sub { in0: 0, in1: 1, out: 3 });
    c.check(&[10.into(), 7.into(), 0.into(), 3.into()]).unwrap();
    assert_eq!(
        c.check(&[10.into(), 7.into(), 0.into(), 4.into()]).unwrap_err(),
        "(3 + 0i) + (0 + 0i)u != (4 + 0i) + (0 + 0i)u"
    );
}

#[test]
fn circuit_mul() {
    let mut c = Circuit::default();
    c.mul.push(Mul { in0: 1, in1: 1, out: 2 });
    c.check(&[1.into(), 4.into(), 16.into()]).unwrap();
    assert_eq!(
        c.check(&[1.into(), 5.into(), 16.into()]).unwrap_err(),
        "(25 + 0i) + (0 + 0i)u != (16 + 0i) + (0 + 0i)u"
    );
}

#[test]
fn circuit_eq() {
    let mut c = Circuit { n_vars: 3, ..Circuit::default() };
    c.eq.push(Eq { in0: 1, in1: 2 });
    c.check(&[1.into(), 2.into(), 2.into()]).unwrap();
    assert_eq!(
        c.check(&[1.into(), 1.into(), 2.into()]).unwrap_err(),
        "(1 + 0i) + (0 + 0i)u != (2 + 0i) + (0 + 0i)u"
    );
    assert_eq!(c.compute_multiplicities(), (vec![0, 1, 1], vec![0, 0, 0]));
}

#[test]
fn circuit_permutation() {
    let mut c = Circuit { n_vars: 6, ..Circuit::default() };
    c.permutation.push(Permutation { inputs: vec![0, 1, 2], outputs: vec![3, 4, 5] });
    c.check(&[1.into(), 2.into(), 2.into(), 2.into(), 1.into(), 2.into()]).unwrap();
    assert_eq!(
        c.check(&[1.into(), 2.into(), 2.into(), 2.into(), 2.into(), 2.into()]).unwrap_err(),
        "Permutation is not valid"
    );
}

#[test]
fn circuit_output() {
    let mut c = Circuit { n_vars: 3, ..Circuit::default() };
    c.output.push(Output { in0: 1 });
    c.check(&[1.into(), 2.into(), 2.into()]).unwrap();
    assert_eq!(c.compute_multiplicities(), (vec![0, 1, 0], vec![0, 0, 0]));
}
