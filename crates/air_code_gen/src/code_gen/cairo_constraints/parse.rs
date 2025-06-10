use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, ExternalState, TraceType,
};
use compiled_casm_air::utils::CONSTRAINT_EVAL_FUNCTION_NAME;
use convert_case::{Case, Casing};
use indexmap::IndexSet;

use super::super::utils::get_variable_name;

pub fn parse_var(
    air_fn: &CompiledAirFn,
    var: &CompiledAirVar,
    relation_offset: &mut usize,
) -> String {
    match var {
        CompiledAirVar::Const(_ty, value) => format!("qm31_const::<{value}, 0, 0, 0>()"),
        CompiledAirVar::Var(_ty, name) => name.clone(),
        CompiledAirVar::State(name) => name.clone(),
        CompiledAirVar::StaticCall(name, args) => {
            if name.ends_with(CONSTRAINT_EVAL_FUNCTION_NAME) {
                return gen_evaluate_call(air_fn, name, args, relation_offset);
            }

            unimplemented!("Unsupported static call: {name}");
        }
        CompiledAirVar::BinaryOp(left, op, right) => {
            let left = parse_var(air_fn, left.as_ref(), relation_offset);
            let right = parse_var(air_fn, right.as_ref(), relation_offset);
            format!("({left} {op} {right})")
        }
        CompiledAirVar::UnaryOp(op, var) => {
            let var = parse_var(air_fn, var.as_ref(), relation_offset);
            format!("({op}{var})")
        }
        CompiledAirVar::Array(vars) => {
            let vars = vars
                .iter()
                .map(|v| parse_var(air_fn, v, relation_offset))
                .collect::<Vec<_>>();
            format!("[{}]", vars.join(", "))
        }
        CompiledAirVar::ExternalState(ExternalState {
            name,
            generic_param: _,
            args,
        }) => {
            if name == "Seq" {
                "seq".to_string()
            } else {
                get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())
            }
        }
        CompiledAirVar::PublicParam(name) => name.clone(),
        v => unimplemented!("Unsupported variable type: {v:?}"),
    }
}

fn gen_evaluate_call(
    air_fn: &CompiledAirFn,
    id: &str,
    args: &[CompiledAirVar],
    relation_offset: &mut usize,
) -> String {
    let mut arg_str = args
        .iter()
        .map(|arg| parse_var(air_fn, arg, relation_offset))
        .collect::<Vec<_>>();

    let inline_fn = id.trim_end_matches(&format!("::{}", CONSTRAINT_EVAL_FUNCTION_NAME));
    let (relations, params, external_states) = air_fn.inline_calls.get(inline_fn).unwrap();
    let relation_names = relations
        .iter()
        .map(|(relation, _)| relation.to_case(Case::Snake))
        .collect::<IndexSet<_>>();
    for relation in &relation_names {
        if air_fn.r#type == TraceType::Inline {
            arg_str.push(format!("{relation}_lookup_elements"));
        } else {
            arg_str.push(format!("self.{relation}_lookup_elements"));
        }
    }
    for param in params {
        arg_str.push(parse_var(
            air_fn,
            &CompiledAirVar::PublicParam(param.name()),
            relation_offset,
        ));
    }
    for ext_state in external_states {
        arg_str.push(parse_var(
            air_fn,
            &CompiledAirVar::ExternalState(ext_state.clone()),
            relation_offset,
        ));
    }
    for (relation, _) in relations {
        arg_str.push(format!(
            "ref {}_sum_{}",
            relation.to_case(Case::Snake),
            *relation_offset
        ));
        *relation_offset += 1;
    }

    arg_str.push("ref sum".to_string());
    arg_str.push("domain_vanishing_eval_inv".to_string());
    arg_str.push("random_coeff".to_string());

    format!(
        "{}_{}(
            {}
        )",
        inline_fn,
        CONSTRAINT_EVAL_FUNCTION_NAME,
        arg_str.join(",\n")
    )
}
