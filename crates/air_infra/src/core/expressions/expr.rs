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
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
use crate::core::Felt;
// Macros

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

#[derive(Clone, Debug)]
pub(super) struct ParentExpr {
    pub(super) name: String,
    pub(super) r#type: String,
    pub(super) parent: Option<Box<ParentExpr>>,
    pub(super) index: Option<usize>,
    pub(super) child_name: String,
}

impl ParentExpr {
    pub(super) fn get_compiled_child(self) -> CompiledAirVar {
        let args = if let Some(i) = self.index {
            let index_var = CompiledAirVar::Const("usize".to_string(), i.to_string());
            vec![index_var]
        } else {
            vec![]
        };

        CompiledAirVar::MethodCall(Box::new(self.clone().into()), self.child_name, args)
    }
}

impl From<ParentExpr> for CompiledAirVar {
    fn from(expr: ParentExpr) -> CompiledAirVar {
        if let Some(parent) = expr.parent {
            return parent.get_compiled_child();
        }
        CompiledAirVar::Var(expr.r#type, expr.name)
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
