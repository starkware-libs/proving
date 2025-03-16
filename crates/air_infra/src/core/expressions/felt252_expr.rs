use stwo_cairo_common::prover_types::cpu::{Felt252, FELT252_N_WORDS};

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;
// Macros
use crate::const_felt252_expr;

pub type Felt252Operation = OpExpr<Felt252>;
pub type Felt252Expr = Expr<Felt252>;
const CHILD_NAME: &str = "get_m31";

impl VarExpr<Felt252> {
    fn get_children(&mut self) -> [&mut FeltExpr; FELT252_N_WORDS] {
        self.complex_or_felt
            .as_complex_mut()
            .iter_mut()
            .map(|c| c.as_felt_mut())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| panic!("Felt252 var must have {FELT252_N_WORDS} felt children."))
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

impl VarExprUpdate for VarExpr<Felt252> {
    fn create_children(&mut self, in_deductions: bool, felts_in_constraints: bool) {
        let children = (0..FELT252_N_WORDS)
            .map(|i| {
                FeltExpr::Var(VarExpr::new(
                    CHILD_NAME.to_string(),
                    self.value.map(|v| v.get_m31(i)),
                    self.is_const,
                    self.in_state(),
                    in_deductions,
                    felts_in_constraints,
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

impl Felt252Expr {
    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        match self {
            Felt252Expr::Var(v) => v.get_child_mut(index),
            Felt252Expr::Op(op) => match op.op {
                Operation::Felt252FromFeltsArray => op.children[0].get_felt_mut(index),
                _ => panic!("Cannot convert to felts"),
            },
        }
    }

    pub fn get_felt(&self, index: usize) -> FeltExpr {
        match self {
            Felt252Expr::Var(v) => v.get_child(index),
            Felt252Expr::Op(op) => match op.op {
                Operation::Felt252FromFeltsArray => op.children[0].get_felt(index),
                _ => panic!("Cannot convert to felts"),
            },
        }
    }
}

impl AirVar for Felt252Expr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            Felt252Expr::Var(v) => v.get_children().into_iter().collect(),
            Felt252Expr::Op(op) => match op.op {
                Operation::Felt252FromFeltsArray => op.children[0].get_felts_mut(),
                _ => panic!("Cannot convert to felts"),
            },
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
    ($low:expr) => {
        if ($low as i128) == -1 {
            const_felt252_expr!(0, 0x8000000000000110000000000000000)
        } else if $low >= 0 {
            const_felt252_expr!($low as u128, 0)
        } else {
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffff - ((-($low) - 2) as u128),
                0x08000000_00000010_ffffffff_ffffffff
            )
        }
    };
    ($low:expr, $high:expr) => {
        Felt252Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            [
                ($low & 0xffffffff_ffffffffu128) as u64,
                ($low as u128 >> 64) as u64,
                ($high & 0xffffffff_ffffffffu128) as u64,
                ($high as u128 >> 64) as u64,
            ]
            .into(),
        ))
    };
}

#[macro_export]
macro_rules! const_felt252_expr_from_felt252 {
    ($felt252:expr) => {
        Felt252Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $felt252,
        ))
    };
}

#[cfg(test)]
macro_rules! felt252_expr {
    ($name:expr, $low:expr, $high:expr) => {
        Felt252Expr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(Felt252::from([
                ($low & 0xffffffff_ffffffffu128) as u64,
                ($low as u128 >> 64) as u64,
                ($high & 0xffffffff_ffffffffu128) as u64,
                ($high as u128 >> 64) as u64,
            ])),
            false,
            false,
            true,
            true,
        ))
    };
}
#[cfg(test)]
pub(super) use felt252_expr;
