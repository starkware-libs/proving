use air_infra::core::air_fn_registry::AirFnEntry;
use air_infra::core::compiled_structs::{
    CompiledAirFn, CompiledAirVar, ConstraintEvalStep, TraceGenStep,
};
use genco::lang::rust;
use genco::quote;
use itertools::{chain, Itertools};

use super::utils::{n_logup_columns, n_trace_cells};
use crate::code_gen::trace_gen::generate_sub_component_imports;
use crate::code_gen::utils::{callee_lookup_length, unique_constraint_function_calls};

pub fn generate_component_structs(
    component_name: &str,
    lists: CompiledAirFn,
    _entry: &AirFnEntry,
) -> rust::Tokens {
    quote! {
        $(imports(&lists.deductions))
        $['\n']
        $(generate_interaction_elements_struct(&lists))
        $['\n']
        $(generate_component_struct(component_name, &lists.constraints))
        $['\n']
        $(generate_claim_struct(&lists))
        $['\n']
        $(generate_interaction_claim_struct())
        $['\n']
        $(generate_framework_impl(component_name, &lists))
    }
}

fn generate_component_struct(
    component_name: &str,
    constraints: &[ConstraintEvalStep],
) -> rust::Tokens {
    let mut members = rust::Tokens::new();

    // Claims.
    members.append(quote! {
        pub claim: Claim,
        pub interaction_claim: InteractionClaim,
        pub self_lookup_elements: ComponentLookupElements,
    });

    // Sub-components Lookup elements.
    for fn_name in unique_constraint_function_calls(constraints) {
        let fn_name = fn_name.to_lowercase();
        members.append(quote! {
            pub $(&fn_name)_lookup_elements: $(fn_name)::ComponentLookupElements,
        });
    }

    quote! {
        pub struct $(component_name)Component {
            $(members)
        }
    }
}

fn generate_claim_struct(lists: &CompiledAirFn) -> rust::Tokens {
    let mut members = rust::Tokens::new();
    members.append(quote! {
        pub log_size: u32,
        pub n_calls: usize,
    });
    let struct_code = quote! {
        #[derive(Copy, Clone)]
        pub struct Claim {
            $(members)
        }
    };

    // impl
    let mut impl_code = rust::Tokens::new();
    let n_logup_columns = n_logup_columns(lists);
    let n_trace_cells = n_trace_cells(&lists.deductions);
    impl_code.append(quote! {
        impl Claim {
            pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
                let interaction_0_log_sizes = vec![self.log_size; $n_trace_cells];
                let interaction_1_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * $n_logup_columns];
                TreeVec::new(vec![interaction_0_log_sizes, interaction_1_log_sizes])
            }
             // TODO(Ohad): better mix_into.
            pub fn mix_into(&self, channel: &mut impl Channel) {
                channel.mix_nonce(self.log_size as u64);
                channel.mix_nonce(self.n_calls as u64);
            }
        }
    });

    chain!(struct_code, impl_code).collect()
}

