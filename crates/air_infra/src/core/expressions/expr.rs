#[cfg(test)]
use std::fmt::Display;

use compiled_casm_air::compiled_structs::CompiledAirVar;
use enum_dispatch::enum_dispatch;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use stwo_cairo_common::prover_types::cpu::{
    BigUInt, Bool, Felt252, Felt252Width27, ProverType, UInt16, UInt32, UInt64,
};

use super::super::air_body::*;
use super::super::variables::*;
use super::biguint_expr::*;
use super::bool_expr::*;
use super::felt252_expr::*;
use super::felt252width27_expr::*;
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
    pub(super) fn as_var_mut(&mut self) -> &mut VarExpr<T> {
        match self {
            Expr::Var(v) => v,
            _ => panic!("Cannot convert non-variable to Var"),
        }
    }

    pub(super) fn as_var(&self) -> &VarExpr<T> {
        match self {
            Expr::Var(v) => v,
            _ => panic!("Cannot convert non-variable to Var"),
        }
    }

    // <as_felt_mut> and <as_felt> work for expressions that are represented by a single felt, for
    // example, Uint16 or Bool.
    pub fn as_felt_mut(&mut self) -> &mut FeltExpr {
        match self {
            Expr::Var(v) => v.as_felt_mut(),
            Expr::Op(o) => o.as_felt_mut(),
        }
    }

    pub fn as_felt(&self) -> FeltExpr {
        match self {
            Expr::Var(v) => v.as_felt(),
            Expr::Op(o) => o.as_felt(),
        }
    }

    // <get_felt_mut> and <get_felt> work for expressions that their children are felts, for
    // example, Felt252 or BigUint.
    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        match self {
            Expr::Var(v) => v.get_felt_mut(index),
            Expr::Op(o) => o.get_felt_mut(index),
        }
    }

    pub fn get_felt(&self, index: usize) -> FeltExpr {
        match self {
            Expr::Var(v) => v.get_felt(index),
            Expr::Op(o) => o.get_felt(index),
        }
    }

    pub fn compile(self, compile_for: CompileFor) -> CompiledAirVar {
        match self {
            Expr::Var(v) => v.compile(compile_for),
            Expr::Op(o) => o.compile(compile_for),
        }
    }
}

pub trait TryIntoFeltExpr {
    fn try_into_felt(&mut self) -> Option<&mut FeltExpr> {
        None
    }
}

impl<T> AirVar for Expr<T>
where
    T: ProverType,
    Self: Into<ExprImpl> + TryIntoFeltExpr,
    VarExpr<T>: VarExprUpdate,
{
    fn new(name: String, in_state: bool) -> Self {
        VarExpr::new(name, None, false, in_state).into()
    }

    fn let_for_deduction(&self, name: String) -> (Self, Intermediate) {
        let interm = Intermediate::new_for_deduction(&name, self);
        let mut var = VarExpr::new_from(name, self);
        var.is_deduction_intermediate = true;
        (var.into(), interm)
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        if self.try_into_felt().is_some() {
            return vec![self.try_into_felt().unwrap()];
        }

        match self {
            Expr::Var(v) => v
                .complex_or_felt
                .as_complex_mut()
                .iter_mut()
                .flat_map(|c| c.as_felts_mut())
                .collect(),
            Expr::Op(o) => o.as_felts_mut(),
        }
    }
}

#[cfg(test)]
impl<T> Display for Expr<T>
where
    T: ProverType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Var(v) => write!(f, "{}", v.clone().compile(CompileFor::Deductions)),
            Expr::Op(o) => write!(f, "{}", o.clone().compile(CompileFor::Deductions)),
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
    Felt252Width27(Felt252Width27Expr),
    BigUInt384(BigUIntExpr<384, 6, 32>),
    BigUInt768(BigUIntExpr<768, 12, 64>),
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
            ExprImpl::Felt252Width27(_) => Felt252Width27::r#type(),
            ExprImpl::BigUInt384(_) => BigUInt::<384, 6, 32>::r#type(),
            ExprImpl::BigUInt768(_) => BigUInt::<768, 12, 64>::r#type(),
        }
    }

    pub fn as_felt_mut(&mut self) -> &mut FeltExpr {
        match self {
            ExprImpl::Felt(f) => f,
            ExprImpl::Bool(b) => b.as_felt_mut(),
            ExprImpl::UInt16(u) => u.as_felt_mut(),
            _ => panic!("Cannot convert to Felt"),
        }
    }

    pub fn as_felt(&self) -> FeltExpr {
        match self {
            ExprImpl::Felt(f) => f.clone(),
            ExprImpl::Bool(b) => b.as_felt(),
            ExprImpl::UInt16(u) => u.as_felt(),
            _ => panic!("Cannot convert to Felt"),
        }
    }

    pub fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            ExprImpl::Felt(f) => vec![f],
            ExprImpl::Bool(b) => vec![b.as_felt_mut()],
            ExprImpl::UInt16(u) => vec![u.as_felt_mut()],
            ExprImpl::UInt32(u) => u.as_felts_mut(),
            ExprImpl::UInt64(u) => u.as_felts_mut(),
            ExprImpl::Felt252(f) => f.as_felts_mut(),
            ExprImpl::Felt252Width27(f) => f.as_felts_mut(),
            ExprImpl::BigUInt384(b) => b.as_felts_mut(),
            ExprImpl::BigUInt768(b) => b.as_felts_mut(),
        }
    }

    pub fn as_felts(&self) -> Vec<FeltExpr> {
        match self {
            ExprImpl::Felt(f) => vec![f.clone()],
            ExprImpl::Bool(b) => vec![b.as_felt()],
            ExprImpl::UInt16(u) => vec![u.as_felt()],
            ExprImpl::UInt32(u) => u.as_felts(),
            ExprImpl::UInt64(u) => u.as_felts(),
            ExprImpl::Felt252(f) => f.as_felts(),
            ExprImpl::Felt252Width27(f) => f.as_felts(),
            ExprImpl::BigUInt384(b) => b.as_felts(),
            ExprImpl::BigUInt768(b) => b.as_felts(),
        }
    }

    pub fn compile(self, compile_for: CompileFor) -> CompiledAirVar {
        match self {
            ExprImpl::Felt(f) => f.compile(compile_for),
            ExprImpl::UInt16(u) => u.compile(compile_for),
            ExprImpl::Bool(b) => b.compile(compile_for),
            ExprImpl::UInt32(u) => u.compile(compile_for),
            ExprImpl::UInt64(u) => u.compile(compile_for),
            ExprImpl::Felt252(f) => f.compile(compile_for),
            ExprImpl::Felt252Width27(f) => f.compile(compile_for),
            ExprImpl::BigUInt384(b) => b.compile(compile_for),
            ExprImpl::BigUInt768(b) => b.compile(compile_for),
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
        expr.serialize_field(
            "name",
            &self.clone().compile(CompileFor::Deductions).to_string(),
        )?;
        expr.serialize_field("type", &self.r#type())?;
        expr.end()
    }
}
