use enum_dispatch::enum_dispatch;

use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::bool_expr::*;
use super::felt252_expr::*;
use super::felt_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;

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
    pub fn r#type(&self) -> String {
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

impl Default for ExprImpl {
    fn default() -> Self {
        ExprImpl::Felt(FeltExpr::default())
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
