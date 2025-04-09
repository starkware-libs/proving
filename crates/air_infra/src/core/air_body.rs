use std::collections::BTreeSet;
use std::fmt::Debug;

use compiled_casm_air::compiled_structs::{
    CompiledAirVar, CompiledIntermediate, ConstraintEvalStep, LookupTerm, TraceGenStep, UseOrYield,
};
use compiled_casm_air::public_params::PublicParam;
use compiled_casm_air::relations::OPCODES_RELATION_NAME;
use compiled_casm_air::utils::CONSTRAINT_EVAL_FUNCTION_NAME;
use indexmap::{IndexMap, IndexSet};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::ProverType;

use super::air_fn_registry::*;
use super::expressions::felt_expr::*;
use super::variables::*;
use crate::const_expr;
use crate::core::Felt;

// A Call is an air_body component that represents a call to another air function.
// It contains the name of the air function, the input argument, the output of the call
// and the air_body of the called function.
#[derive(Clone, Debug)]
pub struct Call {
    pub air_fn_name: String,
    pub air_fn_description: String,
    pub input: AirVarImpl,
    pub output_name: String,
    pub output: AirVarImpl,
    pub state_names: Vec<String>,
    pub air_body: AirBody,
}

// Computes the output of the component into an intermediate variable named <output_name>.
#[derive(Clone, Debug)]
pub struct LookupCall {
    pub air_fn_name: String,
    pub method_name: String,
    pub ext_input: Option<AirVarImpl>,
    pub input: Option<AirVarImpl>,
    pub output_name: String,
    pub output: AirVarImpl,
}

// Each air function has an air_body, which is a vector of AirBodyComponent.
// These describe the steps to execute the function.
#[derive(Clone, Debug)]
pub enum AirBodyComponent {
    // Add a constraint that the given expression equals zero.
    Constraint(FeltExpr, Option<String>),

    // Write the value of the given expression to the next cell in the state.
    Deduction(FeltExpr, Option<String>),

    // An assignment is a constraint and a deduction referring to the same trace cell.
    // For example, when copying a value from one trace cell to another.
    Assignment {
        constraint: FeltExpr,
        deduction: FeltExpr,
        desc: Option<String>,
    },

    // Create a new local variable in the generated code. The visibility controls whether
    // to create the variable in the trace generation code, constraint evaluation code
    // or both.
    Intermediate(Intermediate),

    // Call an inline air function. This component will be replaced by the air_body of
    // the callee during the compilation process.
    Call(Call),

    LookupCall(LookupCall),

    // Adds the input to the lookup table or updates multiplicity.
    LookupAddInput {
        air_fn_name: String,
        ext_input: Option<AirVarImpl>,
        input: Option<AirVarImpl>,
    },

    // Saves the information from the trace needed for the generation of the interaction trace,
    // and creates the constraints between the trace and the interaction trace, and the
    // constraints on the accumulated sum (the logup).
    LookupTerm {
        relation_name: String,
        felts: Vec<FeltExpr>,
        use_or_yield: UseOrYield,
    },
}

// A structure for the air_body of an air_fn.
#[derive(Debug, Clone, Default)]
pub struct AirBody(Vec<AirBodyComponent>);

