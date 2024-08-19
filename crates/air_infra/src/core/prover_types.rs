use std::fmt::Debug;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};

use serde::{Deserialize, Serialize};
use starknet_ff::FieldElement;
use stwo_prover::core::fields::m31::M31;

pub const PRIME: u32 = 2_u32.pow(31) - 1;

pub trait AlgebraicType: ProverType + Add + Sub + Mul + Div {}
impl AlgebraicType for M31 {}
impl AlgebraicType for Felt252 {}

pub trait NumericType: ProverType + Rem + Shl + Shr + BitAnd + BitOr + BitXor {}
impl NumericType for UInt16 {}
impl NumericType for UInt32 {}
impl NumericType for UInt64 {}

pub trait SingleFeltType: ProverType {
    fn as_m31(&self) -> M31;
}

/// Expression Types - the basic type of the variables composing the expression.
/// For exaple, felt or bool. The expression types are devided into group, depending on
/// the operations that can be performed on them.
pub trait ProverType: Debug + Clone + Copy + Default {
    // Returns the calculation of the expression as a string, when all values are known.
    // Used for testing and for creating the name of constant expressions.
    fn calc(&self) -> String;
    fn r#type() -> String;
}

impl ProverType for M31 {
    fn calc(&self) -> String {
        self.to_string()
    }
    fn r#type() -> String {
        "M31".to_string()
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct Bool {
    pub value: bool,
}

impl Bool {
    pub fn from_m31(felt: M31) -> Self {
        assert!(felt.0 == 0 || felt.0 == 1, "M31 value is not a bool");
        Self { value: felt.0 != 0 }
    }
}

impl ProverType for Bool {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn r#type() -> String {
        "Bool".to_string()
    }
}

impl SingleFeltType for Bool {
    fn as_m31(&self) -> M31 {
        M31::from_u32_unchecked(if self.value { 1 } else { 0 })
    }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Bool {
        Bool { value }
    }
}