fn generate_interaction_claim_struct() -> rust::Tokens {
    let struct_code = quote! {
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

fn generate_interaction_elements_struct(lists: &CompiledAirFn) -> rust::Tokens {
    quote! {
        // TODO(Ohad): figure this out.
        pub type ComponentLookupElements = LookupElements<$(callee_lookup_length(lists))>;
    }
}

fn generate_framework_impl(component_name: &str, lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    code.append(quote! {
        impl FrameworkComponent for $(component_name)Component {
            fn log_size(&self) -> u32 {
                self.claim.log_size
            }

            fn max_constraint_log_degree_bound(&self) -> u32 {
                // TODO(Ohad): determine dynamically.
                self.log_size() + 1
            }

            #[allow(unused_parens)]
            #[allow(clippy::double_parens)]
            fn evaluate<E: EvalAtRow>(&self, mut eval:E) -> E{
                $(generate_evaluate(lists))
            }
        }
    });
    code
}

fn generate_evaluate(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    code.append(quote! {
        let mut logup = LogupAtRow::<LOGUP_BATCH_SIZE, E>::new(
            1,
            self.interaction_claim.claimed_sum,
            self.claim.log_size,
        );
    });

    code.append(quote! {
        let trace_row: [_; $(n_trace_cells(&lists.deductions))]
            = std::array::from_fn(|_| eval.next_trace_mask());
    });

    for constraint in lists.constraints.iter() {
        match constraint {
            ConstraintEvalStep::InInstanceConstraint(expr) => {
                code.extend(quote! {
                    eval.add_constraint(
                        $(parse_eval_constraint(expr))
                    );
                });
            }
            ConstraintEvalStep::Intermediate(var, expr) => {
                code.extend(quote! {
                    let $(var) = $(parse_eval_constraint(expr));
                });
            }
            // TODO(Ohad): implement.
            ConstraintEvalStep::LookupConstraint {
                fn_name,
                input_felts,
                output_felts,
            } => {
                code.extend(parse_lookup_constraint(fn_name, input_felts, output_felts));
            }
            // TODO: Implement.
            ConstraintEvalStep::AccessExternalColumn {
                fn_name: _,
                output_name: _,
            } => (),
        }
    }

    code.extend(quote! {
        $['\n']
    });
    let input_values = (0..lists.input_num_of_felts)
        .map(|i| format!("trace_row[{}]", i))
        .join(", ");
    let output_values = lists
        .output_felts
        .iter()
        .map(parse_eval_constraint)
        .join(", ");
    let lookup_values = format!("{}, {}", input_values, output_values);
    code.extend(quote! {
        logup.push_lookup(
            &mut eval,
            -E::EF::one(),
            &[$lookup_values],
            &self.self_lookup_elements,
        );
    });

    code.extend(quote! {

        logup.finalize(&mut eval);

        eval
    });
    code
}

fn parse_eval_constraint(expr: &CompiledAirVar) -> String {
    match expr {
        CompiledAirVar::Const(_, val) => {
            format!("E::F::from(M31::from({val}))")
        }
        CompiledAirVar::State(index) => format!("trace_row[{index}]"),
        CompiledAirVar::StaticCall(id, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_eval_constraint(arg));
            }
            format!("{}({})", id, arg_str)
        }
        CompiledAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_eval_constraint(arg));
            }
            format!("{}.{}({})", parse_eval_constraint(id), func, arg_str)
        }
        CompiledAirVar::Var(_, id) => id.to_string(),
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({} {op} {})",
                parse_eval_constraint(lhs),
                parse_eval_constraint(rhs)
            )
        }
        CompiledAirVar::UnaryOp(op, expr) => {
            format!("({op}{})", parse_eval_constraint(expr))
        }
        CompiledAirVar::Tuple(_) => unimplemented!(),
        CompiledAirVar::Array(_) => unimplemented!(),
        CompiledAirVar::Struct { .. } => {
            todo!()
        }
    }
}

fn imports(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        #![allow(non_camel_case_types)]
        #![allow(unused_imports)]
        use num_traits::One;
        use stwo_prover::constraint_framework::logup::{LogupAtRow, LookupElements};
        use stwo_prover::constraint_framework::{EvalAtRow, FrameworkComponent};
        use stwo_prover::core::channel::Channel;
        use stwo_prover::core::backend::simd::m31::PackedM31;
        use stwo_prover::core::fields::m31::M31;
        use stwo_prover::core::fields::qm31::SecureField;
        use stwo_prover::core::fields::secure_column::SECURE_EXTENSION_DEGREE;
        use stwo_prover::core::pcs::TreeVec;

        use crate::LOGUP_BATCH_SIZE;
        $(generate_sub_component_imports(deductions))
    }
}

fn parse_lookup_constraint(
    fn_name: &str,
    inputs: &[CompiledAirVar],
    outputs: &[CompiledAirVar],
) -> rust::Tokens {
    let lookup_values = chain!(inputs, outputs)
        .map(parse_eval_constraint)
        .collect_vec()
        .join(", ");
    quote! {
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[$lookup_values],
            &self.$(fn_name.to_lowercase())_lookup_elements,
        );
    }
}
