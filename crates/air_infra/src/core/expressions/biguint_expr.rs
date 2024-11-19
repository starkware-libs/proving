use prover_types::cpu::BigUInt;

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint64_expr::*;
use super::var_expr::*;

pub type BigUIntOperation<const B: usize, const L: usize> = OpExpr<BigUInt<B, L>>;
pub type BigUInt384Operation = BigUIntOperation<384, 6>;
pub type BigUInt768Operation = BigUIntOperation<768, 12>;

pub type BigUIntExpr<const B: usize, const L: usize> = Expr<BigUInt<B, L>>;
pub type BigUInt384Expr = BigUIntExpr<384, 6>;
pub type BigUInt768Expr = BigUIntExpr<768, 12>;

const CHILD_NAME: &str = "get_u64";

impl<const B: usize, const L: usize> VarExpr<BigUInt<B, L>> {
    fn get_children(&mut self) -> [&mut UInt64Expr; L] {
        let err_msg = &format!("BigUint var must have {L} uint64 children.");
        if let ComplexOrFelt::Complex(children) = &mut self.complex_or_felt {
            return children
                .iter_mut()
                .map(|c| {
                    if let ExprImpl::UInt64(expr) = c {
                        expr
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

impl<const B: usize, const L: usize> VarExprUpdate for VarExpr<BigUInt<B, L>> {
    fn create_children(&mut self) {
        let children = (0..L)
            .map(|i| {
                UInt64Expr::Var(VarExpr::new(
                    CHILD_NAME.to_string(),
                    self.value.map(|v| v.get_u64(i)),
                    self.is_const,
                    self.in_state(),
                    self.intermediate_type.clone(),
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

impl<const B: usize, const L: usize> BigUIntExpr<B, L> {
    pub fn get_uint64_mut(&mut self, index: usize) -> &mut UInt64Expr {
        match self {
            BigUIntExpr::Var(v) => v.get_children()[index],
            BigUIntExpr::Op(op) => {
                if (op.op == Operation::BigUInt384FromUInt64Array && B == 384)
                    || (op.op == Operation::BigUInt768FromUInt64Array && B == 768)
                {
                    if let AirVarImpl::Array(arr) = &mut op.children[0] {
                        if let AirVarImpl::Expr(ExprImpl::UInt64(expr)) =
                            arr.get_mut(index).expect("index out of bounds")
                        {
                            return expr;
                        }
                    }
                }

                panic!("Cannot convert to u64");
            }
        }
    }

    pub fn get_uint64(&self, index: usize) -> UInt64Expr {
        self.clone().get_uint64_mut(index).clone()
    }
}

impl<const B: usize, const L: usize> AirVar for BigUIntExpr<B, L>
where
    Self: Into<ExprImpl>,
{
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.get_var()
            .get_children()
            .into_iter()
            .flat_map(|e| e.as_felts_mut())
            .collect()
    }
}

#[macro_export]
macro_rules! const_bigu384_expr {
    ($limb0:expr, $limb1:expr, $limb2:expr, $limb3:expr, $limb4:expr, $limb5:expr) => {
        BigUIntExpr::<384, 6>::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            [$limb0, $limb1, $limb2, $limb3, $limb4, $limb5].into(),
        ))
    };
}

#[macro_export]
macro_rules! const_bigu768_expr {
    ($limb0:expr, $limb1:expr, $limb2:expr, $limb3:expr, $limb4:expr, $limb5:expr, $limb6:expr, $limb7:expr, $limb8:expr, $limb9:expr, $limb10:expr, $limb11:expr) => {
        BigUIntExpr::<768, 12>::Var($crate::core::expressions::var_expr::VarExpr::new_const(
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
        BigUIntExpr::<384, 6>::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(BigUInt::<384, 6>::from([
                $limb0, $limb1, $limb2, $limb3, $limb4, $limb5,
            ])),
            false,
            false,
            None,
        ))
    };
}
#[cfg(test)]
pub(super) use bigu384_expr;
