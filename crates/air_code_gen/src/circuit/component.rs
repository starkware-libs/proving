use std::collections::HashSet;

use air_common::{CONSTRAINT_EVAL_FUNCTION_NAME, TraceType, UseOrYield};
use air_compile::compiled_structs::*;
use convert_case::{Case, Casing};
use eval_air_fn_constraints::SampleEvaluation;
use eval_air_fn_constraints::assignment::Assignment;
use genco::lang::{Rust, rust};
use genco::tokens::quoted;
use genco::{Tokens, quote};
use itertools::{Itertools, chain};
use stwo_cairo_common::prover_types::cpu::QM31;

use crate::utils::*;

pub fn generate_circuit_constraints_code(
    air_fn: &CompiledAirFn,
    sample_evaluation: Option<&SampleEvaluation>,
) -> Tokens<Rust> {
    let (return_type, prelude_use) = if air_fn.r#type == TraceType::Inline {
        (quote! { -> Vec<Var> }, quote! { use super::super::prelude::*; })
    } else {
        (quote! {}, quote! { use super::prelude::*; })
    };
    let mut code = quote! {
        $(prelude_use)
        $("\n\n")
    };

    if air_fn.r#type != TraceType::Inline {
        let interaction_columns = air_fn.constraint_lookups.len().div_ceil(2) * 4;

        code.append(quote! {
            pub const N_TRACE_COLUMNS: usize = $(input_names(air_fn).len());
            pub const N_INTERACTION_COLUMNS: usize = $(interaction_columns);$("\n\n")
        });
    }

    code.append(generate_relation_uses(air_fn));
    code.append(quote! { $("\n\n") });

    code.append(quote! {
        #[allow(unused_variables)]
        pub fn accumulate_constraints<Value: IValue>(
            input: &[Var],
            context: &mut Context<Value>,
            component_data: &dyn ComponentDataTrait<Value>,
            acc: &mut CompositionConstraintAccumulator) $(return_type) {
            $(generate_accumulate_constraints(air_fn))
        }
        $("\n\n")
    });

    if air_fn.r#type != TraceType::Inline {
        // In circuit proofs, the prover sends the size of all components, including
        // const-size ones. Verify that the size of const-size components is the
        // correct one.
        let check_component_size = if let Some(log_height) = air_fn.log_height {
            quote! {
                $(format!("// Verify this component has 2 ** {log_height} rows"))
                let size_bit = component_data.get_n_instances_bit(context, $(log_height));
                eq(context, size_bit, context.one());
            }
        } else {
            quote! {}
        };

        let (mut log_size, mut preprocessed_column_log_sizes) =
            (quote! { None }, quote! { _preprocessed_column_log_sizes });
        if let Some(log_height) = air_fn.log_height {
            log_size = quote! { Some($(log_height)) };
        }
        // Gates have external states of known size, so we need to get the log size from the
        // preprocessed column log sizes.
        if air_fn.r#type == TraceType::Gate {
            let col_id =
                air_fn.external_states.iter().next().expect("Expected at least one external state");
            log_size = quote! {
                preprocessed_column_log_sizes
                    .get(&PreProcessedColumnId { id: $(quoted(col_id)).to_string() })
                    .cloned()
            };
            preprocessed_column_log_sizes = quote! { preprocessed_column_log_sizes };
        }

        code.append(quote! {

            pub struct Component {}
            impl<Value: IValue> CircuitEval<Value> for Component {
                fn name(&self) -> String {
                    $(quoted(&air_fn.name)).to_string()
                }

                fn evaluate(
                    &self,
                    context: &mut Context<Value>,
                    component_data: &dyn ComponentDataTrait<Value>,
                    acc: &mut CompositionConstraintAccumulator,
                ) {
                    accumulate_constraints(
                        component_data.trace_columns(),
                        context,
                        component_data,
                        acc
                    );
                    $(check_component_size)
                }

                fn trace_columns(&self) -> usize {
                    N_TRACE_COLUMNS
                }

                fn interaction_columns(&self) -> usize {
                    N_INTERACTION_COLUMNS
                }

                fn relation_uses_per_row(&self) -> &[RelationUse] {
                    &RELATION_USES_PER_ROW
                }

                fn log_size(
                    &self,
                    $(preprocessed_column_log_sizes): &OrderedHashMap<PreProcessedColumnId, u32>
                ) -> Option<u32> {
                    $(log_size)
                }
            }
        });
        code.append(gen_tests_module(
            air_fn,
            &sample_evaluation
                .unwrap_or_else(|| panic!("Missing sample evaluation {}", air_fn.name))
                .assignment,
        ));
    }
    code
}

