use std::array::from_fn;
use std::collections::HashSet;
use std::fmt::Debug;

use compiled_casm_air::compiled_structs::CompiledAirVar;
use compiled_casm_air::public_params::PublicParam;
use enum_dispatch::enum_dispatch;
use inst_def::InstDef;
use prover_types::cpu::ProverType;
use serde::Serialize;

use super::air_fn::*;
use super::expressions::biguint_expr::*;
use super::expressions::bool_expr::*;
use super::expressions::expr::*;
use super::expressions::felt252_expr::*;
use super::expressions::felt_expr::*;
use super::expressions::op_expr::*;
use super::expressions::uint16_expr::*;
use super::expressions::uint32_expr::*;
use super::expressions::uint64_expr::*;
use super::expressions::var_expr::*;
use crate::airs::casm::builtins::modulo::mod_utils::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::opcodes::generic_opcode::generic_opcode::*;
#[cfg(test)]
use crate::core::Felt;
// Macros
use crate::impl_air_var;

pub type ChainRoundVar = FeltExpr;

#[allow(private_bounds)]
/// Every input and output of an air function is an AirVar.
pub trait AirVar: InternalAirVarActions + Debug {
    fn get_felt_descriptions(&self) -> Option<Vec<String>> {
        None
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr>;

    fn as_felts(&self) -> Vec<FeltExpr> {
        self.clone()
            .as_felts_mut()
            .into_iter()
            .map(|f| f.clone())
            .collect()
    }

    fn is_empty() -> bool {
        false
    }

    #[cfg(test)]
    fn to_values(&self) -> Option<Vec<Felt>> {
        self.as_felts()
            .iter()
            .map(|f| f.value())
            .collect::<Option<Vec<_>>>()
    }
}

// Information about air variables used by the air builder.
#[enum_dispatch]
pub trait InternalAirVarInfo {
    fn get_info(&self) -> HashSet<AirVarInfo>;

    fn prover_type(&self) -> String;

    // An AirVar is in_state if it is stored in a trace cell or a polynomial of felts stored in
    // trace cells. Used to verify that expressions of constraints are polynomials of felts
    // written to the trace. We check this in run mode, since when building an air body, we want
    // all constraints to refer to sepecial inputs carrying the AirFn name.
    fn in_state(&self) -> bool {
        self.get_info().iter().all(|i| i.in_state)
    }

    // An AirVar is_const if was created with a value and the flag is_const = true, or if it is the
    // result of operations on other constants.
    // Used to verify that a constant variable is not written to the trace in a top-level AirFn,
    // since this would create a constant column in the trace.
    // Note that in runtime, we allow deduction of constant variables in internal calls, since an
    // AirFn can be called with different inputs in different calls.
    fn is_const(&self) -> bool {
        self.get_info().iter().all(|i| i.is_const)
    }

    // An AirVar is in_constraints if each of its intermediate variables was created with
    // let_for_constraint or with let_. Similarly, an AirVar is in_deductions if each of its
    // intermediate variables was created with let_for_deduction or with let_.
    // If it has no intermediate variables, it is both in_constraints and in_deductions.
    // Used to verify that intermediate variables are used in the correct context.
    fn visibility(&self) -> Visibility {
        let visibilities = self
            .get_info()
            .iter()
            .map(|i| i.visibility.clone())
            .collect::<HashSet<_>>();
        Visibility {
            in_constraints: visibilities.iter().all(|t| t.in_constraints),
            in_deductions: visibilities.iter().all(|t| t.in_deductions),
        }
    }

    fn public_params(&self) -> HashSet<PublicParam> {
        self.get_info()
            .iter()
            .filter_map(|i| i.public_param.clone())
            .collect()
    }

    fn external_states(&self) -> HashSet<(String, Option<usize>)> {
        self.get_info()
            .iter()
            .filter_map(|i| i.external_state.clone())
            .collect()
    }
}

// Actions on air variables used by the air builder.
pub(crate) trait InternalAirVarActions: Clone + Into<AirVarImpl> {
    fn new(name: String, in_state: bool) -> Self;
    fn let_(&self, name: String, visibility: Visibility) -> Self;
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AirVarInfo {
    pub in_state: bool,
    pub is_const: bool,
    pub visibility: Visibility,
    pub public_param: Option<PublicParam>,
    pub external_state: Option<(String, Option<usize>)>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct Visibility {
    pub in_constraints: bool,
    pub in_deductions: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility {
            in_constraints: true,
            in_deductions: true,
        }
    }
}

// Describes an external preprocessed table and its type as used in the air infra.
// Note that we can have two tables with the same CONST_TRACE_ID, but different types (see for
// example Seq and SeqAddr), as long as they are represented by the same number of felts (i.e. the
// number of columns in the table).
pub trait ExtTable: Default + Debug + Clone {
    const CONST_TRACE_ID: &'static str;
    type T: AirVar;

