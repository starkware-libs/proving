use stwo_cairo_common::prover_types::cpu::{FELT252WIDTH27_N_WORDS, Felt252Width27};

use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;

pub type Felt252Width27Operation = OpExpr<Felt252Width27>;
pub type Felt252Width27Expr = Expr<Felt252Width27>;
const CHILD_NAME: &str = "get_m31";

impl TryIntoFeltExpr for Felt252Width27Expr {}

impl VarExprUpdate for VarExpr<Felt252Width27> {
    fn create_complex_or_felt(&mut self, is_const: bool, deg_in_state: Option<usize>) {
        let children = (0..FELT252WIDTH27_N_WORDS)
            .map(|i| {
                FeltExpr::Var(VarExpr::new(
                    CHILD_NAME.to_string(),
                    self.value.map(|v| v.get_m31(i)),
                    is_const,
                    deg_in_state,
                ))
                .into()
            })
            .collect::<Vec<_>>();
        self.complex_or_felt = ComplexOrFelt::Complex(children);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        for (index, felt) in self.get_felt_children().into_iter().enumerate() {
            felt.as_var_mut().set_parent(parent_var, Some(index));
        }
    }
}

// Default is implemented for Felt252Width27Expr because it is returned from an external table.
impl Default for Felt252Width27Expr {
    fn default() -> Self {
        Felt252Width27Expr::Var(VarExpr::new_const(Felt252Width27 { limbs: [0, 0, 0, 0] }))
    }
}

#[macro_export]
macro_rules! const_felt252_width27 {
    ($value:expr) => {
        Felt252Width27Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const($value))
    };
}