fn generate_accumulate_constraints(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    let input_names = input_names(air_fn);

    // Unpack input
    code.append(quote! {
        let [$(input_names.join(",\n"))] = input.try_into().unwrap();$("\n")
    });

    let atoms = get_constraint_atoms(air_fn);
    let used_preprocessed_columns: HashSet<String> = atoms
        .iter()
        .filter_map(|a| {
            if let CompiledAirVar::ExternalState(col_id) = a { Some(col_id.clone()) } else { None }
        })
        .collect();

    // Read preprocessed columns
    for external_col_id in used_preprocessed_columns.iter().sorted() {
        // Seq is the only preprocessed column that is of unfixed size.
        if external_col_id == "Seq" {
            code.append(quote! {
                let seq = seq_of_component_size(context, component_data, &acc.preprocessed_columns);
            });
        } else {
            code.append(quote! {
                let $(external_col_id) = acc.get_preprocessed_column(&$(make_preprocessed_column_id(external_col_id)));
            });
        }
    }

    let used_public_params: HashSet<String> = atoms
        .iter()
        .filter_map(|a| {
            if let CompiledAirVar::PublicParam(param) = a { Some(param.clone()) } else { None }
        })
        .collect();

    for public_param in used_public_params.iter().sorted() {
        code.append(quote! {
            let $(public_param) = *acc.public_params.get($(quoted(public_param))).unwrap();
        });
    }

    // Evaluate constraints
    for (i, constraint) in air_fn.constraints.iter().enumerate() {
        code.append(quote! { $("\n\n") });
        match constraint {
            ConstraintEvalStep::Constraint(expr, desc) => {
                if let Some(desc) = desc {
                    code.append(quote! {
                        $("//")$desc.$("\n")
                    });
                }
                code.append(quote! {
                    let constraint_$(i)_value = $(make_var_for_expr(expr));
                    acc.add_constraint(context, constraint_$(i)_value);
                });
            }
            ConstraintEvalStep::Intermediate(CompiledConstraintIntermediate {
                felt_names,
                var,
            }) => {
                if let CompiledAirVar::StaticCall(..) = var {
                    if felt_names.is_empty() {
                        // This StaticCall just checks constraints - no output
                        code.append(quote! { $(make_var_for_expr(var)); });
                    } else {
                        // This StaticCall also returns output - put it in variables
                        code.append(quote! {
                            let [$(felt_names.join(", "))] = $(make_var_for_expr(var)).try_into().unwrap();
                        })
                    }
                } else {
                    assert_eq!(
                        felt_names.len(),
                        1,
                        "In constraints, only StaticCalls are allowed to produce multiple-felt \
                         outputs"
                    );
                    code.append(quote! {
                        let $(&felt_names[0]) = $(make_var_for_expr(var));
                    });
                }
            }
            ConstraintEvalStep::LookupTerm(LookupTerm {
                relation_name,
                felts,
                use_or_yield,
                multiplicity,
            }) => {
                let felts = remove_trailing_zeroes(felts);
                let mut felt_strings =
                    felts.iter().map(|f| make_var_for_expr(f).to_string().unwrap());

                let numerator = make_eval_body_for_expr(multiplicity);

                let numerator = match use_or_yield {
                    UseOrYield::Use => quote! { eval!(context, $(numerator)) },
                    UseOrYield::Yield => quote! { eval!(context, -($(numerator))) },
                };

                let verb = match use_or_yield {
                    UseOrYield::Use => "Use",
                    UseOrYield::Yield => "Yield",
                };
                code.append(quote! {
                    $("//") $(verb) $(relation_name).
                    let tuple_$(i) = &[$(felt_strings.join(", "))];
                    let numerator_$(i) = $(numerator);
                    acc.add_to_relation(context, numerator_$(i), tuple_$(i));
                });
            }
        }
    }

    // Add return value
    if air_fn.r#type == TraceType::Inline {
        code.append(quote! {
            vec!$(make_var_for_expr(&air_fn.verifier_output.0))
        });
    }
    code
}

