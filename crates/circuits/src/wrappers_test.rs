use expect_test::expect;

use crate::context::Context;
use crate::ivalue::NoValue;
use crate::ops::Guess;
use crate::wrappers::{M31Wrapper, U16Wrapper, U32Wrapper};

#[test]
fn test_m31_wrapper_guess_circuit() {
    let mut context = Context::<NoValue>::default();
    let res = M31Wrapper::from(NoValue).guess(&mut context);
    context.finalize_guessed_vars();
    expect!["M31([3])"].assert_eq(&format!("{res:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [3] = [3] x [1]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_u16_wrapper_guess_circuit() {
    let mut context = Context::<NoValue>::default();
    let res = U16Wrapper::from(NoValue).guess(&mut context);
    context.finalize_guessed_vars();
    expect!["U16([3])"].assert_eq(&format!("{res:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [3] = m31_to_u32([3])
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn test_u32_wrapper_guess_circuit() {
    let mut context = Context::<NoValue>::default();
    let res = U32Wrapper::from(NoValue).guess(&mut context);
    context.finalize_guessed_vars();
    expect!["U32([7])"].assert_eq(&format!("{res:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
            (0 + 1i) + (0 + 0i)u: [5],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [7] = [3] + [6]
        [6] = [4] * [5]
        [3] = m31_to_u32([3])
        [4] = m31_to_u32([4])
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}
