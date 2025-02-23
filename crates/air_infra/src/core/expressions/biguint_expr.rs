use stwo_cairo_common::prover_types::cpu::BigUInt;

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::var_expr::*;

pub type BigUIntOperation<const B: usize, const L: usize, const F: usize> =
    OpExpr<BigUInt<B, L, F>>;
pub type BigUInt384Operation = BigUIntOperation<384, 6, 32>;
pub type BigUInt768Operation = BigUIntOperation<768, 12, 64>;

pub type BigUIntExpr<const B: usize, const L: usize, const F: usize> = Expr<BigUInt<B, L, F>>;
pub type BigUInt384Expr = BigUIntExpr<384, 6, 32>;
pub type BigUInt768Expr = BigUIntExpr<768, 12, 64>;

const CHILD_NAME: &str = "get_m31";

impl<const B: usize, const L: usize, const F: usize> VarExpr<BigUInt<B, L, F>> {
    fn get_children(&mut self) -> [&mut FeltExpr; F] {
        self.complex_or_felt
            .as_complex_mut()
            .iter_mut()
            .map(|c| c.as_felt_mut())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| panic!("BigUint var must have {F} felt children."))
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

impl<const B: usize, const L: usize, const F: usize> VarExprUpdate for VarExpr<BigUInt<B, L, F>> {
    fn create_children(&mut self, in_deductions: bool, felts_in_constraints: bool) {
        let children = (0..F)
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

impl<const B: usize, const L: usize, const F: usize> BigUIntExpr<B, L, F> {
    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        self.as_var_mut().get_child_mut(index)
    }

    pub fn get_felt(&self, index: usize) -> FeltExpr {
        self.as_var().get_child(index)
    }
}

impl<const B: usize, const L: usize, const F: usize> AirVar for BigUIntExpr<B, L, F>
where
    Self: Into<ExprImpl>,
{
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.as_var_mut().get_children().into_iter().collect()
    }
}

#[macro_export]
macro_rules! const_bigu384_expr {
    ($limb0:expr, $limb1:expr, $limb2:expr, $limb3:expr, $limb4:expr, $limb5:expr) => {
        BigUIntExpr::<384, 6, 32>::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            [$limb0, $limb1, $limb2, $limb3, $limb4, $limb5].into(),
        ))
    };
}

#[macro_export]
macro_rules! const_bigu768_expr {
    ($limb0:expr, $limb1:expr, $limb2:expr, $limb3:expr, $limb4:expr, $limb5:expr, $limb6:expr, $limb7:expr, $limb8:expr, $limb9:expr, $limb10:expr, $limb11:expr) => {
        BigUIntExpr::<768, 12, 64>::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            [
                $limb0, $limb1, $limb2, $limb3, $limb4, $limb5, $limb6, $limb7, $limb8, $limb9,
                $limb10, $limb11,
            ]
            .into(),
        ))
    };
}

#[cfg(test)]
macro_rules! bigu384_expr {
    ($name:expr, $limb0:expr, $limb1:expr, $limb2:expr, $limb3:expr, $limb4:expr, $limb5:expr) => {
        BigUIntExpr::<384, 6, 32>::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(BigUInt::<384, 6, 32>::from([
                $limb0, $limb1, $limb2, $limb3, $limb4, $limb5,
            ])),
            false,
            false,
            true,
            true,
        ))
    };
}
#[cfg(test)]
pub(super) use bigu384_expr;
