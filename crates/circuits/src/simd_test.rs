use expect_test::expect;
use itertools::Itertools;
use num_traits::{One, Zero};
use rstest::rstest;
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::context::{Context, TraceContext, Var};
use crate::extract_bits::extract_bits;
use crate::ivalue::{NoValue, qm31_from_u32s};
use crate::ops::{Guess, guess};
use crate::simd::Simd;
use crate::test_utils::{packed_values, simd_from_u32s};
use crate::wrappers::M31Wrapper;

#[test]
fn test_repeat() {
    let mut context = TraceContext::default();
    let a = Simd::repeat(&mut context, M31::from(2), 6);
    let twos = qm31_from_u32s(2, 2, 2, 2);
    assert_eq!(a.len(), 6);
    assert_eq!(packed_values(&context, &a), &[twos, twos]);
    context.validate_circuit();
}

#[test]
fn test_zero() {
    let mut context = TraceContext::default();
    let a = Simd::zero(&mut context, 6);
    let zero = 0.into();
    assert_eq!(a.len(), 6);
    assert_eq!(packed_values(&context, &a), &[zero, zero]);
    context.validate_circuit();
}

#[test]
fn test_one() {
    let mut context = TraceContext::default();
    let a = Simd::one(&mut context, 6);
    let ones = qm31_from_u32s(1, 1, 1, 1);
    assert_eq!(a.len(), 6);
    assert_eq!(packed_values(&context, &a), &[ones, ones]);
    context.validate_circuit();
}
#[test]
fn test_simd_basic_ops() {
    let mut context = TraceContext::default();

    let a = simd_from_u32s(&mut context, vec![1, 2, 3, 4, 5, 6]);
    let b = simd_from_u32s(&mut context, vec![7, 9, 11, 13, 15, 17]);

    assert_eq!(a.len(), 6);

    let a_add_b = Simd::add(&mut context, &a, &b);
    assert_eq!(
        packed_values(&context, &a_add_b),
        &[qm31_from_u32s(8, 11, 14, 17), qm31_from_u32s(20, 23, 0, 0)]
    );

    let b_sub_a = Simd::sub(&mut context, &b, &a);
    assert_eq!(
        packed_values(&context, &b_sub_a),
        &[qm31_from_u32s(6, 7, 8, 9), qm31_from_u32s(10, 11, 0, 0)]
    );

    let a_mul_b = Simd::mul(&mut context, &a, &b);
    assert_eq!(
        packed_values(&context, &a_mul_b),
        &[qm31_from_u32s(7, 18, 33, 52), qm31_from_u32s(75, 102, 0, 0)]
    );

    context.validate_circuit();
}

