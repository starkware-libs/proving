use compiled_casm_air::prover_types::UInt32;

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint16_expr::*;
use super::var_expr::*;

pub type UInt32Operation = OpExpr<UInt32>;
pub type UInt32Expr = Expr<UInt32>;
const LOW_NAME: &str = "low";
const HIGH_NAME: &str = "high";

impl VarExpr<UInt32> {
    // Converts children to low and high.
    fn get_children(&mut self) -> [&mut UInt16Expr; 2] {
        let err_msg = "UInt32 var must have a low and high children.";
        if let ComplexOrFelt::Complex(children) = &mut self.complex_or_felt {
            return children
                .iter_mut()
                .map(|c| {
                    if let ExprImpl::UInt16(expr) = c {
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

impl VarExprUpdate for VarExpr<UInt32> {
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
            UInt16Expr::Var(low).into(),
            UInt16Expr::Var(high).into(),
        ]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        let [low, high] = self.get_children();
        low.get_var().set_parent(parent_var, None);
        high.get_var().set_parent(parent_var, None);
    }
}

impl UInt32Expr {
    pub fn low_mut(&mut self) -> &mut UInt16Expr {
        self.get_var().get_children()[0]
    }

    pub fn high_mut(&mut self) -> &mut UInt16Expr {
        self.get_var().get_children()[1]
    }

    pub fn low(&self) -> UInt16Expr {
        self.clone().low_mut().clone()
    }

    pub fn high(&self) -> UInt16Expr {
        self.clone().high_mut().clone()
    }
}

impl AirVar for UInt32Expr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.get_var()
            .get_children()
            .into_iter()
            .flat_map(|e| e.as_felts_mut())
            .collect()
    }
}

#[macro_export]
macro_rules! const_u32_expr {
    ($val:expr) => {
        UInt32Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $val.into(),
        ))
    };
}

#[cfg(test)]
macro_rules! u32_expr {
    ($name:expr, $val:expr) => {
        UInt32Expr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(UInt32::from($val)),
            false,
            false,
            None,
        ))
    };
}
#[cfg(test)]
pub(super) use u32_expr;