fn input_names(air_fn: &CompiledAirFn) -> Vec<String> {
    if air_fn.r#type == TraceType::Inline {
        chain!(
            air_fn.verifier_input_limbs.clone(),
            ["enabler".to_string()],
            air_fn.state_names.clone()
        )
        .collect()
    } else {
        air_fn.state_names.clone()
    }
}

fn make_var_for_expr(expr: &CompiledAirVar) -> rust::Tokens {
    if let CompiledAirVar::StaticCall(fn_name, args) = expr {
        // StaticCall generates a function call
        assert!(fn_name.ends_with(CONSTRAINT_EVAL_FUNCTION_NAME));
        let called_air_fn_name =
            fn_name.trim_end_matches(&format!("::{CONSTRAINT_EVAL_FUNCTION_NAME}"));
        let CompiledAirVar::Array(first_arg_vars) = args[0].clone() else {
            panic!("Expected an array as StaticCall first argument. Got {:?}", args[0])
        };
        let mut input_strings = first_arg_vars
            .iter()
            .chain(args[1..].iter())
            .map(|arg| make_var_for_expr(arg).to_string().unwrap());
        quote! { $(called_air_fn_name)::accumulate_constraints(&[$(input_strings.join(", "))], context, component_data, acc) }
    } else if let CompiledAirVar::Array(vars) = expr {
        let mut var_strings = vars.iter().map(|var| make_var_for_expr(var).to_string().unwrap());
        quote! { [$(var_strings.join(", "))] }
    } else {
        // All the rest generate an eval!(...) call
        quote! { eval!(context, $(make_eval_body_for_expr(expr))) }
    }
}

/// Create something that can be passed to the eval!(...) macro in stwo-circuits
/// to create a Var for the given expression.
fn make_eval_body_for_expr(expr: &CompiledAirVar) -> rust::Tokens {
    match expr {
        CompiledAirVar::Const(_ty, val) => quote! { $(val) },
        CompiledAirVar::Var(_ty, id) => quote! { $(id) },
        CompiledAirVar::State(name) => quote! { $(name) },
        CompiledAirVar::StaticCall(..) => {
            panic!("StaticCall is only supported at the top level of an expression")
        }
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            let lhs_eval = make_eval_body_for_expr(lhs);
            let rhs_eval = make_eval_body_for_expr(rhs);
            quote! { ($(lhs_eval)) $(op) ($(rhs_eval)) }
        }
        CompiledAirVar::UnaryOp(op, rhs) => {
            let rhs_eval = make_eval_body_for_expr(rhs);
            quote! { $(op) ($(rhs_eval)) }
        }
        CompiledAirVar::Tuple(_vars) => todo!(),
        CompiledAirVar::Array(_vars) => todo!(),
        CompiledAirVar::ExternalState(col_id) => quote! { $(col_id.to_lowercase()) },
        CompiledAirVar::PublicParam(param_name) => quote! { $(param_name) },
        CompiledAirVar::Enabler => quote! { enabler },
        CompiledAirVar::Struct { .. }
        | CompiledAirVar::MethodCall(..)
        | CompiledAirVar::Multiplicity(_) => {
            panic!("Unsupported expression in constraint evaluation: {expr}")
        }
    }
}

fn get_constraint_atoms(air_fn: &CompiledAirFn) -> HashSet<CompiledAirVar> {
    let mut result = HashSet::new();

    let mut insert = |expr: &CompiledAirVar| {
        result.insert(expr.clone());
    };

    for step in air_fn.constraints.iter() {
        match step {
            ConstraintEvalStep::Constraint(compiled_air_var, _) => {
                expr_iterator(compiled_air_var, &mut insert)
            }
            ConstraintEvalStep::LookupTerm(lookup_term) => {
                lookup_term.felts.iter().for_each(|f| expr_iterator(f, &mut insert));
                expr_iterator(&lookup_term.multiplicity, &mut insert);
            }
            ConstraintEvalStep::Intermediate(compiled_constraint_intermediate) => {
                expr_iterator(&compiled_constraint_intermediate.var, &mut insert)
            }
        }
    }
    result
}

