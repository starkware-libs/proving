use std::collections::BTreeSet;
use std::fmt::Debug;

use compiled_casm_air::compiled_structs::{
    CompiledAirVar, ConstraintEvalStep, Intermediate, LookupTerm, TraceGenStep, UseOrYield,
};
use compiled_casm_air::public_params::PublicParam;
use compiled_casm_air::relations::OPCODES_RELATION_NAME;
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use serde::Serialize;

use super::air_fn_registry::*;
use super::expressions::felt_expr::*;
use super::variables::*;

// A Call is an air_body component that represents a call to another air function.
// It contains the name of the air function, the input argument, the output of the call
// and the air_body of the called function.
#[derive(Clone, Debug, Serialize)]
pub struct Call {
    pub air_fn_name: String,
    pub air_fn_description: String,
    pub input: AirVarImpl,
    pub output: AirVarImpl,
    #[serde(skip)]
    pub air_body: AirBody,
}

// Computes the output of the component into an intermediate variable named <output_name>.
#[derive(Clone, Debug, Serialize)]
pub struct LookupCall {
    pub air_fn_name: String,
    pub method_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext_input: Option<AirVarImpl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<AirVarImpl>,
    pub output_name: String,
    pub output_type: String,
}

// Each air function has an air_body, which is a vector of AirBodyComponent.
// These describe the steps to execute the function.
#[derive(Clone, Debug, Serialize)]
pub enum AirBodyComponent {
    // Add a constraint that the given expression equals zero.
    Constraint(
        FeltExpr,
        #[serde(skip_serializing_if = "Option::is_none")] Option<String>,
    ),

    // Write the value of the given expression to the next cell in the state.
    Deduction(
        FeltExpr,
        #[serde(skip_serializing_if = "Option::is_none")] Option<String>,
    ),

    // An assignment is a constraint and a deduction referring to the same trace cell.
    // For example, when copying a value from one trace cell to another.
    Assignment {
        constraint: FeltExpr,
        deduction: FeltExpr,
        #[serde(skip_serializing_if = "Option::is_none")]
        desc: Option<String>,
    },

    // Create a new local variable in the generated code. The visibility controls whether
    // to create the variable in the trace generation code, constraint evaluation code
    // or both.
    Intermediate(String, String, AirVarImpl, Visibility),

    // Call an inline air function. This component will be replaced by the air_body of
    // the callee during the compilation process.
    Call(Call),

    LookupCall(LookupCall),

