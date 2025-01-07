use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;

use compiled_casm_air::compiled_structs::{
    CompiledAirVar, ConstraintEvalStep, ConstraintLeanCompare, Intermediate, LookupTerm,
    TraceGenStep, UseOrYield,
};
use compiled_casm_air::public_params::PublicParam;
use convert_case::{Case, Casing};
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

// Deduces the output and updates inputs / multiplicity of the relation.
#[derive(Clone, Debug, Serialize)]
pub struct LookupCall {
    pub air_fn_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext_input: Option<AirVarImpl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<AirVarImpl>,
    // None if there is no output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_name: Option<String>,
}

// Each air function has an air_body, which is a vector of AirBodyComponent.
// These are the components of the air function.
#[derive(Clone, Debug, Serialize)]
pub enum AirBodyComponent {
    Constraint(
        FeltExpr,
        #[serde(skip_serializing_if = "Option::is_none")] Option<String>,
    ),
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
    Intermediate(String, String, AirVarImpl, Visibility),
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
#[derive(Debug, Clone, Serialize)]
pub struct AirBody(pub Vec<AirBodyComponent>);

impl AirBody {
    // Transforms the air body of an air function into the compiled air fn format.
    pub fn compile(
        &self,
    ) -> (
        Vec<TraceGenStep>,
        Vec<ConstraintEvalStep>,
        HashSet<PublicParam>,
    ) {
        let mut constraints = vec![];
        let mut deductions = vec![];
        let mut public_params = HashSet::new();

        for component in self.0.clone() {
            match component {
                AirBodyComponent::Constraint(constraint, desc) => {
                    constraints.push(ConstraintEvalStep::Constraint(
                        constraint.clone().into(),
                        desc,
                    ));
                    public_params.extend(constraint.public_params());
                }
                AirBodyComponent::Assignment {
                    constraint,
                    deduction,
                    desc,
                } => {
                    constraints.push(ConstraintEvalStep::Constraint(
                        constraint.clone().into(),
                        desc.clone(),
                    ));
                    deductions.push(TraceGenStep::Deduction(deduction.into()));
                    public_params.extend(constraint.public_params());
                }
                AirBodyComponent::Deduction(deduction, _) => {
                    deductions.push(TraceGenStep::Deduction(deduction.clone().into()));
                    public_params.extend(deduction.public_params());
                }
                AirBodyComponent::Intermediate(name, var_ty, var, ty) => {
                    if ty.in_constraints {
                        constraints.push(ConstraintEvalStep::Intermediate(Intermediate {
                            name: name.clone(),
                            r#type: var_ty.clone(),
                            var: var.clone().into(),
                        }));
                    }

                    if ty.in_deductions {
                        deductions.push(TraceGenStep::Intermediate(Intermediate {
                            name,
                            r#type: var_ty,
                            var: var.clone().into(),
                        }));
                    }
                    public_params.extend(var.public_params());
                }
                AirBodyComponent::Call(f) => {
                    let (new_deductions, new_constraints, new_public_params) = f.air_body.compile();
                    if !new_constraints.is_empty() {
                        constraints
                            .push(ConstraintEvalStep::StartBlock(f.air_fn_description.clone()));
                        constraints.extend(new_constraints);
                        constraints.push(ConstraintEvalStep::EndBlock);
                    }
                    if !new_deductions.is_empty() {
                        deductions.push(TraceGenStep::StartBlock(f.air_fn_description));
                        deductions.extend(new_deductions);
                        deductions.push(TraceGenStep::EndBlock);
                    }
                    public_params.extend(new_public_params);
                }
                AirBodyComponent::LookupCall(call) => {
                    deductions.push(TraceGenStep::LookupCall {
                        fn_name: call.air_fn_name,
                        input: AirFnEntry::generate_input(call.ext_input, call.input),
                        output_name: call.output_name,
                    });
                }
                AirBodyComponent::LookupAddInput {
                    air_fn_name,
                    ext_input,
                    input,
                } => {
                    deductions.push(TraceGenStep::LookupAddInput {
                        fn_name: air_fn_name,
                        input: AirFnEntry::generate_input(ext_input, input),
                    });
                }
                AirBodyComponent::LookupTerm {
                    relation_name,
                    felts,
                    use_or_yield,
                } => {
                    constraints.push(ConstraintEvalStep::LookupTerm(LookupTerm {
                        relation_name: relation_name.clone(),
                        felts: felts.clone().into_iter().map(|f| f.into()).collect(),
                        use_or_yield,
                    }));
                    deductions.push(TraceGenStep::LookupTerm(LookupTerm {
                        relation_name,
                        felts: felts.clone().into_iter().map(|f| f.into()).collect(),
                        use_or_yield,
                    }));
                    public_params.extend(felts.iter().flat_map(|f| f.public_params()));
                }
            }
        }

        (deductions, constraints, public_params)
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
            .map(|component| match component {
                AirBodyComponent::Call(f) => f.air_body.get_n_lookup_terms(),
                AirBodyComponent::LookupTerm { .. } => 1,
                _ => 0,
            })
            .sum()
    }

    pub fn get_constraints(&self) -> Vec<ConstraintLeanCompare> {
        let mut res = vec![];

        for comp in self.0.clone().into_iter() {
            match comp {
                AirBodyComponent::Constraint(expr, _) => {
                    res.push(ConstraintLeanCompare::Constraint(
                        CompiledAirVar::from(expr).to_string(),
                    ));
                }
                AirBodyComponent::Assignment {
                    constraint,
                    deduction: _,
                    desc: _,
                } => {
                    res.push(ConstraintLeanCompare::Constraint(
                        CompiledAirVar::from(constraint).to_string(),
                    ));
                }
                AirBodyComponent::Intermediate(
                    name,
                    ty,
                    expr,
                    Visibility {
                        in_constraints: true,
                        in_deductions: _,
                    },
                ) => {
                    res.push(ConstraintLeanCompare::Intermediate {
                        name: name.clone(),
                        r#type: ty,
                        var: CompiledAirVar::from(expr).to_string(),
                    });
                }
                AirBodyComponent::Call(Call {
                    air_fn_name,
                    air_fn_description: _,
                    input,
                    output,
                    air_body: _,
                }) => {
                    res.push(ConstraintLeanCompare::Call {
                        fn_name: air_fn_name,
                        input: CompiledAirVar::from(input).to_string(),
                        output: CompiledAirVar::from(output).to_string(),
                    });
                }
                AirBodyComponent::LookupTerm {
                    relation_name,
                    felts,
                    use_or_yield: UseOrYield::Use,
                } => {
                    let felts = felts
                        .into_iter()
                        .map(|f| CompiledAirVar::from(f).to_string())
                        .collect::<Vec<_>>();
                    res.push(ConstraintLeanCompare::LookupUse {
                        relation_name,
                        felts,
                    });
                }
                _ => {}
            }
        }

        res.sort();
        res
    }
}
