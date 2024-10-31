use prover_types::cpu::UInt64;

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint32_expr::*;
use super::var_expr::*;

pub type UInt64Operation = OpExpr<UInt64>;
pub type UInt64Expr = Expr<UInt64>;
const LOW_NAME: &str = "low";
const HIGH_NAME: &str = "high";

impl VarExpr<UInt64> {
    // Converts children to low and high.
    fn get_children(&mut self) -> [&mut UInt32Expr; 2] {
        let err_msg = "UInt64 var must have a low and high children.";
        if let ComplexOrFelt::Complex(children) = &mut self.complex_or_felt {
            return children
                .iter_mut()
                .map(|c| {
                    if let ExprImpl::UInt32(expr) = c {
                        expr
                    } else {
                        panic!("{}", err_msg);
                    }
                })
                .collect::<Vec<_>>()
                .try_into()
                .expect(err_msg);
        }
        panic!("{}", err_msg);
    }
}

impl VarExprUpdate for VarExpr<UInt64> {
    fn create_children(&mut self) {
        let low = VarExpr::new(
            LOW_NAME.to_string(),
            self.value.map(|v| v.low()),
            self.is_const,
            self.in_state(),
            self.intermediate_type.clone(),
        );
        let high = VarExpr::new(
            HIGH_NAME.to_string(),
            self.value.map(|v| v.high()),
            self.is_const,
            self.in_state(),
            self.intermediate_type.clone(),
        );
        self.complex_or_felt = ComplexOrFelt::Complex(vec![
            UInt32Expr::Var(low).into(),
            UInt32Expr::Var(high).into(),
        ]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        let [low, high] = self.get_children();
        low.get_var().set_parent(parent_var, None);
        high.get_var().set_parent(parent_var, None);
    }
}

impl UInt64Expr {
    pub fn low_mut(&mut self) -> &mut UInt32Expr {
        self.get_var().get_children()[0]
    }

    pub fn high_mut(&mut self) -> &mut UInt32Expr {
        self.get_var().get_children()[1]
    }

    pub fn low(&self) -> UInt32Expr {
        self.clone().low_mut().clone()
    }

    pub fn high(&self) -> UInt32Expr {
        self.clone().high_mut().clone()
    }
}

impl AirVar for UInt64Expr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.get_var()
            .get_children()
            .into_iter()
            .flat_map(|e| e.as_felts_mut())
            .collect()
    }
}

#[macro_export]
macro_rules! const_u64_expr {
    ($val:expr) => {
        UInt64Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $val.into(),
        ))
    };
}

#[cfg(test)]
macro_rules! u64_expr {
    ($name:expr, $val:expr) => {
        UInt64Expr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(UInt64::from($val)),
            false,
            false,
            None,
        ))
    };
}
#[cfg(test)]
pub(super) use u64_expr;
