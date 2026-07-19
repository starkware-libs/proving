use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use crate::const_expr;
use crate::core::expressions::felt_expr::*;
use crate::core::struct_var::*;

pub type BoundedFeltExpr = VarWrapper<FeltExpr, (i32, i32)>;

impl BoundedFeltExpr {
    pub fn new(expr: FeltExpr, max_bound: i32, min_bound: i32) -> Self {
        Self { var: expr, extra_info: Some((min_bound, max_bound)) }
    }

    pub fn max_bound(&self) -> i32 {
        self.extra_info.expect("BoundedFeltExpr must have bounds").1
    }

    pub fn min_bound(&self) -> i32 {
        self.extra_info.expect("BoundedFeltExpr must have bounds").0
    }

    pub fn set_max_bound(&mut self, max_bound: i32) {
        self.extra_info = Some((self.min_bound(), max_bound));
    }

    pub fn set_min_bound(&mut self, min_bound: i32) {
        self.extra_info = Some((min_bound, self.max_bound()));
    }
}

impl Default for BoundedFeltExpr {
    fn default() -> Self {
        Self { var: const_expr!(0), extra_info: Some((0, 0)) }
    }
}

impl Add for BoundedFeltExpr {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            var: self.var.clone() + other.var.clone(),
            extra_info: Some((
                self.min_bound() + other.min_bound(),
                self.max_bound() + other.max_bound(),
            )),
        }
    }
}

impl AddAssign for BoundedFeltExpr {
    fn add_assign(&mut self, other: Self) {
        *self = self.clone() + other;
    }
}

impl Sub for BoundedFeltExpr {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            var: self.var.clone() - other.var.clone(),
            extra_info: Some((
                self.min_bound() - other.max_bound(),
                self.max_bound() - other.min_bound(),
            )),
        }
    }
}

impl SubAssign for BoundedFeltExpr {
    fn sub_assign(&mut self, other: Self) {
        *self = self.clone() - other;
    }
}

impl Mul<u32> for BoundedFeltExpr {
    type Output = Self;

    fn mul(self, other: u32) -> Self {
        Self {
            var: const_expr!(other) * self.var.clone(),
            extra_info: Some((
                (other as i32) * self.min_bound(),
                (other as i32) * self.max_bound(),
            )),
        }
    }
}

impl From<(FeltExpr, i32, i32)> for BoundedFeltExpr {
    fn from((expr, max_bound, min_bound): (FeltExpr, i32, i32)) -> Self {
        Self { var: expr, extra_info: Some((min_bound, max_bound)) }
    }
}
