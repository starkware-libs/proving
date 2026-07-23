use std::array::from_fn;
use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;

use air_common::{ExternalState, TraceType};
use air_compile::compiled_structs::CompiledAirVar;
use enum_dispatch::enum_dispatch;
use serde::Serialize;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;
use stwo_cairo_common::prover_types::cpu::ProverType;

use super::air_body::*;
use super::air_fn::*;
use super::expressions::biguint_expr::*;
use super::expressions::bool_expr::*;
use super::expressions::expr::*;
use super::expressions::felt_expr::*;
use super::expressions::felt252_expr::*;
use super::expressions::felt252width27_expr::*;
use super::expressions::op_expr::*;
use super::expressions::uint16_expr::*;
use super::expressions::uint32_expr::*;
use super::expressions::uint64_expr::*;
use super::expressions::var_expr::*;
use crate::casm_state::*;
#[cfg(any(test, feature = "test"))]
use crate::core::Felt;
use crate::core::public_params::*;
use crate::core::struct_var::VarWrapper;
use crate::felt252_id_memory::memory::*;
// Macros
use crate::impl_air_var;

pub type RoundNumVar = FeltExpr;
pub type ChainIdVar = FeltExpr;

#[allow(private_bounds)]
/// Every input and output of an air function is an AirVar.
pub trait AirVar: Clone + Debug + Into<AirVarImpl> {
    fn new(name: String, deg_in_state: Option<usize>) -> Self;

    fn let_for_deduction(&self, name: String) -> (Self, Intermediate);

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr>;

    // TODO(AnatG): Improve implementation of this function. It should be possible to implement it
    // without clone of self.
    fn as_felts(&self) -> Vec<FeltExpr> {
        self.clone().as_felts_mut().into_iter().map(|f| f.clone()).collect()
    }

    fn is_empty() -> bool {
        false
    }

    #[cfg(any(test, feature = "test"))]
    fn to_values(&self) -> Option<Vec<Felt>> {
        self.as_felts().iter().map(|f| f.value()).collect::<Option<Vec<_>>>()
    }

    // Defines a new variable for the top level variable, visible in deductions, and a variable for
    // each felt, visible in constraints.
    fn rec_let(
        &self,
        name: String,
        expr_descriptions: Option<Vec<Option<String>>>,
    ) -> (Self, Vec<Intermediate>) {
        let (mut res, interm0) = self.let_for_deduction(name.clone());

        // When the expression is a single felt that is directly in state, no intermediates
        // are necessary.
        if let AirVarImpl::Expr(ExprImpl::Felt(f)) = self.clone().into()
            && f.is_directly_in_state()
        {
            return (self.clone(), vec![]);
        }

        // We have to create the variable for <self> before its felts, because <let_> creates
        // the felts as well. Then, we recreate the felts from their original expressions
        // (<orig_felts>) and update <res>.
        let mut orig_felts = self.as_felts();
        let mut vars = vec![interm0];

        for (i, (orig_felt, felt)) in orig_felts.iter_mut().zip(res.as_felts_mut()).enumerate() {
            if !orig_felt.is_directly_in_state() {
                let felt_name = self.clone().into().get_limb_name(&name, i, &expr_descriptions);
                vars.push(Intermediate::new_for_constraint(&felt_name, orig_felt));
                felt.let_for_constraint(felt_name);
            }
        }

        (res, vars)
    }
}

// Information about air variables used by the air builder.
#[enum_dispatch]
pub trait AirVarImplInfo {
    fn var_expr_infos(&self) -> HashSet<VarExprInfo>;

    fn prover_type(&self) -> String;

    fn compile(self, compile_for: CompileFor) -> CompiledAirVar;

    // An AirVar is in_state if it is stored in a trace cell or a polynomial of felts stored in
    // trace cells. Used to verify that expressions of constraints are polynomials of felts
    // written to the trace.
    fn in_state(&self) -> bool {
        self.deg_in_state().is_some()
    }

    // An AirVar which is in_state has an associated degree of the polynomial of the stored trace
    // cells felt (if it is stored directly, the degree is 1). Used to verify that the degrees of
    // expressions of constraints or lookup terms are polynomials of sufficiently small degree.
    fn deg_in_state(&self) -> Option<usize>;

