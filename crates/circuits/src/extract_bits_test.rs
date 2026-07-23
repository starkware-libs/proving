use rstest::rstest;

use crate::context::TraceContext;
use crate::extract_bits::{extract_bits, validate_extract_bits};
use crate::ivalue::qm31_from_u32s;
use crate::test_utils::{packed_values, simd_from_u32s};

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
