use stwo_cairo_common::prover_types::cpu::{SingleFeltType, UInt16};

use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;

pub type UInt16Operation = OpExpr<UInt16>;
pub type UInt16Expr = Expr<UInt16>;
const CHILD_NAME: &str = "as_m31";

impl TryIntoFeltExpr for UInt16Expr {}

impl VarExprUpdate for VarExpr<UInt16> {
    fn create_complex_or_felt(&mut self, is_const: bool, deg_in_state: Option<usize>) {
        let child = VarExpr::new(
            CHILD_NAME.to_string(),
            self.value.map(|v| v.as_m31()),
            is_const,
            deg_in_state,
        );
        self.complex_or_felt = ComplexOrFelt::Complex(vec![FeltExpr::Var(child).into()]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        self.get_felt_mut(0).as_var_mut().set_parent(parent_var, None);
    }
}

#[macro_export]
macro_rules! const_u16_expr {
    ($val:expr) => {
        UInt16Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const($val.into()))
    };
}