    fn new() -> Self::T {
        let mut res = Self::T::new("".to_string(), false);
        Self::to_state(&mut res);
        res
    }

    fn to_state(v: &mut Self::T) {
        for (i, f) in v.as_felts_mut().into_iter().enumerate() {
            f.to_state(StateInfo::ExtTableState {
                name: Self::CONST_TRACE_ID.to_string(),
                col_index: i,
                log_n_rows: None,
            });
        }
    }

    // External tables that can be called with air_builder.call_external_table should implement this
    // method. See for example Seq.
    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        Self::new()
    }
}

#[derive(Clone, Debug, Default, InstDef)]
pub struct ExtTableAirFn<E>
where
    E: ExtTable,
{
    #[instdef(skip)]
    pub(super) ext_table: E,
}

impl<E> AirFn for ExtTableAirFn<E>
where
    E: ExtTable,
{
    type ExtIn = ();
    type In = ();
    type Out = E::T;

    fn call(&self, _air_builder: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        self.ext_table.call_impl(_air_builder)
    }

    fn name(&self) -> String {
        E::CONST_TRACE_ID.to_string()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }
}

#[enum_dispatch]
pub trait AsProverType<T>
where
    T: ProverType,
{
    fn value(&self) -> Option<T>;

    // Returns the calculation of the value as a string, when it is known.
    // Used for testing.
    #[cfg(test)]
    fn calc(&self) -> String {
        self.value()
            .expect("calc was called on a var without a value")
            .calc()
    }
}

// Air variables as represented in the air_body.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AirVarImpl {
    Expr(ExprImpl),
    Tuple(Vec<AirVarImpl>),
    Array(Vec<AirVarImpl>),
    Struct {
        name: Option<String>,
        r#type: String,
        fields: Vec<(String, AirVarImpl)>,
    },
}

impl InternalAirVarInfo for AirVarImpl {
    fn get_info(&self) -> HashSet<AirVarInfo> {
        match self {
            AirVarImpl::Expr(expr) => expr.get_info(),
            AirVarImpl::Tuple(vars) | AirVarImpl::Array(vars) => {
                vars.iter().flat_map(|v| v.get_info()).collect()
            }
            AirVarImpl::Struct {
                name: _,
                r#type: _,
                fields,
            } => fields.iter().flat_map(|(_, v)| v.get_info()).collect(),
        }
    }

