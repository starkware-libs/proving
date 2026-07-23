use air_common::{ExternalState, PaddingType, TraceType, UseOrYield};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledAirFn {
    pub name: String,
    pub relation_names: Vec<String>,
    pub relation_size: Option<usize>,
    // For constant-size component, the log_2 of the number of rows.
    pub log_height: Option<u32>,
    pub description: String,
    pub instance_definition: String,
    pub r#type: TraceType,
    pub padding_type: PaddingType,

    // The input to the air function for write trace.
    // Contains the name of the input, its prover type, and its packed prover type.
    pub prover_input: (String, String, String),
    // The input to the air function for the constraints evaluation.
    // Contains the name of each limb in the input.
    pub verifier_input_limbs: Vec<String>,

    // For const-size components, the IDs of the preprocessed input columns
    pub input_const_columns: Vec<String>,

    // The output of the air function for write trace.
    // Contains the output, its name, its prover type, and its packed prover type.
    pub prover_output: (CompiledAirVar, String, String, String),
    // The output of the air function for the constraints evaluation.
    // Contains the output, and names for each of its felts (used for debugging).
    pub verifier_output: (CompiledAirVar, Vec<String>),

    pub state_names: Vec<String>,

    // The names of the lookup relations used/yielded.
    pub constraint_lookups: Vec<(String, UseOrYield)>,

    // The names of the called air_fn components and their padding type.
    // Some of these may not be used/yielded, see for example `mem_read_unverified`.
    pub sub_components: IndexMap<String, PaddingType>,

    // For each lookup relation, the name of the corresponding air function component, the index of
    // the relation in this air function, and the max number of inputs added to it by each row
    // in this component.
    pub n_inputs_added_per_relation: IndexMap<String, (String, usize, usize)>,

    // The names of the air functions that are inlined into this one, with their lookup names,
    // public params, and external states.
    #[allow(clippy::type_complexity)]
    pub inline_calls:
        IndexMap<String, (Vec<(String, UseOrYield)>, IndexSet<String>, IndexSet<ExternalState>)>,

    // The set of public parameters used in the air function.
    pub public_params: IndexSet<String>,

    // The set of external states used in the air function.
    pub external_states: IndexSet<ExternalState>,

    pub constraints: Vec<ConstraintEvalStep>,
    pub deductions: Vec<TraceGenStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TraceGenStep {
    // Contains a description of the following code block.
    StartBlock(String),

    EndBlock,

    // The argument is a polynomial in in-state values.
    Deduction(CompiledAirVar),

    Intermediate(CompiledTraceGenIntermediate),

    // Adds the input to the lookup table or updates multiplicity.
    LookupAddInput { relation_name: String, input: CompiledAirVar },

    // Saves the information from the trace needed for the generation of the interaction trace.
    LookupTerm(LookupTerm),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConstraintEvalStep {
    // The first argument is a polynomial in in-state values. The constraint requires it
    // to evaluate to zero.
    // The second argument is the description of the constraint.
    Constraint(CompiledAirVar, Option<String>),

    // Used to create the constraints between the trace and the interaction trace, and the
    // constraints on the accumulated sum (the logup).
    LookupTerm(LookupTerm),

    Intermediate(CompiledConstraintIntermediate),
}

// Air variables as represented in the deductions and constrains lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CompiledAirVar {
    // A constant expression represented by the type of the constant and its value.
    Const(String, String),
    // A variable expression represented by the type of the variable and its name.
    Var(String, String),
    // A name for the variable written to the trace at the given index.
    State(String),
    // A static function call. The name of the function and its arguments.
    StaticCall(String, Vec<CompiledAirVar>),
    // A method function call. The self variable, the name of the method and its arguments.
    MethodCall(Box<CompiledAirVar>, String, Vec<CompiledAirVar>),
    // A binary operation represented by the left-hand side, the operator, and the right-hand
    // side.
    BinaryOp(Box<CompiledAirVar>, String, Box<CompiledAirVar>),
    // A unary operation represented by the operator and the expression.
    UnaryOp(String, Box<CompiledAirVar>),
    Tuple(Vec<CompiledAirVar>),
    Array(Vec<CompiledAirVar>),
    Struct { r#type: String, fields: Vec<(String, CompiledAirVar)> },
    // A variable written to a preprocessed column.
    ExternalState(ExternalState),
    // A value passed to the verifier outside the trace. Can influence the constraints
    // that the verifier checks.
    PublicParam(String),
    // A value that is 1 in active rows and 0 in padding rows
    Enabler,
    // A value equal to the number of uses of the i'th yield of this row
    Multiplicity(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CompiledTraceGenIntermediate {
    pub name: String,
    pub r#type: String,
    pub var: CompiledAirVar,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CompiledConstraintIntermediate {
    pub felt_names: Vec<String>,
    pub var: CompiledAirVar,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct LookupTerm {
    pub relation_name: String,
    pub felts: Vec<CompiledAirVar>,
    pub use_or_yield: UseOrYield,
    pub multiplicity: CompiledAirVar,
}
