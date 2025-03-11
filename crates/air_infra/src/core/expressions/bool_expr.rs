use stwo_cairo_common::prover_types::cpu::{Bool, SingleFeltType};

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;

pub type BoolOperation = OpExpr<Bool>;
pub type BoolExpr = Expr<Bool>;
const CHILD_NAME: &str = "as_m31";

impl TryIntoFeltExpr for BoolExpr {}

impl VarExprUpdate for VarExpr<Bool> {
    fn create_children(&mut self) {
        let child = VarExpr::new(
            CHILD_NAME.to_string(),
            self.value.map(|v| v.as_m31()),
            self.is_const,
            self.in_state(),
        );
        self.complex_or_felt = ComplexOrFelt::Complex(vec![FeltExpr::Var(child).into()]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        self.get_felt_mut(0)
            .as_var_mut()
            .set_parent(parent_var, None);
    }
}

#[macro_export]
macro_rules! const_bool_expr {
    ($val:expr) => {
        BoolExpr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $val.into(),
        ))
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! bool_expr {
    ($name:expr, $val:expr) => {
        BoolExpr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(Bool::from($val)),
            false,
            false,
        ))
    };
}
