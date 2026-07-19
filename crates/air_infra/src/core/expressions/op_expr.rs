use std::collections::HashSet;
use std::fmt::Display;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};

use air_compile::compiled_structs::CompiledAirVar;
use serde::{Deserialize, Serialize};
use stwo_cairo_common::prover_types::cpu::{
    BigUInt, Bool, FELT252_N_WORDS, FELT252WIDTH27_N_WORDS, Felt252, Felt252Width27,
    MOD_BUILTIN_WORD_BIT_LEN, ProverType, UInt16, UInt32,
};

use super::super::variables::*;
use super::biguint_expr::*;
use super::bool_expr::*;
use super::felt_expr::*;
use super::felt252_expr::*;
use super::felt252width27_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
use super::var_expr::*;
use crate::core::Felt;
use crate::core::air_body::*;
// Macros
use crate::{const_expr, const_expr_from_m31, impl_binary_op, impl_unary_op};

/// Binary expressions - results of binary operations on expressions.
#[derive(Clone, Debug, Default)]
pub struct OpExpr<T>
where
    T: ProverType,
{
    pub(super) value: Option<T>,
    pub(super) children: Vec<AirVarImpl>,
    pub(super) op: Operation,
}

impl<T> OpExpr<T>
where
    T: ProverType,
{
    pub fn new(op: Operation, children: Vec<AirVarImpl>, value: Option<T>) -> Self {
        OpExpr { value, children, op }
    }

    pub fn as_felt(&self) -> Result<FeltExpr, String> {
        if !self.op.is_from_felts() {
            return Err(format!("Operation {:?} does not allow extracting felts", self.op));
        }
        Ok(self.children[0].as_felt())
    }

    pub fn as_felt_mut(&mut self) -> Result<&mut FeltExpr, String> {
        if !self.op.is_from_felts() {
            return Err(format!("Operation {:?} does not allow extracting felts", self.op));
        }
        Ok(self.children[0].as_felt_mut())
    }

    pub fn get_felt(&self, index: usize) -> Result<FeltExpr, String> {
        if !self.op.is_from_felts() {
            return Err(format!("Operation {:?} does not allow extracting felts", self.op));
        }
        Ok(self.children[0].get_felt(index))
    }

    pub fn get_felt_mut(&mut self, index: usize) -> Result<&mut FeltExpr, String> {
        if !self.op.is_from_felts() {
            return Err(format!("Operation {:?} does not allow extracting felts", self.op));
        }
        Ok(self.children[0].get_felt_mut(index))
    }

    pub fn as_felts_mut(&mut self) -> Result<Vec<&mut FeltExpr>, String> {
        if !self.op.is_from_felts() {
            return Err(format!("Operation {:?} does not allow extracting felts", self.op));
        }
        match &mut self.children[0] {
            AirVarImpl::Expr(expr) => Ok(vec![expr.as_felt_mut()]),
            AirVarImpl::Array(arr) => Ok(arr.iter_mut().map(|v| v.as_felt_mut()).collect()),
            _ => Err("Cannot convert to felts".to_string()),
        }
    }
}

impl<T> AsProverType<T> for OpExpr<T>
where
    T: ProverType,
{
    fn value(&self) -> Option<T> {
        self.value
    }
}

