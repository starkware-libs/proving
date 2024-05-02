use core::array::from_fn;

use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::bool_expr::*;
use super::felt_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
// Macros
use crate::impl_air_var;

/// Experssions can be manipulated with binary and unary operations.
/// They have a type that determines the operations that can be performed on them.
pub trait Expr<T>
where
    T: ProverType,
{
    fn value(&self) -> Option<T>;

    // Returns the calculation of the expression as a string, when all values are known.
    // Used for testing.
    #[cfg(test)]
    fn calc(&self) -> String {
        if let Some(v) = self.value() {
            return v.calc();
        }

        panic!("VarExpr::calc() called on a VarExpr without a value");
    }
}

/// Constant expressions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConstExpr<T>
where
    T: ProverType,
{
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: T,
}

impl<T> ConstExpr<T>
where
    T: ProverType,
{
    pub fn new_const(value: T) -> Self {
        ConstExpr {
            name: value.calc(),
            value,
        }
    }
}

// All expressions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExprImpl {
    Felt(FeltExpr),
    UInt16(UInt16Expr),
    Bool(BoolExpr),
    UInt32(UInt32Expr),
    UInt64(UInt64Expr),
}

impl Default for ExprImpl {
    fn default() -> Self {
        ExprImpl::Felt(FeltExpr::default())
    }
}

impl AirVar for ExprImpl {
    fn new(name: String) -> Self {
        ExprImpl::Felt(FeltExpr::new(name))
    }

    fn name(&self) -> String {
        match self {
            ExprImpl::Felt(f) => f.name(),
            ExprImpl::UInt16(u) => u.name(),
            ExprImpl::Bool(b) => b.name(),
            ExprImpl::UInt32(u) => u.name(),
            ExprImpl::UInt64(u) => u.name(),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            ExprImpl::Felt(f) => f.in_state(),
            ExprImpl::UInt16(u) => u.in_state(),
            ExprImpl::Bool(b) => b.in_state(),
            ExprImpl::UInt32(u) => u.in_state(),
            ExprImpl::UInt64(u) => u.in_state(),
        }
    }

    fn create_intermediate_var_for_deduction(&self, name: String) -> Self {
        match self {
            ExprImpl::Felt(f) => f.create_intermediate_var_for_deduction(name).into(),
            ExprImpl::UInt16(u) => u.create_intermediate_var_for_deduction(name).into(),
            ExprImpl::Bool(b) => b.create_intermediate_var_for_deduction(name).into(),
            ExprImpl::UInt32(u) => u.create_intermediate_var_for_deduction(name).into(),
            ExprImpl::UInt64(u) => u.create_intermediate_var_for_deduction(name).into(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            ExprImpl::Felt(f) => f.as_felts(),
            ExprImpl::UInt16(u) => u.as_felts(),
            ExprImpl::Bool(b) => b.as_felts(),
            ExprImpl::UInt32(u) => u.as_felts(),
            ExprImpl::UInt64(u) => u.as_felts(),
        }
    }
}

impl From<FeltExpr> for ExprImpl {
    fn from(expr: FeltExpr) -> ExprImpl {
        ExprImpl::Felt(expr)
    }
}
impl From<UInt16Expr> for ExprImpl {
    fn from(expr: UInt16Expr) -> ExprImpl {
        ExprImpl::UInt16(expr)
    }
}
impl From<BoolExpr> for ExprImpl {
    fn from(expr: BoolExpr) -> ExprImpl {
        ExprImpl::Bool(expr)
    }
}
impl From<UInt32Expr> for ExprImpl {
    fn from(expr: UInt32Expr) -> ExprImpl {
        ExprImpl::UInt32(expr)
    }
}
impl From<UInt64Expr> for ExprImpl {
    fn from(expr: UInt64Expr) -> ExprImpl {
        ExprImpl::UInt64(expr)
    }
}

impl From<ExprImpl> for GenericAirVar {
    fn from(expr: ExprImpl) -> GenericAirVar {
        GenericAirVar::Expr(expr)
    }
}

impl From<ExprImpl> for ProcessedAirVar {
    fn from(expr: ExprImpl) -> ProcessedAirVar {
        match expr {
            ExprImpl::Felt(f) => f.into(),
            ExprImpl::UInt16(u) => u.into(),
            ExprImpl::Bool(b) => b.into(),
            ExprImpl::UInt32(u) => u.into(),
            ExprImpl::UInt64(u) => u.into(),
        }
    }
}

impl_air_var!([FeltExpr; 2]);
impl_air_var!((BoolExpr, FeltExpr));
impl_air_var!((BoolExpr, UInt16Expr));
impl_air_var!((UInt16Expr, FeltExpr));
impl_air_var!([UInt32Expr; 2]);
impl_air_var!(Vec<BoolExpr>);