    // Adds the input to the lookup table or updates multiplicity.
    LookupAddInput {
        air_fn_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        ext_input: Option<AirVarImpl>,
        #[serde(skip_serializing_if = "Option::is_none")]
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
#[derive(Debug, Clone, Serialize, Default)]
pub struct AirBody(Vec<AirBodyComponent>);

impl AirBody {
    pub fn get_prev_chain_id(&self, chain_round_name: &str) -> Option<FeltExpr> {
        for component in self.0.iter().rev() {
            match component {
                AirBodyComponent::LookupCall(LookupCall {
                    air_fn_name, input, ..
                }) if air_fn_name == chain_round_name => {
                    if let Some(AirVarImpl::Tuple(vars)) = input {
                        return Some(vars[0].as_felt());
                    }
                    panic!("Expected tuple input for chain round lookup call");
                }
                _ => {}
            }
        }
        None
    }

    // Checks visibility and in_state status of the variables in the new component and adds it.
    pub fn push(&mut self, component: AirBodyComponent) {
        match &component {
            AirBodyComponent::Constraint(expr, _) => {
                assert!(
                    expr.visibility().in_constraints && expr.in_state(),
                    "constraint must be in state and have only intermediate variables known in constraints"
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
            AirBodyComponent::Intermediate(_, _, var, visibility) => {
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
            AirBodyComponent::LookupTerm { felts, .. } => {
                for f in felts {
                    assert!(
                        f.visibility().in_deductions && f.visibility().in_constraints && f.in_state(),
                        "lookup term must be in state and have only intermediate variables known in deductions and constraints"
                    );
                }
            }
        };

        self.0.push(component);
    }

    // Transforms the air body of an air function into the compiled air fn format.
    pub fn compile(&self) -> CompiledAirBody {
        let mut compiled = CompiledAirBody::default();

        for component in self.0.clone() {
            match component {
                AirBodyComponent::Constraint(constraint, desc) => {
                    compiled.constraints.push(ConstraintEvalStep::Constraint(
                        constraint.clone().into(),
                        desc,
                    ));
                    compiled.public_params.extend(constraint.public_params());
                    compiled
                        .external_states
                        .extend(constraint.external_states());
                }
                AirBodyComponent::Assignment {
                    constraint,
                    deduction,
                    desc,
                } => {
                    compiled.constraints.push(ConstraintEvalStep::Constraint(
                        constraint.clone().into(),
                        desc.clone(),
                    ));
                    compiled
                        .deductions
                        .push(TraceGenStep::Deduction(deduction.into()));
                    compiled.public_params.extend(constraint.public_params());
                    compiled
                        .external_states
                        .extend(constraint.external_states());
                }
                AirBodyComponent::Deduction(deduction, _) => {
                    compiled
                        .deductions
                        .push(TraceGenStep::Deduction(deduction.clone().into()));
                    compiled.public_params.extend(deduction.public_params());
                    compiled.external_states.extend(deduction.external_states());
                }
                AirBodyComponent::Intermediate(name, var_ty, var, ty) => {
                    if ty.in_constraints {
                        compiled
                            .constraints
                            .push(ConstraintEvalStep::Intermediate(Intermediate {
                                name: name.clone(),
                                r#type: var_ty.clone(),
                                var: var.clone().into(),
                            }));
                    }

                    if ty.in_deductions {
                        compiled
                            .deductions
                            .push(TraceGenStep::Intermediate(Intermediate {
                                name,
                                r#type: var_ty,
                                var: var.clone().into(),
                            }));
                    }
                    compiled.public_params.extend(var.public_params());
                    compiled.external_states.extend(var.external_states());
                }
                AirBodyComponent::Call(f) => {
                    let f_air_body = f.air_body.compile();
                    if !f_air_body.constraints.is_empty() {
                        compiled
                            .constraints
                            .push(ConstraintEvalStep::StartBlock(f.air_fn_description.clone()));
                        compiled.constraints.extend(f_air_body.constraints);
                        compiled.constraints.push(ConstraintEvalStep::EndBlock);
                    }
                    if !f_air_body.deductions.is_empty() {
                        compiled
                            .deductions
                            .push(TraceGenStep::StartBlock(f.air_fn_description));
                        compiled.deductions.extend(f_air_body.deductions);
                        compiled.deductions.push(TraceGenStep::EndBlock);
                    }
                    compiled.public_params.extend(f_air_body.public_params);
                    compiled.external_states.extend(f_air_body.external_states);
                }
                AirBodyComponent::LookupCall(call) => {
                    compiled
                        .deductions
                        .push(TraceGenStep::Intermediate(Intermediate {
                            name: call.output_name,
                            r#type: call.output_type,
                            var: CompiledAirVar::StaticCall(
                                call.method_name,
                                vec![AirFnEntry::generate_input(call.ext_input, call.input)],
                            ),
                        }));
                }
                AirBodyComponent::LookupAddInput {
                    air_fn_name,
                    ext_input,
                    input,
                } => {
                    compiled.deductions.push(TraceGenStep::LookupAddInput {
                        fn_name: air_fn_name,
                        input: AirFnEntry::generate_input(ext_input, input),
                    });
                }
                AirBodyComponent::LookupTerm {
                    relation_name,
                    felts,
                    use_or_yield,
                } => {
                    compiled
                        .constraints
                        .push(ConstraintEvalStep::LookupTerm(LookupTerm {
                            relation_name: relation_name.clone(),
                            felts: felts.clone().into_iter().map(|f| f.into()).collect(),
                            use_or_yield,
                        }));
                    compiled
                        .deductions
                        .push(TraceGenStep::LookupTerm(LookupTerm {
                            relation_name,
                            felts: felts.clone().into_iter().map(|f| f.into()).collect(),
                            use_or_yield,
                        }));
                    compiled
                        .public_params
                        .extend(felts.iter().flat_map(|f| f.public_params()));
                    compiled
                        .external_states
                        .extend(felts.iter().flat_map(|f| f.external_states()));
                }
            }
        }

        compiled
    }

    // Returns the names of the lookup relations used and lookup components called by the air
    // function.
    pub fn get_lookup_names(&self) -> BTreeSet<String> {
        let mut lookup_calls = BTreeSet::new();
        // for deduction in deductions {
        for component in &self.0 {
            match component {
                AirBodyComponent::Call(f) => {
                    lookup_calls.extend(f.air_body.get_lookup_names());
                }
                AirBodyComponent::LookupCall(LookupCall { air_fn_name, .. }) => {
                    lookup_calls.insert(air_fn_name.clone());
                }
                AirBodyComponent::LookupTerm {
                    relation_name,
                    use_or_yield,
                    ..
                } => {
                    if *use_or_yield == UseOrYield::Use {
                        lookup_calls.insert(relation_name.to_case(Case::Snake));
                    }
                }
                _ => (),
            }
        }
        lookup_calls
    }

    // Sums the number of uses and yields.
    pub fn get_n_lookup_terms(&self) -> usize {
        self.0
            .iter()
            .map(|comp| match comp {
                AirBodyComponent::Call(f) => f.air_body.get_n_lookup_terms(),
                AirBodyComponent::LookupTerm { .. } => 1,
                _ => 0,
            })
            .sum()
    }

    // Counts the inputs added per lookup. This is an upper bound on the number of rows.
    pub fn get_lookup_n_rows(&self) -> IndexMap<String, usize> {
        let mut lookup_rows = IndexMap::new();
        self.0.iter().for_each(|comp| {
            if let AirBodyComponent::LookupAddInput { air_fn_name, .. } = comp {
                *lookup_rows.entry(air_fn_name.clone()).or_insert(0) += 1;
            }
            if let AirBodyComponent::Call(f) = comp {
                for (name, cnt) in f.air_body.get_lookup_n_rows() {
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
            if let AirBodyComponent::Call(f) = comp {
                for (name, cnt) in f.air_body.get_lookup_n_use_cols() {
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
                    constraints.push(CompiledAirVar::from(expr).to_string())
                }
                AirBodyComponent::Assignment { constraint, .. } => {
                    constraints.push(CompiledAirVar::from(constraint).to_string())
                }
                AirBodyComponent::Intermediate(
                    name,
                    _,
                    expr,
                    Visibility {
                        in_constraints: true,
                        in_deductions: _,
                    },
                ) => intermediates.push((name, CompiledAirVar::from(expr).to_string())),
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
                        .map(|f| CompiledAirVar::from(f).to_string())
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

#[derive(Debug, Clone, Serialize, Default)]
pub struct Constraints {
    pub intermediates: Vec<(String, String)>,
    pub constraints: Vec<String>,
    pub lookups: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CompiledAirBody {
    pub deductions: Vec<TraceGenStep>,
    pub constraints: Vec<ConstraintEvalStep>,
    pub public_params: BTreeSet<PublicParam>,
    pub external_states: BTreeSet<(String, Vec<String>)>,
}
