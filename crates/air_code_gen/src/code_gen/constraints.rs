use std::collections::HashMap;

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, CompiledConstraintIntermediate, ConstraintEvalStep, LookupTerm,
    PaddingType, TraceType, UseOrYield,
};
use compiled_casm_air::utils::CONSTRAINT_EVAL_FUNCTION_NAME;
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::{chain, Itertools};

use super::parse::{
    constraint_consts, parse_eval_constraint, parse_lookup_constraint, seek_consts,
};
use super::utils::{get_variable_name, relations_used_or_yielded, replace_generics_with_turbofish};
use crate::code_gen::utils::{is_const_size_component, make_preprocessed_column_id};

/// Generate constraints evaluation code for an AirFn that is not called from other AirFns
pub fn generate_toplevel_constraints_code(air_fn: &CompiledAirFn) -> rust::Tokens {
    quote! {
        $(imports(air_fn))
        $['\n']
        $(generate_consts(air_fn))
        $['\n']
        $(generate_component_structs(air_fn))
        $['\n']
        $(generate_claim_struct(air_fn))
        $['\n']
        $(generate_interaction_claim_struct())
        $['\n']
        $(generate_component_type_def())
        $['\n']
        $(generate_framework_impl(air_fn))
        $['\n']
        $(generate_tests(air_fn))
    }
}

pub fn generate_tests(air_fn: &CompiledAirFn) -> rust::Tokens {
    let log_size = if is_const_size_component(air_fn) {
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
            use crate::components::constraints_regression_test_values::$(air_fn.name.to_case(Case::UpperSnake));

            #[test]
            fn $(air_fn.name.clone())_constraints_regression() {
                let mut rng = SmallRng::seed_from_u64(0);
                let eval = Eval {
                    claim: Claim {
                        $log_size
                        $(get_dummy_public_params(air_fn))
                    },
                    $(get_dummy_lookup_elements(air_fn))
                };
                let expr_eval = eval.evaluate(ExprEvaluator::new());
                let assignment = expr_eval.random_assignment();

                let mut sum = QM31::zero();
                for c in expr_eval.constraints {
                    sum += c.assign(&assignment) * rng.gen::<QM31>();
                }

                $(air_fn.name.to_case(Case::UpperSnake)).assert_debug_eq(&sum);
            }
        }
    }
}

fn get_dummy_lookup_elements(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for relation in relations_used_or_yielded(air_fn) {
        code.append(quote! {
            $(relation.to_case(Case::Snake))_lookup_elements: relations::$(relation)::dummy(),
        });
    }
    code
}

fn get_dummy_public_params(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for param in &air_fn.public_params {
        code.append(quote! {
         $(param.name()): rng.gen::<u32>(),
        });
    }
    code
}

