use core::array::from_fn;
use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::felt_expr::*;
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
        }
    }

    fn in_state(&self) -> bool {
        match self {
            ExprImpl::Felt(f) => f.in_state(),
        }
    }

    fn create_intermediate_var(&self, name: String) -> Self {
        match self {
            ExprImpl::Felt(f) => f.create_intermediate_var(name).into(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            ExprImpl::Felt(f) => f.as_felts(),
        }
    }
}

impl From<FeltExpr> for ExprImpl {
    fn from(expr: FeltExpr) -> ExprImpl {
        ExprImpl::Felt(expr)
    }
}

impl From<ExprImpl> for ProcessedAirVar {
    fn from(expr: ExprImpl) -> ProcessedAirVar {
        match expr {
            ExprImpl::Felt(f) => f.into(),
        }
    }
}

impl_air_var!([FeltExpr; 2]);
