use compiled_casm_air::compiled_structs::*;
use compiled_casm_air::utils::CONSTRAINT_EVAL_FUNCTION_NAME;
use genco::lang::{rust, Rust};
use genco::{quote, Tokens};
use itertools::{chain, Itertools};

use crate::components_code_gen::constraints::generate_relation_uses;
use crate::utils::{
    make_preprocessed_column_id, relation_multiplicity_index, remove_trailing_zeroes,
};

pub fn generate_circuit_constraints_code(air_fn: &CompiledAirFn) -> Tokens<Rust> {
    let return_type = if air_fn.r#type == TraceType::Inline {
        quote! { -> Vec<Var> }
    } else {
        quote! {}
    };

    let mut code = quote! {
        use crate::cairo_air::components::prelude::*;
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
        pub fn accumulate_constraints(
            input: &[Var],
            context: &mut Context<impl IValue>,
            component_data: &ComponentData<'_>,
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
        code.append(quote! {

            pub struct Component {}
            impl<Value: IValue> CircuitEval<Value> for Component {
                fn evaluate(
                    &self,
                    context: &mut Context<Value>,
                    component_data: &ComponentData<'_>,
                    acc: &mut CompositionConstraintAccumulator,
                ) {
                    accumulate_constraints(component_data.trace_columns, context, component_data, acc);
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
            }
        })
    }
    code
}

fn generate_accumulate_constraints(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    let input_names = input_names(air_fn);

    // Mark component_data as used
    code.append(quote! {
        let _ = component_data;
    });

    // Unpack input
    code.append(quote! {
        let [$(input_names.join(",\n"))] = input.try_into().unwrap();$("\n")
    });

    // Read preprocessed columns
    for external_col_id in &air_fn.external_states {
        // Seq is the only preprocessed column that is of unfixed size.
        if external_col_id == "Seq" {
            code.append(quote! {
                let seq = seq_of_component_size(context, component_data, acc);
            });
        } else {
            code.append(quote! {
                let $(external_col_id) = acc.get_preprocessed_column(&$(make_preprocessed_column_id(external_col_id)));
            });
        }
    }

    // Evaluate enabler constraint
    if air_fn.padding_type == PaddingType::Enabler {
        code.append(quote! {
            let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
            acc.add_constraint(context, enabler_constraint_value);$("\n")
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
                    assert_eq!(felt_names.len(), 1, "In constraints, only StaticCalls are allowed to produce multiple-felt outputs");
                    code.append(quote! {
                        let $(&felt_names[0]) = $(make_var_for_expr(var));
                    });
                }
            }
            ConstraintEvalStep::LookupTerm(LookupTerm {
                relation_name,
                felts,
                use_or_yield,
            }) => {
                let felts = remove_trailing_zeroes(felts);
                let mut felt_strings = felts
                    .iter()
                    .map(|f| make_var_for_expr(f).to_string().unwrap());

                let is_masked = relation_multiplicity_index(air_fn, relation_name);
                let numerator = match air_fn.padding_type {
                    PaddingType::Enabler if is_masked.is_some() => quote! { enabler },
                    PaddingType::Multiplicity if is_masked.is_some() => {
                        quote! { multiplicity_$(is_masked.unwrap()) }
                    }
                    _ => quote! { 1 },
                };

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
            air_fn.state_names.clone()
        )
        .collect()
    } else {
        let enabler_and_multiplicity_columns = match air_fn.padding_type {
            PaddingType::Enabler => vec!["enabler".to_owned()],
            PaddingType::Multiplicity => (0..air_fn.relation_names.len())
                .map(|i| format!("multiplicity_{i}"))
                .collect(),
            PaddingType::None => vec![],
        };
        chain!(air_fn.state_names.clone(), enabler_and_multiplicity_columns).collect()
    }
}

fn make_var_for_expr(expr: &CompiledAirVar) -> rust::Tokens {
    if let CompiledAirVar::StaticCall(fn_name, args) = expr {
        // StaticCall generates a function call
        assert!(fn_name.ends_with(CONSTRAINT_EVAL_FUNCTION_NAME));
        let called_air_fn_name =
            fn_name.trim_end_matches(&format!("::{CONSTRAINT_EVAL_FUNCTION_NAME}"));
        let CompiledAirVar::Array(first_arg_vars) = args[0].clone() else {
            panic!(
                "Expected an array as StaticCall first argument. Got {:?}",
                args[0]
            )
        };
        let mut input_strings = first_arg_vars
            .iter()
            .chain(args[1..].iter())
            .map(|arg| make_var_for_expr(arg).to_string().unwrap());
        quote! { $(called_air_fn_name)::accumulate_constraints(&[$(input_strings.join(", "))], context, component_data, acc) }
    } else if let CompiledAirVar::Array(vars) = expr {
        let mut var_strings = vars
            .iter()
            .map(|var| make_var_for_expr(var).to_string().unwrap());
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
        CompiledAirVar::PublicParam(_) => todo!(),
        CompiledAirVar::Struct { .. } | CompiledAirVar::MethodCall(..) => {
            panic!("Unsupported expression in constraint evaluation: {expr}")
        }
    }
}