    // An AirVar is_const if was created with a value and the flag is_const = true, or if it is the
    // result of operations on other constants.
    // Used to verify that a constant variable is not written to the trace in a top-level AirFn,
    // since this would create a constant column in the trace.
    // Note that in runtime, we allow deduction of constant variables in internal calls, since an
    // AirFn can be called with different inputs in different calls.
    fn is_const(&self) -> bool {
        self.var_expr_infos().iter().all(|i| i.is_const)
    }

    // An AirVar is visible in constraints if it's a felt and each of its intermediate variables is
    // in constraints. An AirVar is visible in deductions if each of its intermediate variables is
    // in deductions, if it has no intermediate variables, or if you can create it from a parent
    // variable (see VarExpr). Felts that are directly in the state (see FeltExpr) are visible
    // in constraints and in deductions. Used to verify that variables are used in the correct
    // context.
    fn visibility(&self) -> Visibility {
        let visibilities =
            self.var_expr_infos().iter().map(|i| i.visibility).collect::<HashSet<_>>();
        Visibility {
            in_constraints: visibilities.iter().all(|t| t.in_constraints),
            in_deductions: visibilities.iter().all(|t| t.in_deductions),
        }
    }

    fn public_params(&self) -> BTreeSet<PublicParam> {
        self.var_expr_infos().iter().filter_map(|i| i.public_param.clone()).collect()
    }