impl<T> AirVarImplInfo for OpExpr<T>
where
    T: ProverType,
{
    fn var_expr_infos(&self) -> HashSet<VarExprInfo> {
        self.children.iter().flat_map(|v| v.var_expr_infos()).collect()
    }

    fn prover_type(&self) -> String {
        T::r#type()
    }

    fn compile(self, compile_for: CompileFor) -> CompiledAirVar {
        match self.children.len() {
            1 => {
                let child = self.children[0].clone().compile(compile_for);
                match self.op.into() {
                    OpType::Op(op) => CompiledAirVar::UnaryOp(op, Box::new(child)),
                    OpType::Method(op) => CompiledAirVar::MethodCall(Box::new(child), op, vec![]),
                    OpType::Static(op) => CompiledAirVar::StaticCall(op, vec![child]),
                }
            }
            2 => {
                let left = self.children[0].clone().compile(compile_for);
                let right = self.children[1].clone().compile(compile_for);
                match self.op.into() {
                    OpType::Op(op) => CompiledAirVar::BinaryOp(Box::new(left), op, Box::new(right)),
                    OpType::Method(op) => {
                        CompiledAirVar::MethodCall(Box::new(left), op, vec![right])
                    }
                    OpType::Static(op) => CompiledAirVar::StaticCall(op, vec![left, right]),
                }
            }
            _ => panic!("Invalid number of children for operation"),
        }
    }

    fn deg_in_state(&self) -> Option<usize> {
        if T::r#type() != Felt::r#type()
            || self.children.iter().any(|c| c.prover_type() != Felt::r#type())
        {
            panic!("Only felt variables can have a degree in state");
        }

        let degs: Vec<Option<usize>> = self.children.iter().map(|c| c.deg_in_state()).collect();
        if degs.iter().any(|c| c.is_none()) {
            return None;
        }

        // OpExpr-s need unique degree computation logic, to correctly evaluate degrees of products.
        match self.op {
            Operation::Add | Operation::Sub => {
                assert!(degs.len() == 2);
                Some(std::cmp::max(degs[0].unwrap(), degs[1].unwrap()))
            }
            Operation::Mul => {
                assert!(degs.len() == 2);
                Some(degs[0].unwrap() + degs[1].unwrap())
            }
            // Inverting or dividing a non-constant expression is not a low degree polynomial.
            Operation::Div => {
                assert!(degs.len() == 2);
                if degs[1] == Some(0) { degs[0] } else { None }
            }
            Operation::Inverse => {
                if degs[0] == Some(0) {
                    Some(0)
                } else {
                    None
                }
            }
            _ => panic!("Shouldn't reach here"),
        }
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Operation {
    #[default]
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Rem,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    Neg,
    Not,
    WideningMul,
    BoolFromFelt,
    UInt16FromBool,
    UInt16FromFelt,
    Felt252FromFeltsArray,
    Felt252Width27FromFeltsArray,
    Felt252FromFelt,
    Felt252Width27FromFelt252,
    Felt252FromFelt252Width27,
    UInt32FromFelt,
    UInt32FromFeltsPair,
    BigUInt768FromBigUInt384,
    BigUInt384FromBigUInt764,
    BigUInt384FromFelt252,
    BigUInt768FromFelt252,
    BigUInt384FromFelt252Array,
    Inverse,
}

impl Operation {
    // Returns true if one can collect felt expressions directly from the children of the operation.
    pub(super) fn is_from_felts(&self) -> bool {
        matches!(
            self,
            Operation::BoolFromFelt
                | Operation::UInt16FromFelt
                | Operation::Felt252FromFeltsArray
                | Operation::Felt252FromFelt
                | Operation::Felt252Width27FromFeltsArray
                | Operation::UInt32FromFeltsPair
                | Operation::UInt32FromFelt
        )
    }
}

// Note that all operations from the same type should have different names for the code generation.
impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Add => write!(f, "+"),
            Operation::Sub => write!(f, "-"),
            Operation::Mul => write!(f, "*"),
            Operation::Div => write!(f, "/"),
            Operation::Eq => write!(f, "eq"),
            Operation::Rem => write!(f, "%"),
            Operation::Shl => write!(f, "<<"),
            Operation::Shr => write!(f, ">>"),
            Operation::BitAnd => write!(f, "&"),
            Operation::BitOr => write!(f, "|"),
            Operation::BitXor => write!(f, "^"),
            Operation::Neg => write!(f, "-"),
            Operation::Not => write!(f, "!"),
            Operation::WideningMul => write!(f, "widening_mul"),
            Operation::BoolFromFelt => write!(f, "Bool::from_m31"),
            Operation::UInt16FromBool => write!(f, "UInt16::from_bool"),
            Operation::UInt16FromFelt => write!(f, "UInt16::from_m31"),
            Operation::Felt252FromFeltsArray => write!(f, "Felt252::from_limbs"),
            Operation::Felt252Width27FromFeltsArray => write!(f, "Felt252Width27::from_limbs"),
            Operation::Felt252FromFelt => write!(f, "Felt252::from_m31"),
            Operation::Felt252Width27FromFelt252 => write!(f, "Felt252Width27::from_felt252"),
            Operation::Felt252FromFelt252Width27 => write!(f, "Felt252::from_felt252width27"),
            Operation::UInt32FromFelt => write!(f, "UInt32::from_m31"),
            Operation::UInt32FromFeltsPair => write!(f, "UInt32::from_limbs"),
            Operation::BigUInt768FromBigUInt384 => {
                write!(f, "BigUInt::<768, 12, 64>::from_biguint::<384, 6, 32>")
            }
            Operation::BigUInt384FromBigUInt764 => {
                write!(f, "BigUInt::<384, 6, 32>::from_biguint::<768, 12, 64>")
            }
            Operation::BigUInt384FromFelt252 => write!(f, "BigUInt::<384, 6, 32>::from_felt252"),
            Operation::BigUInt768FromFelt252 => write!(f, "BigUInt::<768, 12, 64>::from_felt252"),
            Operation::BigUInt384FromFelt252Array => {
                write!(f, "BigUInt::<384, 6, 32>::from_felt252_array")
            }
            Operation::Inverse => write!(f, "inverse"),
        }
    }
}

