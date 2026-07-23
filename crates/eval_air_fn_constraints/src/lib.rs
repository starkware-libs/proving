pub mod assignment;
mod logup;
mod scope;
mod util;

use std::rc::Rc;

use air_common::{TraceType, UseOrYield};
use air_compile::compiled_structs::{
    CompiledAirFn, CompiledAirVar, ConstraintEvalStep, LookupTerm,
};
use assignment::Assignment;
use indexmap::IndexMap;
use logup::evaluate_logup_constraints;
use num_traits::Zero;
use scope::Scope;
use serde::{Deserialize, Serialize};
use stwo_cairo_common::prover_types::cpu::QM31;
use util::Environment;

/// A random assignment for the variables that appear in a composition polynomial,
/// and the evaluation of the polynomial on these values.
#[derive(Serialize, Deserialize)]
pub struct SampleEvaluation {
    pub assignment: Assignment,
    pub result: QM31,
}

#[derive(Clone)]
struct EvaluatedLookupTerm {
    felt_values: Vec<QM31>,
    use_or_yield_sign: QM31,
    multiplicity_value: QM31,
}

enum EvaluatedStep {
    Constraint(QM31),
    LookupTerm(EvaluatedLookupTerm),
}

pub fn create_sample_evaluation(
    compiled_registry: &IndexMap<String, CompiledAirFn>,
    component_name: &String,
) -> SampleEvaluation {
    let component = compiled_registry
        .get(component_name)
        .unwrap_or_else(|| panic!("Component {component_name} missing"));

    // Inline AirFns don't have a well defined constraint polynomial so we don't support them.
    assert!(component.r#type != TraceType::Inline);

    let assignment = Assignment::new_random_for(component);
    let (_, steps) = run_component_and_collect_steps(
        compiled_registry,
        component,
        &[],
        &assignment.base_trace,
        assignment.environment.clone(),
    );
    SampleEvaluation { result: evaluate_composition_polynomial(steps, &assignment), assignment }
}

/// Run an AirFn with the given input, trace and environment. Return its output and
/// the list of constraints steps it performed.
fn run_component_and_collect_steps(
    compiled_registry: &IndexMap<String, CompiledAirFn>,
    component: &CompiledAirFn,
    input: &[QM31],
    rest_params: &[QM31],
    environment: Rc<Environment>,
) -> (Vec<QM31>, Vec<EvaluatedStep>) {
    let verifier_input_names = &component.verifier_input_limbs;
    let (enabler, base_trace) = if component.r#type == TraceType::Inline {
        // Inline components receive the enabler as the first parameter after the input
        (Some(rest_params[0]), &rest_params[1..])
    } else {
        (None, rest_params)
    };
    let mut scope = Scope::new(Rc::clone(&environment), enabler);
    assert_eq!(base_trace.len(), component.state_names.len());
    assert_eq!(input.len(), verifier_input_names.len());

    for (name, value) in component.state_names.iter().zip(base_trace) {
        scope.add_new_var(name.clone(), *value);
    }

    for (name, value) in verifier_input_names.iter().zip(input) {
        scope.add_new_var(name.clone(), *value);
    }

    let mut steps = vec![];
    for constraint in component.constraints.iter() {
        match constraint {
            ConstraintEvalStep::Constraint(compiled_air_var, _) => {
                steps.push(EvaluatedStep::Constraint(scope.evaluate(compiled_air_var)))
            }
            ConstraintEvalStep::LookupTerm(LookupTerm {
                felts,
                use_or_yield,
                relation_name: _,
                multiplicity,
            }) => {
                let multiplicity_value = scope.evaluate(multiplicity);
                let felt_values = felts.iter().map(|f| scope.evaluate(f)).collect();
                let use_or_yield_sign = match use_or_yield {
                    UseOrYield::Use => 1.into(),
                    UseOrYield::Yield => (-1).into(),
                };
                steps.push(EvaluatedStep::LookupTerm(EvaluatedLookupTerm {
                    felt_values,
                    use_or_yield_sign,
                    multiplicity_value,
                }));
            }
            ConstraintEvalStep::Intermediate(compiled_intermediate) => {
                match &compiled_intermediate.var {
                    CompiledAirVar::StaticCall(fn_name, params) => {
                        let (return_values, mut callee_steps) = run_static_call_and_collect_steps(
                            compiled_registry,
                            fn_name,
                            params,
                            &scope,
                        );
                        steps.append(&mut callee_steps);

                        // Add the return values to the scope
                        for (name, value) in
                            compiled_intermediate.felt_names.iter().zip(return_values)
                        {
                            scope.add_new_var(name.clone(), value);
                        }
                    }
                    CompiledAirVar::BinaryOp(..)
                    | CompiledAirVar::Const(..)
                    | CompiledAirVar::State(..)
                    | CompiledAirVar::Var(..)
                    | CompiledAirVar::ExternalState(..)
                    | CompiledAirVar::PublicParam(..) => {
                        // These intermediate types are expected to be one felt in size
                        assert_eq!(compiled_intermediate.felt_names.len(), 1);
                        let value = scope.evaluate(&compiled_intermediate.var);
                        scope.add_new_var(compiled_intermediate.felt_names[0].clone(), value);
                    }
                    _ => panic!("Unexpected value for intermediate: {compiled_intermediate:?}"),
                }
            }
        }
    }

    let CompiledAirVar::Array(exprs) = &component.verifier_output.0 else {
        panic!("Invalid output expression in {}", &component.name)
    };
    let output_values = exprs.iter().map(|e| scope.evaluate(e)).collect();
    (output_values, steps)
}

fn run_static_call_and_collect_steps(
    compiled_registry: &IndexMap<String, CompiledAirFn>,
    fn_name: &String,
    params: &[CompiledAirVar],
    scope: &Scope,
) -> (Vec<QM31>, Vec<EvaluatedStep>) {
    let called_component_name = fn_name
        .strip_suffix("::evaluate")
        .unwrap_or_else(|| panic!("StaticCall name {fn_name} doesn't end with '::evaluate'"));
    let called_component = compiled_registry
        .get(&called_component_name.to_string())
        .unwrap_or_else(|| panic!("Component {fn_name} is missing"));
    let CompiledAirVar::Array(input) = params[0].clone() else {
        panic!("Unexpected StaticCall input")
    };
    let input_values = input.iter().map(|input_var| scope.evaluate(input_var)).collect::<Vec<_>>();
    let state_values = params[1..].iter().map(|var| scope.evaluate(var)).collect::<Vec<_>>();
    run_component_and_collect_steps(
        compiled_registry,
        called_component,
        &input_values,
        &state_values,
        scope.environment(),
    )
}

fn evaluate_composition_polynomial(steps: Vec<EvaluatedStep>, assignment: &Assignment) -> QM31 {
    let mut constraint_evals = vec![];
    let mut lookup_terms: Vec<EvaluatedLookupTerm> = vec![];

    // Split the steps to local and lookup constraints
    for step in steps {
        match step {
            EvaluatedStep::Constraint(value) => constraint_evals.push(value),
            EvaluatedStep::LookupTerm(evaluated_lookup_term) => {
                lookup_terms.push(evaluated_lookup_term)
            }
        }
    }

    // Evaluate the logup constraints
    constraint_evals.append(&mut evaluate_logup_constraints(assignment, &lookup_terms));

    // Combine all constraint evaluations to get the composition poynomial.
    let mut result = QM31::zero();
    for eval in constraint_evals {
        result = result * assignment.random_coeff + eval;
    }

    result
}
