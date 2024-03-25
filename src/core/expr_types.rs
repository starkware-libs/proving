use std::fmt::Debug;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};

use enum_dispatch::enum_dispatch;
use num_integer::Integer;
use serde::{Deserialize, Serialize};

pub const PRIME: u32 = 2_u32.pow(31) - 1;

pub trait AlgebraicType: ExprType + Add + Sub + Mul + Div {}
impl AlgebraicType for Felt {}

pub trait NumericType: ExprType + Rem + Shl + Shr + BitAnd + BitOr + BitXor {}
impl NumericType for UInt16 {}
impl NumericType for UInt32 {}

pub trait AsFelt {
    fn as_felt(&self) -> Felt;
}
pub trait SingleFeltType: ExprType + AsFelt {}
impl SingleFeltType for Felt {}
impl SingleFeltType for UInt16 {}
impl SingleFeltType for Bool {}

/// Expression Types - the basic type of the variables composing the expression.
/// For exaple, felt or bool. The expression types are devided into group, depending on
/// the operations that can be performed on them.
#[enum_dispatch]
pub trait ExprType: Debug + Clone + Copy + Default {
    // Returns the calculation of the expression as a string, when all values are known.
    // Used for testing and for creating the name of constant expressions.
    fn calc(&self) -> String;
    fn as_felts(&self) -> Vec<Felt>;
    fn r#type() -> String;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Felt {
    pub value: u32,
}

impl ExprType for Felt {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn as_felts(&self) -> Vec<Felt> {
        vec![*self]
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
}

impl AsFelt for Felt {
    fn as_felt(&self) -> Felt {
        *self
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Bool {
    pub value: bool,
}

impl ExprType for Bool {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn as_felts(&self) -> Vec<Felt> {
        vec![Felt {
            value: self.value as u32,
        }]
    }
    fn r#type() -> String {
        "Bool".to_string()
    }
}

impl Bool {
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.value == other.value,
        }
    }
}

impl AsFelt for Bool {
    fn as_felt(&self) -> Felt {
        Felt {
            value: self.value as u32,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct UInt16 {
    pub value: u16,
}

impl ExprType for UInt16 {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn as_felts(&self) -> Vec<Felt> {
        vec![Felt {
            value: self.value as u32,
        }]
    }
    fn r#type() -> String {
        "UInt16".to_string()
    }
}

impl UInt16 {
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.value == other.value,
        }
    }
}

impl AsFelt for UInt16 {
    fn as_felt(&self) -> Felt {
        Felt {
            value: self.value as u32,
        }
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct UInt32 {
    pub value: u32,
}

impl UInt32 {
    pub fn eq(&self, other: &Self) -> Bool {
        Bool {
            value: self.value == other.value,
        }
    }
}

impl ExprType for UInt32 {
    fn calc(&self) -> String {
        self.value.to_string()
    }
    fn as_felts(&self) -> Vec<Felt> {
        vec![
            Felt {
                value: self.value % 2_u32.pow(16),
            },
            Felt {
                value: self.value / 2_u32.pow(16),
            },
        ]
    }
    fn r#type() -> String {
        "UInt32".to_string()
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
