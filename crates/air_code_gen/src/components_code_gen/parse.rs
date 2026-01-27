//! Parsing logic to extract information from [`CompiledAirFn`].

use std::collections::{BTreeSet, HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, CompiledConstraintIntermediate, ConstraintEvalStep, LookupTerm,
    PaddingType, TraceType, UseOrYield,
};
use compiled_casm_air::utils::CONSTRAINT_EVAL_FUNCTION_NAME;
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

use crate::utils::{relation_multiplicity_index, remove_trailing_zeroes};

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
                    ..
                }) => {
                    let felts = remove_trailing_zeroes(felts);
                    const_defs.extend(felts.iter().flat_map(seek_consts))
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
        CompiledAirVar::ExternalState { .. } => f(expr),
        CompiledAirVar::StaticCall(_, vars) => iter_many(vars),
        CompiledAirVar::MethodCall(self_var, _, vars) => {
            iter_many(vars);
            expr_iterator(self_var, f);
        }
        CompiledAirVar::BinaryOp(lhs, _, rhs) => iter_many(&[*lhs.clone(), *rhs.clone()]),
        CompiledAirVar::UnaryOp(_, var) => f(var),
        CompiledAirVar::Tuple(vars) => iter_many(vars),
        CompiledAirVar::Array(vars) => iter_many(vars),
        CompiledAirVar::Struct { r#type: _, fields } => {
            iter_many(&fields.iter().cloned().map(|(_, var)| var).collect_vec())
        }
        CompiledAirVar::PublicParam(_) => f(expr),
    }
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
            constant_names
                .get(&(ty.to_owned(), val.to_owned()))
                .unwrap()
                .to_string()
                + ".clone()"
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
            format!(
                "({op}{})",
                parse_eval_constraint(air_fn, expr, constant_names)
            )
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
        CompiledAirVar::Struct { .. } | CompiledAirVar::MethodCall(..) => {
            panic!("Unsupported expression in constraint evaluation: {expr}")
        }
        CompiledAirVar::ExternalState(col_id) => col_id.to_lowercase() + ".clone()",
        CompiledAirVar::PublicParam(public_param) => {
            if air_fn.r#type == TraceType::Inline {
                public_param.clone() + ".clone()"
            } else {
                format!("E::F::from(M31::from(self.claim.{public_param}))")
            }
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
            &CompiledAirVar::PublicParam(param.name()),
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
    relation_name: &str,
    felts: &[CompiledAirVar],
    use_or_yield: &UseOrYield,
    constant_defs: &HashMap<(String, String), String>,
) -> rust::Tokens {
    let lookup_values = remove_trailing_zeroes(felts)
        .iter()
        .map(|felt| parse_eval_constraint(air_fn, felt, constant_defs))
        .collect_vec();
    // TODO(AnatG): Assumes how parse_eval_constraint formats the output. Find a better way.
    let lookup_values_str = if lookup_values.len() == 1 {
        if lookup_values[0].ends_with(".clone()") {
            format!(
                "std::slice::from_ref(&{})",
                lookup_values[0].replace(".clone()", "")
            )
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
    let is_masked = relation_multiplicity_index(air_fn, relation_name);
    let numerator = match air_fn.padding_type {
        PaddingType::Enabler if is_masked.is_some() => quote! {E::EF::from(enabler.clone())},
        PaddingType::Multiplicity if is_masked.is_some() => {
            quote! {E::EF::from(multiplicity_$(is_masked.unwrap()))}
        }
        _ => quote! {E::EF::one()},
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
