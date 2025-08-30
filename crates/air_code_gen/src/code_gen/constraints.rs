use std::collections::HashMap;

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, CompiledConstraintIntermediate, ConstraintEvalStep,
    ExternalState, LookupTerm, PaddingType, TraceType, UseOrYield,
};
use compiled_casm_air::utils::CONSTRAINT_EVAL_FUNCTION_NAME;
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use indexmap::IndexSet;
use itertools::{chain, Itertools};

use super::parse::{
    constraint_consts, is_const_size_component, parse_eval_constraint, parse_lookup_constraint,
    seek_consts,
};
use super::utils::{filter_lookup_terms, get_variable_name, replace_generics_with_turbofish};

/// Generate constraints evaluation code for an AirFn that is not called from other AirFns
pub fn generate_toplevel_constraints_code(lists: &CompiledAirFn) -> rust::Tokens {
    quote! {
        $(imports(lists))
        $['\n']
        $(generate_consts(lists))
        $['\n']
        $(generate_component_structs(lists))
        $['\n']
        $(generate_claim_struct(lists))
        $['\n']
        $(generate_interaction_claim_struct())
        $['\n']
        $(generate_component_type_def())
        $['\n']
        $(generate_framework_impl(lists))
        $['\n']
        $(generate_tests(lists))
    }
}

pub fn generate_tests(lists: &CompiledAirFn) -> rust::Tokens {
    let log_size = if is_const_size_component(lists) {
        quote! {}
    } else {
        quote! {log_size: 4,}
    };

    quote! {
        #[cfg(test)]
        mod tests {
            use num_traits::Zero;
            use rand::rngs::SmallRng;
            use rand::{Rng, SeedableRng};
            use stwo_constraint_framework::expr::ExprEvaluator;
            use stwo::core::fields::qm31::QM31;

            use super::*;
            use crate::components::constraints_regression_test_values::$(lists.name.to_case(Case::UpperSnake));

            #[test]
            fn $(lists.name.clone())_constraints_regression() {
                let mut rng = SmallRng::seed_from_u64(0);
                let eval = Eval {
                    claim: Claim {
                        $log_size
                        $(get_dummy_public_params(lists))
                    },
                    $(get_dummy_lookup_elements(lists))
                };
                let expr_eval = eval.evaluate(ExprEvaluator::new());
                let assignment = expr_eval.random_assignment();

                let mut sum = QM31::zero();
                for c in expr_eval.constraints {
                    sum += c.assign(&assignment) * rng.gen::<QM31>();
                }

                assert_eq!(sum, $(lists.name.to_case(Case::UpperSnake)));
            }
        }
    }
}

fn get_dummy_lookup_elements(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for relation in lists
        .constraint_lookups
        .iter()
        .map(|(r, _)| r)
        .collect::<IndexSet<_>>()
    {
        code.append(quote! {
            $(relation.to_case(Case::Snake))_lookup_elements: relations::$(relation)::dummy(),
        });
    }
    code
}

fn get_dummy_public_params(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for param in &lists.public_params {
        code.append(quote! {
         $(param.name()): rng.gen::<u32>(),
        });
    }
    code
}

/// Generate constraints evaluation code for an inline AirFn (AirFn that is only called from
/// other AirFns)
pub fn generate_inline_constraints_code(lists: &CompiledAirFn) -> rust::Tokens {
    let CompiledAirVar::Array(ref output_array) = lists.verifier_output.0 else {
        panic!("Verifier output is not array in {}", &lists.name)
    };
    let name = lists.name.to_case(Case::Pascal);
    let input_name = format!("[{}]", lists.verifier_input_limbs.join(", "));
    let input_type = format!("[E::F; {}]", lists.verifier_input_limbs.len());
    let output_type = format!("[E::F; {}]", output_array.len());

    // TODO(AnatG): Find a way to remove <#[allow(unused_variables)]> below.
    quote! {
        $(imports(lists))
        $['\n']
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
        pub struct $(name.clone()) {}
        $['\n']
        impl $(name) {
            #[allow(unused_parens)]
            #[allow(clippy::double_parens)]
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            #[allow(unused_variables)]
            #[allow(clippy::too_many_arguments)]
                pub fn evaluate<E: EvalAtRow>(
                    $(input_name.clone()): $(input_type.clone()),
                    $(get_inline_args(lists))
                    eval: &mut E,
            ) -> $(output_type)
            {
                $(generate_evaluate(lists))
            }
        }
    }
}

