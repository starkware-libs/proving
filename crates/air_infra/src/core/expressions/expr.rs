use std::fmt::Display;

use enum_dispatch::enum_dispatch;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::bool_expr::*;
use super::felt252_expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
use super::var_expr::*;

use crate::core::Felt;

/// Experssions can be manipulated with binary and unary operations.
/// They have a type that determines the operations that can be performed on them.
#[enum_dispatch]
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

#[derive(Clone, Debug)]
#[enum_dispatch(InternalAirVarInfo, Expr<T>)]
pub enum GenericExprImpl<T>
where
    T: ProverType,
{
    Var(VarExpr<T>),
    Op(OpExpr<T>),
}

impl<T> GenericExprImpl<T>
where
    T: ProverType,
{
    pub(super) fn get_var(&mut self) -> &mut VarExpr<T> {
        match self {
            GenericExprImpl::Var(v) => v,
            _ => panic!("Cannot convert non-variable to Var"),
        }
    }
}

impl<T> InternalAirVarActions for GenericExprImpl<T>
where
    T: ProverType,
    Self: Into<ExprImpl>,
    VarExpr<T>: VarExprUpdate,
{
    fn new(name: String) -> Self {
        VarExpr::new(name, None, false).into()
    }

    fn let_(&self, name: String) -> Self {
        VarExpr::new(name, self.value(), self.is_const()).into()
    }
}

impl<T> From<GenericExprImpl<T>> for CompiledAirVar
where
    T: ProverType,
{
    fn from(expr: GenericExprImpl<T>) -> CompiledAirVar {
        match expr {
            GenericExprImpl::Var(v) => v.into(),
            GenericExprImpl::Op(o) => o.into(),
        }
    }
}

// All expressions.
#[derive(Clone, Debug)]
#[enum_dispatch(InternalAirVarInfo)]
pub enum ExprImpl {
    Felt(FeltExpr),
    UInt16(UInt16Expr),
    Bool(BoolExpr),
    UInt32(UInt32Expr),
    UInt64(UInt64Expr),
    Felt252(Felt252Expr),
}

impl ExprImpl {
    fn r#type(&self) -> String {
        match self {
            ExprImpl::Felt(_) => Felt::r#type(),
            ExprImpl::UInt16(_) => UInt16::r#type(),
            ExprImpl::Bool(_) => Bool::r#type(),
            ExprImpl::UInt32(_) => UInt32::r#type(),
            ExprImpl::UInt64(_) => UInt64::r#type(),
            ExprImpl::Felt252(_) => Felt252::r#type(),
        }
    }
}

impl<E> From<E> for AirVarImpl
where
    E: Into<ExprImpl>,
{
    fn from(expr: E) -> AirVarImpl {
        AirVarImpl::Expr(expr.into())
    }
}

impl Display for ExprImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", CompiledAirVar::from(self.clone()),)
    }
}

impl Serialize for ExprImpl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut expr = serializer.serialize_struct("Expr", 2)?;
        expr.serialize_field("name", &CompiledAirVar::from(self.clone()).to_string())?;
        expr.serialize_field("type", &self.r#type())?;
        expr.end()
    }
}

impl From<ExprImpl> for CompiledAirVar {
    fn from(expr: ExprImpl) -> CompiledAirVar {
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
