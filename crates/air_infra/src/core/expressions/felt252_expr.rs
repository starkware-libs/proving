use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;

// Macros
use crate::const_felt252_expr;

pub type Felt252Operation = OpExpr<Felt252>;
pub type Felt252Expr = GenericExprImpl<Felt252>;
const CHILD_NAME: &str = "get_m31";

impl VarExpr<Felt252> {
    // Converts children to felts.
    fn get_children(&mut self) -> [&mut FeltExpr; FELT252_N_WORDS] {
        let err_msg = &format!("Felt252 var must have {FELT252_N_WORDS} felt children.");
        if let ComplexOrFelt::Complex(children) = &mut self.complex_or_felt {
            return children
                .iter_mut()
                .map(|c| {
                    if let ExprImpl::Felt(felt_expr) = c {
                        felt_expr
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

impl VarExprUpdate for VarExpr<Felt252> {
    fn create_children(&mut self) {
        let children = (0..FELT252_N_WORDS)
            .map(|i| {
                FeltExpr::Var(VarExpr::new(
                    CHILD_NAME.to_string(),
                    self.value.map(|v| v.get_m31(i)),
                    self.is_const,
                ))
                .into()
            })
            .collect::<Vec<_>>();
        self.complex_or_felt = ComplexOrFelt::Complex(children);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        for (index, felt) in self.get_children().into_iter().enumerate() {
            felt.get_var().set_parent(parent_var, Some(index));
        }
    }
}

impl Felt252Expr {
    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        match self {
            Felt252Expr::Var(v) => v.get_children()[index],
            Felt252Expr::Op(op) => {
                if op.op == Operation::Felt252FromFeltsArray {
                    if let AirVarImpl::Array(arr) = &mut op.children[0] {
                        if let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) =
                            arr.get_mut(index).expect("index out of bounds")
                        {
                            return felt_expr;
                        }
                    }
                }
                panic!("Cannot convert to felts");
            }
        }
    }

    pub fn get_felt(&self, index: usize) -> FeltExpr {
        self.clone().get_felt_mut(index).clone()
    }
}

impl AirVar for Felt252Expr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            Felt252Expr::Var(v) => v.get_children().into_iter().collect(),
            Felt252Expr::Op(op) => {
                if op.op == Operation::Felt252FromFeltsArray {
                    if let AirVarImpl::Array(arr) = &mut op.children[0] {
                        let len = arr.len();
                        let mut felts = vec![];
                        for g in arr {
                            if let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) = g {
                                felts.push(felt_expr);
                            }
                        }
                        if felts.len() == len {
                            return felts;
                        }
                    }
                }
                panic!("Cannot convert to felts");
            }
        }
    }
}

// Default is implemented for Felt252Expr because it is stored in memory.
impl Default for Felt252Expr {
    fn default() -> Self {
        const_felt252_expr!(0, 0)
    }
}

#[macro_export]
macro_rules! const_felt252_expr {
    ($low:expr, $high:expr) => {
        Felt252Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            ($low, $high).into(),
        ))
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! felt252_expr {
    ($name:expr, $low:expr, $high:expr) => {
        Felt252Expr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some($crate::core::prover_types::Felt252::from(($low, $high))),
            false,
        ))
    };
}
