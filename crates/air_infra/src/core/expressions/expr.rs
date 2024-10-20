use enum_dispatch::enum_dispatch;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use compiled_casm_air::compiled_structs::CompiledAirVar;
use compiled_casm_air::prover_types::{BigUInt, Bool, Felt252, ProverType, UInt16, UInt32, UInt64};

use super::super::variables::*;
use super::biguint_expr::*;
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
#[derive(Clone, Debug)]
#[enum_dispatch(InternalAirVarInfo, AsProverType<T>)]
pub enum Expr<T>
where
    T: ProverType,
{
    Var(VarExpr<T>),
    Op(OpExpr<T>),
}

impl<T> Expr<T>
where
    T: ProverType,
{
    pub(super) fn get_var(&mut self) -> &mut VarExpr<T> {
        match self {
            Expr::Var(v) => v,
            _ => panic!("Cannot convert non-variable to Var"),
        }
    }
}

impl<T> InternalAirVarActions for Expr<T>
where
    T: ProverType,
    Self: Into<ExprImpl>,
    VarExpr<T>: VarExprUpdate,
{
    fn new(name: String) -> Self {
        VarExpr::new(name, None, false, false, None).into()
    }

    fn let_(&self, name: String, intermediate_type: IntermediateType) -> Self {
        VarExpr::new(
            name,
            self.value(),
            self.is_const(),
            self.in_state(),
            Some(intermediate_type),
        )
        .into()
    }
}

impl<T> From<Expr<T>> for CompiledAirVar
where
    T: ProverType,
{
    fn from(expr: Expr<T>) -> CompiledAirVar {
        match expr {
            Expr::Var(v) => v.into(),
            Expr::Op(o) => o.into(),
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
    BigUInt256(BigUIntExpr<256, 4>),
    BigUInt512(BigUIntExpr<512, 8>),
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
            ExprImpl::BigUInt256(_) => BigUInt::<256, 4>::r#type(),
            ExprImpl::BigUInt512(_) => BigUInt::<512, 8>::r#type(),
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
            ExprImpl::BigUInt256(b) => b.into(),
            ExprImpl::BigUInt512(b) => b.into(),
        }
    }
}
