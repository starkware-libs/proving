use itertools::Itertools;

use crate::context::{Context, GuessVar, TraceContext, U_VAR_IDX};
use crate::ivalue::qm31_from_u32s;

#[test]
fn test_constants() {
    let x = qm31_from_u32s(1, 2, 3, 4);
    let mut context = Context::default();
    let a = context.constant(x);
    let _b = context.constant(x + x);
    // The second time `x` is requested, the same variable is returned.
    let c = context.constant(x);
    assert_eq!(a.idx, c.idx);

    let u = qm31_from_u32s(0, 0, 1, 0);
    assert_eq!(context.values(), &vec![0.into(), 1.into(), u, x, x + x]);

    let context = context.finalize(false);
    context.validate_circuit();
}

#[test]
fn test_set_outputs() {
    let mut context = TraceContext::default();

    let a = context.constant(qm31_from_u32s(3, 0, 0, 0));

    let reserved0 = context.reserve();
    let reserved1 = context.reserve();
    context.set_outputs(&[a, a]);
    assert_eq!(context.get(reserved0), qm31_from_u32s(3, 0, 0, 0));
    assert_eq!(context.get(reserved1), qm31_from_u32s(3, 0, 0, 0));

    for (actual, expected) in
        context.circuit.output.iter().zip_eq([U_VAR_IDX, reserved0.idx, reserved1.idx])
    {
        assert_eq!(actual.in0, expected);
    }
    let context = context.finalize(false);
    context.circuit().check_yields();

    context.validate_circuit();
}

#[test]
#[should_panic(expected = "were never assigned")]
fn test_unfulfilled_reservation_panics_at_finalize() {
    let mut context = TraceContext::default();
    context.reserve();
    context.finalize(false);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "read of reserved variable")]
fn test_read_before_fulfill_panics_in_debug() {
    let mut context = TraceContext::default();
    let r = context.reserve();
    context.get(r);
}

/// Builds a circuit with a single `U16` guess holding `value`, finalizes the guessed
/// variables, and returns the result of checking the resulting constraints.
fn check_u16_guess(value: u32) -> Result<(), String> {
    let mut context = TraceContext::default();
    let var = context.new_var(qm31_from_u32s(value, 0, 0, 0));
    context.guessed_vars.as_mut().unwrap().push(GuessVar::U16(var));
    context.finalize_guessed_vars();
    context.circuit.check(context.values())
}

#[test]
fn test_u16_guess_accepts_16_bit_values() {
    // Zero, a generic value, and the largest 16-bit value all satisfy the constraint.
    for value in [0, 12345, 0xFFFF] {
        assert!(check_u16_guess(value).is_ok(), "value {value} should be accepted");
    }
}

#[test]
fn test_u16_guess_rejects_out_of_range_values() {
    // Anything that does not fit in 16 bits has a non-zero high limb after `m31_to_u32`,
    // so the `x == m31_to_u32(x)` constraint fails.
    for value in [0x1_0000, 70_000] {
        assert!(check_u16_guess(value).is_err(), "value {value} should be rejected");
    }
}

#[test]
fn test_zero_and_one() {
    let mut context = TraceContext::default();

    let zero = context.constant(0.into());
    let one = context.constant(1.into());

    assert_eq!(context.zero().idx, zero.idx);
    assert_eq!(context.one().idx, one.idx);
}
