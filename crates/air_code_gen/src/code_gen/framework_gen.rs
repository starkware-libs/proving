use std::collections::HashMap;

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, ConstraintEvalStep, Intermediate, LookupTerm, TraceGenStep,
};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::chain;

use crate::code_gen::parse::{
    constraint_consts, get_public_params_from_lookup_terms, parse_eval_constraint,
    parse_lookup_constraint,
};
use crate::code_gen::utils::{block_doc, unique_constraint_relations};

pub fn generate_component_code(lists: CompiledAirFn) -> rust::Tokens {
    quote! {
        $(imports(&lists.deductions))
        $['\n']
        $(generate_component_structs(&lists.constraints))
        $['\n']
        $(generate_claim_struct(&lists))
        $['\n']
        $(generate_interaction_claim_struct())
        $['\n']
        $(generate_component_type_def())
        $['\n']
        $(generate_framework_impl(&lists))
    }
}

fn imports(_deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        #![allow(non_camel_case_types)]
        #![allow(unused_imports)]
        use num_traits::{One, Zero};
        use serde::{Deserialize, Serialize};
        use stwo_cairo_serialize::CairoSerialize;
        use stwo_prover::constraint_framework::logup::{LogupAtRow, LogupSums, LookupElements};
        use stwo_prover::constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry};
        use stwo_prover::core::backend::simd::m31::LOG_N_LANES;
        use stwo_prover::core::channel::Channel;
        use stwo_prover::core::fields::m31::M31;
        use stwo_prover::core::fields::qm31::SecureField;
        use stwo_prover::core::fields::secure_column::SECURE_EXTENSION_DEGREE;
        use stwo_prover::core::pcs::TreeVec;
        use crate::relations;
    }
}

fn generate_component_structs(constraints: &[ConstraintEvalStep]) -> rust::Tokens {
    let mut members = rust::Tokens::new();

    // Claims.
    members.append(quote! {
        pub claim: Claim,
    });

    // Sub-components Lookup elements.
    for relation in unique_constraint_relations(constraints) {
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
    let mut channel_mix_code = quote! {
        channel.mix_u64(self.n_calls as u64);
    };
    let mut members = quote! {
        pub n_calls: usize,
    };
    let public_params = get_public_params_from_lookup_terms(&lists.constraints);
    for public_param in &public_params {
        // TODO(Gali): Get the types of the public params from air_infra.
        members.append(quote! {
            pub $public_param: u32,
        });
        channel_mix_code.append(quote! {
            channel.mix_u64(self.$public_param as u64);
        });
    }

    let struct_code = quote! {
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
        pub struct Claim {
            $(members)
        }
    };

    let n_logup_columns = match lists.n_lookup_terms {
        0 => unimplemented!(),
        1 => quote!(SECURE_EXTENSION_DEGREE),
        _ => {
            let n_batches = lists.n_lookup_terms.div_ceil(2);
            quote!(SECURE_EXTENSION_DEGREE * $(n_batches))
        }
    };
    let impl_code = quote! {
        impl Claim {
            pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
                let log_size = std::cmp::max(self.n_calls.next_power_of_two().ilog2(), LOG_N_LANES);
                let trace_log_sizes = vec![log_size; $(lists.state_names.len())];
                let interaction_log_sizes = vec![log_size; $(n_logup_columns)];
                let preprocessed_log_sizes = vec![log_size];
                TreeVec::new(vec![
                    preprocessed_log_sizes,
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
            pub logup_sums: LogupSums,
        }
    };
    let mut impl_code = rust::Tokens::new();
    impl_code.append(quote! {
        impl InteractionClaim {
            pub fn mix_into(&self, channel: &mut impl Channel) {
                let (total_sum, claimed_sum) = self.logup_sums;
                channel.mix_felts(&[total_sum]);
                if let Some(claimed_sum) = claimed_sum {
                    channel.mix_felts(&[claimed_sum.0]);
                    channel.mix_u64(claimed_sum.1 as u64);
                }
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
                std::cmp::max(self.claim.n_calls.next_power_of_two().ilog2(), LOG_N_LANES)
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
    let constants = constraint_consts(&lists.constraints);
    let mut const_names = HashMap::new();
    for (ty, val) in constants.into_iter() {
        let name = format!("{ty}_{val}");
        const_names.insert((ty.clone(), val.clone()), name.clone());
        if ty == "M31" {
            code.append(quote! {
                let $(name) = E::F::from($(ty)::from($(val)));
            });
        } else {
            code.append(quote! {
                let $(name) = $(ty)::from($(val));
            });
        }
    }

    // TODO(Ohad): handle next_trace_mask for external states.
    if !lists.state_names.is_empty() {
        for name in &lists.state_names {
            code.append(quote! {
                let $name = eval.next_trace_mask();
            });
        }
    }

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
            ConstraintEvalStep::Intermediate(Intermediate { name, r#type, var }) => {
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
    code.extend(quote! {

        eval.finalize_logup_in_pairs();
        eval
    });
    code
}