fn gen_tests_module(air_fn: &CompiledAirFn, assignment: &Assignment) -> rust::Tokens {
    let trace_values =
        assignment.base_trace.iter().map(|v| make_qm31(*v).to_string().unwrap()).collect_vec();
    let interaction_values = assignment
        .interaction_trace
        .iter()
        .map(|v| make_qm31(*v).to_string().unwrap())
        .collect_vec();
    let preprocessed_columns = air_fn
        .external_states
        .iter()
        .map(|col| {
            let col_id = if col == "Seq" {
                &format!("seq_{}", assignment.log_height)
            } else { col };
            let value = assignment.environment.external_states.get(col).unwrap();
            quote! { (PreProcessedColumnId { id: $(quoted(col_id)).to_owned() }, context.constant($(make_qm31(*value)))) }
                .to_string()
                .unwrap()
        })
        .join(",\n");
    let public_params = air_fn
        .public_params
        .iter()
        .map(|param| {
            let value = assignment.environment.public_params.get(param).unwrap();
            let tokens: rust::Tokens =
                quote! { ($(quoted(param)).to_owned(), context.constant($(value.0).into())) };
            tokens.to_string().unwrap()
        })
        .join(",\n");
    let expected_result_name =
        format!("{}{}", air_fn.name.to_case(Case::UpperSnake), SAMPLE_EVALUATION_RESULT_SUFFIX);
    quote! {
        #[cfg(test)]
        mod tests {
            use std::collections::HashMap;
            use stwo::core::fields::qm31::QM31;

            #[allow(unused_imports)]
            use crate::components::prelude::PreProcessedColumnId;
            use crate::sample_evaluations::*;
            use circuits::context::Context;
            use circuits::ivalue::qm31_from_u32s;
            use circuits_stark_verifier::constraint_eval::*;
            use circuits_stark_verifier::test_utils::TestComponentData;

            use super::Component;

            #[test]
            fn test_evaluation_result() {
                let component = Component {};
                let mut context: Context::<QM31> = Default::default();
                context.enable_assert_eq_on_eval();
                let trace_columns = [
                    $(trace_values.join(",\n"))
                ];
                let interaction_columns = [
                    $(interaction_values.join(",\n"))
                ];
                let component_data = TestComponentData::from_values(
                    &mut context,
                    &trace_columns,
                    &interaction_columns,
                    $(make_qm31(assignment.last_row_sum)),
                    $(1 << assignment.log_height)
                );
                let random_coeff = context.new_var($(make_qm31(assignment.random_coeff)));
                let interaction_elements = [
                    context.new_var($(make_qm31(assignment.common_lookup_elements.z))),
                    context.new_var($(make_qm31(assignment.common_lookup_elements.alpha))),
                ];
                let preprocessed_columns = HashMap::from([$(preprocessed_columns)]);
                let public_params = HashMap::from([$(public_params)]);
                let mut accumulator = CompositionConstraintAccumulator::new(&mut context, preprocessed_columns, public_params, random_coeff, interaction_elements);
                component.evaluate(&mut context, &component_data, &mut accumulator);
                let claimed_sum = context.new_var($(make_qm31(assignment.claimed_sum)));
                accumulator.finalize_logup_in_pairs(
                    &mut context,
                    <TestComponentData as ComponentDataTrait<QM31>>::interaction_columns(&component_data),
                    &component_data,
                    claimed_sum,
                );

                let result = accumulator.finalize();
                let result_value = context.get(result);
                assert_eq!(result_value, $(expected_result_name))
            }
        }
    }
}

fn make_qm31(value: QM31) -> rust::Tokens {
    let m31s = value.to_m31_array();

    quote! { qm31_from_u32s($(m31s[0].0), $(m31s[1].0), $(m31s[2].0), $(m31s[3].0)) }
}
