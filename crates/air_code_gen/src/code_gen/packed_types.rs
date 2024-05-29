use std::mem::transmute;
use std::ops::{Add, BitAnd, BitOr, BitXor, Rem, Shl, Shr};
use std::simd::num::SimdUint;
use std::simd::Simd;

use air_infra::core::prover_types::{UInt16, UInt32, UInt64, PRIME};
use stwo_prover::core::backend::simd::m31::PackedM31;

pub const LOG_N_LANES: u32 = 4;

pub const N_LANES: usize = 1 << LOG_N_LANES;

pub const P_BROADCAST: Simd<u32, N_LANES> = Simd::from_array([PRIME; N_LANES]);

pub type PackedFelt = PackedM31;

pub trait PackedFeltType {
    fn as_felt(&self) -> PackedFelt;
}

#[derive(Clone, Copy, Debug)]
pub struct PackedBool {
    pub(crate) value: Simd<u8, N_LANES>,
}

impl PackedFeltType for PackedBool {
    fn as_felt(&self) -> PackedFelt {
        // Safe.
        unsafe { PackedFelt::from_simd_unchecked(self.value.cast()) }
    }
}
#[derive(Copy, Clone, Debug, Default)]
pub struct PackedUInt16 {
    value: Simd<u16, N_LANES>,
}

impl PackedUInt16 {
    pub fn broadcast(value: UInt16) -> Self {
        Self {
            value: Simd::splat(value.value),
        }
    }
    pub fn from_array(arr: [UInt16; N_LANES]) -> Self {
        // Safe because UInt16 is u16.
        unsafe {
            Self {
                value: Simd::from_array(transmute(arr)),
            }
        }
    }
    pub fn as_array(&self) -> [UInt16; N_LANES] {
        // Safe because UInt16 is u16.
        unsafe { transmute(self.value.to_array()) }
    }
}

impl PackedFeltType for PackedUInt16 {
    fn as_felt(&self) -> PackedFelt {
        // Safe.
        unsafe { PackedFelt::from_simd_unchecked(self.value.cast()) }
    }
}

impl Add for PackedUInt16 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

impl Rem for PackedUInt16 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value % rhs.value,
        }
    }
}
impl Shl for PackedUInt16 {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value << rhs.value,
        }
    }
}
impl Shr for PackedUInt16 {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value >> rhs.value,
        }
    }
}
impl BitAnd for PackedUInt16 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value & rhs.value,
        }
    }
}
impl BitOr for PackedUInt16 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value | rhs.value,
        }
    }
}
impl BitXor for PackedUInt16 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value ^ rhs.value,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PackedUInt32 {
    pub(crate) value: Simd<u32, N_LANES>,
}

impl PackedUInt32 {
    pub fn broadcast(value: UInt32) -> Self {
        Self {
            value: Simd::splat(value.value),
        }
    }
    pub fn from_array(arr: [UInt32; N_LANES]) -> Self {
        // Safe because UInt32 is u32.
        unsafe {
            Self {
                value: Simd::from_array(transmute(arr)),
            }
        }
    }

    pub fn as_array(&self) -> [UInt32; N_LANES] {
        // Safe because UInt32 is u32.
        unsafe { transmute(self.value.to_array()) }
    }
}

impl Rem for PackedUInt32 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value % rhs.value,
        }
    }
}
impl Shl for PackedUInt32 {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value << rhs.value,
        }
    }
}
impl Shr for PackedUInt32 {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value >> rhs.value,
        }
    }
}
impl BitAnd for PackedUInt32 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value & rhs.value,
        }
    }
}
impl BitOr for PackedUInt32 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value | rhs.value,
        }
    }
}
impl BitXor for PackedUInt32 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value ^ rhs.value,
        }
    }
}
impl Add for PackedUInt32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PackedUInt64 {
    pub(crate) value: Simd<u64, N_LANES>,
}