impl From<Operation> for OpType {
    fn from(op: Operation) -> OpType {
        match op {
            Operation::Eq => OpType::Method(op.to_string()),
            Operation::WideningMul => OpType::Method(op.to_string()),
            Operation::BoolFromFelt => OpType::Static(op.to_string()),
            Operation::UInt16FromBool => OpType::Static(op.to_string()),
            Operation::UInt16FromFelt => OpType::Static(op.to_string()),
            Operation::Felt252FromFeltsArray => OpType::Static(op.to_string()),
            Operation::Felt252Width27FromFeltsArray => OpType::Static(op.to_string()),
            Operation::Felt252FromFelt => OpType::Static(op.to_string()),
            Operation::Felt252Width27FromFelt252 => OpType::Static(op.to_string()),
            Operation::Felt252FromFelt252Width27 => OpType::Static(op.to_string()),
            Operation::UInt32FromFelt => OpType::Static(op.to_string()),
            Operation::UInt32FromFeltsPair => OpType::Static(op.to_string()),
            Operation::BigUInt768FromBigUInt384 => OpType::Static(op.to_string()),
            Operation::BigUInt384FromBigUInt764 => OpType::Static(op.to_string()),
            Operation::BigUInt384FromFelt252 => OpType::Static(op.to_string()),
            Operation::BigUInt768FromFelt252 => OpType::Static(op.to_string()),
            Operation::BigUInt384FromFelt252Array => OpType::Static(op.to_string()),
            Operation::Inverse => OpType::Method(op.to_string()),
            // Currently, the rest of the operations are represented as operators.
            _ => OpType::Op(op.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpType {
    Op(String),
    Method(String),
    Static(String),
}

impl_binary_op!(Eq, eq, BoolExpr, BoolExpr, BoolOperation);
impl_binary_op!(ops BitAnd, bitand, BoolExpr, BoolOperation);
impl_binary_op!(ops BitOr, bitor, BoolExpr, BoolOperation);
impl_unary_op!(from UInt16FromBool, from_bool, BoolExpr, UInt16Expr, UInt16);
impl_unary_op!(ops Not, not, BoolExpr);
impl_unary_op!(Inverse, inverse, inverse, FeltExpr, FeltExpr);

impl_binary_op!(Eq, eq, FeltExpr, BoolExpr, BoolOperation);
impl_unary_op!(from BoolFromFelt, from_m31, FeltExpr, BoolExpr, Bool);
impl_unary_op!(from UInt16FromFelt, from_m31, FeltExpr, UInt16Expr, UInt16);
impl_unary_op!(from UInt32FromFelt, from_m31, FeltExpr, UInt32Expr, UInt32);
impl_unary_op!(from Felt252FromFelt, from_m31, FeltExpr, Felt252Expr, Felt252);
impl_unary_op!(from Felt252Width27FromFelt252, from, Felt252Expr, Felt252Width27Expr, Felt252Width27);
impl_unary_op!(from Felt252FromFelt252Width27, from, Felt252Width27Expr, Felt252Expr, Felt252);

impl_binary_op!(ops Add, add, Felt252Expr, Felt252Operation);
impl_binary_op!(ops Sub, sub, Felt252Expr, Felt252Operation);
impl_binary_op!(ops Mul, mul, Felt252Expr, Felt252Operation);
impl_binary_op!(ops Div, div, Felt252Expr, Felt252Operation);
impl_binary_op!(Eq, eq, Felt252Expr, BoolExpr, BoolOperation);

impl_unary_op!(from BigUInt384FromBigUInt764, from_biguint, BigUInt768Expr, BigUInt384Expr, BigUInt);
impl_unary_op!(from BigUInt768FromBigUInt384, from_biguint, BigUInt384Expr, BigUInt768Expr, BigUInt);
impl_unary_op!(from BigUInt384FromFelt252, from_felt252, Felt252Expr, BigUInt384Expr, BigUInt);
impl_unary_op!(from BigUInt768FromFelt252, from_felt252, Felt252Expr, BigUInt768Expr, BigUInt);
impl_binary_op!(ops Add, add, BigUInt384Expr, BigUInt384Operation);
impl_binary_op!(ops Sub, sub, BigUInt384Expr, BigUInt384Operation);
impl_binary_op!(ops Mul, mul, BigUInt384Expr, BigUInt384Operation);
impl_binary_op!(ops Div, div, BigUInt384Expr, BigUInt384Operation);
impl_binary_op!(ops Add, add, BigUInt768Expr, BigUInt768Operation);
impl_binary_op!(ops Sub, sub, BigUInt768Expr, BigUInt768Operation);
impl_binary_op!(ops Mul, mul, BigUInt768Expr, BigUInt768Operation);
impl_binary_op!(ops Div, div, BigUInt768Expr, BigUInt768Operation);
impl_binary_op!(Eq, eq, BigUInt384Expr, BoolExpr, BoolOperation);

impl_binary_op!(ops Add, add, UInt16Expr, UInt16Operation);
impl_binary_op!(ops Sub, sub, UInt16Expr, UInt16Operation);
impl_binary_op!(ops Rem, rem, UInt16Expr, UInt16Operation);
impl_binary_op!(ops Shl, shl, UInt16Expr, UInt16Operation);
impl_binary_op!(ops Shr, shr, UInt16Expr, UInt16Operation);
impl_binary_op!(ops BitAnd, bitand, UInt16Expr, UInt16Operation);
impl_binary_op!(ops BitOr, bitor, UInt16Expr, UInt16Operation);
impl_binary_op!(ops BitXor, bitxor, UInt16Expr, UInt16Operation);
impl_binary_op!(Eq, eq, UInt16Expr, BoolExpr, BoolOperation);

impl_binary_op!(ops Add, add, UInt32Expr, UInt32Operation);
impl_binary_op!(ops Rem, rem, UInt32Expr, UInt32Operation);
impl_binary_op!(ops Shl, shl, UInt32Expr, UInt32Operation);
impl_binary_op!(ops Shr, shr, UInt32Expr, UInt32Operation);
impl_binary_op!(ops BitAnd, bitand, UInt32Expr, UInt32Operation);
impl_binary_op!(ops BitOr, bitor, UInt32Expr, UInt32Operation);
impl_binary_op!(ops BitXor, bitxor, UInt32Expr, UInt32Operation);
impl_binary_op!(Eq, eq, UInt32Expr, BoolExpr, BoolOperation);

impl_binary_op!(ops Add, add, UInt64Expr, UInt64Operation);
impl_binary_op!(ops Rem, rem, UInt64Expr, UInt64Operation);
impl_binary_op!(ops Shl, shl, UInt64Expr, UInt64Operation);
impl_binary_op!(ops Shr, shr, UInt64Expr, UInt64Operation);
impl_binary_op!(ops BitAnd, bitand, UInt64Expr, UInt64Operation);
impl_binary_op!(ops BitOr, bitor, UInt64Expr, UInt64Operation);
impl_binary_op!(ops BitXor, bitxor, UInt64Expr, UInt64Operation);
impl_binary_op!(Eq, eq, UInt64Expr, BoolExpr, BoolOperation);

impl From<Vec<Felt252Expr>> for BigUIntExpr<384, 6, 32> {
    fn from(mod_words: Vec<Felt252Expr>) -> BigUIntExpr<384, 6, 32> {
        // only takes MOD_BUILTIN_WORD_BIT_LEN from each Felt252
        let needed_bits = mod_words.len() * MOD_BUILTIN_WORD_BIT_LEN;
        assert!(needed_bits <= 384, "BigUIntExpr<384,6,32> can have at most 384 bits");

        let values = mod_words.iter().filter_map(|n| n.value()).collect::<Vec<Felt252>>();
        let value = if values.len() == mod_words.len() {
            Some(BigUInt::<384, 6, 32>::from_felt252_array(values))
        } else {
            None
        };

        let arr = mod_words.into_iter().map(|f| f.into()).collect::<Vec<AirVarImpl>>();
        BigUIntExpr::Op(OpExpr::new(
            Operation::BigUInt384FromFelt252Array,
            vec![AirVarImpl::Array(arr)],
            value,
        ))
    }
}

impl<const B: usize, const L: usize, const F: usize> BigUIntExpr<B, L, F> {
    pub fn widening_mul<const DB: usize, const DL: usize, const DF: usize>(
        self,
        other: BigUIntExpr<B, L, F>,
    ) -> BigUIntExpr<DB, DL, DF>
    where
        BigUIntExpr<B, L, F>: Into<AirVarImpl>,
    {
        let value = self.value().zip(other.value()).map(|(l, r)| l.widening_mul(r));
        BigUIntExpr::Op(BigUIntOperation::new(
            Operation::WideningMul,
            vec![self.into(), other.into()],
            value,
        ))
    }
}

impl From<Vec<FeltExpr>> for UInt32Expr {
    fn from(felts: Vec<FeltExpr>) -> UInt32Expr {
        assert!(felts.len() == 2, "UInt32Expr must have exactly 2 felts");
        let value = felts[0].value().zip(felts[1].value()).map(|(l, h)| UInt32::from_limbs(l, h));

        UInt32Expr::Op(OpExpr::new(
            Operation::UInt32FromFeltsPair,
            vec![AirVarImpl::Array(vec![felts[0].clone().into(), felts[1].clone().into()])],
            value,
        ))
    }
}

impl From<Vec<FeltExpr>> for Felt252Expr {
    fn from(mut felts: Vec<FeltExpr>) -> Felt252Expr {
        assert!(
            felts.len() <= FELT252_N_WORDS,
            "Felt252Expr can have at most {FELT252_N_WORDS} felts"
        );

        let values = felts.iter().filter_map(|f| f.value()).collect::<Vec<Felt>>();
        let value =
            if values.len() == felts.len() { Some(Felt252::from_limbs(&values)) } else { None };

        felts.resize(FELT252_N_WORDS, FeltExpr::Var(VarExpr::new_const(Felt::from(0))));
        let arr = felts.into_iter().map(|f| f.into()).collect::<Vec<AirVarImpl>>();
        Felt252Expr::Op(OpExpr::new(
            Operation::Felt252FromFeltsArray,
            vec![AirVarImpl::Array(arr)],
            value,
        ))
    }
}

impl From<Vec<FeltExpr>> for Felt252Width27Expr {
    fn from(mut felts: Vec<FeltExpr>) -> Felt252Width27Expr {
        assert!(
            felts.len() <= FELT252WIDTH27_N_WORDS,
            "Felt252Width27Expr can have at most {FELT252WIDTH27_N_WORDS} felts"
        );

        let values = felts.iter().filter_map(|f| f.value()).collect::<Vec<Felt>>();
        let value = if values.len() == felts.len() {
            Some(Felt252Width27::from_limbs(&values))
        } else {
            None
        };

        felts.resize(FELT252WIDTH27_N_WORDS, FeltExpr::Var(VarExpr::new_const(Felt::from(0))));
        let arr = felts.into_iter().map(|f| f.into()).collect::<Vec<AirVarImpl>>();
        Felt252Width27Expr::Op(OpExpr::new(
            Operation::Felt252Width27FromFeltsArray,
            vec![AirVarImpl::Array(arr)],
            value,
        ))
    }
}

impl Div for FeltExpr {
    type Output = FeltExpr;

    fn div(self, other: FeltExpr) -> FeltExpr {
        let value = self.value().zip(other.value()).map(|(l, r)| l.div(r));

        if self.is_const() && other.is_const() {
            return FeltExpr::Var(VarExpr::new_const(
                value.expect("Div operands are consts yet one is missing a value"),
            ));
        }

        if other.is_const() {
            return FeltExpr::Op(FeltOperation::new(
                Operation::Mul,
                vec![
                    self.into(),
                    FeltExpr::Var(VarExpr::new_const(
                        other.value().expect("Divisor is const yet its value is missing").inverse(),
                    ))
                    .into(),
                ],
                value,
            ));
        }

        FeltExpr::Op(FeltOperation::new(Operation::Div, vec![self.into(), other.into()], value))
    }
}

impl Add for FeltExpr {
    type Output = FeltExpr;

    fn add(self, other: FeltExpr) -> FeltExpr {
        let value = self.value().zip(other.value()).map(|(l, r)| l.add(r));
        if self.is_const() && other.is_const() {
            return const_expr_from_m31!(value.unwrap());
        }

        if let Some(val) = self.value()
            && val == 0.into()
        {
            return other;
        }

        if let Some(val) = other.value()
            && val == 0.into()
        {
            return self;
        }

        FeltExpr::Op(FeltOperation::new(Operation::Add, vec![self.into(), other.into()], value))
    }
}

impl Sub for FeltExpr {
    type Output = FeltExpr;

    fn sub(self, other: FeltExpr) -> FeltExpr {
        let value = self.value().zip(other.value()).map(|(l, r)| l.sub(r));
        if self.is_const() && other.is_const() {
            return const_expr_from_m31!(value.unwrap());
        }

        if let Some(val) = other.value()
            && val == 0.into()
        {
            return self;
        }

        FeltExpr::Op(FeltOperation::new(Operation::Sub, vec![self.into(), other.into()], value))
    }
}

impl Mul for FeltExpr {
    type Output = FeltExpr;

    fn mul(self, other: FeltExpr) -> FeltExpr {
        let value = self.value().zip(other.value()).map(|(l, r)| l.mul(r));
        if self.is_const() && other.is_const() {
            return const_expr_from_m31!(value.unwrap());
        }

        if self.is_const() {
            let val = self.value().unwrap();
            if val == 1.into() {
                return other;
            }
            if val == 0.into() {
                return const_expr!(0);
            }
        }

        if other.is_const() {
            let val = other.value().unwrap();
            if val == 1.into() {
                return self;
            }
            if val == 0.into() {
                return const_expr!(0);
            }
        }

        FeltExpr::Op(FeltOperation::new(Operation::Mul, vec![self.into(), other.into()], value))
    }
}

#[macro_export]
macro_rules! impl_binary_op {
    (ops $op:ident, $op_lower:ident, $t:ident, $b:ident) => {
        impl $op for $t {
            type Output = $t;
            fn $op_lower(self, other: $t) -> $t {
                let value = self.value().zip(other.value()).map(|(l, r)| l.$op_lower(r));
                if self.is_const() && other.is_const() {
                    return $t::Var(VarExpr::new_const(value.unwrap()));
                }

                $t::Op($b::new(Operation::$op, vec![self.into(), other.into()], value))
            }
        }
    };

    ($op:ident, $op_lower:ident, $it:ident, $ot:ident, $b:ident) => {
        impl $it {
            pub fn $op_lower(self, other: $it) -> $ot {
                let value = self.value().zip(other.value()).map(|(l, r)| l.$op_lower(&r).into());
                $ot::Op($b::new(Operation::$op, vec![self.into(), other.into()], value))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_unary_op {
    (ops $op:ident, $op_lower:ident, $t:ident) => {
        impl $op for $t {
            type Output = $t;
            fn $op_lower(self) -> $t {
                let value = self.value().map(|c| c.$op_lower());
                if self.is_const() {
                    return $t::Var(VarExpr::new_const(value.unwrap()));
                }

                $t::Op(OpExpr::new(Operation::$op, vec![self.into()], value))
            }
        }
    };

    (from $op:ident, $op_lower:ident, $it:ident, $ot:ident, $ot_lower:ident) => {
        impl From<$it> for $ot {
            fn from(input: $it) -> Self {
                let value = input.value().map($ot_lower::$op_lower);
                if input.is_const() {
                    return $ot::Var(VarExpr::new_const(value.unwrap()));
                }

                $ot::Op(OpExpr::new(Operation::$op, vec![input.into()], value))
            }
        }
    };

    ($op:ident, $name:ident, $op_lower:ident, $it:ident, $ot:ident) => {
        impl $it {
            pub fn $name(self) -> $ot {
                let value = self.value().map(|c| c.$op_lower());
                if self.is_const() {
                    return $ot::Var(VarExpr::new_const(value.unwrap()));
                }

                $ot::Op(OpExpr::new(Operation::$op, vec![self.into()], value))
            }
        }
    };

    (static $op:ident, $name:ident, $op_lower:ident, $it:ident, $ot:ident) => {
        impl $it {
            pub fn $name(self) -> $ot {
                let value = self.value().map(|c| $op_lower(c));
                $ot::Op(OpExpr::new(Operation::$op, vec![self.into()], value))
            }
        }
    };
}
