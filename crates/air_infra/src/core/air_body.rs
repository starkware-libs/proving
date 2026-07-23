use std::collections::BTreeSet;
use std::fmt::Debug;
use std::iter::once;
use std::rc::Rc;

use air_common::{CONSTRAINT_EVAL_FUNCTION_NAME, ExternalState, UseOrYield};
use air_compile::compiled_structs::{
    CompiledAirVar, CompiledConstraintIntermediate, CompiledTraceGenIntermediate,
    ConstraintEvalStep, LookupTerm, TraceGenStep,
};
use indexmap::{IndexMap, IndexSet};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::ProverType;

use super::air_fn_registry::*;
use super::expressions::felt_expr::*;
use super::variables::*;
use crate::core::Felt;
use crate::core::public_params::PublicParam;

// A Call is an air_body component that represents a call to another air function.
// It contains the name of the air function, the input argument, the output of the call
// and the air_body of the called function.
#[derive(Clone, Debug)]
pub struct Call {
    pub entry: Rc<AirFnEntry>,
    pub input: AirVarImpl,
    pub enabler: FeltExpr,
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
#[allow(clippy::large_enum_variant)]
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
        relation_name: String,
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
        multiplicity: FeltExpr,
    },
}

pub enum ConstraintComponent {
    Constraint(FeltExpr),
    Intermediate {
        name: String,
        value: FeltExpr,
    },
    LookupTerm {
        relation_name: String,
        felts: Vec<FeltExpr>,
        use_or_yield: UseOrYield,
        multiplicity: FeltExpr,
    },
}

// A structure for the air_body of an air_fn.
#[derive(Debug, Clone, Default)]
pub struct AirBody(pub Vec<AirBodyComponent>);