/// Generate constraints evaluation code for an inline AirFn (AirFn that is only called from
/// other AirFns)
pub fn generate_inline_constraints_code(air_fn: &CompiledAirFn) -> rust::Tokens {
    let CompiledAirVar::Array(ref output_array) = air_fn.verifier_output.0 else {
        panic!("Verifier output is not array in {}", &air_fn.name)
    };
    let name = air_fn.name.to_case(Case::Pascal);
    let input_name = format!("[{}]", air_fn.verifier_input_limbs.join(", "));
    let input_type = format!("[E::F; {}]", air_fn.verifier_input_limbs.len());
    let output_type = format!("[E::F; {}]", output_array.len());

    // TODO(AnatG): Find a way to remove <#[allow(unused_variables)]> below.
    quote! {
        $(imports(air_fn))
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
                    $(get_inline_args(air_fn))
                    eval: &mut E,
            ) -> $(output_type)
            {
                $(generate_evaluate(air_fn))
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

fn get_inline_args(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for state_name in &air_fn.state_names {
        code.append(quote! {
            $(state_name): E::F,
        });
    }
    for relation in relations_used_or_yielded(air_fn) {
        code.append(quote! {
            $(relation.to_case(Case::Snake))_lookup_elements: &relations::$(relation),
        });
    }
    for param in &air_fn.public_params {
        code.append(quote! {
            $(param.name()): E::F,
        });
    }
    for external_col_id in &air_fn.external_states {
        code.append(quote! {
            $(external_col_id.to_lowercase()): E::F,
        });
    }
    code
}

fn imports(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut res = rust::Tokens::new();
    res.append(quote! {
        use crate::components::prelude::*;
    });
    for (inline_fn, _) in &air_fn.inline_calls {
        res.append(quote! {
            use crate::components::subroutines::$(inline_fn)::$(inline_fn.to_case(Case::Pascal));
        });
    }
    res
}

fn generate_consts(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut consts = match air_fn.padding_type {
        PaddingType::Enabler => {
            // Add a padding column to the trace
            quote! {
                pub const N_TRACE_COLUMNS: usize = $(air_fn.state_names.len() + 1);
            }
        }
        PaddingType::Multiplicity => {
            // Add multiplicity columns to the trace
            quote! {
                pub const N_TRACE_COLUMNS: usize = $(air_fn.state_names.len() + air_fn.relation_names.len());
            }
        }
        _ => quote! {
            pub const N_TRACE_COLUMNS: usize = $(air_fn.state_names.len());
        },
    };
    if is_const_size_component(air_fn) {
        consts.extend(quote! {
            pub const LOG_SIZE: u32 = $(air_fn.log_height.unwrap());
        });
    }

    consts.extend(generate_relation_uses(air_fn));

    consts
}

/// Counts the number of times each relation is used (not including yield) in the component, for
/// each row.
fn generate_relation_uses(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut relation_use_count = HashMap::new();
    for (relation_name, use_or_yield) in &air_fn.constraint_lookups {
        if *use_or_yield == UseOrYield::Use {
            let offset = relation_use_count.entry(relation_name.clone()).or_insert(0);
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
                relation_id: $("\"")$(relation)$("\""),
                uses: $(*uses),
            },
        });
    }
    quote! {
        pub const RELATION_USES_PER_ROW: [RelationUse; $(relation_use_count.len())] = [$(code)];
    }
}

fn generate_component_structs(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut members = rust::Tokens::new();

    // Claims.
    members.append(quote! {
        pub claim: Claim,
    });

    // Sub-components Lookup elements.
    for relation in relations_used_or_yielded(air_fn) {
        members.append(quote! {
            pub $(lookup_elements_field_name(&relation)): relations::$(relation),
        });
    }

    quote! {
        pub struct Eval {
            $(members)
        }
    }
}

fn lookup_elements_field_name(relation_name: &str) -> String {
    format!("{}_lookup_elements", relation_name.to_case(Case::Snake))
}

fn generate_claim_struct(air_fn: &CompiledAirFn) -> rust::Tokens {
    let log_size = if is_const_size_component(air_fn) {
        quote! { LOG_SIZE }
    } else {
        quote! { self.log_size }
    };

    let mut channel_mix_code = quote! {};
    if !is_const_size_component(air_fn) {
        channel_mix_code.append(quote! { channel.mix_u64($(&log_size) as u64); });
    }
    for public_param in &air_fn.public_params {
        channel_mix_code.append(quote! {
            channel.mix_u64(self.$(public_param.name()) as u64);
        });
    }

    let struct_code = quote! {
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize, CairoDeserialize)]
        pub struct Claim {
            $(get_claim_members(air_fn))
        }
    };

    // Clippy wants us to prefix the parameter name with a '_' iff it is unused
    let channel_param = if is_const_size_component(air_fn) {
        quote! { _channel }
    } else {
        quote! { channel }
    };

    let impl_code = quote! {
        impl Claim {
            pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
                let trace_log_sizes = vec![$(&log_size); N_TRACE_COLUMNS];
                let interaction_log_sizes = vec![$(&log_size); $(get_n_logup_columns(air_fn))];
                TreeVec::new(vec![
                    vec![],
                    trace_log_sizes,
                    interaction_log_sizes,
                ])
            }
             // TODO(Ohad): better mix_into.
            pub fn mix_into(&self, $(channel_param): &mut impl Channel) {
                $(channel_mix_code)
            }
        }
    };

    chain!(struct_code, impl_code).collect()
}

pub fn get_n_logup_columns(air_fn: &CompiledAirFn) -> rust::Tokens {
    let n_lookup_terms: usize = air_fn.constraint_lookups.len();
    match n_lookup_terms {
        0 => unimplemented!(),
        1..=2 => quote!(SECURE_EXTENSION_DEGREE),
        n => {
            let n_batches = n.div_ceil(2);
            quote!(SECURE_EXTENSION_DEGREE * $(n_batches))
        }
    }
}

