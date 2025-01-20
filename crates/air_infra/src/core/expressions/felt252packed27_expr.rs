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
    fn get_children(&mut self) -> [&mut FeltExpr; FELT252PACKED27_N_WORDS] {
        self.complex_or_felt
            .as_complex_mut()
            .iter_mut()
            .map(|c| c.as_felt_mut())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| {
                panic!("Felt252Packed27 var must have {FELT252PACKED27_N_WORDS} felt children.")
            })
    }

    fn get_child_mut(&mut self, index: usize) -> &mut FeltExpr {
        self.complex_or_felt
            .as_complex_mut()
            .get_mut(index)
            .expect("Invalid index")
            .as_felt_mut()
    }

    fn get_child(&self, index: usize) -> FeltExpr {
        self.complex_or_felt
            .as_complex()
            .get(index)
            .expect("Invalid index")
            .as_felt()
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
            felt.as_var_mut().set_parent(parent_var, Some(index));
        }
    }
}

impl Felt252Packed27Expr {
    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        match self {
            Felt252Packed27Expr::Var(v) => v.get_child_mut(index),
            Felt252Packed27Expr::Op(op) => match op.op {
                Operation::Felt252Packed27FromFeltsArray => op.children[0].get_felt_mut(index),
                _ => panic!("Cannot convert to felts"),
            },
        }
    }

    pub fn get_felt(&self, index: usize) -> FeltExpr {
        match self {
            Felt252Packed27Expr::Var(v) => v.get_child(index),
            Felt252Packed27Expr::Op(op) => match op.op {
                Operation::Felt252Packed27FromFeltsArray => op.children[0].get_felt(index),
                _ => panic!("Cannot convert to felts"),
            },
        }
    }
}

impl AirVar for Felt252Packed27Expr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            Felt252Packed27Expr::Var(v) => v.get_children().into_iter().collect(),
            Felt252Packed27Expr::Op(op) => match op.op {
                Operation::Felt252Packed27FromFeltsArray => op.children[0].get_felts_mut(),
                _ => panic!("Cannot convert to felts"),
            },
        }
    }
}
