use std::array::from_fn;
use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;

use compiled_casm_air::compiled_structs::CompiledAirVar;
use compiled_casm_air::public_params::PublicParam;
use enum_dispatch::enum_dispatch;
use inst_def::InstDef;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::ProverType;

use super::air_body::*;
use super::air_fn::*;
use super::expressions::biguint_expr::*;
use super::expressions::bool_expr::*;
use super::expressions::expr::*;
use super::expressions::felt252_expr::*;
use super::expressions::felt252width27_expr::*;
use super::expressions::felt_expr::*;
use super::expressions::op_expr::*;
use super::expressions::uint16_expr::*;
use super::expressions::uint32_expr::*;
use super::expressions::uint64_expr::*;
use super::expressions::var_expr::*;
use crate::airs::casm::builtins::modulo::mod_utils::*;
use crate::airs::casm::builtins::pedersen::partial_ec_mul::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::opcodes::blake::create_blake_output::*;
use crate::airs::casm::opcodes::blake::decode_blake_opcode::*;
use crate::airs::casm::opcodes::blake::round::*;
use crate::airs::casm::opcodes::generic_opcode::generic_opcode::*;
#[cfg(test)]
use crate::core::Felt;
// Macros
use crate::impl_air_var;

pub type RoundNumVar = FeltExpr;
pub type ChainIdVar = FeltExpr;

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

    // Defines a new variable for the top level variable, visibile in deductions, and a variable for
    // each felt, visible in constraints.
    fn rec_let(&self, name: String) -> (Self, Vec<Intermediate>) {
        let mut res = self.let_for_deduction(name.clone());

        // When the expression is a single felt, create an intermediate known both in deductions and
        // constraints.
        if let AirVarImpl::Expr(ExprImpl::Felt(f)) = self.clone().into() {
            if f.is_directly_in_state() {
                return (self.clone(), vec![]);
            }
            let var = self.clone().into();
            // Cast <res> into a mut felt expression.
            let res_as_felt = res.as_felts_mut().into_iter().next().expect("No felts");
            res_as_felt.let_for_constraint(name.clone());
            return (
                res,
                vec![Intermediate {
                    name,
                    var,
                    visibility: Visibility::new(true, true),
                }],
            );
        }

        // We have to create the variable for <self> before its felts, because <let_> creates
        // the felts as well. Then, we recreate the felts from their original expressions
        // (<orig_felts>) and update <res>.
        let mut orig_felts = self.as_felts();
        let mut vars = vec![Intermediate {
            name: name.clone(),
            var: self.clone().into(),
            visibility: Visibility::new(true, false),
        }];

        for (i, (orig_felt, felt)) in orig_felts.iter_mut().zip(res.as_felts_mut()).enumerate() {
            let parent_source = felt.clone();
            if orig_felt.is_directly_in_state() {
                *felt = orig_felt.clone();
                felt.copy_parent(&parent_source);
                continue;
            }

            let felt_name = format!("{}_limb_{}", name, i);
            vars.push(Intermediate {
                name: felt_name.clone(),
                var: AirVarImpl::Expr(orig_felt.clone().into()),
                visibility: Visibility::new(false, true),
            });
            felt.let_for_constraint(felt_name);
        }

        (res, vars)
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

    // An AirVar is visible in constraints if it's a felt and each of its intermediate variables is
    // in constraints. An AirVar is visible in deductions if each of its intermediate variables is
    // in deductions, if it has no intermediate variables, or if you can create it from a parent
    // variable (see VarExpr). Felts that are directly in the state (see FeltExpr) are visible
    // in constraints and in deductions. Used to verify that variables are used in the correct
    // context.
    fn visibility(&self) -> Visibility {
        let visibilities = self
            .get_info()
            .iter()
            .map(|i| i.visibility)
            .collect::<HashSet<_>>();
        Visibility {
            in_constraints: visibilities.iter().all(|t| t.in_constraints),
            in_deductions: visibilities.iter().all(|t| t.in_deductions),
        }
    }

    fn public_params(&self) -> BTreeSet<PublicParam> {
        self.get_info()
            .iter()
            .filter_map(|i| i.public_param.clone())
            .collect()
    }

    fn external_states(&self) -> BTreeSet<(String, Vec<String>)> {
        self.get_info()
            .iter()
            .filter_map(|i| i.external_state.clone())
            .collect()
    }
}

