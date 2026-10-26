use expect_test::expect;
use rstest::rstest;

use crate::context::{Context, TraceContext};
use crate::extract_bits::{combine_bits, extract_bits, validate_extract_bits};
use crate::ivalue::{NoValue, qm31_from_u32s};
use crate::ops::guess;
use crate::simd::Simd;
use crate::test_utils::{packed_values, simd_from_u32s};

/// Constructs a single-lane [Simd] variable with no value.
fn guess_simd_no_value(context: &mut Context<NoValue>) -> Simd {
    Simd::from_packed(vec![guess(context, NoValue)], 1)
}

#[test]
fn test_extract_bits() {
    let mut context = TraceContext::default();
    // Note that `2_u32.pow(31) - 1` is identical to `0` after the `simd_from_u32s` call.
    // It's here only as a sanity check.
    let input = simd_from_u32s(&mut context, vec![0, 12, 2_u32.pow(31) - 1, 2_u32.pow(31) - 2]);
    let bits = extract_bits(&mut context, &input, 31);
    assert_eq!(packed_values(&context, &bits[0]), &[qm31_from_u32s(0, 0, 0, 0)]);
    assert_eq!(packed_values(&context, &bits[1]), &[qm31_from_u32s(0, 0, 0, 1)]);
    assert_eq!(packed_values(&context, &bits[2]), &[qm31_from_u32s(0, 1, 0, 1)]);
    assert_eq!(packed_values(&context, &bits[3]), &[qm31_from_u32s(0, 1, 0, 1)]);
    for bit in &bits[4..31] {
        assert_eq!(packed_values(&context, bit), &[qm31_from_u32s(0, 0, 0, 1)]);
    }

    context.validate_circuit();
}

#[rstest]
#[case::success(true)]
#[case::failure(false)]
fn test_validate_extract_bits(#[case] success: bool) {
    let mut context = TraceContext::default();
    // If input is not zero, both 0 and 1 are allowed by `validate_extract_bits`.
    // If it is zero, only lsb=0 is allowed.
    let lsb = simd_from_u32s(&mut context, vec![if success { 0 } else { 1 }, 1, 0, 1, 1, 1]);
    let input = simd_from_u32s(&mut context, vec![0, 1, 2, 3, 4, 5]);

    validate_extract_bits(&mut context, &input, &lsb);

    assert_eq!(context.is_circuit_valid(), success);
}

#[rstest]
#[case::full_simd(4)]
#[case::partial_simd(3)]
fn test_extract_bits_as_range_check(#[case] len: usize) {
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
