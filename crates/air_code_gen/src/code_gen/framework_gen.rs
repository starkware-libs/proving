use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, ConstraintEvalStep, LookupTerm, TraceGenStep, UseOrYield,
};
use genco::lang::rust;
use genco::quote;
use itertools::{chain, Itertools};

use super::utils::block_doc;
use crate::code_gen::simd_prover_gen::remove_trailing_zeroes;
use crate::code_gen::utils::unique_constraint_relations;

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

fn generate_component_structs(constraints: &[ConstraintEvalStep]) -> rust::Tokens {
    let mut members = rust::Tokens::new();

    // Claims.
    members.append(quote! {
        pub claim: Claim,
    });

    // Sub-components Lookup elements.
    for relation in unique_constraint_relations(constraints) {
        members.append(quote! {
            pub $(&relation.to_lowercase())_lookup_elements: relations::$(relation),
        });
    }

    quote! {
        pub struct Eval {
            $(members)
        }
    }
}

fn generate_claim_struct(lists: &CompiledAirFn) -> rust::Tokens {
    let mut members = rust::Tokens::new();
    members.append(quote! {
        pub n_calls: usize,
    });
    let struct_code = quote! {
        #[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
        pub struct Claim {
            $(members)
        }
    };

    // impl
    let mut impl_code = rust::Tokens::new();
    let n_logup_columns = match lists.n_lookup_terms {
        0 => unimplemented!(),
        1 => quote!(SECURE_EXTENSION_DEGREE),
        _ => quote!(SECURE_EXTENSION_DEGREE * $(lists.n_lookup_terms)),
    };
    impl_code.append(quote! {
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
                channel.mix_u64(self.n_calls as u64);
            }
        }
    });

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

/// Collects all M31 variables from an air variable.
fn collect_m31_vars_from_air_var(air_var: &CompiledAirVar) -> Vec<String> {
    match air_var {
        CompiledAirVar::Const(..) => vec![],
        CompiledAirVar::Var(ty, name) => {
            if ty == "M31" {
                vec![name.to_string()]
            } else {
                vec![]
            }
        }
        CompiledAirVar::State(_) => vec![],
        CompiledAirVar::StaticCall(_, vars) => vars
            .iter()
            .map(collect_m31_vars_from_air_var)
            .collect_vec()
            .concat(),
        CompiledAirVar::MethodCall(_, _, vars) => vars
            .iter()
            .map(collect_m31_vars_from_air_var)
            .collect_vec()
            .concat(),
        CompiledAirVar::BinaryOp(var_l, _, var_r) => [var_l, var_r]
            .map(|var| collect_m31_vars_from_air_var(var))
            .concat(),
        CompiledAirVar::UnaryOp(_, var) => collect_m31_vars_from_air_var(var),
        CompiledAirVar::Tuple(vars) => vars
            .iter()
            .map(collect_m31_vars_from_air_var)
            .collect_vec()
            .concat(),
        CompiledAirVar::Array(vars) => vars
            .iter()
            .map(collect_m31_vars_from_air_var)
            .collect_vec()
            .concat(),
        CompiledAirVar::Struct { r#type: _, fields } => fields
            .iter()
            .map(|(_, var)| collect_m31_vars_from_air_var(var))
            .collect_vec()
            .concat(),
        CompiledAirVar::ExternalState(..) => vec![],
        CompiledAirVar::PublicParam(_) => vec![],
    }
}

/// Collects all M31 variables from a ConstraintEvalStep.
fn collect_m31_vars_from_expr(constraint: &ConstraintEvalStep) -> Vec<String> {
    match constraint {
        ConstraintEvalStep::StartBlock(_) => vec![],
        ConstraintEvalStep::EndBlock => vec![],
        ConstraintEvalStep::Constraint(var, _) => collect_m31_vars_from_air_var(var),
        ConstraintEvalStep::LookupTerm(term) => term
            .felts
            .iter()
            .map(collect_m31_vars_from_air_var)
            .collect_vec()
            .concat(),
        ConstraintEvalStep::Intermediate(_, var) => collect_m31_vars_from_air_var(var),
    }
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

    let m31_vars = &lists
        .constraints
        .iter()
        .map(collect_m31_vars_from_expr)
        .collect_vec()
        .concat();

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
            ConstraintEvalStep::Intermediate(var, expr) => {
                // TODO(Ohad): change this condition once intermediate types are exported.
                if m31_vars.contains(var) {
                    code.extend(quote! {
                        let $(var) = eval.add_intermediate(
                            $(parse_eval_constraint(expr,&const_names))
                        );
                    });
                } else {
                    // TODO(alont) consdier producing a warning to indicate that the intermediate
                    // does not translate into expression efficiency.
                    code.extend(quote! {
                        let $(var) = $(parse_eval_constraint(expr,&const_names));
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

        eval.finalize_logup();
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
                ConstraintEvalStep::Constraint(compiled_air_var, ..) => {
                    const_defs.extend(seek_consts(compiled_air_var))
                }
                ConstraintEvalStep::LookupTerm(LookupTerm {
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
        CompiledAirVar::ExternalState(..) => f(expr),
        CompiledAirVar::StaticCall(_, vars) => iter_many(vars),
        CompiledAirVar::MethodCall(_, _, vars) => iter_many(vars),
        CompiledAirVar::BinaryOp(lhs, _, rhs) => iter_many(&[*lhs.clone(), *rhs.clone()]),
        CompiledAirVar::UnaryOp(_, var) => f(var),
        CompiledAirVar::Tuple(vars) => iter_many(vars),
        CompiledAirVar::Array(vars) => iter_many(vars),
        CompiledAirVar::Struct { r#type: _, fields } => {
            iter_many(&fields.iter().cloned().map(|(_, var)| var).collect_vec())
        }
        CompiledAirVar::PublicParam(_) => todo!(),
    }
}

pub fn seek_consts(expr: &CompiledAirVar) -> HashSet<(String, String)> {
    let mut hashset = HashSet::new();
    let mut insert = |expr: &CompiledAirVar| {
        if let CompiledAirVar::Const(ty, val) = expr {
            // Usize are used for array indexing, handled differently.
            if ty != "usize" {
                hashset.insert((ty.to_string(), val.to_string()));
            }
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
        CompiledAirVar::Const(ty, val) => {
            constant_names
                .get(&(ty.to_owned(), val.to_owned()))
                .unwrap()
                .to_string()
                + ".clone()"
        }
        CompiledAirVar::State(name) => format!("{}.clone()", name),
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
        CompiledAirVar::Var(_, id) => id.to_string() + ".clone()",
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
        CompiledAirVar::PublicParam(_) => todo!(),
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

fn parse_lookup_constraint(
    relation_name: &str,
    felts: &[CompiledAirVar],
    use_or_yield: &UseOrYield,
    constant_defs: &HashMap<(String, String), String>,
) -> rust::Tokens {
    let lookup_values = felts
        .iter()
        .map(|felt| parse_eval_constraint(felt, constant_defs))
        .collect_vec();
    let lookup_values = remove_trailing_zeroes(lookup_values);
    let sign = match use_or_yield {
        UseOrYield::Use => "",
        UseOrYield::Yield => "-",
    };
    quote! {
        eval.add_to_relation(RelationEntry::new(&self.
            $(relation_name.to_lowercase())_lookup_elements,
            $(sign)E::EF::one(), &[$(lookup_values.join(","))]));
    }
}