pub fn generate_constraints_code(air_fn: &CompiledAirFn) -> rust::Tokens {
    match air_fn.r#type {
        TraceType::Inline => generate_inline_constraints_code(air_fn),
        _ => generate_toplevel_constraints_code(air_fn),
    }
}

fn get_inline_args(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for state_name in &lists.state_names {
        code.append(quote! {
            $(state_name): E::F,
        });
    }
    for relation in lists
        .constraint_lookups
        .iter()
        .map(|(r, _)| r)
        .collect::<IndexSet<_>>()
    {
        code.append(quote! {
            $(relation.to_case(Case::Snake))_lookup_elements: &relations::$(relation),
        });
    }
    for param in &lists.public_params {
        code.append(quote! {
            $(param.name()): E::F,
        });
    }
    for ExternalState {
        name,
        generic_param: _,
        args,
    } in &lists.external_states
    {
        if name == "Seq" {
            code.append(quote! {
                seq: E::F,
            });
        } else {
            code.append(quote! {
                $(get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())): E::F,
            });
        }
    }
    code
}

fn imports(lists: &CompiledAirFn) -> rust::Tokens {
    let mut res = rust::Tokens::new();
    res.append(quote! {
        use crate::components::prelude::*;
    });
    for (inline_fn, _) in &lists.inline_calls {
        res.append(quote! {
            use crate::components::subroutines::$(inline_fn)::$(inline_fn.to_case(Case::Pascal));
        });
    }
    res
}

fn generate_consts(lists: &CompiledAirFn) -> rust::Tokens {
    let mut consts = match lists.padding_type {
        PaddingType::Enabler | PaddingType::Multiplicity => {
            // Add a padding column to the trace
            quote! {
                pub const N_TRACE_COLUMNS: usize = $(lists.state_names.len() + 1);
            }
        }
        _ => quote! {
            pub const N_TRACE_COLUMNS: usize = $(lists.state_names.len());
        },
    };
    if is_const_size_component(lists) {
        consts.extend(quote! {
            pub const LOG_SIZE: u32 = 4; // Implement manually, set to 4 initially so LOG_SIZE - LOG_N_LANES >= 0.
        });
    }

    consts.extend(generate_relation_uses(lists));

    consts
}

/// Counts the number of times each relation is used (not including yield) in the component, for
/// each row.
fn generate_relation_uses(lists: &CompiledAirFn) -> rust::Tokens {
    let mut relation_use_count = HashMap::new();
    for LookupTerm {
        relation_name,
        use_or_yield,
        ..
    } in filter_lookup_terms(&lists.deductions)
    {
        if use_or_yield == UseOrYield::Use {
            let offset = relation_use_count.entry(relation_name).or_insert(0);
            *offset += 1;
        }
    }

    let mut code = rust::Tokens::new();
    for (relation, uses) in relation_use_count
        .iter()
        .sorted_by_key(|(relation, _)| *relation)
    {
        code.append(quote! {
            RelationUse {
                relation_id: $("\"")$(relation.clone())$("\""),
                uses: $(*uses),
            },
        });
    }
    quote! {
        pub const RELATION_USES_PER_ROW: [RelationUse; $(relation_use_count.len())] = [$(code)];
    }
}

fn generate_component_structs(lists: &CompiledAirFn) -> rust::Tokens {
    let mut members = rust::Tokens::new();

    // Claims.
    members.append(quote! {
        pub claim: Claim,
    });

    // Sub-components Lookup elements.
    for relation in lists
        .constraint_lookups
        .iter()
        .map(|(r, _)| r)
        .collect::<IndexSet<_>>()
    {
        members.append(quote! {
            pub $(&relation.to_case(Case::Snake))_lookup_elements: relations::$(relation),
        });
    }

    quote! {
        pub struct Eval {
            $(members)
        }
    }
}

