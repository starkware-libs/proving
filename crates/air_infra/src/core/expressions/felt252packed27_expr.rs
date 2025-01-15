use prover_types::cpu::{Felt252Packed27, FELT252PACKED27_N_WORDS};

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;

pub type Felt252Packed27Operation = OpExpr<Felt252Packed27>;
pub type Felt252Packed27Expr = Expr<Felt252Packed27>;
const CHILD_NAME: &str = "get_m31";

impl VarExpr<Felt252Packed27> {
    // Converts children to felts.
    fn get_children(&mut self) -> [&mut FeltExpr; FELT252PACKED27_N_WORDS] {
        let err_msg =
            &format!("Felt252Packed27 var must have {FELT252PACKED27_N_WORDS} felt children.");
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

impl VarExprUpdate for VarExpr<Felt252Packed27> {
    fn create_children(&mut self) {
        let children = (0..FELT252PACKED27_N_WORDS)
            .map(|i| {
                FeltExpr::Var(VarExpr::new(
                    CHILD_NAME.to_string(),
                    self.value.map(|v| v.get_m31(i)),
                    self.is_const,
                    self.in_state(),
                    self.visibility.clone(),
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

impl Felt252Packed27Expr {
    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        match self {
            Felt252Packed27Expr::Var(v) => v.get_children()[index],
            Felt252Packed27Expr::Op(op) => {
                if op.op == Operation::Felt252Packed27FromFeltsArray {
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

impl AirVar for Felt252Packed27Expr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            Felt252Packed27Expr::Var(v) => v.get_children().into_iter().collect(),
            Felt252Packed27Expr::Op(op) => {
                if op.op == Operation::Felt252Packed27FromFeltsArray {
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
