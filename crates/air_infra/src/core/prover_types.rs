use std::fmt::Debug;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};

use num_integer::Integer;
use serde::{Deserialize, Serialize};
use stwo_prover::core::fields::m31::BaseField;

pub const PRIME: u32 = 2_u32.pow(31) - 1;

pub trait AlgebraicType: ProverType + Add + Sub + Mul + Div {}
impl AlgebraicType for Felt {}

pub trait NumericType: ProverType + Rem + Shl + Shr + BitAnd + BitOr + BitXor {}
impl NumericType for UInt16 {}
impl NumericType for UInt32 {}
impl NumericType for UInt64 {}

pub trait SingleFeltType: ProverType {
    fn as_felt(&self) -> Felt;
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct Felt {
    pub value: u32,
}

impl ProverType for Felt {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn r#type() -> String {
        "Felt".to_string()
    }
}

impl Felt {
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.value == other.value,
        }
    }

    pub fn as_bool(&self) -> Bool {
        assert!(
            self.value == 0 || self.value == 1,
            "Felt value is not a bool"
        );
        Bool {
            value: self.value != 0,
        }
    }
}

impl From<u32> for Felt {
    fn from(value: u32) -> Felt {
        Felt { value }
    }
}

impl From<Felt> for BaseField {
    fn from(f: Felt) -> BaseField {
        BaseField::from_u32_unchecked(f.value)
    }
}

impl Add for Felt {
    type Output = Felt;
    fn add(self, other: Felt) -> Felt {
        Felt {
            value: ((self.value + other.value) % PRIME),
        }
    }
}
impl Sub for Felt {
    type Output = Felt;
    fn sub(self, other: Felt) -> Felt {
        Felt {
            value: ((self.value + (PRIME - other.value)) % PRIME),
        }
    }
}
impl Mul for Felt {
    type Output = Felt;
    fn mul(self, other: Felt) -> Felt {
        Felt {
            value: ((self.value as u64 * other.value as u64) % PRIME as u64) as u32,
        }
    }
}
impl Div for Felt {
    type Output = Felt;
    fn div(self, other: Felt) -> Felt {
        let egcd = Integer::extended_gcd(&(other.value as i64), &(PRIME as i64));
        let inv_other = ((egcd.x + PRIME as i64) % PRIME as i64) as u32;
        Felt {
            value: ((self.value as u64 * inv_other as u64) % PRIME as u64) as u32,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct Bool {
    pub value: bool,
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
    fn as_felt(&self) -> Felt {
        Felt {
            value: if self.value { 1 } else { 0 },
        }
    }
}

impl Bool {
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.value == other.value,
        }
    }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Bool {
        Bool { value }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct UInt16 {
    pub value: u16,
}

impl ProverType for UInt16 {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn r#type() -> String {
        "UInt16".to_string()
    }
}

impl SingleFeltType for UInt16 {
    fn as_felt(&self) -> Felt {
        Felt {
            value: self.value as u32,
        }
    }
}

impl UInt16 {
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.value == other.value,
        }
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
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.value == other.value,
        }
    }

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
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.value == other.value,
        }
    }

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

pub const FELT252_N_WORDS: usize = 21;
pub const FELT252_BITS_PER_WORD: usize = 12;

// A non-redundant representation of a 252-bit element in the field of numbers
// modulo the prime 2**251 + 17 * 2**192 + 1.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
pub struct Felt252 {
    pub low: u128,
    pub high: u128,
}

impl Felt252 {
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.low == other.low && self.high == other.high,
        }
    }

    pub fn get_felt(&self, index: usize) -> Felt {
        let shift = FELT252_BITS_PER_WORD * index;
        let value = if shift + FELT252_BITS_PER_WORD <= 128 {
            ((self.low >> shift) & 0xFFF) as u32
        } else if shift >= 128 {
            ((self.high >> (shift - 128)) & 0xFFF) as u32
        } else {
            let low_bits = 128 - shift;
            let high_shift = 128 - (FELT252_BITS_PER_WORD - low_bits);
            ((self.low >> shift) | (((self.high << high_shift) >> high_shift) << low_bits)) as u32
        };
        Felt { value }
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