// Actions on air variables used by the air builder.
pub(crate) trait InternalAirVarActions: Clone + Into<AirVarImpl> {
    fn new(name: String, in_state: bool) -> Self;
    // TODO(AnatG): Consider returning a tuple of Self and the new Intermediate.
    fn let_for_deduction(&self, name: String) -> Self;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AirVarInfo {
    pub in_state: bool,
    pub is_const: bool,
    pub visibility: Visibility,
    pub public_param: Option<PublicParam>,
    pub external_state: Option<(String, Vec<String>)>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq, Hash, Default)]
pub struct Visibility {
    pub in_constraints: bool,
    pub in_deductions: bool,
}

impl Visibility {
    pub fn new(in_deductions: bool, in_constraints: bool) -> Self {
        Self {
            in_constraints,
            in_deductions,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Intermediate {
    pub name: String,
    pub var: AirVarImpl,
    pub visibility: Visibility,
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

    // The arguments to the constructor of the preprocessed column object in stwo-cairo, except for
    // the column index.
    fn args() -> Vec<String> {
        vec![]
    }

    fn to_state(v: &mut Self::T) {
        let felts = v.as_felts_mut();
        let n = felts.len();

        for (i, f) in felts.into_iter().enumerate() {
            let mut args = Self::args();
            if n > 1 {
                args.extend_from_slice(&[i.to_string()]);
            };

            f.to_state(StateInfo::ExtTableState(
                Self::CONST_TRACE_ID.to_string(),
                args,
            ));
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

impl AirVarImpl {
    pub fn as_felt(&self) -> FeltExpr {
        match self {
            AirVarImpl::Expr(expr) => expr.as_felt(),
            _ => panic!("Cannot convert to a Felt"),
        }
    }

    pub fn as_felt_mut(&mut self) -> &mut FeltExpr {
        match self {
            AirVarImpl::Expr(expr) => expr.as_felt_mut(),
            _ => panic!("Cannot convert to a Felt"),
        }
    }

    pub fn get_felt(&self, index: usize) -> FeltExpr {
        match self {
            AirVarImpl::Array(arr) => arr.get(index).expect("Invalid index").as_felt(),
            _ => panic!("Cannot convert to a Felt"),
        }
    }

    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        match self {
            AirVarImpl::Array(arr) => arr.get_mut(index).expect("Invalid index").as_felt_mut(),
            _ => panic!("Cannot convert to a Felt"),
        }
    }

    pub fn get_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            AirVarImpl::Array(arr) => arr.iter_mut().map(|v| v.as_felt_mut()).collect(),
            AirVarImpl::Tuple(vars) => vars.iter_mut().map(|v| v.as_felt_mut()).collect(),
            _ => panic!("Cannot convert to felts"),
        }
    }

    pub fn compile(self, compile_for: CompileFor) -> CompiledAirVar {
        match self {
            AirVarImpl::Expr(expr) => expr.compile(compile_for),
            AirVarImpl::Tuple(v) => {
                CompiledAirVar::Tuple(v.into_iter().map(|v| v.compile(compile_for)).collect())
            }
            AirVarImpl::Array(v) => {
                CompiledAirVar::Array(v.into_iter().map(|v| v.compile(compile_for)).collect())
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
                            .map(|(name, v)| (name, v.compile(compile_for)))
                            .collect(),
                    }
                }
            }
        }
    }

    // Returns the prover type with a "Packed" prefix.
    pub fn packed_prover_type(&self) -> String {
        match self {
            AirVarImpl::Expr(expr) => format!("Packed{}", expr.prover_type()),
            AirVarImpl::Tuple(vars) => {
                format!(
                    "({})",
                    vars.iter()
                        .map(|v| v.packed_prover_type())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            AirVarImpl::Array(vars) => {
                format!("[{}; {}]", vars[0].packed_prover_type(), vars.len())
            }
            AirVarImpl::Struct { r#type, .. } => format!("Packed{}", r#type),
        }
    }
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
                format!("[{}; {}]", vars[0].prover_type(), vars.len())
            }
            AirVarImpl::Struct {
                name: _,
                r#type,
                fields: _,
            } => r#type.to_string(),
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
    fn let_for_deduction(&self, _name: String) -> Self {}
}

impl ExtTable for () {
    const CONST_TRACE_ID: &'static str = "";
    type T = ();
}

// Examples + tests

impl_air_var!((BoolExpr, UInt16Expr));
impl_air_var!((BoolExpr, FeltExpr));
impl_air_var!([UInt32Expr]);
impl_air_var!([BoolExpr]);
type TestState = FeltExpr;
impl_air_var!((FeltExpr, RoundNumVar, TestState));

// Inline airs

// SingleKaratsuba + DoubleKaratsuba
impl_air_var!([[FeltExpr]]);
// MemVerify
impl_air_var!((CasmAddress, Felt252Expr));
// ReadPositive + CondDecodeSmallSign + CondFelt252AsAddr + CondFelt252AsRelImm
impl_air_var!((Felt252Expr, FeltExpr));
// Add252 + Div252 + Mul252 + Sub252 + VerifyAdd252 + VerifyMul252
impl_air_var!([Felt252Expr]);
// MemVerifyEqual
impl_air_var!([CasmAddress]);
type Flags = [FeltExpr; 15];
type FlagsFelts = [FeltExpr; 2];
type Offsets = [FeltExpr; 3];
// DecodeInstruction
impl_air_var!((Offsets, Flags, FeltExpr));
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
// Felt252UnpackFrom27 + RangeCheckFelt252Width27 + Cube252 + LinearCombination +
// PoseidonHadesPermutation
impl_air_var!([Felt252Width27Expr]);
// VerifyBlakeWord
impl_air_var!((CasmAddress, UInt32Expr));
// CreateBlakeRoundInput
impl_air_var!((CasmAddress, UInt32Expr, BoolExpr));
// CreateBlakeOutput
impl_air_var!((BlakeH, BlakeState));
// DecodeBlakeOpcode
impl_air_var!((BlakePointers, UInt32Expr, BlakeFlags));
// PartialECMul
impl_air_var!((FeltExpr, PackedECMultiplier, ECPoint));
impl_air_var!((ChainIdVar, RoundNumVar, PartialECMulState));
// QM31ReadReduced
type QM31Coordinates = [FeltExpr; 4];
impl_air_var!((QM31Coordinates, FeltExpr));

// Components

// RangeCheck + VerifyBitwiseXor (+ BitwiseXor + EncodeFlags + EncodeOffsets)
impl_air_var!([FeltExpr]);
// VerifyInstruction
impl_air_var!((CasmAddress, Offsets, FlagsFelts, FeltExpr));

// ChainRound

// BlakeRound
impl_air_var!((BlakeState, CasmAddress));
impl_air_var!((ChainIdVar, RoundNumVar, BlakeRoundInput));
// PoseidonFullRound
type PoseidonFullRoundState = [Felt252Width27Expr; 3];
impl_air_var!((ChainIdVar, RoundNumVar, PoseidonFullRoundState));
// PoseidonPartialRound
type PoseidonPartialRoundState = [Felt252Width27Expr; 4];
impl_air_var!((PoseidonPartialRoundState, Felt252Width27Expr));
impl_air_var!((ChainIdVar, RoundNumVar, PoseidonPartialRoundState));

// Implements AirVar for arrays, tuples and arrays of arrays of air vars.
#[macro_export]
macro_rules! impl_air_var {
    ( [[$s:ty]] ) => {
        impl<const N:usize, const M:usize> AirVar for [[$s;N];M] where $s: AirVar
        {
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                self.into_iter().flat_map(|s| s.as_felts_mut()).collect()
            }
        }

        impl<const N:usize, const M:usize> InternalAirVarActions for [[$s;N];M] where $s: InternalAirVarActions {
            fn let_for_deduction(&self, name: String) -> Self {
                let mut res = self.clone();
                for (i, s) in res.iter_mut().enumerate() {
                    *s = s.let_for_deduction(format!("{}[{}]", name, i));
                }
                res
            }
            fn new(name: String, in_state: bool) -> Self {
                from_fn(|j| from_fn(|i| <$s as InternalAirVarActions>::new(format!("{}_{}[{}]", name, j, i), in_state)))
            }
        }

        impl<const N:usize, const M:usize> From<[[$s;N];M]> for AirVarImpl {
            fn from(array: [[$s;N];M]) -> AirVarImpl {
                AirVarImpl::Array(array.into_iter().map(|s| s.into()).collect())
            }
        }
    };

    ( [$s:ty] ) => {
        impl<const N:usize> AirVar for [$s;N] where $s: AirVar
        {
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                self.into_iter().flat_map(|s| s.as_felts_mut()).collect()
            }
        }

        impl<const N:usize> InternalAirVarActions for [$s;N] where $s: InternalAirVarActions {
            fn let_for_deduction(&self, name: String) -> Self {
                from_fn(|i| self[i].let_for_deduction(format!("{}[{}]", name, i)))
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
            fn let_for_deduction(&self, name: String) -> Self {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                let mut i = 0;
                ($($s.let_for_deduction(format!("{}.{}", name, { i += 1; i - 1 })),)+)
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
