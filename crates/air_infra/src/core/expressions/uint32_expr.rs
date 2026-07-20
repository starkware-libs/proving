use stwo_cairo_common::prover_types::cpu::UInt32;

use super::expr::*;
use super::op_expr::*;
use super::uint16_expr::*;
use super::var_expr::*;

pub type UInt32Operation = OpExpr<UInt32>;
pub type UInt32Expr = Expr<UInt32>;
const LOW_NAME: &str = "low";
const HIGH_NAME: &str = "high";

impl TryIntoFeltExpr for UInt32Expr {}

impl VarExpr<UInt32> {
    fn get_child_mut(&mut self, index: usize) -> &mut UInt16Expr {
        match self.complex_or_felt.as_complex_mut().get_mut(index).expect("Invalid index") {
            ExprImpl::UInt16(e) => e,
            _ => panic!("Invalid child type"),
        }
    }

    fn get_child(&self, index: usize) -> UInt16Expr {
        match self.complex_or_felt.as_complex().get(index).expect("Invalid index") {
            ExprImpl::UInt16(e) => e.clone(),
            _ => panic!("Invalid child type"),
        }
    }
}

impl VarExprUpdate for VarExpr<UInt32> {
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
            ComplexOrFelt::Complex(vec![UInt16Expr::Var(low).into(), UInt16Expr::Var(high).into()]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        self.get_child_mut(0).as_var_mut().set_parent(parent_var, None);
        self.get_child_mut(1).as_var_mut().set_parent(parent_var, None);
    }
}

impl UInt32Expr {
    pub fn low_mut(&mut self) -> &mut UInt16Expr {
        self.as_var_mut().get_child_mut(0)
    }

    pub fn high_mut(&mut self) -> &mut UInt16Expr {
        self.as_var_mut().get_child_mut(1)
    }

    pub fn low(&self) -> UInt16Expr {
        self.as_var().get_child(0)
    }

    pub fn high(&self) -> UInt16Expr {
        self.as_var().get_child(1)
    }
}

#[macro_export]
macro_rules! const_u32_expr {
    ($val:expr) => {
        UInt32Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const($val.into()))
    };
}

#[cfg(test)]
macro_rules! u32_expr {
    ($name:expr, $val:expr) => {
        UInt32Expr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(UInt32::from($val)),
            false,
            None,
        ))
    };
}
#[cfg(test)]
pub(super) use u32_expr;
