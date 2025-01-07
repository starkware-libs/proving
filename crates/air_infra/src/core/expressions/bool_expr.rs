use prover_types::cpu::{Bool, SingleFeltType};

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;

pub type BoolOperation = OpExpr<Bool>;
pub type BoolExpr = Expr<Bool>;
const CHILD_NAME: &str = "as_m31";

impl VarExpr<Bool> {
    // Converts children to FeltExpr.
    fn get_child(&mut self) -> &mut FeltExpr {
        let err_msg = "Bool var must have a felt child.";
        if let ComplexOrFelt::Complex(children) = &mut self.complex_or_felt {
            let child = children.get_mut(0).expect(err_msg);
            if let ExprImpl::Felt(felt_expr) = child {
                return felt_expr;
            }
        }
        panic!("{}", err_msg);
    }
}

impl VarExprUpdate for VarExpr<Bool> {
    fn create_children(&mut self) {
        let child = VarExpr::new(
            CHILD_NAME.to_string(),
            self.value.map(|v| v.as_m31()),
            self.is_const,
            self.in_state(),
            self.visibility.clone(),
        );
        self.complex_or_felt = ComplexOrFelt::Complex(vec![FeltExpr::Var(child).into()]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        self.get_child().get_var().set_parent(parent_var, None);
    }
}

impl BoolExpr {
    pub fn as_felt_mut(&mut self) -> &mut FeltExpr {
        match self {
            BoolExpr::Var(v) => v.get_child(),
            BoolExpr::Op(u) => {
                if u.op == Operation::BoolFromFelt {
                    if let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) = &mut u.children[0] {
                        return felt_expr;
                    }
                }
                panic!("Cannot convert to a Felt");
            }
        }
    }

    pub fn as_felt(&self) -> FeltExpr {
        self.clone().as_felt_mut().clone()
    }
}

impl AirVar for BoolExpr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![self.as_felt_mut()]
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
            Visibility::default(),
        ))
    };
}