    fn external_states(&self) -> BTreeSet<ExternalState> {
        self.var_expr_infos().iter().filter_map(|i| i.external_state.clone()).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarExprInfo {
    pub is_const: bool,
    pub visibility: Visibility,
    pub public_param: Option<PublicParam>,
    pub external_state: Option<ExternalState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Visibility {
    pub in_constraints: bool,
    pub in_deductions: bool,
}

impl Visibility {
    pub fn new(in_deductions: bool, in_constraints: bool) -> Self {
        Self { in_constraints, in_deductions }
    }
}

#[derive(Clone, Debug)]
pub struct Intermediate {
    pub name: String,
    pub var: AirVarImpl,
    pub visibility: Visibility,
}

impl Intermediate {
    pub fn new_for_deduction<V>(name: &str, var: &V) -> Self
    where
        V: Into<AirVarImpl> + Clone,
    {
        Self {
            name: name.to_string(),
            var: var.clone().into(),
            visibility: Visibility::new(true, false),
        }
    }

    pub fn new_for_constraint<V>(name: &str, var: &V) -> Self
    where
        V: Into<AirVarImpl> + Clone,
    {
        Self {
            name: name.to_string(),
            var: var.clone().into(),
            visibility: Visibility::new(false, true),
        }
    }
}

// Describes an external table and its type as used in the air infra.
// Note that we can have two tables with the same CONST_TRACE_ID, but different types (see for
// example Seq and SeqAddr), as long as they are represented by the same number of felts (i.e. the
// number of columns in the table).
pub trait ExtTable: Default + Debug + Clone {
    type T: AirVar;

    fn new() -> Self::T {
        let mut res = Self::T::new("".to_string(), None);
        Self::to_state(&mut res);
        res
    }

    fn column_ids() -> Vec<String> {
        let preprocessed = Self::preprocessed_columns();
        assert_eq!(
            preprocessed.len(),
            Self::T::new("".to_string(), None).as_felts().len(),
            "Not implemented for non-const tables"
        );
        preprocessed.iter().map(|col| col.id().id).collect()
    }

    fn to_state(v: &mut Self::T) {
        let felts = v.as_felts_mut();

        for (felt, col_id) in felts.into_iter().zip(Self::column_ids()) {
            felt.set_value(ValueInfo::ExternalState(col_id));
        }
    }

    // External tables that can be called with air_builder.call_external_table should implement this
    // method. See for example Seq.
    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        Self::new()
    }

    /// For const-size tables, return a column of the table
    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>>;

    /// For const-size tables, return the log number of rows
    fn log_size() -> Option<u32> {
        Self::preprocessed_columns().first().map(|ppc| ppc.log_size())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ExtTableAirFn<E>
where
    E: ExtTable,
{
    #[serde(skip)]
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
    #[cfg(any(test, feature = "test"))]
    fn calc(&self) -> String {
        self.value().expect("calc was called on a var without a value").calc()
    }
}

// Air variables as represented in the air_body.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum AirVarImpl {
    Expr(ExprImpl),
    Tuple(Vec<AirVarImpl>),
    Array(Vec<AirVarImpl>),
    Struct { name: Option<String>, r#type: String, fields: Vec<(String, AirVarImpl)> },
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
            // Note that this implementation is not suitable for AirVarImpl::Tuple because a
            // tuple might have elements that are larger than a single felt, so the i'th felt
            // isn't necessarily in the i'th position.
            AirVarImpl::Array(arr) => arr.get(index).expect("Invalid index").as_felt(),
            _ => panic!("Cannot get a Felt"),
        }
    }

    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        match self {
            // Note that this implementation is not suitable for AirVarImpl::Tuple because a
            // tuple might have elements that are larger than a single felt, so the i'th felt
            // isn't necessarily in the i'th position.
            AirVarImpl::Array(arr) => arr.get_mut(index).expect("Invalid index").as_felt_mut(),
            _ => panic!("Cannot get a Felt"),
        }
    }

    pub fn get_limb_name(
        &self,
        name: &str,
        index: usize,
        expr_descriptions: &Option<Vec<Option<String>>>,
    ) -> String {
        if let Some(descriptions) = expr_descriptions {
            assert_eq!(self.as_exprs().len(), descriptions.len(), "Incorrect descriptions length");
        }
        let (expr_i, expr_size, felt_in_expr_i) = self.expr_felt_index(index);
        let expr_desc = expr_descriptions
            .as_ref()
            .map(|descs| descs.get(expr_i))
            .unwrap_or(None)
            .unwrap_or(&None)
            .clone();

        match (expr_desc, expr_size) {
            (Some(expr_desc), 1) => format!("{name}_{expr_desc}"),
            (Some(expr_desc), _) => format!("{name}_{expr_desc}_limb_{felt_in_expr_i}"),
            (None, _) => {
                if self.as_felts().len() == 1 {
                    name.to_string()
                } else {
                    format!("{name}_limb_{index}")
                }
            }
        }
    }

    fn as_exprs(&self) -> Vec<ExprImpl> {
        match self {
            AirVarImpl::Expr(expr) => vec![expr.clone()],
            AirVarImpl::Tuple(vars) => vars.iter().flat_map(|v| v.as_exprs()).collect(),
            AirVarImpl::Array(vars) => vars.iter().flat_map(|v| v.as_exprs()).collect(),
            AirVarImpl::Struct { fields, .. } => {
                fields.iter().flat_map(|(_, v)| v.as_exprs()).collect()
            }
        }
    }

    // Given a felt index in the air variable, returns the index of the expression that contains it,
    // the number of felts in the expression, and the index of the felt in the expression.
    fn expr_felt_index(&self, mut felt_index: usize) -> (usize, usize, usize) {
        let exprs = self.as_exprs();
        let num_of_felts = exprs.iter().map(|e| e.as_felts().len()).collect::<Vec<_>>();
        for (i, n) in num_of_felts.iter().enumerate() {
            if felt_index < *n {
                return (i, *n, felt_index);
            }
            felt_index -= n;
        }
        panic!("Felt index {felt_index} is out of bounds")
    }

    pub fn as_felts(&self) -> Vec<FeltExpr> {
        match self {
            AirVarImpl::Expr(expr) => expr.as_felts(),
            AirVarImpl::Tuple(vars) => vars.iter().flat_map(|v| v.as_felts()).collect(),
            AirVarImpl::Array(vars) => vars.iter().flat_map(|v| v.as_felts()).collect(),
            AirVarImpl::Struct { fields, .. } => {
                fields.iter().flat_map(|(_, v)| v.as_felts()).collect()
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
                    vars.iter().map(|v| v.packed_prover_type()).collect::<Vec<_>>().join(", ")
                )
            }
            AirVarImpl::Array(vars) => {
                if vars.is_empty() {
                    return "[PackedM31; 0]".to_string();
                }

                format!("[{}; {}]", vars[0].packed_prover_type(), vars.len())
            }
            AirVarImpl::Struct { r#type, .. } => format!("Packed{type}"),
        }
    }
}

impl AirVarImplInfo for AirVarImpl {
    fn var_expr_infos(&self) -> HashSet<VarExprInfo> {
        match self {
            AirVarImpl::Expr(expr) => expr.var_expr_infos(),
            AirVarImpl::Tuple(vars) | AirVarImpl::Array(vars) => {
                vars.iter().flat_map(|v| v.var_expr_infos()).collect()
            }
            AirVarImpl::Struct { name: _, r#type: _, fields } => {
                fields.iter().flat_map(|(_, v)| v.var_expr_infos()).collect()
            }
        }
    }

