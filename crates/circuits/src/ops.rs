use itertools::Itertools;
use stwo::core::circle::CirclePoint;

use crate::circuit::{Add, Eq, Mul, Output, Permutation, PointwiseMul, Sub};
use crate::context::{Context, GuessVar, Var};
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::wrappers::M31Wrapper;

#[cfg(test)]
#[path = "ops_test.rs"]
pub mod test;

/// A macro for writing arithmetic expressions on circuit variables.
///
/// Usage: `eval!(context, expression)`
///
/// Note: Parentheses are required for all arithmetic operations, including literals and variables.
///
/// Example:
/// ```plain
/// let result = eval!(context, ((value) * (value)) - (1));
/// ```
#[macro_export]
macro_rules! eval {
    ($ctx:expr, ($($a:tt)+) + ($($b:tt)+)) => {{
        let __tmp0 = $crate::eval!($ctx, $($a)+);
        let __tmp1 = $crate::eval!($ctx, $($b)+);
        $crate::ops::add($ctx, __tmp0, __tmp1)
    }};

    ($ctx:expr, ($($a:tt)+) - ($($b:tt)+)) => {{
        let __tmp0 = $crate::eval!($ctx, $($a)+);
        let __tmp1 = $crate::eval!($ctx, $($b)+);
        $crate::ops::sub($ctx, __tmp0, __tmp1)
    }};

    ($ctx:expr, - ($($a:tt)+)) => {{
        let __tmp0 = $ctx.zero();
        let __tmp1 = $crate::eval!($ctx, $($a)+);
        $crate::ops::sub($ctx, __tmp0, __tmp1)
    }};

    ($ctx:expr, ($($a:tt)+) * ($($b:tt)+)) => {{
        let __tmp0 = $crate::eval!($ctx, $($a)+);
        let __tmp1 = $crate::eval!($ctx, $($b)+);
        $crate::ops::mul($ctx, __tmp0, __tmp1)
    }};

    ($ctx:expr, $lit:literal) => {
        $ctx.constant($lit.into())
    };

    ($ctx:expr, $id:expr) => {
        $id
    };
}

/// Adds an equality gate to the circuit.
pub fn eq<Value: IValue>(context: &mut Context<Value>, a: Var, b: Var) {
    context.stats.equals += 1;
    if context.assert_eq_on_eval {
        assert_eq!(context.get(a), context.get(b), "Eq failed: Vars {a:?} and {b:?}");
    }
    context.circuit.eq.push(Eq { in0: a.idx, in1: b.idx });
}

/// Returns a variable constrained to hold the result of `a + b`.
///
/// Typically adds an addition gate to the circuit, but as an optimization, if either operand is the
/// canonical zero variable, no gate is added and the other operand is returned directly.
pub fn add(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let zero = context.zero();
    // Note that if both are zero, we return zero.
    if a.idx == zero.idx {
        return b;
    }
    if b.idx == zero.idx {
        return a;
    }

    let out = context.new_var(context.get(a) + context.get(b));
    add_into(context, a, b, out);
    out
}

/// Adds an addition gate `a + b = out` to the circuit, using the given existing variable as the
/// output. The caller is responsible for the value of `out`.
pub fn add_into(context: &mut Context<impl IValue>, a: Var, b: Var, out: Var) {
    context.stats.add += 1;
    context.circuit.add.push(Add { in0: a.idx, in1: b.idx, out: out.idx });
}

/// Adds a subtraction gate to the circuit, and returns the output variable.
pub fn sub(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let out = context.new_var(context.get(a) - context.get(b));
    sub_into(context, a, b, out);
    out
}

/// Adds a subtraction gate `a - b = out` to the circuit, using the given existing variable as the
/// output. The caller is responsible for the value of `out`.
pub fn sub_into(context: &mut Context<impl IValue>, a: Var, b: Var, out: Var) {
    context.stats.sub += 1;
    context.circuit.sub.push(Sub { in0: a.idx, in1: b.idx, out: out.idx });
}

