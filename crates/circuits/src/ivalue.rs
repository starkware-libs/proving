use blake2::{Blake2s256, Digest};
use itertools::Itertools;
use num_traits::Zero;
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue};
use crate::utils::le_u32s_from_bytes;
use crate::wrappers::U32Wrapper;

#[cfg(test)]
#[path = "ivalue_test.rs"]
pub mod test;

pub fn qm31_from_u32s(a: u32, b: u32, c: u32, d: u32) -> QM31 {
    QM31(CM31(a.into(), b.into()), CM31(c.into(), d.into()))
}

/// Represents a value that can be used in a [`Circuit`](crate::circuit::Circuit).
///
/// We use [QM31] for a circuit with concrete values and [NoValue] for a circuit with no
/// concrete values.
pub trait IValue:
    Sized
    + Copy
    + Clone
    + std::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + PartialEq
{
    fn from_qm31(value: QM31) -> Self;

    /// A dummy value used by [`Context::reserve`](crate::context::Context::reserve) for variables
    /// whose value will be supplied later.
    fn placeholder() -> Self;

    /// Computes pointwise multiplication of two [QM31] values.
    ///
    /// If `a = x0 + x1 * i + x2 * u + x3 * iu`, and `b = y0 + y1 * i + y2 * u + y3 * iu`,
    /// then the pointwise multiplication is
    /// `(x0 * y0) + (x1 * y1) * i + (x2 * y2) * u + (x3 * y3) * iu`.
    fn pointwise_mul(a: Self, b: Self) -> Self;

    /// For each of the four [M31] coordinates, returns `1/x` if `x != 0` and `0` if `x = 0`.
    fn pointwise_inv_or_zero(&self) -> Self;

    /// Returns a [QM31] value that consists of the LSB of each of the four [M31] coordinates.
    fn pointwise_lsb(&self) -> Self;

    /// Hashes the [QM31] inputs with Blake2s, returning the eight raw output words as
    /// a [`HashValue`] (no `M31::P` reduction).
    fn blake2s(input: &[Self], n_bytes: usize) -> HashValue<Self>;

    /// Sorts the input by the u coordinate.
    fn sort_by_u_coordinate(input: &[Self]) -> Vec<Self>;

    /// Converts an M31 value `(x, 0, 0, 0)` into limb representation `(low_u16, high_u15, 0, 0)`.
    fn m31_to_u32(&self) -> Self;

    /// Packs a `u32` value into a QM31 limb representation `(low_u16, high_u16, 0, 0)`.
    fn pack_u32(value: u32) -> Self;

    /// Unpacks a QM31 limb representation `(low_u16, high_u16, 0, 0)` back into a `u32`.
    fn unpack_u32(&self) -> u32;
}

impl IValue for QM31 {
    /// Constructs an [IValue] from the given [QM31].
    fn from_qm31(value: QM31) -> Self {
        value
    }

    /// Dummy QM31 value.
    fn placeholder() -> Self {
        qm31_from_u32s(0xabcdef, 0xabcdef, 0xabcdef, 0xabcdef)
    }

    fn pointwise_mul(x: Self, y: Self) -> Self {
        QM31(CM31(x.0.0 * y.0.0, x.0.1 * y.0.1), CM31(x.1.0 * y.1.0, x.1.1 * y.1.1))
    }

    fn pointwise_inv_or_zero(&self) -> Self {
        QM31(
            CM31(inv_or_zero(self.0.0), inv_or_zero(self.0.1)),
            CM31(inv_or_zero(self.1.0), inv_or_zero(self.1.1)),
        )
    }

    fn pointwise_lsb(&self) -> Self {
        qm31_from_u32s(self.0.0.0 % 2, self.0.1.0 % 2, self.1.0.0 % 2, self.1.1.0 % 2)
    }

    fn blake2s(input: &[Self], n_bytes: usize) -> HashValue<Self> {
        // Sanity check: check the number of bytes is consistent with the number of [QM31] values.
        assert_eq!(input.len(), n_bytes.div_ceil(16));

        // Serialize each QM31's four M31 coordinates as little-endian u32 words.
        let mut input_bytes: Vec<u8> = Vec::new();
        for x in input {
            for coord in x.to_m31_array() {
                input_bytes.extend_from_slice(&coord.0.to_le_bytes());
            }
        }

        let mut hasher = Blake2s256::new();
        hasher.update(&input_bytes[0..n_bytes]);
        let hash: [u8; 32] = hasher.finalize().into();

        let words: [u32; BLAKE2S_DIGEST_N_WORDS] = le_u32s_from_bytes(hash);
        HashValue::from(words)
    }

    fn sort_by_u_coordinate(input: &[Self]) -> Vec<Self> {
        input.iter().cloned().sorted_by_key(|val| val.1.0).collect_vec()
    }

    fn m31_to_u32(&self) -> Self {
        let x = self.0.0.0;
        qm31_from_u32s(x & 0xFFFF, x >> 16, 0, 0)
    }

    fn pack_u32(value: u32) -> Self {
        qm31_from_u32s(value & 0xFFFF, value >> 16, 0, 0)
    }

    fn unpack_u32(&self) -> u32 {
        let [low, high, 0, 0] = self.to_m31_array().map(|m| m.0) else {
            panic!("value does not have zeroes in the last two coordinates as expected");
        };
        assert!(low <= 0xFFFF && high <= 0xFFFF, "low and high coordinates must be u16 values");
        low | (high << 16)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct NoValue;

impl IValue for NoValue {
    fn from_qm31(_: QM31) -> Self {
        Self
    }

    fn placeholder() -> Self {
        Self
    }

    fn pointwise_mul(_: Self, _: Self) -> Self {
        Self
    }

    fn pointwise_inv_or_zero(&self) -> Self {
        Self
    }

    fn pointwise_lsb(&self) -> Self {
        Self
    }

    fn m31_to_u32(&self) -> Self {
        Self
    }

    fn pack_u32(_value: u32) -> Self {
        Self
    }

    fn unpack_u32(&self) -> u32 {
        0
    }

    fn blake2s(_: &[Self], _: usize) -> HashValue<Self> {
        HashValue([U32Wrapper::new_unsafe(Self); BLAKE2S_DIGEST_N_WORDS])
    }

    fn sort_by_u_coordinate(input: &[Self]) -> Vec<Self> {
        input.to_vec()
    }
}

impl std::ops::Add for NoValue {
    type Output = NoValue;
    fn add(self, _: NoValue) -> NoValue {
        Self
    }
}
impl std::ops::Sub for NoValue {
    type Output = NoValue;
    fn sub(self, _: NoValue) -> NoValue {
        Self
    }
}
impl std::ops::Mul for NoValue {
    type Output = NoValue;
    fn mul(self, _: NoValue) -> NoValue {
        Self
    }
}
impl std::ops::Div for NoValue {
    type Output = NoValue;
    fn div(self, _: NoValue) -> NoValue {
        Self
    }
}

/// Computes the inverse of a value. Returns zero if the value is zero.
fn inv_or_zero(value: M31) -> M31 {
    if value.is_zero() { M31::zero() } else { value.inverse() }
}