impl AirBody {
    // Checks visibility and in_state status of the variables in the new component and adds it.
    pub fn push(&mut self, component: AirBodyComponent) {
        match &component {
            AirBodyComponent::Constraint(expr, desc) => {
                assert!(
                    expr.visibility().in_constraints && expr.in_state(),
                    "constraint must be in state and have only intermediate variables known in constraints"
                );
                let deg = expr.deg_in_state().unwrap();
                assert!(
                    deg <= 3,
                    "constraint must have degree <= 3, encountered degree {} in constraint named '{}' with expression\n{:#?}",
                    deg,
                    desc.clone().unwrap_or_default(),
                    expr
                );
            }
            AirBodyComponent::Deduction(expr, _) => {
                assert!(
                    expr.visibility().in_deductions,
                    "deduction must have only intermediate variables known in deductions"
                );
            }
            AirBodyComponent::Assignment {
                constraint,
                deduction,
                desc: _,
            } => {
                assert!(
                    constraint.visibility().in_constraints && constraint.in_state(),
                    "constraint must be in state and have only intermediate variables known in constraints"
                );
                assert!(
                    deduction.visibility().in_deductions,
                    "deduction must have only intermediate variables known in deductions"
                );
            }
            AirBodyComponent::Intermediate(Intermediate {
                name: _,
                var,
                visibility,
            }) => {
                assert!(
                    visibility.in_deductions || visibility.in_constraints,
                    "visibility of intermediates must be set"
                );
                if visibility.in_constraints {
                    assert!(
                        var.prover_type() == Felt::r#type(),
                        "only felts can be intermediates in constraints"
                    );
                }
                if visibility.in_constraints {
                    // We check that the variable is in_state since we don't want to create
                    // variables for constraints before deduction.
                    assert!(
                        var.in_state() && var.visibility().in_constraints,
                        "intermediate variable must be in state and have only intermediate variables known in constraints"
                    );
                }
                if visibility.in_deductions {
                    assert!(
                        var.visibility().in_deductions,
                        "intermediate variable must have only intermediate variables known in deductions"
                    );
                }
            }
            AirBodyComponent::Call(_) => {}
            AirBodyComponent::LookupCall(LookupCall {
                ext_input, input, ..
            }) => {
                if let Some(ext_input) = ext_input {
                    assert!(
                        ext_input.visibility().in_deductions,
                        "lookup call must have only intermediate variables known in deductions"
                    );
                }
                if let Some(input) = input {
                    assert!(
                        input.visibility().in_deductions,
                        "lookup call must have only intermediate variables known in deductions"
                    );
                }
            }
            AirBodyComponent::LookupAddInput {
                ext_input, input, ..
            } => {
                if let Some(ext_input) = ext_input {
                    assert!(
                        ext_input.visibility().in_deductions,
                        "lookup add input must have only intermediate variables known in deductions"
                    );
                }
                if let Some(input) = input {
                    assert!(
                        input.visibility().in_deductions,
                        "lookup add input must have only intermediate variables known in deductions"
                    );
                }
            }
            AirBodyComponent::LookupTerm {
                felts,
                relation_name,
                ..
            } => {
                for f in felts {
                    assert!(
                        f.visibility().in_deductions && f.visibility().in_constraints && f.in_state(),
                        "lookup term must be in state and have only intermediate variables known in deductions and constraints"
                    );
                    let deg = f.deg_in_state().unwrap();
                    assert!(
                        deg <= 1,
                        "lookup term must have degree <= 1, encountered degree {} in term named '{}' with expression\n{:#?}",
                        deg,
                        relation_name,
                        f
                    );
                }
            }
        };

        self.0.push(component);
    }

    pub fn get_external_states(&self) -> IndexSet<(String, Vec<String>)> {
        let mut external_states = IndexSet::<(String, Vec<String>)>::default();

        for component in self.0.clone() {
            match component {
                AirBodyComponent::Constraint(felt_expr, _)
                | AirBodyComponent::Assignment {
                    constraint: felt_expr,
                    ..
                }
                | AirBodyComponent::Deduction(felt_expr, _) => {
                    external_states.extend(felt_expr.external_states());
                }
                AirBodyComponent::Intermediate(Intermediate { var, .. }) => {
                    external_states.extend(var.external_states());
                }
                AirBodyComponent::Call(f) => {
                    external_states.extend(f.air_body.get_external_states());
                }
                AirBodyComponent::LookupCall(_) => {}
                AirBodyComponent::LookupAddInput { input, .. } => {
                    if let Some(input) = input {
                        external_states.extend(input.external_states());
                    }
                }
                AirBodyComponent::LookupTerm { felts, .. } => {
                    external_states.extend(felts.iter().flat_map(|f| f.external_states()));
                }
            }
        }

        external_states
    }

    pub fn get_public_params(&self) -> IndexSet<PublicParam> {
        let mut public_params = IndexSet::<PublicParam>::default();

        for component in self.0.clone() {
            match component {
                AirBodyComponent::Constraint(felt_expr, _)
                | AirBodyComponent::Assignment {
                    constraint: felt_expr,
                    ..
                }
                | AirBodyComponent::Deduction(felt_expr, _) => {
                    public_params.extend(felt_expr.public_params());
                }
                AirBodyComponent::Intermediate(Intermediate { var, .. }) => {
                    public_params.extend(var.public_params());
                }
                AirBodyComponent::Call(f) => {
                    public_params.extend(f.air_body.get_public_params());
                }
                AirBodyComponent::LookupCall(_) => {}
                AirBodyComponent::LookupAddInput { input, .. } => {
                    if let Some(input) = input {
                        public_params.extend(input.public_params());
                    }
                }
                AirBodyComponent::LookupTerm { felts, .. } => {
                    public_params.extend(felts.iter().flat_map(|f| f.public_params()));
                }
            }
        }

        public_params
    }