/// Returns a variable constrained to hold the result of `a * b`.
///
/// Typically adds a multiplication gate to the circuit, but as an optimization, no gate is added if
/// either operand is the canonical zero variable (the canonical zero is returned) or the canonical
/// one variable (the other operand is returned).
pub fn mul(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let zero = context.zero();
    if a.idx == zero.idx || b.idx == zero.idx {
        return zero;
    }

    let one = context.one();
    // Note that if both are one, we return one.
    if a.idx == one.idx {
        return b;
    }
    if b.idx == one.idx {
        return a;
    }

    let out = context.new_var(context.get(a) * context.get(b));
    mul_into(context, a, b, out);
    out
}

/// Adds a multiplication gate `a * b = out` to the circuit, using the given existing variable as
/// the output. The caller is responsible for the value of `out`.
pub fn mul_into(context: &mut Context<impl IValue>, a: Var, b: Var, out: Var) {
    context.stats.mul += 1;
    context.circuit.mul.push(Mul { in0: a.idx, in1: b.idx, out: out.idx });
}

/// Computes `a / b` by guessing `a / b`, constraining `out * b = a`, and returning `out`.
/// Does not constrain `b != 0` — the caller must ensure this through other means.
pub fn div(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    context.stats.div += 1;
    let out = guess(context, context.get(a) / context.get(b));
    let mul_res = mul(context, out, b);
    eq(context, mul_res, a);
    out
}

/// Computes `b`'s inverse by guessing it, constraining `b * b_inv = 1` (which proves
/// `b != 0`), and returning `b_inv`.
pub fn inv(context: &mut Context<impl IValue>, b: Var) -> Var {
    context.stats.inv += 1;
    let one = context.one();
    let b_inv = guess(context, context.get(one) / context.get(b));
    let b_times_b_inv = mul(context, b, b_inv);
    eq(context, b_times_b_inv, one);
    b_inv
}

pub fn pointwise_mul<Value: IValue>(context: &mut Context<Value>, a: Var, b: Var) -> Var {
    let out = context.new_var(Value::pointwise_mul(context.get(a), context.get(b)));
    pointwise_mul_into(context, a, b, out);
    out
}

/// Adds a pointwise multiplication gate `a .* b = out` to the circuit, using the given existing
/// variable as the output. The caller is responsible for the value of `out`.
pub fn pointwise_mul_into<Value: IValue>(context: &mut Context<Value>, a: Var, b: Var, out: Var) {
    context.stats.pointwise_mul += 1;
    context.circuit.pointwise_mul.push(PointwiseMul { in0: a.idx, in1: b.idx, out: out.idx });
}

/// Permutes the input values using the given function and returns the new variables.
pub fn permute<Value: IValue>(
    context: &mut Context<Value>,
    inputs: &[Var],
    permute_fn: impl FnOnce(&[Value]) -> Vec<Value>,
) -> Vec<Var> {
    context.stats.permutation_inputs += inputs.len();
    let outputs: Vec<Var> = permute_fn(&inputs.iter().map(|var| context.get(*var)).collect_vec())
        .iter()
        .map(|value| context.new_var(*value))
        .collect();
    context.circuit.permutation.push(Permutation {
        inputs: inputs.iter().map(|var| var.idx).collect(),
        outputs: outputs.iter().map(|var| var.idx).collect(),
    });
    outputs
}

/// Adds an output gate to the circuit.
pub fn output<Value: IValue>(context: &mut Context<Value>, a: Var) {
    context.stats.outputs += 1;
    context.circuit.output.push(Output { in0: a.idx });
}

/// Returns `(a, b)` if `selector` is 0, and `(b, a)` if `selector` is 1.
/// Assumption: `selector` is either 0 or 1.
pub fn cond_flip(context: &mut Context<impl IValue>, selector: Var, a: Var, b: Var) -> (Var, Var) {
    let diff = eval!(context, (selector) * ((b) - (a)));
    let res_a = eval!(context, (a) + (diff));
    let res_b = eval!(context, (b) - (diff));
    (res_a, res_b)
}

