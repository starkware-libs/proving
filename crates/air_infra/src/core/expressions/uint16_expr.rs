use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;

pub type UInt16Operation = OpExpr<UInt16>;
pub type UInt16Expr = Expr<UInt16>;
const CHILD_NAME: &str = "as_m31";

impl VarExpr<UInt16> {
    // Converts children to FeltExpr.
    fn get_child(&mut self) -> &mut FeltExpr {
        let err_msg = "UInt16 var must have a felt child.";
        if let ComplexOrFelt::Complex(children) = &mut self.complex_or_felt {
            let child = children.get_mut(0).expect(err_msg);
            if let ExprImpl::Felt(felt_expr) = child {
                return felt_expr;
            }
        }
        panic!("{}", err_msg);
    }
}

impl VarExprUpdate for VarExpr<UInt16> {
    fn create_children(&mut self) {
        let child = VarExpr::new(
            CHILD_NAME.to_string(),
            self.value.map(|v| v.as_m31()),
            self.is_const,
            self.in_state(),
            self.intermediate_type.clone(),
        );
        self.complex_or_felt = ComplexOrFelt::Complex(vec![FeltExpr::Var(child).into()]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        self.get_child().get_var().set_parent(parent_var, None);
    }
}

impl UInt16Expr {
    pub fn as_felt_mut(&mut self) -> &mut FeltExpr {
        match self {
            UInt16Expr::Var(v) => v.get_child(),
            UInt16Expr::Op(op) => {
                if op.op == Operation::UInt16FromBool {
                    if let AirVarImpl::Expr(ExprImpl::Bool(bool_expr)) = &mut op.children[0] {
                        return bool_expr.as_felt_mut();
                    }
                } else if op.op == Operation::UInt16FromFelt {
                    if let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) = &mut op.children[0] {
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

impl AirVar for UInt16Expr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![self.as_felt_mut()]
    }
}

#[macro_export]
macro_rules! const_u16_expr {
    ($val:expr) => {
        UInt16Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $val.into(),
        ))
    };
}