pub fn get_claim_members(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut members = quote! {};
    if !is_const_size_component(air_fn) {
        members.append(quote! { pub log_size: u32, });
    };

    for public_param in &air_fn.public_params {
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

fn generate_framework_impl(air_fn: &CompiledAirFn) -> rust::Tokens {
    let log_size = if is_const_size_component(air_fn) {
        quote! { LOG_SIZE }
    } else {
        quote! { self.claim.log_size }
    };

    quote! {
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
                $(generate_evaluate(air_fn))
            }
        }
    }
}

fn generate_evaluate(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    // Constants.
    let mut constants = constraint_consts(&air_fn.constraints);
    if air_fn.r#type == TraceType::Inline {
        constants.extend(seek_consts(&air_fn.verifier_output.0));
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

    if air_fn.r#type != TraceType::Inline {
        for external_col_id in &air_fn.external_states {
            // Seq is the only preprocessed column that is of unfixed size.
            if external_col_id == "Seq" {
                code.append(quote! {
                    let seq = eval.get_preprocessed_column(Seq::new(self.log_size()).id());
                });
            } else {
                code.append(quote! {
                    let $(external_col_id) = eval.get_preprocessed_column($(make_preprocessed_column_id(external_col_id)));
                });
            }
        }
    }

    if air_fn.r#type != TraceType::Inline && !air_fn.state_names.is_empty() {
        for name in &air_fn.state_names {
            code.append(quote! {
                let $name = eval.next_trace_mask();
            });
        }
    }

    match air_fn.padding_type {
        PaddingType::Enabler => {
            // Add enabler column to the trace
            code.append(quote! {
                let enabler = eval.next_trace_mask();
                // Check enabler column is a bit.
                eval.add_constraint(enabler.clone() * enabler.clone() - enabler.clone());
            });
        }
        PaddingType::Multiplicity => {
            // Add multiplicity columns to the trace
            for i in 0..air_fn.relation_names.len() {
                code.append(quote! { let multiplicity_$i = eval.next_trace_mask();});
            }
        }
        _ => {}
    }

    code.extend(quote! { $("\n\n") });

    for constraint in air_fn.constraints.iter() {
        match constraint {
            ConstraintEvalStep::Constraint(expr, desc) => {
                if let Some(desc) = desc {
                    code.append(quote! {
                        $("//")$desc.$("\n")
                    });
                }
                code.extend(quote! {
                    eval.add_constraint(
                        $(parse_eval_constraint(air_fn, expr,&const_names))
                    );
                });
            }
            ConstraintEvalStep::Intermediate(CompiledConstraintIntermediate {
                felt_names,
                var,
            }) => {
                if felt_names.is_empty() {
                    code.extend(quote! {
                        $(parse_eval_constraint(air_fn, var, &const_names));
                    });
                } else if let CompiledAirVar::StaticCall(fn_name, _) = var {
                    if air_fn.r#type != TraceType::Inline {
                        // TODO(AnatG): Consider adding to StaticCall a predicate.
                        if fn_name.ends_with(CONSTRAINT_EVAL_FUNCTION_NAME) {
                            code.extend(quote! {
                                #[allow(clippy::unused_unit)]
                                #[allow(unused_variables)]
                            });
                        }
                    }
                    code.extend(quote! {
                        let [$(felt_names.join(", "))] = $(parse_eval_constraint(air_fn, var, &const_names));
                    });
                } else {
                    assert_eq!(felt_names.len(), 1, "In constraints, only StaticCalls are allowed to produce multiple-felt outputs");
                    code.extend(quote! {
                        let $(&felt_names[0]) = eval.add_intermediate($(parse_eval_constraint(air_fn, var, &const_names)));
                    });
                }
            }
            ConstraintEvalStep::LookupTerm(LookupTerm {
                relation_name,
                felts,
                use_or_yield,
            }) => {
                code.extend(parse_lookup_constraint(
                    air_fn,
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
    if air_fn.r#type == TraceType::Inline {
        code.extend(quote! {

            $(parse_eval_constraint(air_fn, &air_fn.verifier_output.0, &const_names))
        });
    } else {
        code.extend(quote! {

            eval.finalize_logup_in_pairs();
            eval
        });
    }
    code
}
