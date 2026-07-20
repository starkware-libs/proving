use stwo_cairo_common::prover_types::cpu::UInt64;

use super::expr::*;
use super::op_expr::*;
use super::uint32_expr::*;
use super::var_expr::*;

pub type UInt64Operation = OpExpr<UInt64>;
pub type UInt64Expr = Expr<UInt64>;
const LOW_NAME: &str = "low";
const HIGH_NAME: &str = "high";

impl TryIntoFeltExpr for UInt64Expr {}

impl VarExpr<UInt64> {
    fn get_child_mut(&mut self, index: usize) -> &mut UInt32Expr {
        match self.complex_or_felt.as_complex_mut().get_mut(index).expect("Invalid index") {
            ExprImpl::UInt32(e) => e,
            _ => panic!("Invalid child type"),
        }
    }

    fn get_child(&self, index: usize) -> UInt32Expr {
        match self.complex_or_felt.as_complex().get(index).expect("Invalid index") {
            ExprImpl::UInt32(e) => e.clone(),
            _ => panic!("Invalid child type"),
        }
    }
}

impl VarExprUpdate for VarExpr<UInt64> {
    fn create_complex_or_felt(&mut self, is_const: bool, deg_in_state: Option<usize>) {
        let low =
            VarExpr::new(LOW_NAME.to_string(), self.value.map(|v| v.low()), is_const, deg_in_state);
        let high = VarExpr::new(
            HIGH_NAME.to_string(),
            self.value.map(|v| v.high()),
            is_const,
            deg_in_state,
        );
        self.complex_or_felt =
            ComplexOrFelt::Complex(vec![UInt32Expr::Var(low).into(), UInt32Expr::Var(high).into()]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        self.get_child_mut(0).as_var_mut().set_parent(parent_var, None);
        self.get_child_mut(1).as_var_mut().set_parent(parent_var, None);
    }
}

impl UInt64Expr {
    pub fn low_mut(&mut self) -> &mut UInt32Expr {
        self.as_var_mut().get_child_mut(0)
    }

    pub fn high_mut(&mut self) -> &mut UInt32Expr {
        self.as_var_mut().get_child_mut(1)
    }

    pub fn low(&self) -> UInt32Expr {
        self.as_var().get_child(0)
    }

    pub fn high(&self) -> UInt32Expr {
        self.as_var().get_child(1)
    }
}

#[macro_export]
macro_rules! const_u64_expr {
    ($val:expr) => {
        UInt64Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const($val.into()))
    };
}

#[cfg(test)]
macro_rules! u64_expr {
    ($name:expr, $val:expr) => {
        UInt64Expr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(UInt64::from($val)),
            false,
            None,
        ))
    };
}
#[cfg(test)]
pub(super) use u64_expr;