impl Not for Bool {
    type Output = Bool;
    fn not(self) -> Bool {
        Bool { value: !self.value }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct UInt16 {
    pub value: u16,
}

impl UInt16 {
    pub fn from_bool(val: Bool) -> Self {
        Self {
            value: val.value as u16,
        }
    }

    pub fn from_m31(felt: M31) -> Self {
        assert!(
            felt < M31::from_u32_unchecked(2_u32.pow(16)),
            "M31 value is not a u16"
        );
        Self {
            value: felt.0 as u16,
        }
    }
}

impl ProverType for UInt16 {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn r#type() -> String {
        "UInt16".to_string()
    }
}

impl Add for UInt16 {
    type Output = UInt16;
    fn add(self, other: UInt16) -> UInt16 {
        UInt16 {
            value: self.value.wrapping_add(other.value),
        }
    }
}

impl Sub for UInt16 {
    type Output = UInt16;
    fn sub(self, rhs: UInt16) -> UInt16 {
        UInt16 {
            value: self.value.wrapping_sub(rhs.value),
        }
    }
}

impl SingleFeltType for UInt16 {
    fn as_m31(&self) -> M31 {
        M31::from_u32_unchecked(self.value as u32)
    }
}

impl From<u16> for UInt16 {
    fn from(value: u16) -> UInt16 {
        UInt16 { value }
    }
}

impl Rem for UInt16 {
    type Output = UInt16;
    fn rem(self, other: UInt16) -> UInt16 {
        UInt16 {
            value: self.value % other.value,
        }
    }
}
impl Shl for UInt16 {
    type Output = UInt16;
    fn shl(self, other: UInt16) -> UInt16 {
        UInt16 {
            value: self.value << other.value,
        }
    }
}
impl Shr for UInt16 {
    type Output = UInt16;
    fn shr(self, other: UInt16) -> UInt16 {
        UInt16 {
            value: self.value >> other.value,
        }
    }
}
impl BitAnd for UInt16 {
    type Output = UInt16;
    fn bitand(self, other: UInt16) -> UInt16 {
        UInt16 {
            value: self.value & other.value,
        }
    }
}
impl BitOr for UInt16 {
    type Output = UInt16;
    fn bitor(self, other: UInt16) -> UInt16 {
        UInt16 {
            value: self.value | other.value,
        }
    }
}
impl BitXor for UInt16 {
    type Output = UInt16;
    fn bitxor(self, other: UInt16) -> UInt16 {
        UInt16 {
            value: self.value ^ other.value,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct UInt32 {
    pub value: u32,
}

impl UInt32 {
    pub fn low(&self) -> UInt16 {
        UInt16 {
            value: (self.value & 0xFFFF) as u16,
        }
    }

    pub fn high(&self) -> UInt16 {
        UInt16 {
            value: (self.value >> 16) as u16,
        }
    }

    pub fn from_m31(felt: M31) -> Self {
        Self { value: felt.0 }
    }
}

impl From<u32> for UInt32 {
    fn from(value: u32) -> UInt32 {
        UInt32 { value }
    }
}

impl ProverType for UInt32 {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn r#type() -> String {
        "UInt32".to_string()
    }
}

impl Add for UInt32 {
    type Output = UInt32;
    fn add(self, other: UInt32) -> UInt32 {
        UInt32 {
            value: self.value.wrapping_add(other.value),
        }
    }
}
impl Rem for UInt32 {
    type Output = UInt32;
    fn rem(self, other: UInt32) -> UInt32 {
        UInt32 {
            value: self.value % other.value,
        }
    }
}
impl Shl for UInt32 {
    type Output = UInt32;
    fn shl(self, other: UInt32) -> UInt32 {
        UInt32 {
            value: self.value << other.value,
        }
    }
}
impl Shr for UInt32 {
    type Output = UInt32;
    fn shr(self, other: UInt32) -> UInt32 {
        UInt32 {
            value: self.value >> other.value,
        }
    }
}
impl BitAnd for UInt32 {
    type Output = UInt32;
    fn bitand(self, other: UInt32) -> UInt32 {
        UInt32 {
            value: self.value & other.value,
        }
    }
}
impl BitOr for UInt32 {
    type Output = UInt32;
    fn bitor(self, other: UInt32) -> UInt32 {
        UInt32 {
            value: self.value | other.value,
        }
    }
}
impl BitXor for UInt32 {
    type Output = UInt32;
    fn bitxor(self, other: UInt32) -> UInt32 {
        UInt32 {
            value: self.value ^ other.value,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct UInt64 {
    pub value: u64,
}

impl UInt64 {
    pub fn low(&self) -> UInt32 {
        UInt32 {
            value: (self.value & 0xFFFFFFFF) as u32,
        }
    }

    pub fn high(&self) -> UInt32 {
        UInt32 {
            value: (self.value >> 32) as u32,
        }
    }
}

impl From<u64> for UInt64 {
    fn from(value: u64) -> UInt64 {
        UInt64 { value }
    }
}

impl ProverType for UInt64 {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn r#type() -> String {
        "UInt64".to_string()
    }
}

impl Add for UInt64 {
    type Output = UInt64;
    fn add(self, other: UInt64) -> UInt64 {
        UInt64 {
            value: self.value.wrapping_add(other.value),
        }
    }
}
impl Rem for UInt64 {
    type Output = UInt64;
    fn rem(self, other: UInt64) -> UInt64 {
        UInt64 {
            value: self.value % other.value,
        }
    }
}
impl Shl for UInt64 {
    type Output = UInt64;
    fn shl(self, other: UInt64) -> UInt64 {
        UInt64 {
            value: self.value << other.value,
        }
    }
}
impl Shr for UInt64 {
    type Output = UInt64;
    fn shr(self, other: UInt64) -> UInt64 {
        UInt64 {
            value: self.value >> other.value,
        }
    }
}
impl BitAnd for UInt64 {
    type Output = UInt64;
    fn bitand(self, other: UInt64) -> UInt64 {
        UInt64 {
            value: self.value & other.value,
        }
    }
}
impl BitOr for UInt64 {
    type Output = UInt64;
    fn bitor(self, other: UInt64) -> UInt64 {
        UInt64 {
            value: self.value | other.value,
        }
    }
}
impl BitXor for UInt64 {
    type Output = UInt64;
    fn bitxor(self, other: UInt64) -> UInt64 {
        UInt64 {
            value: self.value ^ other.value,
        }
    }
}

pub const FELT252_N_WORDS: usize = 28;
pub const FELT252_BITS_PER_WORD: usize = 9;

// NOTE! This assumes Felt252 has shape (28, 9).
pub const P_FELTS: [u32; FELT252_N_WORDS] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 136, 0, 0, 0, 0, 0, 256,
];

// A non-redundant representation of a 252-bit element in the field of numbers
// modulo the prime 2**251 + 17 * 2**192 + 1.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct Felt252 {
    // TODO: Consider replacing this representation with a [u64; 4] limb representation,
    // to simplify FieldElement conversions.
    pub low: u128,
    pub high: u128,
}

impl Felt252 {
    pub fn get_m31(&self, index: usize) -> M31 {
        let shift = FELT252_BITS_PER_WORD * index;
        let value = if shift + FELT252_BITS_PER_WORD <= 128 {
            ((self.low >> shift) & 0x1FF) as u32
        } else if shift >= 128 {
            ((self.high >> (shift - 128)) & 0x1FF) as u32
        } else {
            let low_bits = 128 - shift;
            let high_shift = 128 - (FELT252_BITS_PER_WORD - low_bits);
            ((self.low >> shift) | (((self.high << high_shift) >> high_shift) << low_bits)) as u32
        };
        M31::from_u32_unchecked(value)
    }

    pub fn from_m31_(felts: Vec<M31>) -> Self {
        assert!(felts.len() <= FELT252_N_WORDS, "Invalid number of felts");
        let mut low = 0;
        let mut high = 0;
        for (index, felt) in felts.iter().enumerate() {
            let shift = FELT252_BITS_PER_WORD * index;
            if shift + FELT252_BITS_PER_WORD <= 128 {
                low |= (felt.0 as u128) << shift;
            } else if shift >= 128 {
                high |= (felt.0 as u128) << (shift - 128);
            } else {
                let low_bits = 128 - shift;
                let high_felt = (felt.0 as u128) << low_bits;
                low |= ((felt.0 as u128) - (high_felt >> low_bits)) << shift;
                high |= high_felt;
            }
        }

        Self { low, high }
    }

    pub fn from_m31(felt: M31) -> Self {
        Self {
            low: felt.0 as u128,
            high: 0,
        }
    }
}

// Convert between Felt252 and FieldElement for performing field operations.
// Note that FieldElements are in Montgomery form, and for efficiency and simplicity, we skip the
// conversion in both direction. We thus have to compensate with extra factors when performing
// multiplication and division.
impl From<Felt252> for FieldElement {
    fn from(n: Felt252) -> FieldElement {
        let mut limbs = [0u64; 4];
        limbs[0] = (n.low & (u64::MAX as u128)) as u64;
        limbs[1] = (n.low >> 64) as u64;
        limbs[2] = (n.high & (u64::MAX as u128)) as u64;
        limbs[3] = (n.high >> 64) as u64;

        FieldElement::from_mont(limbs)
    }
}
impl From<FieldElement> for Felt252 {
    fn from(n: FieldElement) -> Felt252 {
        let limbs = n.into_mont();
        let low = (limbs[0] as u128) + ((limbs[1] as u128) << 64);
        let high = (limbs[2] as u128) + ((limbs[3] as u128) << 64);

        Felt252 { low, high }
    }
}

impl Add for Felt252 {
    type Output = Felt252;
    fn add(self, other: Felt252) -> Felt252 {
        let self_ff: FieldElement = self.into();
        let other_ff: FieldElement = other.into();

        (self_ff + other_ff).into()
    }
}

impl Sub for Felt252 {
    type Output = Felt252;
    fn sub(self, other: Felt252) -> Felt252 {
        let self_ff: FieldElement = self.into();
        let other_ff: FieldElement = other.into();

        (self_ff - other_ff).into()
    }
}

// This value is equal to 2**512 % PRIME, which compensates for the two Montgomery divisions
// by 2**256 performed in the two multiplications below.
const FELT252_MONT_MUL_FACTOR: FieldElement = FieldElement::from_mont([
    18446741271209837569,
    5151653887,
    18446744073700081664,
    576413109808302096,
]);

impl Mul for Felt252 {
    type Output = Felt252;
    fn mul(self, other: Felt252) -> Felt252 {
        let self_ff: FieldElement = self.into();
        let other_ff: FieldElement = other.into();

        (self_ff * other_ff * FELT252_MONT_MUL_FACTOR).into()
    }
}

// The Montgomery inversion adds a factor of 2**512 to the inverse of `other`, so it is only
// necessary to perform one more Montgomery reduction after computing self * other.invert().
// The reduction is accessible by multipliying by 1 (i.e. the Montgomery form of 2**-256).
const FELT252_MONT_DIV_FACTOR: FieldElement = FieldElement::from_mont([1, 0, 0, 0]);

impl Div for Felt252 {
    type Output = Felt252;
    fn div(self, other: Felt252) -> Felt252 {
        let self_ff: FieldElement = self.into();
        let other_ff: FieldElement = other.into();

        (self_ff * other_ff.invert().expect("Division by zero") * FELT252_MONT_DIV_FACTOR).into()
    }
}

impl From<(u128, u128)> for Felt252 {
    fn from((low, high): (u128, u128)) -> Felt252 {
        Felt252 { low, high }
    }
}

impl ProverType for Felt252 {
    fn calc(&self) -> String {
        format!("({}, {})", self.low, self.high)
    }
    fn r#type() -> String {
        "Felt252".to_string()
    }
}
