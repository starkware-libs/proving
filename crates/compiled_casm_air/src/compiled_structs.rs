use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

use crate::public_params::PublicParam;

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
    pub inline_calls: IndexMap<
        String,
        (
            Vec<(String, UseOrYield)>,
            IndexSet<PublicParam>,
            IndexSet<ExternalState>,
        ),
    >,

    // The set of public parameters used in the air function.
    pub public_params: IndexSet<PublicParam>,

    // The set of external states used in the air function.
    pub external_states: IndexSet<ExternalState>,

    pub constraints: Vec<ConstraintEvalStep>,
    pub deductions: Vec<TraceGenStep>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaddingType {
    // The multiplicity column is used to pad the lookups to const columns, memory, and verify
    // instruction.
    Multiplicity,
    // The enabler column is used to pad the chain lookups, as "Opcodes" and "BlakeRound", and
    // every lookup with no multiplicity.
    Enabler,
    // For air functions that are not a component in the trace, as inline air functions.
    None,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceType {
    // Doesn't have its own component in the trace, always inlined into its caller.
    // Can be called only with call.
    Inline,

    // Has its own component in the trace. Each call generates a new row in that component.
    // Can be called only with lookup_call. Yields lookup data.
    Component,

    // Has its own component in the trace. The trace for this component is pre-filled with rows
    // for all possible inputs by external means. Doesn't generate deductions or constraints.
    // Has no input, only output. Can be called only with call_external_table. Doesn't yield
    // lookup data.
    Const,

    // Has its own component in the trace. Has no input and no output. Cannot be called from
    // another component. Doesn't yield lookup data.
    Builtin,

    // Has its own component in the trace. Its input and output are casm states.
    // Cannot be called from another component. Doesn't yield multiplicity column.
    // Generates accumulated sum column where the input
    // is used and the output is yielded (chain lookup constraint).
    // Their chain lookup relation is called OPCODES_RELATION_NAME.
    Opcode,

    // Memory components are pre-filled. Their trace consists of only input and output columns, or
    // only output columns, if the input is const. They don't generate deductions. They can
    // generate constraints, and they yield lookup data. They implement the IsMemory trait.
    Memory,

    // Has its own component in the trace. Its input and output are of the same type ([FeltExpr;
    // 2], S), where S is some AirVar. Doesn't yield multiplicity column.
    // Generates accumulated sum column where the input
    // is used and the output is yielded (chain lookup constraint).
    //
    // Important:
    // - A ChainRound can be called from a single caller. This is because we use the caller Seq
    //   column to identify the chain (see chain_lookup_call).
    // - A ChainRound must have consts per round that are returned from a lookup component with a
    //   const round number column in its external input. Without this the chain lookup is not
    //   sound (for example, a malicious prover can run for more rounds than intended by
    //   overflowing the round number).
    ChainRound,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TraceGenStep {
    // Constains a description of the following code block.
    StartBlock(String),

    EndBlock,

    // The argument is a polynomial in in-state values.
    Deduction(CompiledAirVar),

    Intermediate(CompiledTraceGenIntermediate),

    // Adds the input to the lookup table or updates multiplicity.
    LookupAddInput {
        relation_name: String,
        input: CompiledAirVar,
    },

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
    Struct {
        r#type: String,
        fields: Vec<(String, CompiledAirVar)>,
    },
    // A variable written to a preprocessed column.
    ExternalState(ExternalState),
    // A value passed to the verifier outside the trace. Can influence the constraints
    // that the verifier checks.
    PublicParam(String),
}

// A preprocessed column represented by its id in stwo-cairo. The special
// value "Seq" is used to represent the Seq column whose size equals the size
// of the current component.
pub type ExternalState = String;

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum UseOrYield {
    Use,
    Yield,
}

/// See `casm_registry.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledAirFnStat {
    pub trace_type: TraceType,
    pub num_state_cols: usize,
    pub use_lookup_cols: IndexMap<String, usize>,
    pub yield_lookup_cols: IndexMap<String, usize>,
    pub lookup_rows: IndexMap<String, usize>,
    pub padding_type: PaddingType,
    pub total_num_trace_cols: usize,
    // To this we should add the number of trace cells in:
    // - Const tables and their corresponding lookup components (multiplicity and logup columns)
    // - The memory tables (and their corresponding multiplicity and logup columns)
    // - The table of verify instruction (with number of rows equals the number of different pc
    //   values)
    pub trace_cells_upper_bound: usize,
    // An upper bound on the multiplicity values for lookups to const tables and memory tables.
    pub uses_upper_bound: IndexMap<String, usize>,
    // An upper bound on the number of rows in the trace for each called lookup relation.
    pub rows_upper_bound: IndexMap<String, usize>,
    // The uses upper bound is limited by the size of the field.
    pub max_instances_uses_limit: usize,
    // The rows upper bound is currently limited to 2**27.
    pub max_instances_rows_limit: usize,
}

/// See `non_components.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonComponentStat {
    pub trace_cells_upper_bound: usize,
    pub uses_upper_bound: IndexMap<String, usize>,
    pub steps: usize,
    pub max_num_instances_uses: usize,
    pub max_num_instances_steps: usize,
}

/// See `constraints.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct LeanCompare {
    pub state_names: Vec<String>,
    pub intermediates: Vec<(String, String)>,
    pub constraints: Vec<String>,
    pub lookups: Vec<(String, String)>,
}