#[test]
fn test_eq_circuit8() {
    let mut context = Context::<NoValue>::default();
    let a_simd = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 8);
    let b_simd = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 8);
    Simd::eq(&mut context, &a_simd, &b_simd);
    expect!["[[3], [4]]"].assert_eq(&format!("{:?}", a_simd.data));
    expect!["[[5], [6]]"].assert_eq(&format!("{:?}", b_simd.data));
    expect![[r#"
        [3] = [5]
        [4] = [6]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_eq_circuit7() {
    let mut context = Context::<NoValue>::default();
    let a_simd = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 7);
    let b_simd = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 7);
    Simd::eq(&mut context, &a_simd, &b_simd);
    expect!["[[3], [4]]"].assert_eq(&format!("{:?}", a_simd.data));
    expect!["[[5], [6]]"].assert_eq(&format!("{:?}", b_simd.data));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (1 + 1i) + (1 + 0i)u: [8],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [7] = [4] - [6]
        [9] = [7] x [8]
        [3] = [5]
        [9] = [0]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_eq() {
    // Go over possible lengths.
    for len in 0..=8_usize {
        let n_qm31s = len.div_ceil(4);
        // Go over all possible coordinates in the padded array, including coordinates greater than
        // the actual length.
        for wrong_coord in 0..(4 * n_qm31s) {
            let mut context = TraceContext::default();

            let mut vals: Vec<u32> = (0..8).collect();
            let a = vec![
                qm31_from_u32s(vals[0], vals[1], vals[2], vals[3]),
                qm31_from_u32s(vals[4], vals[5], vals[6], vals[7]),
            ]
            .guess(&mut context);

            vals[wrong_coord] += 1;
            let b = vec![
                qm31_from_u32s(vals[0], vals[1], vals[2], vals[3]),
                qm31_from_u32s(vals[4], vals[5], vals[6], vals[7]),
            ]
            .guess(&mut context);

            Simd::eq(
                &mut context,
                &Simd::from_packed(a[..n_qm31s].to_vec(), len),
                &Simd::from_packed(b[..n_qm31s].to_vec(), len),
            );

            // There should be an error if the wrong coordinate is within the range of the actual
            // length.
            assert_eq!(context.is_circuit_valid(), wrong_coord >= len);
        }
    }
}

#[test]
fn test_guess_inv_or_zero() {
    let mut context = TraceContext::default();

    let a = simd_from_u32s(&mut context, vec![2, 0, 3]);
    let a_inv = a.guess_inv_or_zero(&mut context);
    assert_eq!(a_inv.len(), 3);

    let one = M31::one();
    assert_eq!(
        packed_values(&context, &a_inv),
        &[QM31(CM31(one / M31::from(2), 0.into()), CM31(one / M31::from(3), 0.into()))]
    );

    let mut values = context.values().clone();
    context.circuit.check(&values).unwrap();

    // As the value is not enforced, the circuit passes with a changed value as well.
    values[a_inv.get_packed()[0].idx] += QM31::from(1);
    context.circuit.check(&values).unwrap();
}

#[rstest]
#[case::success(None)]
#[case::has_zero2(Some(2))]
#[case::has_zero5(Some(5))]
fn test_inv(#[case] zero_idx: Option<usize>) {
    let mut context = TraceContext::default();

    let mut input_vals = vec![2, 3, 4, 5, 6, 7];
    let mut expected_invs = input_vals.iter().map(|v| M31::one() / M31::from(*v)).collect_vec();

    if let Some(zero_idx) = zero_idx {
        input_vals[zero_idx] = 0;
        expected_invs[zero_idx] = M31::zero();
    }

    let a = simd_from_u32s(&mut context, input_vals);
    let a_inv = a.inv(&mut context);
    assert_eq!(a_inv.len(), 6);

    assert_eq!(
        packed_values(&context, &a_inv),
        &[
            QM31(
                CM31(expected_invs[0], expected_invs[1]),
                CM31(expected_invs[2], expected_invs[3])
            ),
            QM31(CM31(expected_invs[4], expected_invs[5]), CM31::zero()),
        ]
    );

    assert_eq!(context.is_circuit_valid(), zero_idx.is_none());
}

#[test]
fn test_inv_circuit() {
    let mut context = Context::<NoValue>::default();
    let input = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 6);
    let res = input.inv(&mut context);

    expect!["Simd { data: [[3], [4]], len: 6 }"].assert_eq(&format!("{input:?}"));
    expect!["Simd { data: [[5], [6]], len: 6 }"].assert_eq(&format!("{res:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (1 + 1i) + (1 + 1i)u: [9],
            (1 + 1i) + (0 + 0i)u: [11],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [10] = [8] - [9]
        [7] = [5] x [3]
        [8] = [6] x [4]
        [12] = [10] x [11]
        [7] = [9]
        [12] = [0]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[rstest]
#[case::zeros([0, 0, 0, 0], true)]
#[case::ones([1, 1, 1, 0], true)]
#[case::mix([1, 0, 1, 0], true)]
#[case::two([2, 0, 0, 0], false)]
#[case::last_coord_ignored([0, 0, 0, 2], true)]
fn test_assert_bits(#[case] vals: [u32; 4], #[case] success: bool) {
    let mut context = TraceContext::default();

    let a = Simd::from_packed(
        vec![guess(&mut context, qm31_from_u32s(vals[0], vals[1], vals[2], vals[3]))],
        3,
    );
    a.assert_bits(&mut context);

    assert_eq!(context.is_circuit_valid(), success);
}

#[test]
fn test_guess_lsb() {
    let mut context = TraceContext::default();

    let a = simd_from_u32s(&mut context, vec![7, 1, 4, 6, 11, 8]);
    let b = a.guess_lsb(&mut context);

    assert_eq!(b.len(), 6);
    assert_eq!(
        packed_values(&context, &b),
        [qm31_from_u32s(1, 1, 0, 0), qm31_from_u32s(1, 0, 0, 0)]
    );

    context.validate_circuit();
}

#[test]
fn test_select() {
    let mut context = TraceContext::default();

    let selector = simd_from_u32s(&mut context, vec![1, 0, 1]);
    let if_zero = simd_from_u32s(&mut context, vec![1, 2, 3]);
    let if_one = simd_from_u32s(&mut context, vec![4, 5, 6]);

    let result = Simd::select(&mut context, &selector, &if_zero, &if_one);

    assert_eq!(result.len(), 3);
    assert_eq!(packed_values(&context, &result), &[qm31_from_u32s(4, 2, 6, 0)]);

    context.validate_circuit();
}

#[test]
fn test_unpack() {
    let mut context = TraceContext::default();
    let input = simd_from_u32s(&mut context, vec![12, 6, 5, 20, 1]);
    let res = Simd::unpack(&mut context, &input);
    assert_eq!(res.len(), 5);
    assert_eq!(context.get(res[0]), qm31_from_u32s(12, 0, 0, 0));
    assert_eq!(context.get(res[1]), qm31_from_u32s(6, 0, 0, 0));
    assert_eq!(context.get(res[2]), qm31_from_u32s(5, 0, 0, 0));
    assert_eq!(context.get(res[3]), qm31_from_u32s(20, 0, 0, 0));
    assert_eq!(context.get(res[4]), qm31_from_u32s(1, 0, 0, 0));
}

#[test]
fn test_unpack_idx() {
    let mut context = TraceContext::default();
    let input = vec![12, 6, 5, 20, 1];
    let packed = simd_from_u32s(&mut context, input.clone());

    for (i, expected) in input.iter().enumerate() {
        let unpacked = Simd::unpack_idx(&mut context, &packed, i);
        assert_eq!(context.get(unpacked), qm31_from_u32s(*expected, 0, 0, 0));
    }
}

#[test]
fn test_unpack_circuit() {
    let mut context = Context::<NoValue>::default();
    let input = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 6);
    let res = Simd::unpack(&mut context, &input);
    expect!["Simd { data: [[3], [4]], len: 6 }"].assert_eq(&format!("{input:?}"));
    expect!["[[5], [9], [12], [16], [17], [19]]"].assert_eq(&format!("{res:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (0 + 1i) + (0 + 0i)u: [6],
            (0 + 2147483646i) + (0 + 0i)u: [8],
            (0 + 0i) + (1717986918 + 1288490188i)u: [11],
            (0 + 0i) + (0 + 1i)u: [13],
            (0 + 0i) + (1288490188 + 429496729i)u: [15],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [9] = [7] * [8]
        [12] = [10] * [11]
        [16] = [14] * [15]
        [19] = [18] * [8]
        [5] = [3] x [1]
        [7] = [3] x [6]
        [10] = [3] x [2]
        [14] = [3] x [13]
        [17] = [4] x [1]
        [18] = [4] x [6]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_pack() {
    let mut context = TraceContext::default();
    let input =
        [12, 6, 5, 20, 1, 4, 8, 10].map(|i| M31Wrapper::from(M31::from(i))).guess(&mut context);

    let res = Simd::pack(&mut context, &[]);
    assert_eq!(res.len(), 0);

    let res = Simd::pack(&mut context, &input[0..1]);
    assert_eq!(res.len(), 1);
    assert_eq!(packed_values(&context, &res), vec![qm31_from_u32s(12, 0, 0, 0)]);

    let res = Simd::pack(&mut context, &input[0..2]);
    assert_eq!(res.len(), 2);
    assert_eq!(packed_values(&context, &res), vec![qm31_from_u32s(12, 6, 0, 0)]);

    let res = Simd::pack(&mut context, &input[0..3]);
    assert_eq!(res.len(), 3);
    assert_eq!(packed_values(&context, &res), vec![qm31_from_u32s(12, 6, 5, 0)]);

    let res = Simd::pack(&mut context, &input[0..4]);
    assert_eq!(res.len(), 4);
    assert_eq!(packed_values(&context, &res), vec![qm31_from_u32s(12, 6, 5, 20)]);

    let res = Simd::pack(&mut context, &input[0..5]);
    assert_eq!(res.len(), 5);
    assert_eq!(
        packed_values(&context, &res),
        vec![qm31_from_u32s(12, 6, 5, 20), qm31_from_u32s(1, 0, 0, 0)]
    );

    let res = Simd::pack(&mut context, &input[0..8]);
    assert_eq!(res.len(), 8);
    assert_eq!(
        packed_values(&context, &res),
        vec![qm31_from_u32s(12, 6, 5, 20), qm31_from_u32s(1, 4, 8, 10)]
    );

    context.validate_circuit();
}

#[test]
fn test_pack_circuit() {
    let mut context = Context::<NoValue>::default();
    let input = vec![NoValue; 6]
        .guess(&mut context)
        .iter()
        .map(|v| M31Wrapper::new_unsafe(*v))
        .collect_vec();
    let res = Simd::pack(&mut context, &input);

    expect!["[M31([3]), M31([4]), M31([5]), M31([6]), M31([7]), M31([8])]"]
        .assert_eq(&format!("{input:?}"));
    expect!["Simd { data: [[16], [18]], len: 6 }"].assert_eq(&format!("{res:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (0 + 1i) + (0 + 0i)u: [9],
            (0 + 0i) + (0 + 1i)u: [10],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [12] = [3] + [11]
        [14] = [12] + [13]
        [16] = [14] + [15]
        [18] = [7] + [17]
        [11] = [9] * [4]
        [13] = [2] * [5]
        [15] = [10] * [6]
        [17] = [9] * [8]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_scalar_mul_circuit() {
    let mut context = TraceContext::default();
    let input_vals = vec![12, 6, 5, 3, 4];

    let scalar = 5;
    let expected_res = input_vals.iter().map(|v| scalar * v).collect_vec();
    let scalar_as_m31 = M31Wrapper::from(M31::from(scalar)).guess(&mut context);

    let input = simd_from_u32s(&mut context, input_vals);

    let res = Simd::scalar_mul(&mut context, &input, &scalar_as_m31);

    assert_eq!(res.len(), 5);
    assert_eq!(
        packed_values(&context, &res),
        &[
            qm31_from_u32s(expected_res[0], expected_res[1], expected_res[2], expected_res[3]),
            qm31_from_u32s(expected_res[4], 0, 0, 0)
        ]
    );

    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [6] = [4] * [3]
        [7] = [5] * [3]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_pow2_circuit() {
    let mut context = TraceContext::default();
    let input_vals = vec![12, 6, 5, 3, 4];
    let expected_res = input_vals.iter().map(|v| 1 << v).collect_vec();

    let input = simd_from_u32s(&mut context, input_vals);
    let n_bits = 5;
    let bits = extract_bits(&mut context, &input, n_bits);
    let res = Simd::pow2(&mut context, &bits);

    assert_eq!(res.len() as u32, n_bits);
    assert_eq!(
        packed_values(&context, &res),
        &[
            qm31_from_u32s(expected_res[0], expected_res[1], expected_res[2], expected_res[3]),
            qm31_from_u32s(expected_res[4], 1, 1, 1)
        ]
    );
}

#[test]
fn test_combine_bits_circuit() {
    let mut context = TraceContext::default();

    let input = simd_from_u32s(&mut context, vec![12, 6, 5, 3, 4]);
    let n_bits = 5;
    let bits = extract_bits(&mut context, &input, n_bits);
    let res = Simd::combine_bits(&mut context, &bits);

    let get_vals = |vars: &[Var]| -> Vec<QM31> { vars.iter().map(|v| context.get(*v)).collect() };

    assert_eq!(get_vals(input.get_packed()), get_vals(res.get_packed()));
}
