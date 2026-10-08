use expect_test::expect;
use rstest::rstest;

use crate::context::{Context, TraceContext, Var};
use crate::extract_bits::{combine_bits, extract_bits};
use crate::ivalue::{NoValue, qm31_from_u32s};
use crate::ops::guess;
use crate::simd::Simd;
use crate::test_utils::{guess_m31_from_u32, guess_m31_no_value, packed_values, simd_from_u32s};
use crate::wrappers::M31Wrapper;

/// Constructs a single-lane [Simd] variable with no value.
fn guess_simd_no_value(context: &mut Context<NoValue>) -> Simd {
    Simd::from_packed(vec![guess(context, NoValue)], 1)
}

#[test]
fn test_extract_bits_simd() {
    let mut context = TraceContext::default();
    let input = simd_from_u32s(&mut context, vec![0, 12, 2_u32.pow(30) - 1, 2_u32.pow(30) - 2]);
    let bits = extract_bits(&mut context, &input, 30);
    assert_eq!(bits.len(), 30);
    assert_eq!(packed_values(&context, &bits[0]), &[qm31_from_u32s(0, 0, 1, 0)]);
    assert_eq!(packed_values(&context, &bits[1]), &[qm31_from_u32s(0, 0, 1, 1)]);
    assert_eq!(packed_values(&context, &bits[2]), &[qm31_from_u32s(0, 1, 1, 1)]);
    assert_eq!(packed_values(&context, &bits[3]), &[qm31_from_u32s(0, 1, 1, 1)]);
    for bit in &bits[4..30] {
        assert_eq!(packed_values(&context, bit), &[qm31_from_u32s(0, 0, 1, 1)]);
    }

    context.validate_circuit();
}

#[rstest]
#[case::zero(0)]
#[case::small(12)]
#[case::max(2_u32.pow(30) - 1)]
fn test_extract_bits_m31_wrapper(#[case] value: u32) {
    let mut context = TraceContext::default();
    let input = guess_m31_from_u32(&mut context, value);
    let bits = extract_bits(&mut context, &input, 30);

    assert_eq!(bits.len(), 30);
    for (i, bit) in bits.iter().enumerate() {
        assert_eq!(context.get(*bit.raw()), qm31_from_u32s((value >> i) & 1, 0, 0, 0));
    }

    context.validate_circuit();
}

#[rstest]
#[case::full_simd(4)]
#[case::partial_simd(3)]
fn test_extract_bits_out_of_range(#[case] len: usize) {
    let mut context = TraceContext::default();

    let n_bits = 5;

    let out_of_range_value = 1 << n_bits;
    let mut input_values = vec![out_of_range_value];
    input_values.resize(len, 3);
    let input = simd_from_u32s(&mut context, input_values);
    let _bits = extract_bits(&mut context, &input, n_bits);

    assert!(!context.is_circuit_valid());
}

#[test]
fn test_extract_bits_out_of_range_m31_wrapper() {
    let mut context = TraceContext::default();

    let n_bits = 5;
    let input = guess_m31_from_u32(&mut context, 1 << n_bits);
    let _bits = extract_bits(&mut context, &input, n_bits);

    assert!(!context.is_circuit_valid());
}

#[test]
fn test_extract_bits_circuit_single_m31_wrapper() {
    let mut context = Context::<NoValue>::default();
    let input = guess_m31_no_value(&mut context);
    let bits = extract_bits(&mut context, &input, 3);

    expect!["M31([3])"].assert_eq(&format!("{input:?}"));
    expect!["[M31([5]), M31([9]), M31([12])]"].assert_eq(&format!("{bits:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (1073741824 + 0i) + (0 + 0i)u: [4],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [7] = [3] - [5]
        [11] = [8] - [9]
        [6] = [5] * [5]
        [8] = [7] * [4]
        [10] = [9] * [9]
        [12] = [11] * [4]
        [13] = [12] * [12]
        [5] = [6]
        [9] = [10]
        [12] = [13]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_extract_bits_circuit_single_simd() {
    let mut context = Context::<NoValue>::default();
    let input = guess_simd_no_value(&mut context);
    let bits = extract_bits(&mut context, &input, 3);

    expect!["Simd { data: [[3]], len: 1 }"].assert_eq(&format!("{input:?}"));
    expect![
        "[Simd { data: [[5]], len: 1 }, Simd { data: [[11]], len: 1 }, Simd { data: [[16]], len: \
         1 }]"
    ]
    .assert_eq(&format!("{bits:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (1073741824 + 0i) + (0 + 0i)u: [4],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [7] = [5] - [6]
        [9] = [3] - [5]
        [13] = [11] - [12]
        [15] = [10] - [11]
        [18] = [16] - [17]
        [10] = [9] * [4]
        [16] = [15] * [4]
        [6] = [5] x [5]
        [8] = [7] x [1]
        [12] = [11] x [11]
        [14] = [13] x [1]
        [17] = [16] x [16]
        [19] = [18] x [1]
        [8] = [0]
        [14] = [0]
        [19] = [0]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_combine_bits_circuit() {
    let mut context = TraceContext::default();

    let input = simd_from_u32s(&mut context, vec![12, 6, 5, 3, 4]);
    let n_bits = 5;
    let bits = extract_bits(&mut context, &input, n_bits);
    let res = combine_bits(&mut context, &bits);

    assert_eq!(packed_values(&context, &input), packed_values(&context, &res));
    context.validate_circuit();
}

#[test]
fn test_combine_bits_circuit_m31_wrapper() {
    let mut context = TraceContext::default();

    // Little-endian bits of 12 = 0b01100.
    let bits: Vec<M31Wrapper<Var>> =
        [0, 0, 1, 1, 0].into_iter().map(|bit| guess_m31_from_u32(&mut context, bit)).collect();
    let res = combine_bits(&mut context, &bits);

    assert_eq!(context.get(*res.raw()), qm31_from_u32s(12, 0, 0, 0));
    context.validate_circuit();
}

#[test]
fn test_combine_bits_circuit_single() {
    let mut context = Context::<NoValue>::default();
    let bits = [
        guess_m31_no_value(&mut context),
        guess_m31_no_value(&mut context),
        guess_m31_no_value(&mut context),
    ];
    let res = combine_bits(&mut context, &bits);

    expect!["[M31([3]), M31([4]), M31([5])]"].assert_eq(&format!("{bits:?}"));
    expect!["M31([10])"].assert_eq(&format!("{res:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (2 + 0i) + (0 + 0i)u: [6],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [8] = [7] + [4]
        [10] = [9] + [3]
        [7] = [5] * [6]
        [9] = [8] * [6]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);

    // The simd version of length 1 creates an identical circuit.
    let mut context_simd = Context::<NoValue>::default();
    let bits_simd = [
        guess_simd_no_value(&mut context_simd),
        guess_simd_no_value(&mut context_simd),
        guess_simd_no_value(&mut context_simd),
    ];
    combine_bits(&mut context_simd, &bits_simd);
    assert_eq!(context.circuit, context_simd.circuit);
}
