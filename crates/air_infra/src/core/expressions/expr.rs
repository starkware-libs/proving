use std::array::from_fn;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::bool_expr::*;
use super::felt252_expr::*;
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

// All expressions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExprImpl {
    Felt(FeltExpr),
    UInt16(UInt16Expr),
    Bool(BoolExpr),
    UInt32(UInt32Expr),
    UInt64(UInt64Expr),
    Felt252(Felt252Expr),
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
            ExprImpl::Felt252(f) => f.name(),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            ExprImpl::Felt(f) => f.in_state(),
            ExprImpl::UInt16(u) => u.in_state(),
            ExprImpl::Bool(b) => b.in_state(),
            ExprImpl::UInt32(u) => u.in_state(),
            ExprImpl::UInt64(u) => u.in_state(),
            ExprImpl::Felt252(f) => f.in_state(),
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
        match self {
            ExprImpl::Felt(f) => f.let_for_deduction(name).into(),
            ExprImpl::UInt16(u) => u.let_for_deduction(name).into(),
            ExprImpl::Bool(b) => b.let_for_deduction(name).into(),
            ExprImpl::UInt32(u) => u.let_for_deduction(name).into(),
            ExprImpl::UInt64(u) => u.let_for_deduction(name).into(),
            ExprImpl::Felt252(f) => f.let_for_deduction(name).into(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            ExprImpl::Felt(f) => f.as_felts(),
            ExprImpl::UInt16(u) => u.as_felts(),
            ExprImpl::Bool(b) => b.as_felts(),
            ExprImpl::UInt32(u) => u.as_felts(),
            ExprImpl::UInt64(u) => u.as_felts(),
            ExprImpl::Felt252(f) => f.as_felts(),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            ExprImpl::Felt(f) => f.is_const(),
            ExprImpl::UInt16(u) => u.is_const(),
            ExprImpl::Bool(b) => b.is_const(),
            ExprImpl::UInt32(u) => u.is_const(),
            ExprImpl::UInt64(u) => u.is_const(),
            ExprImpl::Felt252(f) => f.is_const(),
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
impl From<Felt252Expr> for ExprImpl {
    fn from(expr: Felt252Expr) -> ExprImpl {
        ExprImpl::Felt252(expr)
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
            ExprImpl::Felt252(f) => f.into(),
        }
    }
}

impl Display for ExprImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprImpl::Felt(e) => write!(f, "{}", e),
            ExprImpl::UInt16(u) => write!(f, "{}", u),
            ExprImpl::Bool(b) => write!(f, "{}", b),
            ExprImpl::UInt32(u) => write!(f, "{}", u),
            ExprImpl::UInt64(u) => write!(f, "{}", u),
            ExprImpl::Felt252(e) => write!(f, "{}", e),
        }
    }
}

impl_air_var!([FeltExpr; 2]);
impl_air_var!((BoolExpr, FeltExpr));
impl_air_var!((BoolExpr, UInt16Expr));
impl_air_var!((UInt16Expr, FeltExpr));
impl_air_var!([UInt32Expr; 2]);
impl_air_var!(Vec<BoolExpr>);
impl_air_var!(Vec<FeltExpr>);
