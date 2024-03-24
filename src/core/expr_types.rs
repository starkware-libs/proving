use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};

use enum_dispatch::enum_dispatch;
use num_integer::Integer;
use serde::{Deserialize, Serialize};

pub const PRIME: u32 = 2_u32.pow(31) - 1;

pub trait NumericType: ExprType + Add + Sub + Mul + Div {}
impl NumericType for Felt {}

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
        "felt".to_string()
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