/// Computes the conjugate (with respect to `CM31`) of a `QM31` value:
///   `a + b * i + c * u + d * iu -> a + b * i - c * u - d * iu`.
pub fn conj(c: &mut Context<impl IValue>, a: Var) -> Var {
    let coefs = c.constant(qm31_from_u32s(1, 1, 0, 0) - qm31_from_u32s(0, 0, 1, 1));
    pointwise_mul(c, a, coefs)
}

/// Computes the twisted imaginary part (with respect to `CM31`) of a `QM31` value:
///   `z = a + b * i + c * u + d * iu ->  (c + d * i)*u = (z - conj(z))/2`.
pub fn im(c: &mut Context<impl IValue>, a: Var) -> Var {
    let coefs = c.constant(qm31_from_u32s(0, 0, 1, 1));
    pointwise_mul(c, a, coefs)
}

/// Returns a new unconstrained variable with the given value.
pub fn guess<Value: IValue>(context: &mut Context<Value>, value: Value) -> Var {
    context.stats.guess += 1;
    let out = context.new_var(value);
    context.guessed_vars.as_mut().unwrap().push(GuessVar::QM31(out));
    out
}

/// Returns a new variable constrained to the base field `M31` with the given value.
pub fn guess_m31<Value: IValue>(
    context: &mut Context<Value>,
    value: M31Wrapper<Value>,
) -> M31Wrapper<Var> {
    context.stats.guess += 1;
    let out = context.new_var(*value.get());
    context.guessed_vars.as_mut().unwrap().push(GuessVar::M31(out));
    M31Wrapper::new_unsafe(out)
}

/// Computes the map `(a, b, c, d) -> a + b * i + c * u + d * iu`. Note that the input values are
/// not necessarily in the base field `M31`.
pub fn from_partial_evals(context: &mut Context<impl IValue>, values: [Var; 4]) -> Var {
    let i = context.constant(qm31_from_u32s(0, 1, 0, 0));
    let u = context.constant(qm31_from_u32s(0, 0, 1, 0));
    let iu = context.constant(qm31_from_u32s(0, 0, 0, 1));

    eval!(
        context,
        (((values[0]) + ((values[1]) * (i))) + ((values[2]) * (u))) + ((values[3]) * (iu))
    )
}

/// A trait for creating [Var]s from values in a recursive structure.
///
/// For example, given a `Vec<Vec<QM31>>` we can create a `Vec<Vec<Var>>`.
pub trait Guess<Value: IValue> {
    type Target;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target;
}

/// Implementation of [Guess] for a single value.
impl<Value: IValue> Guess<Value> for Value {
    type Target = Var;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        guess(context, *self)
    }
}

/// Implementation of [Guess] for [Vec].
impl<Value: IValue, T: Guess<Value>> Guess<Value> for Vec<T> {
    type Target = Vec<T::Target>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        self.iter().map(|value| value.guess(context)).collect()
    }
}

/// Implementation of [Guess] for `[T; N]`.
impl<Value: IValue, T: Guess<Value>, const N: usize> Guess<Value> for [T; N] {
    type Target = [T::Target; N];

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        self.each_ref().map(|value| value.guess(context))
    }
}

/// Implementation of [Guess] for `(T, S)`.
impl<Value: IValue, T: Guess<Value>, S: Guess<Value>> Guess<Value> for (T, S) {
    type Target = (T::Target, S::Target);

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        (self.0.guess(context), self.1.guess(context))
    }
}

impl<Value: IValue, T: Guess<Value>> Guess<Value> for CirclePoint<T> {
    type Target = CirclePoint<T::Target>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        CirclePoint { x: self.x.guess(context), y: self.y.guess(context) }
    }
}