    fn deg_in_state(&self) -> Option<usize> {
        match self {
            AirVarImpl::Expr(expr) => expr.deg_in_state(),
            _ => panic!("Cannot get degree in state for tuples, arrays or structs"),
        }
    }

    fn prover_type(&self) -> String {
        match self {
            AirVarImpl::Expr(expr) => expr.prover_type(),
            AirVarImpl::Tuple(vars) => {
                format!("({})", vars.iter().map(|v| v.prover_type()).collect::<Vec<_>>().join(", "))
            }
            AirVarImpl::Array(vars) => {
                if vars.is_empty() {
                    return "[M31; 0]".to_string();
                }

                format!("[{}; {}]", vars[0].prover_type(), vars.len())
            }
            AirVarImpl::Struct { name: _, r#type, fields: _ } => r#type.to_string(),
        }
    }

    fn compile(self, compile_for: CompileFor) -> CompiledAirVar {
        match self {
            AirVarImpl::Expr(expr) => expr.compile(compile_for),
            AirVarImpl::Tuple(v) => {
                CompiledAirVar::Tuple(v.into_iter().map(|v| v.compile(compile_for)).collect())
            }
            AirVarImpl::Array(v) => {
                CompiledAirVar::Array(v.into_iter().map(|v| v.compile(compile_for)).collect())
            }
            AirVarImpl::Struct { name, r#type, fields } => {
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
    fn new(_name: String, _deg_in_state: Option<usize>) -> Self {}
    fn let_for_deduction(&self, _name: String) -> (Self, Intermediate) {
        ((), Intermediate::new_for_deduction("", self))
    }
}

impl ExtTable for () {
    type T = ();

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
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
type BoundedFeltExpr = VarWrapper<FeltExpr, (i32, i32)>;
// DoubleKaratsuba
impl_air_var!([BoundedFeltExpr]);
// MemVerify
impl_air_var!((CasmAddress, Felt252Expr));
// ReadPositive
impl_air_var!((Felt252Expr, CasmId));
// CondFelt252AsAddr + CondFelt252AsRelImm
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
impl_air_var!((CasmAddress, CasmId, Cond));
// ReadSmall
impl_air_var!((FeltExpr, CasmId));
type GenericFlags = [FeltExpr; 20]; // GENERIC_FLAGS_SIZE = 20
type Operands = [Felt252Expr; 3];
// DecodeGenericInstruction
impl_air_var!((GenericFlags, Offsets));
// EvalOperands
impl_air_var!((CasmStateVar, GenericFlags, Offsets));
// HandleOpcodes
impl_air_var!((CasmStateVar, GenericFlags, Offsets, Operands));
// UpdateRegisters
impl_air_var!((CasmStateVar, GenericFlags, Operands));
type ModValue = [Felt252Expr; 4]; // MOD_BUILTIN_N_WORDS = 4
// ModUtils
impl_air_var!([ModValue]);
// ModUtils
impl_air_var!((CasmAddress, FeltExpr));
type Addresses<const N: usize> = [CasmAddress; N];
// MemVerifyAll
impl_air_var!((Addresses<const N: usize>, Felt252Expr));
// Felt252UnpackFrom27 + RangeCheck252Width27 + Cube252 + LinearCombination +
// PoseidonHadesPermutation
impl_air_var!([Felt252Width27Expr]);
// VerifyU32
impl_air_var!((CasmAddress, UInt32Expr));
// CreateBlakeRoundInput
impl_air_var!((CasmAddress, UInt32Expr, BoolExpr));
// CreateBlakeOutput
type BlakeH = [UInt32Expr; 8];
type BlakeState = [UInt32Expr; 16];
impl_air_var!((BlakeH, BlakeState));
// DecodeBlakeOpcode
type BlakePointers = [CasmAddress; 3];
type BlakeFlags = [BoolExpr; 2];
impl_air_var!((BlakePointers, UInt32Expr, BlakeFlags));
// QM31ReadReduced
type QM31Coordinates = [FeltExpr; 4];
impl_air_var!((QM31Coordinates, CasmId));

// Components

// RangeCheck + VerifyBitwiseXor (+ BitwiseXor + EncodeFlags + EncodeOffsets)
impl_air_var!([FeltExpr]);
// VerifyInstruction
impl_air_var!((CasmAddress, Offsets, FlagsFelts, FeltExpr));
// PoseidonAggregator
type PoseidonInputIds = [CasmId; 3];
type PoseidonOutputIds = [CasmId; 3];
impl_air_var!((PoseidonInputIds, PoseidonOutputIds));
// PedersenAggregator
type PedersenInputIds = [CasmId; 2];
type PedersenOutputId = CasmId;
impl_air_var!((PedersenInputIds, PedersenOutputId));

// ChainRound

// BlakeRound
impl_air_var!((BlakeState, CasmAddress));
type BlakeRoundInput = (BlakeState, CasmAddress);
impl_air_var!((ChainIdVar, RoundNumVar, BlakeRoundInput));
// PoseidonFullRound
type PoseidonFullRoundState = [Felt252Width27Expr; 3];
impl_air_var!((ChainIdVar, RoundNumVar, PoseidonFullRoundState));
// PoseidonPartialRound
type PoseidonPartialRoundState = [Felt252Width27Expr; 4];
impl_air_var!((PoseidonPartialRoundState, Felt252Width27Expr));
impl_air_var!((ChainIdVar, RoundNumVar, PoseidonPartialRoundState));
// PartialECMul
type ECPoint = [Felt252Expr; 2];
type PackedECMultiplier<const NUM_WINDOWS: usize> = [FeltExpr; NUM_WINDOWS];
type PartialECMulState<const NUM_WINDOWS: usize> = (PackedECMultiplier<NUM_WINDOWS>, ECPoint);
impl_air_var!((PackedECMultiplier<const NUM_WINDOWS: usize>, ECPoint));
impl_air_var!((ChainIdVar, RoundNumVar, PartialECMulState<const NUM_WINDOWS: usize>));
// PartialECMulGeneric
type ECPointB = ECPoint;
type PartialECMulGenericState = (Felt252Width27Expr, ECPoint, ECPoint, FeltExpr);
impl_air_var!((Felt252Width27Expr, ECPoint, ECPointB, FeltExpr));
impl_air_var!((ChainIdVar, RoundNumVar, PartialECMulGenericState));

// Gates
// M31ToU32
impl_air_var!((FeltExpr, UInt32Expr));
type H = [UInt32Expr; 8];
impl_air_var!((H, BoolExpr));
impl_air_var!([H]);
type Hes = [H; 2];
type QM31Message = [FeltExpr; 16];
impl_air_var!((QM31Message, FeltExpr));
impl_air_var!((Hes, QM31Message));
type State = [UInt32Expr; 16];
impl_air_var!((State, FeltExpr));
type StateMessageId = (State, FeltExpr);
impl_air_var!((ChainIdVar, RoundNumVar, StateMessageId));

// Implements AirVar for arrays, tuples and arrays of arrays of air vars.
#[macro_export]
macro_rules! impl_air_var {
    ( [[$s:ty]] ) => {
        impl<const N:usize, const M:usize> AirVar for [[$s;N];M] where $s: AirVar
        {
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                self.into_iter().flat_map(|s| s.as_felts_mut()).collect()
            }
            fn let_for_deduction(&self, name: String) -> (Self, Intermediate) {
                let interm = Intermediate::new_for_deduction(&name, self);
                (from_fn(|i| self[i].let_for_deduction(format!("{}[{}]", name, i)).0), interm)
            }
            fn new(name: String, deg_in_state: Option<usize>) -> Self {
                from_fn(|i| <[$s;N] as AirVar>::new(format!("{}[{}]", name, i), deg_in_state))
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
            fn let_for_deduction(&self, name: String) -> (Self, Intermediate) {
                let interm = Intermediate::new_for_deduction(&name, self);
                (from_fn(|i| self[i].let_for_deduction(format!("{}[{}]", name, i)).0), interm)
            }
            fn new(name: String, deg_in_state: Option<usize>) -> Self {
                from_fn(|i| <$s as AirVar>::new(format!("{}[{}]", name, i), deg_in_state))
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
            fn let_for_deduction(&self, name: String) -> (Self, Intermediate) {
                let interm = Intermediate::new_for_deduction(&name, self);
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                let mut i = 0;
                (($($s.let_for_deduction(format!("{}.{}", name, { i += 1; i - 1 })).0,)+), interm)
            }
            fn new(name: String, deg_in_state: Option<usize>) -> Self {
                let mut i = 0;
                ($(<$s$(< $( $lt ),+ >)? as AirVar>::new(format!("{}.{}", name, { i += 1; i - 1 }), deg_in_state),)+)
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
