use std::collections::HashMap;

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledIntermediate, ConstraintEvalStep, LookupTerm, PaddingType, TraceType,
};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::chain;

use super::parse::seek_consts;
use super::utils::{block_doc, get_variable_name, replace_generics_with_turbofish};
use crate::code_gen::parse::{constraint_consts, parse_eval_constraint, parse_lookup_constraint};

pub fn generate_constraints_code(lists: &CompiledAirFn) -> rust::Tokens {
    quote! {
        $(imports())
        $['\n']
        $(generate_n_trace_columns(lists))
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

pub fn generate_tests(air_fn: &CompiledAirFn) -> rust::Tokens {
    quote! {
        #[cfg(test)]
        mod tests {
            use num_traits::Zero;
            use rand::rngs::SmallRng;
            use rand::{Rng, SeedableRng};
            use stwo_prover::constraint_framework::expr::ExprEvaluator;
            use stwo_prover::core::fields::qm31::QM31;

            use super::*;
            use crate::components::constraints_regression_test_values::$(air_fn.name.to_case(Case::UpperSnake));

            #[test]
            fn $(air_fn.name.clone())_constraints_regression() {
                let mut rng = SmallRng::seed_from_u64(0);
                let eval = Eval {
                    claim: Claim {
                        log_size: 4,
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

                assert_eq!(sum, $(air_fn.name.to_case(Case::UpperSnake)));
            }
        }
    }
}

fn get_dummy_lookup_elements(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for relation in air_fn.lookup_names.keys() {
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

pub fn generate_inline_code(lists: &CompiledAirFn) -> rust::Tokens {
    let name = lists.name.to_case(Case::Pascal);
    let input_name = lists.verifier_input.0.clone();
    let input_type = lists.verifier_input.1.clone();
    let output_type = lists.verifier_output.1.clone().replace("M31", "E::F");

    quote! {
        $(imports())
        $['\n']
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
        pub struct $(name.clone()) {}
        $['\n']
        impl $(name) {
            #[allow(unused_parens)]
            #[allow(clippy::double_parens)]
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
                pub fn evaluate<E: EvalAtRow>(
                    $(input_name.clone()): $(input_type.clone().replace("M31", "E::F")),
                    $(get_state_names(lists))
                    eval: &mut E,
                    $(get_lookup_elements(lists))
            ) -> $(output_type)
            {
                $(generate_evaluate(lists))
            }
        }
    }
}

fn get_lookup_elements(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for relation in lists.lookup_names.keys() {
        code.append(quote! {
            $(relation.to_case(Case::Snake))_lookup_elements: &relations::$(relation),
        });
    }
    code
}

fn get_state_names(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for state_name in &lists.state_names {
        code.append(quote! {
            $(state_name): E::F,
        });
    }
    code
}

fn imports() -> rust::Tokens {
    quote! {
        use crate::components::prelude::*;
    }
}

fn generate_n_trace_columns(lists: &CompiledAirFn) -> rust::Tokens {
    // TODO(Gali): Add mults column support.
    if lists.padding_type == PaddingType::Enabler {
        quote! {
            pub const N_TRACE_COLUMNS: usize = $(lists.state_names.len() + 1);
        }
    } else {
        quote! {
            pub const N_TRACE_COLUMNS: usize = $(lists.state_names.len());
        }
    }
}

fn generate_component_structs(lists: &CompiledAirFn) -> rust::Tokens {
    let mut members = rust::Tokens::new();

    // Claims.
    members.append(quote! {
        pub claim: Claim,
    });

    // Sub-components Lookup elements.
    for relation in lists.lookup_names.keys() {
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
    let mut channel_mix_code = quote! { channel.mix_u64(self.log_size as u64); };
    let mut members = quote! { pub log_size: u32, };
    for public_param in &lists.public_params {
        // TODO(Gali): Get the types of the public params from air_infra.
        members.append(quote! {
            pub $(public_param.name()): u32,
        });
        channel_mix_code.append(quote! {
            channel.mix_u64(self.$(public_param.name()) as u64);
        });
    }

    let struct_code = quote! {
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
        pub struct Claim {
            $(members)
        }
    };

    let n_lookup_terms: usize = lists.lookup_names.values().sum();
    let n_logup_columns = match n_lookup_terms {
        0 => unimplemented!(),
        1..=2 => quote!(SECURE_EXTENSION_DEGREE),
        n => {
            let n_batches = n.div_ceil(2);
            quote!(SECURE_EXTENSION_DEGREE * $(n_batches))
        }
    };

    let impl_code = quote! {
        impl Claim {
            pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
                let trace_log_sizes = vec![self.log_size; N_TRACE_COLUMNS];
                let interaction_log_sizes = vec![self.log_size; $(n_logup_columns)];
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

fn generate_interaction_claim_struct() -> rust::Tokens {
    let struct_code = quote! {
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
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
    code.append(quote! {
        impl FrameworkEval for Eval {
            fn log_size(&self) -> u32 {
                self.claim.log_size
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

    for (name, args) in &lists.external_states {
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

    if lists.r#type != TraceType::Inline && !lists.state_names.is_empty() {
        for name in &lists.state_names {
            code.append(quote! {
                let $name = eval.next_trace_mask();
            });
        }
    }

    if lists.padding_type == PaddingType::Enabler {
        // Add enabler column to the trace
        code.append(quote! {
            let enabler = eval.next_trace_mask();
            // Check enabler column is a bit.
            eval.add_constraint(enabler.clone() * enabler.clone() - enabler.clone());
        })
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
                        $(parse_eval_constraint(expr,&const_names))
                    );
                });
            }
            ConstraintEvalStep::Intermediate(CompiledIntermediate { name, r#type, var }) => {
                if r#type == "M31" {
                    code.extend(quote! {
                        let $(name) = eval.add_intermediate(
                            $(parse_eval_constraint(var,&const_names))
                        );
                    });
                } else {
                    // TODO(alont) consdier producing a warning to indicate that the intermediate
                    // does not translate into expression efficiency.
                    code.extend(quote! {
                        let $(name) = $(parse_eval_constraint(var,&const_names));
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
            ConstraintEvalStep::StartBlock(msg) => {
                code.extend(block_doc(msg));
            }
            ConstraintEvalStep::EndBlock => {
                code.extend(quote!(
                    $['\n']
                ));
            }
        }
        code.extend(quote! {
            $("\n")
        });
    }
    if lists.r#type == TraceType::Inline {
        code.extend(quote! {

            $(parse_eval_constraint(&lists.verifier_output.0, &const_names))
        });
    } else {
        code.extend(quote! {

            eval.finalize_logup_in_pairs();
            eval
        });
    }
    code
}
