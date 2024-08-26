use std::fmt::Display;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};

use serde::{Deserialize, Serialize};

use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::biguint_expr::*;
use super::bool_expr::*;
use super::felt252_expr::*;
use super::felt_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
use super::var_expr::*;
use crate::core::Felt;
// Macros
use crate::impl_binary_op;
use crate::impl_unary_op;

/// Binary expressions - results of binary operations on expressions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OpExpr<T>
where
    T: ProverType,
{
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: Option<T>,
    #[serde(skip)]
    pub(super) children: Vec<AirVarImpl>,
    #[serde(skip)]
    pub(super) op: Operation,
}

impl<T> OpExpr<T>
where
    T: ProverType,
{
    pub fn new(op: Operation, children: Vec<AirVarImpl>, value: Option<T>) -> Self {
        let name = match op.into() {
            OpType::Op(_) => match children.len() {
                1 => format!("({}{})", op, children[0]),
                2 => format!("({} {} {})", children[0], op, children[1]),
                _ => panic!("Invalid number of children for operation"),
            },
            OpType::Method(_) => match children.len() {
                1 => format!("({}.{}())", children[0], op),
                2 => format!("({}.{}({}))", children[0], op, children[1]),
                _ => panic!("Invalid number of children for operation"),
            },
            OpType::Static(_) => match children.len() {
                1 => format!("({}({}))", op, children[0]),
                2 => format!("({}({}, {}))", op, children[0], children[1]),
                _ => panic!("Invalid number of children for operation"),
            },
        };

        OpExpr {
            name,
            value,
            children,
            op,
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

impl<T> InternalAirVarInfo for OpExpr<T>
where
    T: ProverType,
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn in_state(&self) -> bool {
        self.children.iter().all(|v| v.in_state())
    }

    fn is_const(&self) -> bool {
        self.children.iter().all(|v| v.is_const())
    }

    fn get_intermediate_types(&self) -> Vec<IntermediateType> {
        self.children
            .iter()
            .flat_map(|v| v.get_intermediate_types())
            .collect()
    }
}

impl<T> From<OpExpr<T>> for CompiledAirVar
where
    T: ProverType,
{
    fn from(expr: OpExpr<T>) -> CompiledAirVar {
        match expr.children.len() {
            1 => {
                let child = expr.children[0].clone().into();
                match expr.op.into() {
                    OpType::Op(op) => CompiledAirVar::UnaryOp(op, Box::new(child)),
                    OpType::Method(op) => CompiledAirVar::MethodCall(Box::new(child), op, vec![]),
                    OpType::Static(op) => CompiledAirVar::StaticCall(op, vec![child]),
                }
            }
            2 => {
                let left = expr.children[0].clone().into();
                let right = expr.children[1].clone().into();
                match expr.op.into() {
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
    Felt252FromFelt,
    Felt252FromBigUInt256,
    UInt32FromFelt,
    BigUInt512FromUInt64Array,
    BigUInt256FromUInt64Array,
    BigUInt256FromBigUInt512,
    BigUInt512FromBigUInt256,
    BigUInt256FromFelt252,
    BigUInt512FromFelt252,
}

// Note that all operations from the same type should have different names for the code generation.
impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Add => write!(f, "+"),
            Operation::Sub => write!(f, "-"),
            Operation::Mul => write!(f, "*"),
            Operation::Div => write!(f, "//"),
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
            Operation::Felt252FromFelt => write!(f, "Felt252::from_m31"),
            Operation::Felt252FromBigUInt256 => write!(f, "Felt252::from_biguint256"),
            Operation::UInt32FromFelt => write!(f, "UInt32::from_m31"),
            Operation::BigUInt512FromUInt64Array => write!(f, "BigUInt::<512, 8>::from_limbs"),
            Operation::BigUInt256FromUInt64Array => write!(f, "BigUInt::<256, 4>::from_limbs"),
            Operation::BigUInt256FromBigUInt512 => {
                write!(f, "BigUInt::<256, 4>::from_biguint::<512, 8>")
            }
            Operation::BigUInt512FromBigUInt256 => {
                write!(f, "BigUInt::<512, 8>::from_biguint::<256, 4>")
            }
            Operation::BigUInt256FromFelt252 => write!(f, "BigUInt::<256, 4>::from_felt252"),
            Operation::BigUInt512FromFelt252 => write!(f, "BigUInt::<512, 8>::from_felt252"),
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
            Operation::Felt252FromFelt => OpType::Static(op.to_string()),
            Operation::Felt252FromBigUInt256 => OpType::Static(op.to_string()),
            Operation::UInt32FromFelt => OpType::Static(op.to_string()),
            Operation::BigUInt512FromUInt64Array => OpType::Static(op.to_string()),
            Operation::BigUInt256FromUInt64Array => OpType::Static(op.to_string()),
            Operation::BigUInt256FromBigUInt512 => OpType::Static(op.to_string()),
            Operation::BigUInt512FromBigUInt256 => OpType::Static(op.to_string()),
            Operation::BigUInt256FromFelt252 => OpType::Static(op.to_string()),
            Operation::BigUInt512FromFelt252 => OpType::Static(op.to_string()),
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
impl_unary_op!(from UInt16FromBool, from_bool, BoolExpr, UInt16Expr, UInt16);
impl_unary_op!(ops Not, not, BoolExpr);

impl_binary_op!(ops Add, add, FeltExpr, FeltOperation);
impl_binary_op!(ops Sub, sub, FeltExpr, FeltOperation);
impl_binary_op!(ops Mul, mul, FeltExpr, FeltOperation);
impl_binary_op!(ops Div, div, FeltExpr, FeltOperation);
impl_binary_op!(Eq, eq, FeltExpr, BoolExpr, BoolOperation);
impl_unary_op!(from BoolFromFelt, from_m31, FeltExpr, BoolExpr, Bool);
impl_unary_op!(from UInt16FromFelt, from_m31, FeltExpr, UInt16Expr, UInt16);
impl_unary_op!(from UInt32FromFelt, from_m31, FeltExpr, UInt32Expr, UInt32);
impl_unary_op!(from Felt252FromFelt, from_m31, FeltExpr, Felt252Expr, Felt252);

impl_unary_op!(from Felt252FromBigUInt256, from_biguint256, BigUInt256Expr, Felt252Expr, Felt252);
impl_binary_op!(ops Add, add, Felt252Expr, Felt252Operation);
impl_binary_op!(ops Sub, sub, Felt252Expr, Felt252Operation);
impl_binary_op!(ops Mul, mul, Felt252Expr, Felt252Operation);
impl_binary_op!(ops Div, div, Felt252Expr, Felt252Operation);
impl_binary_op!(Eq, eq, Felt252Expr, BoolExpr, BoolOperation);

impl_unary_op!(from BigUInt256FromBigUInt512, from_biguint, BigUInt512Expr, BigUInt256Expr, BigUInt);
impl_unary_op!(from BigUInt512FromBigUInt256, from_biguint, BigUInt256Expr, BigUInt512Expr, BigUInt);
impl_unary_op!(from BigUInt256FromFelt252, from_felt252, Felt252Expr, BigUInt256Expr, BigUInt);
impl_unary_op!(from BigUInt512FromFelt252, from_felt252, Felt252Expr, BigUInt512Expr, BigUInt);
impl_binary_op!(ops Add, add, BigUInt256Expr, BigUInt256Operation);
impl_binary_op!(ops Sub, sub, BigUInt256Expr, BigUInt256Operation);
impl_binary_op!(ops Mul, mul, BigUInt256Expr, BigUInt256Operation);
impl_binary_op!(ops Div, div, BigUInt256Expr, BigUInt256Operation);
impl_binary_op!(ops Add, add, BigUInt512Expr, BigUInt512Operation);
impl_binary_op!(ops Sub, sub, BigUInt512Expr, BigUInt512Operation);
impl_binary_op!(ops Mul, mul, BigUInt512Expr, BigUInt512Operation);
impl_binary_op!(ops Div, div, BigUInt512Expr, BigUInt512Operation);

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

impl<const B: usize, const L: usize> BigUIntExpr<B, L> {
    pub fn widening_mul<const DB: usize, const DL: usize>(
        self,
        other: BigUIntExpr<B, L>,
    ) -> BigUIntExpr<DB, DL>
    where
        BigUIntExpr<B, L>: Into<AirVarImpl>,
    {
        let value = self
            .value()
            .zip(other.value())
            .map(|(l, r)| l.widening_mul(r));
        BigUIntExpr::Op(BigUIntOperation::new(
            Operation::WideningMul,
            vec![self.into(), other.into()],
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

        let values = felts
            .iter()
            .filter_map(|f| f.value())
            .collect::<Vec<Felt>>();
        let value = if values.len() == felts.len() {
            Some(Felt252::from_limbs(values))
        } else {
            None
        };

        felts.resize(
            FELT252_N_WORDS,
            FeltExpr::Var(VarExpr::new_const(Felt::from(0))),
        );
        let arr = felts
            .into_iter()
            .map(|f| f.into())
            .collect::<Vec<AirVarImpl>>();
        Felt252Expr::Op(OpExpr::new(
            Operation::Felt252FromFeltsArray,
            vec![AirVarImpl::Array(arr)],
            value,
        ))
    }
}

impl<const B: usize, const L: usize> From<Vec<UInt64Expr>> for BigUIntExpr<B, L> {
    fn from(mut limbs: Vec<UInt64Expr>) -> BigUIntExpr<B, L> {
        assert!(limbs.len() <= L, "BigUIntExpr can have at most {L} felts");

        let values = limbs
            .iter()
            .filter_map(|n| n.value())
            .collect::<Vec<UInt64>>();
        let value = if values.len() == limbs.len() {
            Some(BigUInt::from_limbs(values))
        } else {
            None
        };

        limbs.resize(L, UInt64Expr::Var(VarExpr::new_const(UInt64::from(0))));
        let arr = limbs
            .into_iter()
            .map(|f| f.into())
            .collect::<Vec<AirVarImpl>>();
        match B {
            512 => BigUIntExpr::Op(OpExpr::new(
                Operation::BigUInt512FromUInt64Array,
                vec![AirVarImpl::Array(arr)],
                value,
            )),
            256 => BigUIntExpr::Op(OpExpr::new(
                Operation::BigUInt256FromUInt64Array,
                vec![AirVarImpl::Array(arr)],
                value,
            )),
            _ => panic!("Unsupported BigUInt size"),
        }
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

                $t::Op($b::new(
                    Operation::$op,
                    vec![self.into(), other.into()],
                    value,
                ))
            }
        }
    };

    ($op:ident, $op_lower:ident, $it:ident, $ot:ident, $b:ident) => {
        impl $it {
            pub fn $op_lower(self, other: $it) -> $ot {
                let value = self
                    .value()
                    .zip(other.value())
                    .map(|(l, r)| l.$op_lower(&r).into());
                $ot::Op($b::new(
                    Operation::$op,
                    vec![self.into(), other.into()],
                    value,
                ))
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

    (from $op:ident, $op_lower:ident, $it:ident, $ot:ident, $ot_lower: ident) => {
        impl From<$it> for $ot {
            fn from(input: $it) -> Self {
                let value = input.value().map(|c| $ot_lower::$op_lower(c));
                $ot::Op(OpExpr::new(Operation::$op, vec![input.into()], value))
            }
        }
    };

    ($op:ident, $name:ident, $op_lower:ident, $it:ident, $ot:ident) => {
        impl $it {
            pub fn $name(self) -> $ot {
                let value = self.value().map(|c| c.$op_lower());
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