fn generate_claim_struct(lists: &CompiledAirFn) -> rust::Tokens {
    let log_size = if is_const_size_component(lists) {
        quote! { LOG_SIZE }
    } else {
        quote! { self.log_size }
    };

    let mut channel_mix_code = quote! { channel.mix_u64($(&log_size) as u64); };
    for public_param in &lists.public_params {
        channel_mix_code.append(quote! {
            channel.mix_u64(self.$(public_param.name()) as u64);
        });
    }

    let struct_code = quote! {
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize, CairoDeserialize)]
        pub struct Claim {
            $(get_claim_members(lists))
        }
    };

    let impl_code = quote! {
        impl Claim {
            pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
                let trace_log_sizes = vec![$(&log_size); N_TRACE_COLUMNS];
                let interaction_log_sizes = vec![$(&log_size); $(get_n_logup_columns(lists))];
                TreeVec::new(vec![
                    vec![],
                    trace_log_sizes,
                    interaction_log_sizes,
                ])
            }
             // TODO(Ohad): better mix_into.
            pub fn mix_into(&self, channel: &mut impl Channel) {
                $(channel_mix_code)
            }
        }
    };

    chain!(struct_code, impl_code).collect()
}

pub fn get_n_logup_columns(lists: &CompiledAirFn) -> rust::Tokens {
    let n_lookup_terms: usize = lists.constraint_lookups.len();
    match n_lookup_terms {
        0 => unimplemented!(),
        1..=2 => quote!(SECURE_EXTENSION_DEGREE),
        n => {
            let n_batches = n.div_ceil(2);
            quote!(SECURE_EXTENSION_DEGREE * $(n_batches))
        }
    }
}

pub fn get_claim_members(lists: &CompiledAirFn) -> rust::Tokens {
    let mut members = quote! {};
    if !is_const_size_component(lists) {
        members.append(quote! { pub log_size: u32, });
    };

    for public_param in &lists.public_params {
        members.append(quote! {
            pub $(public_param.name()): u32,
        });
    }
    members
}

fn generate_interaction_claim_struct() -> rust::Tokens {
    let struct_code = quote! {
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize, CairoDeserialize)]
        pub struct InteractionClaim {
            pub claimed_sum: SecureField,
        }
    };
    let mut impl_code = rust::Tokens::new();
    impl_code.append(quote! {
        impl InteractionClaim {
            pub fn mix_into(&self, channel: &mut impl Channel) {
                channel.mix_felts(&[self.claimed_sum]);
            }
        }
    });

    chain!(struct_code, impl_code).collect()
}

fn generate_component_type_def() -> rust::Tokens {
    quote! {
        pub type Component = FrameworkComponent<Eval>;
    }
}

fn generate_framework_impl(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    let log_size = if is_const_size_component(lists) {
        quote! { LOG_SIZE }
    } else {
        quote! { self.claim.log_size }
    };
    code.append(quote! {
        impl FrameworkEval for Eval {
            fn log_size(&self) -> u32 {
                $log_size
            }

            fn max_constraint_log_degree_bound(&self) -> u32 {
                // TODO(Ohad): determine dynamically.
                self.log_size() + 1
            }

            #[allow(unused_parens)]
            #[allow(clippy::double_parens)]
            #[allow(non_snake_case)]
            fn evaluate<E: EvalAtRow>(&self, mut eval:E) -> E{
                $(generate_evaluate(lists))
            }
        }
    });
    code
}

