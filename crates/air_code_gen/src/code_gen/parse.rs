//! Parsing logic to extract information from [`CompiledAirFn`].

use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirVar, CompiledIntermediate, ConstraintEvalStep, LookupTerm, UseOrYield,
};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

use crate::code_gen::utils::remove_trailing_zeroes;

// TODO(Ohad): Optimize small constantF252 values initialization.
pub fn constraint_consts(constraints: &[ConstraintEvalStep]) -> Vec<(String, String)> {
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
                ConstraintEvalStep::Intermediate(CompiledIntermediate {
                    name: _,
                    r#type: _,
                    var,
                }) => const_defs.extend(seek_consts(var)),
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
        CompiledAirVar::ExternalState { .. } => f(expr),
        CompiledAirVar::StaticCall(_, vars) => iter_many(vars),
        CompiledAirVar::MethodCall(_, _, vars) => iter_many(vars),
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

pub fn parse_eval_constraint(
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
        CompiledAirVar::ExternalState(name, ..) => name.to_lowercase() + ".clone()",
        CompiledAirVar::PublicParam(public_param) => {
            format!("E::F::from(M31::from(self.claim.{public_param}))")
        }
    }
}

pub fn parse_lookup_constraint(
    relation_name: &str,
    felts: &[CompiledAirVar],
    use_or_yield: &UseOrYield,
    constant_defs: &HashMap<(String, String), String>,
) -> rust::Tokens {
    let lookup_values = remove_trailing_zeroes(felts)
        .iter()
        .map(|felt| parse_eval_constraint(felt, constant_defs))
        .collect_vec();
    let sign = match use_or_yield {
        UseOrYield::Use => "",
        UseOrYield::Yield => "-",
    };
    let numerator = if relation_name.eq("Opcodes") {
        quote! {E::EF::from(padding.clone())}
    } else {
        quote! {E::EF::one()}
    };
    quote! {
        eval.add_to_relation(RelationEntry::new(&self.
            $(relation_name.to_case(Case::Snake))_lookup_elements,
            $(sign)$numerator, &[$(lookup_values.join(","))]));
    }
}
