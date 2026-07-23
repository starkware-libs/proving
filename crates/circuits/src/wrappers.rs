use std::fmt::Debug;

use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::context::{Context, GuessVar, Var};
use crate::eval;
use crate::ivalue::{IValue, NoValue, qm31_from_u32s};
use crate::ops::{Guess, guess_m31};

#[cfg(test)]
#[path = "wrappers_test.rs"]
pub mod test;

/// Represents a value that should be in the base field `M31`.
///
/// Using the [Guess] trait on [M31Wrapper] and gates that guarantee that the guessed value is
/// indeed in the base field `M31`.
#[derive(Clone, PartialEq)]
pub struct M31Wrapper<T>(T);

impl<T> M31Wrapper<T> {
    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn new_unsafe(var: T) -> Self {
        Self(var)
    }
}

impl From<M31> for M31Wrapper<QM31> {
    fn from(value: M31) -> Self {
        M31Wrapper(value.into())
    }
}

impl M31Wrapper<Var> {
    /// Adds a multiplication gate to the circuit, and returns the output variable.
    pub fn mul(context: &mut Context<impl IValue>, a: Self, b: Self) -> Self {
        Self(eval!(context, (*a.get()) * (*b.get())))
    }
}

impl From<NoValue> for M31Wrapper<NoValue> {
    fn from(_: NoValue) -> Self {
        M31Wrapper(NoValue)
    }
}

impl<Value: IValue> Guess<Value> for M31Wrapper<Value> {
    type Target = M31Wrapper<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        // `guess_m31` constrains the guessed variable to the base field `M31` during
        // finalization, so no further masking is required here.
        guess_m31(context, self.clone())
    }
}

impl<T: Debug> Debug for M31Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "M31({:?})", self.0)
    }
}

/// Represents a value that should be a 16-bit unsigned integer.
///
/// A `u16` is stored in a single `M31` limb as `(value, 0, 0, 0)`. Using the [Guess] trait on
/// [U16Wrapper] guesses that limb and range-constrains it to `[0, 2^16)` during finalization, so
/// the guessed value is guaranteed to be a valid `u16`.
#[derive(Clone, Copy, PartialEq)]
pub struct U16Wrapper<T>(T);

impl<T> U16Wrapper<T> {
    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn new_unsafe(var: T) -> Self {
        Self(var)
    }
}

impl From<u16> for U16Wrapper<QM31> {
    fn from(value: u16) -> Self {
        U16Wrapper(qm31_from_u32s(value as u32, 0, 0, 0))
    }
}

impl From<NoValue> for U16Wrapper<NoValue> {
    fn from(_: NoValue) -> Self {
        U16Wrapper(NoValue)
    }
}

impl<Value: IValue> Guess<Value> for U16Wrapper<Value> {
    type Target = U16Wrapper<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        // The guessed variable is range-constrained to `[0, 2^16)` during finalization (see
        // `Context::finalize_guessed_vars`), so no further masking is required here.
        context.stats.guess += 1;
        let out = context.new_var(*self.get());
        context.guessed_vars.as_mut().unwrap().push(GuessVar::U16(out));
        U16Wrapper::new_unsafe(out)
    }
}

impl<T: Debug> Debug for U16Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "U16({:?})", self.0)
    }
}

/// Represents a value that should be a 32-bit unsigned integer.
///
/// A `u32` is packed into a `QM31` as `(low_u16, high_u16, 0, 0)` (see
/// [`IValue::pack_u32`]). Using the [Guess] trait on [U32Wrapper] guesses the two 16-bit limbs
/// separately — each range-constrained to `[0, 2^16)` as a [U16Wrapper] — and recombines them, so
/// the guessed value is guaranteed to be a valid `u32`.
#[derive(Clone, Copy, PartialEq)]
pub struct U32Wrapper<T>(T);

impl<T> U32Wrapper<T> {
    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn new_unsafe(var: T) -> Self {
        Self(var)
    }
}

impl From<u32> for U32Wrapper<QM31> {
    fn from(value: u32) -> Self {
        U32Wrapper(QM31::pack_u32(value))
    }
}

impl From<NoValue> for U32Wrapper<NoValue> {
    fn from(_: NoValue) -> Self {
        U32Wrapper(NoValue)
    }
}

impl<Value: IValue> Guess<Value> for U32Wrapper<Value> {
    type Target = U32Wrapper<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        // A `u32` is stored as `(low_u16, high_u16, 0, 0)`. Guess each 16-bit limb separately so
        // that each is range-constrained to `[0, 2^16)` during finalization, then recombine them as
        // `low + high * i` to rebuild the packed representation.
        let value = Value::unpack_u32(self.get());
        let low = U16Wrapper::new_unsafe(Value::from_qm31(qm31_from_u32s(value & 0xFFFF, 0, 0, 0)))
            .guess(context);
        let high = U16Wrapper::new_unsafe(Value::from_qm31(qm31_from_u32s(value >> 16, 0, 0, 0)))
            .guess(context);
        let i = context.constant(qm31_from_u32s(0, 1, 0, 0));
        U32Wrapper(eval!(context, (*low.get()) + ((*high.get()) * (i))))
    }
}

impl<T: Debug> Debug for U32Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "U32({:?})", self.0)
    }
}