    fn prover_type(&self) -> String {
        match self {
            AirVarImpl::Expr(expr) => expr.prover_type(),
            AirVarImpl::Tuple(vars) => {
                format!(
                    "({})",
                    vars.iter()
                        .map(|v| v.prover_type())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            AirVarImpl::Array(vars) => {
                format!(
                    "[{}]",
                    vars.iter()
                        .map(|v| v.prover_type())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            AirVarImpl::Struct {
                name: _,
                r#type,
                fields: _,
            } => r#type.to_string(),
        }
    }
}

impl From<AirVarImpl> for CompiledAirVar {
    fn from(var: AirVarImpl) -> CompiledAirVar {
        match var {
            AirVarImpl::Expr(expr) => expr.into(),
            AirVarImpl::Tuple(v) => {
                CompiledAirVar::Tuple(v.into_iter().map(|v| v.into()).collect())
            }
            AirVarImpl::Array(v) => {
                CompiledAirVar::Array(v.into_iter().map(|v| v.into()).collect())
            }
            AirVarImpl::Struct {
                name,
                r#type,
                fields,
            } => {
                if let Some(n) = name {
                    CompiledAirVar::Var(r#type, n)
                } else {
                    CompiledAirVar::Struct {
                        r#type,
                        fields: fields
                            .into_iter()
                            .map(|(name, v)| (name, v.into()))
                            .collect(),
                    }
                }
            }
        }
    }
}

impl From<()> for AirVarImpl {
    fn from(_value: ()) -> Self {
        AirVarImpl::Tuple(vec![])
    }
}

impl AirVar for () {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![]
    }

    fn is_empty() -> bool {
        true
    }
}

impl InternalAirVarActions for () {
    fn new(_name: String, _in_state: bool) -> Self {}
    fn let_(&self, _name: String, _intermediate_type: Visibility) -> Self {}
}

impl ExtTable for () {
    const CONST_TRACE_ID: &'static str = "";
    type T = ();

    fn to_state(_: &mut Self::T) {}
}

// Examples + tests

impl_air_var!((BoolExpr, UInt16Expr));
impl_air_var!((BoolExpr, FeltExpr));
impl_air_var!([UInt32Expr]);
impl_air_var!([BoolExpr]);
type TestState = FeltExpr;
impl_air_var!((FeltExpr, ChainRoundVar, TestState));

// Inline airs

// MemVerify
impl_air_var!((CasmAddress, Felt252Expr));
// ReadPositive + CondDecodeSmallSign + CondFelt252AsAddr + CondFelt252AsRelImm
impl_air_var!((Felt252Expr, FeltExpr));
// Add252 + Div252 + Mul252 + Sub252 + VerifyAdd252 + VerifyMul252
impl_air_var!([Felt252Expr]);
// MemVerifyEqual
impl_air_var!([CasmAddress]);
type Flags = [FeltExpr; 15];
type Offsets = [FeltExpr; 3];
// DecodeInstruction
impl_air_var!((Offsets, Flags));
type Cond = FeltExpr;
// MemCondVerifyEqualKnownId
impl_air_var!((CasmAddress, FeltExpr, Cond));
type Id = FeltExpr;
// ReadSmall
impl_air_var!((FeltExpr, Id));
type GenericFlags = [FeltExpr; GENERIC_FLAGS_SIZE];
type Operands = [Felt252Expr; 3];
// DecodeGenericInstruction
impl_air_var!((GenericFlags, Offsets));
// EvalOperands
impl_air_var!((CasmStateVar, GenericFlags, Offsets));
// HandleOpcodes
impl_air_var!((CasmStateVar, GenericFlags, Offsets, Operands));
// UpdateRegisters
impl_air_var!((CasmStateVar, GenericFlags, Operands));
type ModValue = [Felt252Expr; MOD_BUILTIN_N_WORDS];
// ModUtils
impl_air_var!([ModValue]);
// ModUtils
impl_air_var!((CasmAddress, FeltExpr));
type Addresses<const N: usize> = [CasmAddress; N];
// MemVerifyAll
impl_air_var!((Addresses<const N: usize>, Felt252Expr));

// Components

// RangeCheck + VerifyBitwiseXor (+ BitwiseXor + EncodeFlags + EncodeOffsets)
impl_air_var!([FeltExpr]);
// VerifyInstruction
impl_air_var!((CasmAddress, Offsets, Flags));

// Implements AirVar for arrays and tuples of air vars.
#[macro_export]
macro_rules! impl_air_var {
    ( [$s:ty] ) => {
        impl<const N:usize> AirVar for [$s;N] where $s: AirVar
        {
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                self.into_iter().flat_map(|s| s.as_felts_mut()).collect()
            }
        }

        impl<const N:usize> InternalAirVarActions for [$s;N] where $s: InternalAirVarActions {
            fn let_(&self, name: String, visibility: Visibility) -> Self {
                let mut res = self.clone();
                for (i, s) in res.iter_mut().enumerate() {
                    *s = s.let_(format!("{}[{}]", name, i), visibility.clone());
                }
                res
            }
            fn new(name: String, in_state: bool) -> Self {
                from_fn(|i| <$s as InternalAirVarActions>::new(format!("{}[{}]", name, i), in_state))
            }
        }

        impl<const N:usize> From<[$s;N]> for AirVarImpl {
            fn from(array: [$s;N]) -> AirVarImpl {
                AirVarImpl::Array(array.into_iter().map(|s| s.into()).collect())
            }
        }
    };

    ( ($($s:ident $(<$(const $lt:tt$(: $clt:tt )?),+>)?),+) ) => {
        impl $($(<$(const $lt$(: $clt )?),+>)?)+ AirVar for ($($s$(< $( $lt ),+ >)?),+)
            where $($s$(< $( $lt ),+ >)?: AirVar),+
        {
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                let mut res = vec!();
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                $(res.extend($s.as_felts_mut());)+
                res
            }
        }

        impl $($(<$(const $lt$(: $clt )?),+>)?)+ InternalAirVarActions for ($($s$(< $( $lt ),+ >)?),+)
            where $($s$(< $( $lt ),+ >)?: InternalAirVarActions),+
        {
            fn let_(&self, name: String, visibility: Visibility) -> Self {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                let mut i = 0;
                ($($s.let_(format!("{}.{}", name, { i += 1; i - 1 }), visibility.clone()),)+)
            }
            fn new(name: String, in_state: bool) -> Self {
                let mut i = 0;
                ($(<$s$(< $( $lt ),+ >)? as InternalAirVarActions>::new(format!("{}.{}", name, { i += 1; i - 1 }), in_state),)+)
            }
        }

        impl $($(<$(const $lt$(: $clt )?),+>)?)+ From<($($s$(< $( $lt ),+ >)?),+)> for AirVarImpl {
            fn from(tuple: ($($s$(< $( $lt ),+ >)?),+)) -> AirVarImpl {
                #[allow(non_snake_case)]
                let ($($s),+) = tuple.clone();
                AirVarImpl::Tuple(vec![$($s.into(),)+])
            }
        }
    };
}
