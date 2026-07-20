use stwo_cairo_common::prover_types::cpu::{FELT252_N_WORDS, Felt252};

use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;
// Macros
use crate::const_felt252_expr;

pub type Felt252Operation = OpExpr<Felt252>;
pub type Felt252Expr = Expr<Felt252>;
const CHILD_NAME: &str = "get_m31";

impl TryIntoFeltExpr for Felt252Expr {}

impl VarExprUpdate for VarExpr<Felt252> {
    fn create_complex_or_felt(&mut self, is_const: bool, deg_in_state: Option<usize>) {
        let children = (0..FELT252_N_WORDS)
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

// Default is implemented for Felt252Expr because it is stored in memory.
impl Default for Felt252Expr {
    fn default() -> Self {
        const_felt252_expr!(0, 0)
    }
}

#[macro_export]
macro_rules! const_felt252_expr {
    ($low:expr) => {{
        #[allow(clippy::int_plus_one)]
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
    }};
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
        Felt252Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const($felt252))
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
            None,
        ))
    };
}
#[cfg(test)]
pub(super) use felt252_expr;