impl PackedUInt64 {
    pub fn broadcast(value: UInt64) -> Self {
        Self {
            value: Simd::splat(value.value),
        }
    }
    pub fn from_array(arr: [UInt64; N_LANES]) -> Self {
        // Safe because UInt64is u64.
        unsafe {
            Self {
                value: Simd::from_array(transmute(arr)),
            }
        }
    }
    pub fn as_array(&self) -> [UInt64; N_LANES] {
        // Safe because UInt64 is u64.
        unsafe { transmute(self.value.to_array()) }
    }
}

impl Add for PackedUInt64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

impl Rem for PackedUInt64 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value % rhs.value,
        }
    }
}
impl Shl for PackedUInt64 {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value << rhs.value,
        }
    }
}
impl Shr for PackedUInt64 {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value >> rhs.value,
        }
    }
}
impl BitAnd for PackedUInt64 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value & rhs.value,
        }
    }
}
impl BitOr for PackedUInt64 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value | rhs.value,
        }
    }
}
impl BitXor for PackedUInt64 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value ^ rhs.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use air_infra::core::prover_types::{UInt16, UInt32, UInt64};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};
    use stwo_prover::core::backend::simd::m31::N_LANES;

    use super::{PackedUInt16, PackedUInt32, PackedUInt64};
    use crate::code_gen::packed_types::PackedFeltType;

    macro_rules! packed_uint_test {
        ($packed_ty:ty, $prover_ty: ty, $inner_ty: ty) => {
            let mut rng = SmallRng::seed_from_u64(0);
            let a = <$packed_ty>::from_array(std::array::from_fn(|_| {
                <$prover_ty>::from(rng.gen::<$inner_ty>())
            }));
            let b = <$packed_ty>::from_array(std::array::from_fn(|_| {
                <$prover_ty>::from(rng.gen::<$inner_ty>())
            }));
            let c = <$packed_ty>::from_array(
                (0..N_LANES as $inner_ty)
                    .map(<$prover_ty>::from)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            );

            let add = a + b;
            let rem = a % b;
            let shl = b << c;
            let shr = a >> c;
            let and = a & b;
            let or = a | b;
            let xor = a ^ b;

            add.as_array().into_iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    x.value,
                    (a.as_array()[i] + b.as_array()[i]).value,
                    "Add Failed."
                );
            });

            rem.as_array().into_iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    x.value,
                    (a.as_array()[i] % b.as_array()[i]).value,
                    "Rem Failed."
                );
            });
            shl.as_array().into_iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    x.value,
                    (b.as_array()[i] << c.as_array()[i]).value,
                    "Shl Failed."
                );
            });
            shr.as_array().into_iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    x.value,
                    (a.as_array()[i] >> c.as_array()[i]).value,
                    "Shr Failed."
                );
            });
            and.as_array().into_iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    x.value,
                    (a.as_array()[i] & b.as_array()[i]).value,
                    "And Failed."
                );
            });
            or.as_array().into_iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    x.value,
                    (a.as_array()[i] | b.as_array()[i]).value,
                    "Or Failed."
                );
            });
            xor.as_array().into_iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    x.value,
                    (a.as_array()[i] ^ b.as_array()[i]).value,
                    "Xor Failed."
                );
            });
        };
    }

    #[test]
    fn packed_uint16_test() {
        packed_uint_test!(PackedUInt16, UInt16, u16);
    }

    #[test]
    fn packed_uint32_test() {
        packed_uint_test!(PackedUInt32, UInt32, u32);
    }

    #[test]
    fn packed_uint64_test() {
        packed_uint_test!(PackedUInt64, UInt64, u64);
    }

    #[test]
    fn packed_uint_16_as_felt_test() {
        let mut rng = SmallRng::seed_from_u64(0);
        let a = PackedUInt16::from_array(std::array::from_fn(|_| UInt16::from(rng.gen::<u16>())));
        let felt = a.as_felt();
        felt.to_array().into_iter().enumerate().for_each(|(i, x)| {
            assert_eq!(x, felt.to_array()[i], "As Felt Failed.");
        });
    }
}