fn generate_evaluate(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    // Constants.
    let mut constants = constraint_consts(&lists.constraints);
    if lists.r#type == TraceType::Inline {
        constants.extend(seek_consts(&lists.verifier_output.0));
    }
    let mut const_names = HashMap::new();
    for (ty, val) in constants.into_iter() {
        let name = get_variable_name(&ty, &val);
        const_names.insert((ty.clone(), val.clone()), name.clone());
        if ty == "M31" {
            code.append(quote! {
                let $(name) = E::F::from($(replace_generics_with_turbofish(&ty))::from($(val)));
            });
        } else {
            code.append(quote! {
                let $(name) = $(replace_generics_with_turbofish(&ty))::from($(val));
            });
        }
    }

    if lists.r#type != TraceType::Inline {
        for ExternalState {
            name,
            generic_param: _,
            args,
        } in &lists.external_states
        {
            // Seq is the only preprocessed column that is of unfixed size.
            if name == "Seq" {
                code.append(quote! {
                    let seq = eval.get_preprocessed_column(Seq::new(self.log_size()).id());
                });
            } else {
                code.append(quote! {
                    let $(&get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())) = eval.get_preprocessed_column(($name::new($(args.join(", ")))).id());
                });
            }
        }
    }

    if lists.r#type != TraceType::Inline && !lists.state_names.is_empty() {
        for name in &lists.state_names {
            code.append(quote! {
                let $name = eval.next_trace_mask();
            });
        }
    }

    match lists.padding_type {
        PaddingType::Enabler => {
            // Add enabler column to the trace
            code.append(quote! {
                let enabler = eval.next_trace_mask();
                // Check enabler column is a bit.
                eval.add_constraint(enabler.clone() * enabler.clone() - enabler.clone());
            });
        }
        PaddingType::Multiplicity => {
            // Add multiplicity column to the trace
            code.append(quote! { let multiplicity = eval.next_trace_mask();});
        }
        _ => {}
    }

    code.extend(quote! { $("\n\n") });

    for constraint in lists.constraints.iter() {
        match constraint {
            ConstraintEvalStep::Constraint(expr, desc) => {
                if let Some(desc) = desc {
                    code.append(quote! {
                        $("//")$desc.$("\n")
                    });
                }
                code.extend(quote! {
                    eval.add_constraint(
                        $(parse_eval_constraint(lists, expr,&const_names))
                    );
                });
            }
            ConstraintEvalStep::Intermediate(CompiledConstraintIntermediate {
                felt_names,
                var,
            }) => {
                if felt_names.is_empty() {
                    code.extend(quote! {
                        $(parse_eval_constraint(lists, var, &const_names));
                    });
                } else if let CompiledAirVar::StaticCall(fn_name, _) = var {
                    if lists.r#type != TraceType::Inline {
                        // TODO(AnatG): Consider adding to StaticCall a predicate.
                        if fn_name.ends_with(CONSTRAINT_EVAL_FUNCTION_NAME) {
                            code.extend(quote! {
                                #[allow(clippy::unused_unit)]
                                #[allow(unused_variables)]
                            });
                        }
                    }
                    code.extend(quote! {
                        let [$(felt_names.join(", "))] = $(parse_eval_constraint(lists, var, &const_names));
                    });
                } else {
                    assert_eq!(felt_names.len(), 1, "In constraints, only StaticCalls are allowed to produce multiple-felt outputs");
                    code.extend(quote! {
                        let $(&felt_names[0]) = eval.add_intermediate($(parse_eval_constraint(lists, var, &const_names)));
                    });
                }
            }
            // TODO(Ohad): implement.
            ConstraintEvalStep::LookupTerm(LookupTerm {
                relation_name,
                felts,
                use_or_yield,
            }) => {
                code.extend(parse_lookup_constraint(
                    lists,
                    relation_name,
                    felts,
                    use_or_yield,
                    &const_names,
                ));
            }
        }
        code.extend(quote! {
            $("\n")
        });
    }
    if lists.r#type == TraceType::Inline {
        code.extend(quote! {

            $(parse_eval_constraint(lists, &lists.verifier_output.0, &const_names))
        });
    } else {
        code.extend(quote! {

            eval.finalize_logup_in_pairs();
            eval
        });
    }
    code
}
