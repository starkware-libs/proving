use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, ConstraintEvalStep, LookupData, TraceGenStep, UseOrYield,
};
use genco::lang::rust;
use genco::quote;
use itertools::{chain, Itertools};

use super::utils::{n_logup_columns, n_trace_cells};
use crate::code_gen::simd_prover_gen::remove_trailing_zeroes;
use crate::code_gen::trace_gen::generate_sub_component_imports;
use crate::code_gen::utils::{callee_lookup_length, unique_constraint_relations};

pub fn generate_component_structs(component_name: &str, lists: CompiledAirFn) -> rust::Tokens {
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
        $(generate_component_type_def(component_name))
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
    });

    // Sub-components Lookup elements.
    for fn_name in unique_constraint_relations(constraints) {
        let fn_name = fn_name.to_lowercase();
        members.append(quote! {
            pub $(&fn_name)_lookup_elements: $(fn_name)::ComponentLookupElements,
        });
    }

    quote! {
        pub struct $(component_name)Eval {
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

fn generate_component_type_def(component_name: &str) -> rust::Tokens {
    quote! {
        // TODO(Ohad): remove this after names are changed.
        #[allow(non_snake_case)]
        pub type $(component_name)Component = FrameworkComponent<$(component_name)Eval>;
    }
}

fn generate_framework_impl(component_name: &str, lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    code.append(quote! {
        impl FrameworkEval for $(component_name)Eval {
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

    // Constants.
    let constants = constraint_consts(&lists.constraints);
    let mut const_names = HashMap::new();
    for (ty, val) in constants.into_iter() {
        let name = format!("{ty}_{val}");
        const_names.insert((ty.clone(), val.clone()), name.clone());
        let ty = if ty == "M31" { "E::F" } else { &ty };
        code.append(quote! {
            let $(name) = $(ty)::from(M31::from($(val)));
        });
    }

    // Logup.
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
            ConstraintEvalStep::Constraint(expr) => {
                code.extend(quote! {
                    eval.add_constraint(
                        $(parse_eval_constraint(expr,&const_names))
                    );
                });
            }
            ConstraintEvalStep::Intermediate(var, expr) => {
                code.extend(quote! {
                    let $(var) = $(parse_eval_constraint(expr,&const_names));
                });
            }
            // TODO(Ohad): implement.
            ConstraintEvalStep::LookupData(LookupData {
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
            ConstraintEvalStep::StartBlock(_) => (),
            ConstraintEvalStep::EndBlock => (),
        }
    }
    code.extend(quote! {

        logup.finalize(&mut eval);

        eval
    });
    code
}

// TODO(Ohad): Optimize small constantF252 values initialization.
fn constraint_consts(constraints: &[ConstraintEvalStep]) -> Vec<(String, String)> {
    constraints
        .iter()
        .fold(HashSet::new(), |mut const_defs, constraint| {
            match constraint {
                ConstraintEvalStep::Constraint(compiled_air_var) => {
                    const_defs.extend(seek_consts(compiled_air_var))
                }
                ConstraintEvalStep::LookupData(LookupData {
                    relation_name: _,
                    felts,
                    ..
                }) => const_defs.extend(felts.iter().flat_map(seek_consts)),
                ConstraintEvalStep::Intermediate(_, var) => const_defs.extend(seek_consts(var)),
                ConstraintEvalStep::StartBlock(_) => {}
                ConstraintEvalStep::EndBlock => {}
            };
            const_defs
        })
        .into_iter()
        .sorted()
        .collect()
}

fn expr_iterator<F>(expr: &CompiledAirVar, f: &mut F)
where
    F: FnMut(&CompiledAirVar),
{
    let mut iter_many =
        |vars: &[CompiledAirVar]| vars.iter().for_each(|var| expr_iterator::<F>(var, f));

    match expr {
        CompiledAirVar::Const(..) => f(expr),
        CompiledAirVar::Var(..) => f(expr),
        CompiledAirVar::State(..) => f(expr),
        CompiledAirVar::StaticCall(_, vars) => iter_many(vars),
        CompiledAirVar::MethodCall(_, _, vars) => iter_many(vars),
        CompiledAirVar::BinaryOp(lhs, _, rhs) => iter_many(&[*lhs.clone(), *rhs.clone()]),
        CompiledAirVar::UnaryOp(_, var) => f(var),
        CompiledAirVar::Tuple(vars) => iter_many(vars),
        CompiledAirVar::Array(vars) => iter_many(vars),
        CompiledAirVar::Struct { r#type: _, fields } => {
            iter_many(&fields.iter().cloned().map(|(_, var)| var).collect_vec())
        }
        CompiledAirVar::ExternalState(..) => todo!(),
    }
}

pub fn seek_consts(expr: &CompiledAirVar) -> HashSet<(String, String)> {
    let mut hashset = HashSet::new();
    let mut insert = |expr: &CompiledAirVar| {
        if let CompiledAirVar::Const(ty, val) = expr {
            hashset.insert((ty.to_string(), val.to_string()));
        }
    };
    expr_iterator(expr, &mut insert);
    hashset
}

fn parse_eval_constraint(
    expr: &CompiledAirVar,
    constant_names: &HashMap<(String, String), String>,
) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => constant_names
            .get(&(ty.to_owned(), val.to_owned()))
            .unwrap()
            .to_string(),
        CompiledAirVar::State(index) => format!("trace_row[{index}]"),
        CompiledAirVar::StaticCall(id, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_eval_constraint(arg, constant_names));
            }
            format!("{}({})", id, arg_str)
        }
        CompiledAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_eval_constraint(arg, constant_names));
            }
            format!(
                "{}.{}({})",
                parse_eval_constraint(id, constant_names),
                func,
                arg_str
            )
        }
        CompiledAirVar::Var(_, id) => id.to_string(),
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({} {op} {})",
                parse_eval_constraint(lhs, constant_names),
                parse_eval_constraint(rhs, constant_names)
            )
        }
        CompiledAirVar::UnaryOp(op, expr) => {
            format!("({op}{})", parse_eval_constraint(expr, constant_names))
        }
        CompiledAirVar::Tuple(_) => unimplemented!(),
        CompiledAirVar::Array(_) => unimplemented!(),
        CompiledAirVar::Struct { .. } => {
            todo!()
        }
        CompiledAirVar::ExternalState(..) => "todo!()".to_string(),
    }
}

fn imports(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        #![allow(non_camel_case_types)]
        #![allow(unused_imports)]
        use num_traits::One;
        use stwo_prover::constraint_framework::logup::{LogupAtRow, LookupElements};
        use stwo_prover::constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval};
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
    relation_name: &str,
    felts: &[CompiledAirVar],
    use_or_yield: &UseOrYield,
    constant_defs: &HashMap<(String, String), String>,
) -> rust::Tokens {
    let lookup_values = felts
        .iter()
        .map(|felt| parse_eval_constraint(felt, constant_defs))
        .collect_vec()
        .join(", ");
    let lookup_values = remove_trailing_zeroes(&lookup_values);
    let sign = match use_or_yield {
        UseOrYield::Use => "",
        UseOrYield::Yield => "-",
    };
    quote! {
        logup.push_lookup(
            &mut eval,
            $(sign)E::EF::one(),
            &[$lookup_values],
            &self.$(relation_name.to_lowercase())_lookup_elements,
        );
    }
}