impl AirBody {
    // Checks visibility and in_state status of the variables in the new component and adds it.
    pub fn push(&mut self, component: AirBodyComponent) {
        match &component {
            AirBodyComponent::Constraint(expr, desc) => {
                assert!(
                    expr.visibility().in_constraints && expr.in_state(),
                    "constraint must be in state and have only intermediate variables known in \
                     constraints"
                );
                let deg = expr.deg_in_state().unwrap();
                assert!(
                    deg <= 3,
                    "constraint must have degree <= 3, encountered degree {} in constraint named \
                     '{}' with expression\n{:#?}",
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
            AirBodyComponent::Assignment { constraint, deduction, desc: _ } => {
                assert!(
                    constraint.visibility().in_constraints && constraint.in_state(),
                    "constraint must be in state and have only intermediate variables known in \
                     constraints"
                );
                assert!(
                    deduction.visibility().in_deductions,
                    "deduction must have only intermediate variables known in deductions"
                );
            }
            AirBodyComponent::Intermediate(Intermediate { name: _, var, visibility }) => {
                assert!(
                    visibility.in_deductions || visibility.in_constraints,
                    "visibility of intermediates must be set"
                );
                if visibility.in_constraints {
                    assert!(
                        var.prover_type() == Felt::r#type(),
                        "only felts can be intermediates in constraints"
                    );
                    // We check that the variable is in_state since we don't want to create
                    // variables for constraints before deduction.
                    assert!(
                        var.in_state() && var.visibility().in_constraints,
                        "intermediate variable must be in state and have only intermediate \
                         variables known in constraints"
                    );
                }
                if visibility.in_deductions {
                    assert!(
                        var.visibility().in_deductions,
                        "intermediate variable must have only intermediate variables known in \
                         deductions"
                    );
                }
            }
            AirBodyComponent::Call(call) => {
                assert!(
                    call.entry.filter_input_limbs(call.input.clone()).visibility().in_constraints,
                    "call input must have only intermediate variables known in constraints"
                );
                assert!(
                    call.entry.filter_output_limbs(call.output.clone()).visibility().in_deductions,
                    "call output must have only intermediate variables known in constraints"
                );
            }
            AirBodyComponent::LookupCall(LookupCall { ext_input, input, .. }) => {
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
            AirBodyComponent::LookupAddInput { ext_input, input, .. } => {
                if let Some(ext_input) = ext_input {
                    assert!(
                        ext_input.visibility().in_deductions,
                        "lookup add input must have only intermediate variables known in \
                         deductions"
                    );
                }
                if let Some(input) = input {
                    assert!(
                        input.visibility().in_deductions,
                        "lookup add input must have only intermediate variables known in \
                         deductions"
                    );
                }
            }
            AirBodyComponent::LookupTerm { felts, relation_name, .. } => {
                for f in felts {
                    assert!(
                        f.visibility().in_deductions
                            && f.visibility().in_constraints
                            && f.in_state(),
                        "lookup term must be in state and have only intermediate variables known \
                         in deductions and constraints"
                    );
                    let deg = f.deg_in_state().unwrap();
                    assert!(
                        deg <= 1,
                        "lookup term must have degree <= 1, encountered degree {deg} in term \
                         named '{relation_name}' with expression {f:#?}",
                    );
                }
            }
        };

        self.0.push(component);
    }

    pub fn get_external_states(&self) -> IndexSet<ExternalState> {
        let mut external_states = IndexSet::<ExternalState>::default();

        for component in self.0.clone() {
            match component {
                AirBodyComponent::Constraint(felt_expr, _)
                | AirBodyComponent::Assignment { constraint: felt_expr, .. }
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
                AirBodyComponent::LookupTerm { felts, multiplicity, .. } => {
                    external_states.extend(
                        felts.iter().chain(once(&multiplicity)).flat_map(|f| f.external_states()),
                    );
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
                | AirBodyComponent::Assignment { constraint: felt_expr, .. }
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
                AirBodyComponent::LookupTerm { felts, multiplicity, .. } => {
                    public_params.extend(
                        felts.iter().chain(once(&multiplicity)).flat_map(|f| f.public_params()),
                    );
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
                    deductions
                        .push(TraceGenStep::Deduction(deduction.compile(CompileFor::Deductions)));
                }
                AirBodyComponent::Deduction(deduction, _) => {
                    deductions
                        .push(TraceGenStep::Deduction(deduction.compile(CompileFor::Deductions)));
                }
                AirBodyComponent::Intermediate(Intermediate { name, var, visibility }) => {
                    if visibility.in_deductions {
                        deductions.push(TraceGenStep::Intermediate(CompiledTraceGenIntermediate {
                            name,
                            r#type: var.prover_type(),
                            var: var.compile(CompileFor::Deductions),
                        }));
                    }
                }
                AirBodyComponent::Call(call) => {
                    let call_deductions = call.air_body.compile_for_deductions();
                    if !call_deductions.is_empty() {
                        deductions.push(TraceGenStep::StartBlock(call.entry.description.clone()));
                        deductions.extend(call_deductions);
                        deductions.push(TraceGenStep::EndBlock);
                    }
                }
                AirBodyComponent::LookupCall(call) => {
                    deductions.push(TraceGenStep::Intermediate(CompiledTraceGenIntermediate {
                        name: call.output_name,
                        r#type: call.output.prover_type(),
                        var: CompiledAirVar::StaticCall(
                            call.method_name,
                            vec![
                                AirFnEntry::join_inputs(call.ext_input, call.input)
                                    .compile(CompileFor::Deductions),
                            ],
                        ),
                    }));
                }
                AirBodyComponent::LookupAddInput {
                    relation_name,
                    air_fn_name: _,
                    ext_input,
                    input,
                } => {
                    deductions.push(TraceGenStep::LookupAddInput {
                        relation_name,
                        input: AirFnEntry::join_inputs(ext_input, input)
                            .compile(CompileFor::Deductions),
                    });
                }
                AirBodyComponent::LookupTerm {
                    relation_name,
                    felts,
                    use_or_yield,
                    multiplicity,
                } => {
                    deductions.push(TraceGenStep::LookupTerm(LookupTerm {
                        relation_name,
                        felts: felts
                            .into_iter()
                            .map(|f| f.compile(CompileFor::Deductions))
                            .collect(),
                        use_or_yield,
                        multiplicity: multiplicity.compile(CompileFor::Deductions),
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
                AirBodyComponent::Assignment { constraint, desc, .. } => {
                    constraints.push(ConstraintEvalStep::Constraint(
                        constraint.compile(CompileFor::Constraints),
                        desc,
                    ));
                }
                AirBodyComponent::Deduction(..) => {}
                AirBodyComponent::Intermediate(Intermediate { name, var, visibility }) => {
                    if visibility.in_constraints {
                        // These are only felt expressions (see assert in <push>).
                        constraints.push(ConstraintEvalStep::Intermediate(
                            CompiledConstraintIntermediate {
                                felt_names: vec![name],
                                var: var.compile(CompileFor::Constraints),
                            },
                        ));
                    }
                }
                AirBodyComponent::Call(call) => {
                    let call_constraints = call.air_body.compile_for_constraints();
                    if !call_constraints.is_empty() {
                        let state_vars = call
                            .state_names
                            .iter()
                            .map(|s| CompiledAirVar::State(s.clone()))
                            .collect::<Vec<_>>();
                        let input = call.entry.filter_input_limbs(call.input);
                        let input = input.compile(CompileFor::Constraints);
                        let enabler = call.enabler.compile(CompileFor::Constraints);

                        constraints.push(ConstraintEvalStep::Intermediate(
                            CompiledConstraintIntermediate {
                                felt_names: call.entry.output_limb_names(call.output_name),
                                var: CompiledAirVar::StaticCall(
                                    format!(
                                        "{}::{}",
                                        call.entry.name, CONSTRAINT_EVAL_FUNCTION_NAME
                                    ),
                                    vec![input]
                                        .into_iter()
                                        .chain(once(enabler))
                                        .chain(state_vars.into_iter())
                                        .collect(),
                                ),
                            },
                        ));
                    }
                }
                AirBodyComponent::LookupCall(..) => {}
                AirBodyComponent::LookupAddInput { .. } => {}
                AirBodyComponent::LookupTerm {
                    relation_name,
                    felts,
                    use_or_yield,
                    multiplicity,
                } => {
                    constraints.push(ConstraintEvalStep::LookupTerm(LookupTerm {
                        relation_name,
                        felts: felts
                            .into_iter()
                            .map(|f| f.compile(CompileFor::Constraints))
                            .collect(),
                        use_or_yield,
                        multiplicity: multiplicity.compile(CompileFor::Constraints),
                    }));
                }
            }
        }

        constraints
    }

    // Returns the names of the lookup relations used or yielded by the air function.
    pub fn get_constraint_lookups(&self) -> Vec<(String, UseOrYield)> {
        let mut lookup_calls = vec![];
        for component in &self.0 {
            match component {
                AirBodyComponent::Call(f) => {
                    lookup_calls.extend(f.air_body.get_constraint_lookups());
                }
                AirBodyComponent::LookupTerm { relation_name, use_or_yield, .. } => {
                    lookup_calls.push((relation_name.clone(), *use_or_yield));
                }
                _ => (),
            }
        }
        lookup_calls
    }

    // Returns the names of the lookup relations called by the air function.
    pub fn get_sub_components(&self) -> IndexSet<String> {
        let mut lookup_calls = IndexSet::new();
        for component in &self.0 {
            match component {
                AirBodyComponent::Call(f) => {
                    lookup_calls.extend(f.air_body.get_sub_components());
                }
                AirBodyComponent::LookupCall(LookupCall { air_fn_name, .. })
                | AirBodyComponent::LookupAddInput { air_fn_name, .. } => {
                    lookup_calls.insert(air_fn_name.clone());
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
                inline_calls.insert(call.entry.name.clone());
            }
        }
        inline_calls
    }

    // Counts the inputs added per lookup. This is an upper bound on the number of rows in the air
    // function table. The value in the output map is (air_fn_name, count).
    pub fn get_n_inputs_added_per_relation(&self) -> IndexMap<String, (String, usize)> {
        let mut lookup_rows = IndexMap::new();
        self.0.iter().for_each(|comp| {
            if let AirBodyComponent::LookupAddInput { relation_name, air_fn_name, .. } = comp {
                lookup_rows.entry(relation_name.clone()).or_insert((air_fn_name.clone(), 0)).1 += 1;
            }
            if let AirBodyComponent::Call(call) = comp {
                for (relation_name, (air_fn_name, cnt)) in
                    call.air_body.get_n_inputs_added_per_relation()
                {
                    lookup_rows.entry(relation_name).or_insert((air_fn_name, 0)).1 += cnt;
                }
            }
        });
        lookup_rows
    }

    pub fn get_used_constraint_intermediates(&self) -> IndexSet<String> {
        let mut result = IndexSet::new();
        for eval_component in self.get_flattened_constraint_components() {
            match eval_component {
                ConstraintComponent::Constraint(expr) => {
                    result.extend(expr.get_used_constraint_intermediates())
                }
                ConstraintComponent::Intermediate { value, .. } => {
                    result.extend(value.get_used_constraint_intermediates());
                }
                ConstraintComponent::LookupTerm { felts, multiplicity, .. } => {
                    for f in felts {
                        result.extend(f.get_used_constraint_intermediates());
                    }
                    result.extend(multiplicity.get_used_constraint_intermediates());
                }
            }
        }
        result
    }

    // Return a list of all constraint-evaluation-related operations performed by this AirFn,
    // including all operations performed by inline AirFns that it calls.
    pub fn get_flattened_constraint_components(&self) -> Vec<ConstraintComponent> {
        let mut result = vec![];
        for component in self.0.iter() {
            match component {
                AirBodyComponent::Constraint(expr, _)
                | AirBodyComponent::Assignment { constraint: expr, .. } => {
                    result.push(ConstraintComponent::Constraint(expr.clone()))
                }
                AirBodyComponent::Intermediate(intermediate) => {
                    if intermediate.visibility.in_constraints {
                        result.push(ConstraintComponent::Intermediate {
                            name: intermediate.name.clone(),
                            value: intermediate.var.as_felt(),
                        })
                    }
                }
                AirBodyComponent::Call(call) => {
                    result.extend(call.air_body.get_flattened_constraint_components());
                }
                AirBodyComponent::Deduction(..)
                | AirBodyComponent::LookupCall(..)
                | AirBodyComponent::LookupAddInput { .. } => (),
                AirBodyComponent::LookupTerm {
                    felts,
                    relation_name,
                    use_or_yield,
                    multiplicity,
                } => result.push(ConstraintComponent::LookupTerm {
                    relation_name: relation_name.to_string(),
                    felts: felts.clone(),
                    use_or_yield: *use_or_yield,
                    multiplicity: multiplicity.clone(),
                }),
            }
        }
        result
    }
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
pub enum CompileFor {
    Constraints,
    Deductions,
}