    // Transforms the air body of an air function into the compiled deductions air fn format.
    pub fn compile_for_deductions(&self) -> Vec<TraceGenStep> {
        let mut deductions = vec![];

        for component in self.0.clone() {
            match component {
                AirBodyComponent::Constraint(..) => {}
                AirBodyComponent::Assignment { deduction, .. } => {
                    deductions.push(TraceGenStep::Deduction(
                        deduction.compile(CompileFor::Deductions),
                    ));
                }
                AirBodyComponent::Deduction(deduction, _) => {
                    deductions.push(TraceGenStep::Deduction(
                        deduction.compile(CompileFor::Deductions),
                    ));
                }
                AirBodyComponent::Intermediate(Intermediate {
                    name,
                    var,
                    visibility,
                }) => {
                    if visibility.in_deductions {
                        deductions.push(TraceGenStep::Intermediate(CompiledIntermediate {
                            name,
                            r#type: var.prover_type(),
                            var: var.compile(CompileFor::Deductions),
                        }));
                    }
                }
                AirBodyComponent::Call(call) => {
                    let call_deductions = call.air_body.compile_for_deductions();
                    if !call_deductions.is_empty() {
                        deductions.push(TraceGenStep::StartBlock(call.air_fn_description));
                        deductions.extend(call_deductions);
                        deductions.push(TraceGenStep::EndBlock);
                    }
                }
                AirBodyComponent::LookupCall(call) => {
                    deductions.push(TraceGenStep::Intermediate(CompiledIntermediate {
                        name: call.output_name,
                        r#type: call.output.prover_type(),
                        var: CompiledAirVar::StaticCall(
                            call.method_name,
                            vec![AirFnEntry::generate_input(call.ext_input, call.input)
                                .compile(CompileFor::Deductions)],
                        ),
                    }));
                }
                AirBodyComponent::LookupAddInput {
                    air_fn_name,
                    ext_input,
                    input,
                } => {
                    deductions.push(TraceGenStep::LookupAddInput {
                        fn_name: air_fn_name,
                        input: AirFnEntry::generate_input(ext_input, input)
                            .compile(CompileFor::Deductions),
                    });
                }
                AirBodyComponent::LookupTerm {
                    relation_name,
                    felts,
                    use_or_yield,
                } => {
                    deductions.push(TraceGenStep::LookupTerm(LookupTerm {
                        relation_name,
                        felts: felts
                            .into_iter()
                            .map(|f| f.compile(CompileFor::Deductions))
                            .collect(),
                        use_or_yield,
                    }));
                }
            }
        }

        deductions
    }

    // Transforms the air body of an air function into the compiled constraints air fn format.
    pub fn compile_for_constraints(&self) -> Vec<ConstraintEvalStep> {
        let mut constraints = vec![];

        for component in self.0.clone() {
            match component {
                AirBodyComponent::Constraint(constraint, desc) => {
                    constraints.push(ConstraintEvalStep::Constraint(
                        constraint.compile(CompileFor::Constraints),
                        desc,
                    ));
                }
                AirBodyComponent::Assignment {
                    constraint, desc, ..
                } => {
                    constraints.push(ConstraintEvalStep::Constraint(
                        constraint.compile(CompileFor::Constraints),
                        desc,
                    ));
                }
                AirBodyComponent::Deduction(..) => {}
                AirBodyComponent::Intermediate(Intermediate {
                    name,
                    var,
                    visibility,
                }) => {
                    if visibility.in_constraints {
                        // These are only felt expressions (see assert in <push>).
                        constraints.push(ConstraintEvalStep::Intermediate(CompiledIntermediate {
                            name,
                            r#type: var.prover_type(),
                            var: var.compile(CompileFor::Constraints),
                        }));
                    }
                }
                AirBodyComponent::Call(mut call) => {
                    let call_constraints = call.air_body.compile_for_constraints();
                    if !call_constraints.is_empty() {
                        // TODO(AnatG): Consider changing the signature of the function instead of
                        // sending zeros.
                        for f in call.input.as_felts_mut() {
                            if !f.visibility().in_constraints {
                                *f = const_expr!(0);
                            }
                        }

                        let state_vars = call
                            .state_names
                            .iter()
                            .map(|s| CompiledAirVar::State(s.clone()))
                            .collect::<Vec<_>>();

                        constraints.push(ConstraintEvalStep::Intermediate(CompiledIntermediate {
                            name: call.output.verifier_name(call.output_name),
                            r#type: call.output.verifier_type(),
                            var: CompiledAirVar::StaticCall(
                                format!("{}::{}", call.air_fn_name, CONSTRAINT_EVAL_FUNCTION_NAME),
                                vec![call.input.as_limbs().compile(CompileFor::Constraints)]
                                    .into_iter()
                                    .chain(state_vars.into_iter())
                                    .collect(),
                            ),
                        }));
                    }
                }
                AirBodyComponent::LookupCall(..) => {}
                AirBodyComponent::LookupAddInput { .. } => {}
                AirBodyComponent::LookupTerm {
                    relation_name,
                    felts,
                    use_or_yield,
                } => {
                    constraints.push(ConstraintEvalStep::LookupTerm(LookupTerm {
                        relation_name,
                        felts: felts
                            .into_iter()
                            .map(|f| f.compile(CompileFor::Constraints))
                            .collect(),
                        use_or_yield,
                    }));
                }
            }
        }

        constraints
    }

