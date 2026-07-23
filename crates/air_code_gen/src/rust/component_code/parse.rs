//! Parsing logic to extract information from [`CompiledAirFn`].

use std::collections::{BTreeSet, HashMap, HashSet};

use air_common::{CONSTRAINT_EVAL_FUNCTION_NAME, TraceType, UseOrYield};
use air_compile::compiled_structs::{
    CompiledAirFn, CompiledAirVar, CompiledConstraintIntermediate, ConstraintEvalStep, LookupTerm,
};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

use crate::utils::{expr_iterator, remove_trailing_zeroes};

// TODO(Ohad): Optimize small constantF252 values initialization.
pub fn constraint_consts(constraints: &[ConstraintEvalStep]) -> BTreeSet<(String, String)> {
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
                    multiplicity,
                    ..
                }) => {
                    let felts = remove_trailing_zeroes(felts);
                    const_defs.extend(felts.iter().flat_map(seek_consts));
                    const_defs.extend(seek_consts(multiplicity));
                }
                ConstraintEvalStep::Intermediate(CompiledConstraintIntermediate {
                    var, ..
                }) => const_defs.extend(seek_consts(var)),
            };
            const_defs
        })
        .into_iter()
        .sorted()
        .collect()
}

pub fn seek_consts(expr: &CompiledAirVar) -> BTreeSet<(String, String)> {
    let mut hashset = BTreeSet::new();
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

pub fn parse_eval_constraint(
    air_fn: &CompiledAirFn,
    expr: &CompiledAirVar,
    constant_names: &HashMap<(String, String), String>,
) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => {
            constant_names.get(&(ty.to_owned(), val.to_owned())).unwrap().to_string() + ".clone()"
        }
        CompiledAirVar::State(name) => format!("{name}.clone()"),
        CompiledAirVar::StaticCall(id, args) => {
            assert!(id.ends_with(CONSTRAINT_EVAL_FUNCTION_NAME));
            gen_evaluate_call(air_fn, id, args, constant_names)
        }
        CompiledAirVar::Var(_, id) => id.to_string() + ".clone()",
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({} {op} {})",
                parse_eval_constraint(air_fn, lhs, constant_names),
                parse_eval_constraint(air_fn, rhs, constant_names)
            )
        }
        CompiledAirVar::UnaryOp(op, expr) => {
            format!("({op}{})", parse_eval_constraint(air_fn, expr, constant_names))
        }
        CompiledAirVar::Tuple(vars) => {
            let vars_str = vars
                .iter()
                .map(|var| parse_eval_constraint(air_fn, var, constant_names))
                .collect_vec()
                .join(", ");
            format!("({vars_str})")
        }
        CompiledAirVar::Array(vars) => {
            let vars_str = vars
                .iter()
                .map(|var| parse_eval_constraint(air_fn, var, constant_names))
                .collect_vec()
                .join(", ");
            format!("[{vars_str}]")
        }
        CompiledAirVar::Enabler => {
            assert_eq!(air_fn.r#type, TraceType::Inline);
            "enabler.clone()".to_string()
        }
        CompiledAirVar::ExternalState(col_id) => col_id.to_lowercase() + ".clone()",
        CompiledAirVar::PublicParam(public_param) => {
            if air_fn.r#type == TraceType::Inline {
                public_param.clone() + ".clone()"
            } else {
                format!("E::F::from(M31::from(self.{public_param}))")
            }
        }
        CompiledAirVar::Struct { .. }
        | CompiledAirVar::MethodCall(..)
        | CompiledAirVar::Multiplicity(_) => {
            panic!("Unsupported expression in constraint evaluation: {expr}")
        }
    }
}

fn gen_evaluate_call(
    air_fn: &CompiledAirFn,
    id: &str,
    args: &[CompiledAirVar],
    constant_names: &HashMap<(String, String), String>,
) -> String {
    let mut arg_str = args
        .iter()
        .map(|arg| parse_eval_constraint(air_fn, arg, constant_names))
        .collect::<Vec<_>>();
    let inline_fn = id.trim_end_matches(&format!("::{CONSTRAINT_EVAL_FUNCTION_NAME}"));
    let (_, params, external_states) = air_fn.inline_calls.get(inline_fn).unwrap();
    if air_fn.r#type == TraceType::Inline {
        arg_str.push("common_lookup_elements".to_string());
    } else {
        arg_str.push("&self.common_lookup_elements".to_string());
    }
    for param in params {
        arg_str.push(parse_eval_constraint(
            air_fn,
            &CompiledAirVar::PublicParam(param.clone()),
            constant_names,
        ));
    }
    for ext_state in external_states {
        arg_str.push(parse_eval_constraint(
            air_fn,
            &CompiledAirVar::ExternalState(ext_state.clone()),
            constant_names,
        ));
    }
    if air_fn.r#type == TraceType::Inline {
        arg_str.push("eval".to_string());
    } else {
        arg_str.push("&mut eval".to_string());
    }

    format!(
        "{}::{}({})\n",
        inline_fn.to_case(Case::Pascal),
        CONSTRAINT_EVAL_FUNCTION_NAME,
        arg_str.join(", ")
    )
}

pub fn parse_lookup_constraint(
    air_fn: &CompiledAirFn,
    felts: &[CompiledAirVar],
    use_or_yield: &UseOrYield,
    multiplicity: &CompiledAirVar,
    constant_defs: &HashMap<(String, String), String>,
) -> rust::Tokens {
    let lookup_values = remove_trailing_zeroes(felts)
        .iter()
        .map(|felt| parse_eval_constraint(air_fn, felt, constant_defs))
        .collect_vec();
    // TODO(AnatG): Assumes how parse_eval_constraint formats the output. Find a better way.
    let lookup_values_str = if lookup_values.len() == 1 {
        if lookup_values[0].ends_with(".clone()") {
            format!("std::slice::from_ref(&{})", lookup_values[0].replace(".clone()", ""))
        } else {
            format!("std::slice::from_ref(&{})", lookup_values[0])
        }
    } else {
        format!("&[{}]", lookup_values.join(", "))
    };
    let sign = match use_or_yield {
        UseOrYield::Use => "",
        UseOrYield::Yield => "-",
    };
    let numerator = quote! {
        E::EF::from($(parse_eval_constraint(air_fn, multiplicity, constant_defs)))
    };
    if air_fn.r#type == TraceType::Inline {
        quote! {
            eval.add_to_relation(RelationEntry::new(
                common_lookup_elements,
                $(sign)$numerator, $(lookup_values_str)));
        }
    } else {
        quote! {
            eval.add_to_relation(RelationEntry::new(&self.
                common_lookup_elements,
                $(sign)$numerator, $(lookup_values_str)));
        }
    }
}
