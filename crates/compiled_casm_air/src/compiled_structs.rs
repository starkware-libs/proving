use std::collections::{BTreeSet, HashSet};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::public_params::PublicParam;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledAirFn {
    pub name: String,
    pub relation_name: Option<String>,
    pub description: String,
    pub input: CompiledAirVar,

    pub state_names: Vec<String>,

    // The index of the multiplicity column in the lookup table that is used / yielded.
    // None for chain lookup relations, such as "Opcodes".
    pub multiplicity_col_index: Option<usize>,

    // The names of the lookup relations used and lookup components called.
    pub lookup_names: BTreeSet<String>,

    // The number of lookup terms (use or yield) in the air function.
    pub n_lookup_terms: usize,

    // The set of public parameters used in the air function.
    pub public_params: HashSet<PublicParam>,

    // The set of external states used in the air function.
    pub external_states: HashSet<(String, Option<usize>)>,

    pub constraints: Vec<ConstraintEvalStep>,
    pub deductions: Vec<TraceGenStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TraceGenStep {
    // Constains a description of the following code block.
    StartBlock(String),

    EndBlock,

    // The argument is a polynomial in in-state values.
    Deduction(CompiledAirVar),

    Intermediate(Intermediate),

    // Gets the output of the component (fast deduction).
    LookupCall {
        fn_name: String,
        input: CompiledAirVar,
        // output_name is the name of the intermediate variable into which the lookup result should
        // be placed. If it is None, there is no output and no intermediate variable is created.
        output_name: Option<String>,
    },

    // Adds the input to the lookup table or updates multiplicity.
    LookupAddInput {
        fn_name: String,
        // TODO(AnatG): Add row index.
        input: CompiledAirVar,
    },

    // Saves the information from the trace needed for the generation of the interaction trace.
    LookupTerm(LookupTerm),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConstraintEvalStep {
    // Constains a description of the following code block.
    StartBlock(String),

    EndBlock,

    // The first argument is a polynomial in in-state values. The constraint requires it
    // to evaluate to zero.
    // The second argument is the description of the constraint.
    Constraint(CompiledAirVar, Option<String>),

    // Used to create the constraints between the trace and the interaction trace, and the
    // constraints on the accumulated sum (the logup).
    LookupTerm(LookupTerm),

    Intermediate(Intermediate),
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
    Struct {
        r#type: String,
        fields: Vec<(String, CompiledAirVar)>,
    },
    // A variable written to the trace of a const table at the given (column) index.
    // If the table is Seq of constant length, the log length is also provided.
    ExternalState {
        name: String,
        col_index: usize,
        log_n_rows: Option<usize>,
    },
    // A value passed to the verifier outside the trace. Can influence the constraints
    // that the verifier checks.
    PublicParam(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Intermediate {
    pub name: String,
    pub r#type: String,
    pub var: CompiledAirVar,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct LookupTerm {
    pub relation_name: String,
    pub felts: Vec<CompiledAirVar>,
    pub use_or_yield: UseOrYield,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum UseOrYield {
    Use,
    Yield,
}

/// See `casm_registry.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledAirFnStat {
    pub trace_type: String,
    pub num_state_cols: usize,
    pub lookup_use_cols: IndexMap<String, usize>,
    pub lookup_rows: IndexMap<String, usize>,
    pub lookup_yield: bool,
    pub lookup_multiplicity: bool,
    pub total_num_trace_cols: usize,
    // To this we should add the number of trace cells in:
    // - Const tables and their corresponding lookup components (multiplicity and logup columns)
    // - The two memory tables (and their corresponding multiplicity and logup columns)
    // - The table of verify instruction (with number of rows equals the number of different pc
    //   values)
    pub trace_cells_upper_bound: usize,
}

/// See `constraints.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum ConstraintLeanCompare {
    Intermediate {
        name: String,
        r#type: String,
        var: String,
    },
    Constraint(String),
    Call {
        fn_name: String,
        input: String,
        output: String,
    },
    LookupUse {
        relation_name: String,
        felts: Vec<String>,
    },
}