    // Returns the names of the lookup relations used or yielded by the air function, and the number
    // of terms per relation.
    pub fn get_lookup_names(&self) -> IndexMap<String, usize> {
        let mut lookup_calls = IndexMap::new();
        for component in &self.0 {
            match component {
                AirBodyComponent::Call(f) => {
                    for (relation_name, n_uses) in f.air_body.get_lookup_names() {
                        let v = lookup_calls.entry(relation_name.clone()).or_insert(0);
                        *v += n_uses;
                    }
                }
                AirBodyComponent::LookupTerm {
                    relation_name,
                    use_or_yield: _,
                    ..
                } => {
                    let v = lookup_calls.entry(relation_name.clone()).or_insert(0);
                    *v += 1;
                }
                _ => (),
            }
        }
        lookup_calls
    }

    pub fn get_inline_calls(&self) -> BTreeSet<String> {
        let mut inline_calls = BTreeSet::new();
        for component in &self.0 {
            if let AirBodyComponent::Call(call) = component {
                inline_calls.insert(call.air_fn_name.clone());
            }
        }
        inline_calls
    }

    // Counts the inputs added per lookup. This is an upper bound on the number of rows.
    pub fn get_lookup_n_rows(&self) -> IndexMap<String, usize> {
        let mut lookup_rows = IndexMap::new();
        self.0.iter().for_each(|comp| {
            if let AirBodyComponent::LookupAddInput { air_fn_name, .. } = comp {
                *lookup_rows.entry(air_fn_name.clone()).or_insert(0) += 1;
            }
            if let AirBodyComponent::Call(call) = comp {
                for (name, cnt) in call.air_body.get_lookup_n_rows() {
                    *lookup_rows.entry(name).or_insert(0) += cnt;
                }
            }
        });
        lookup_rows
    }

    // Counts the number of uses per lookup.
    pub fn get_lookup_n_use_cols(&self) -> IndexMap<String, usize> {
        let mut lookup_uses = IndexMap::new();
        self.0.iter().for_each(|comp| {
            if let AirBodyComponent::LookupTerm {
                relation_name,
                use_or_yield,
                ..
            } = comp
            {
                if *use_or_yield == UseOrYield::Use {
                    *lookup_uses.entry(relation_name.clone()).or_insert(0) += 1;
                }
            }
            if let AirBodyComponent::Call(call) = comp {
                for (name, cnt) in call.air_body.get_lookup_n_use_cols() {
                    *lookup_uses.entry(name).or_insert(0) += cnt;
                }
            }
        });
        lookup_uses
    }

    pub fn get_constraints(&self) -> Constraints {
        let mut intermediates = vec![];
        let mut constraints = vec![];
        let mut lookups = vec![];

        for comp in self.0.clone().into_iter() {
            match comp {
                AirBodyComponent::Constraint(expr, _) => {
                    constraints.push(expr.compile(CompileFor::Constraints).to_string())
                }
                AirBodyComponent::Assignment { constraint, .. } => {
                    constraints.push(constraint.compile(CompileFor::Constraints).to_string())
                }
                AirBodyComponent::Intermediate(Intermediate {
                    name,
                    var,
                    visibility,
                }) if visibility.in_constraints => {
                    intermediates.push((name, var.compile(CompileFor::Constraints).to_string()))
                }
                AirBodyComponent::Call(Call { air_body, .. }) => {
                    let call = air_body.get_constraints();
                    constraints.extend(call.constraints);
                    intermediates.extend(call.intermediates);
                    lookups.extend(call.lookups);
                }
                AirBodyComponent::LookupTerm {
                    relation_name,
                    felts,
                    use_or_yield: UseOrYield::Use,
                } => {
                    if relation_name == OPCODES_RELATION_NAME {
                        continue;
                    }
                    let felts = felts
                        .into_iter()
                        .map(|f| f.compile(CompileFor::Constraints).to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    lookups.push((relation_name, felts));
                }
                _ => {}
            }
        }

        Constraints {
            intermediates,
            constraints,
            lookups,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
pub enum CompileFor {
    Constraints,
    Deductions,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Constraints {
    pub intermediates: Vec<(String, String)>,
    pub constraints: Vec<String>,
    pub lookups: Vec<(String, String)>,
}
