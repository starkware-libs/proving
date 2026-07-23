use air_common::{CONSTRAINT_EVAL_FUNCTION_NAME, TraceType};
use air_compile::compiled_structs::{
    CompiledAirFn, CompiledAirVar, CompiledConstraintIntermediate, ConstraintEvalStep, LookupTerm,
};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;

pub fn parse_constraints(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    let mut relation_offset = 0;
    for constraint in air_fn.constraints.iter() {
        match constraint {
            ConstraintEvalStep::Constraint(c, desc) => {
                code.append(quote! {
                    $("\n")
                    $("//") Constraint - $(desc.clone().unwrap_or("".to_string()))
                    let constraint_quotient = ($(parse_var(air_fn, c, &mut relation_offset)));
                    sum = sum * random_coeff + constraint_quotient;
                });
            }
            ConstraintEvalStep::Intermediate(CompiledConstraintIntermediate {
                felt_names,
                var,
            }) => {
                match felt_names.len() {
                    0 =>
                    code.append(quote! { $(parse_var(air_fn, var, &mut relation_offset)); }),
                    1 => code.append(quote! {
                        let $(felt_names[0].clone()): QM31 = $(parse_var(air_fn, var, &mut relation_offset));
                    }),
                    _ => code.append(quote! {
                        let [$(felt_names.join(", "))] = $(parse_var(air_fn, var, &mut relation_offset));
                    })
                }
            }
            ConstraintEvalStep::LookupTerm(LookupTerm {
                relation_name,
                felts,
                use_or_yield: _,
                multiplicity
            }) => {
                let felts = felts
                    .iter()
                    .map(|f| parse_var(air_fn, f, &mut relation_offset))
                    .collect::<Vec<_>>();
                let relation_name = relation_name.to_case(Case::Snake);
                let lookup_elements = if air_fn.r#type == TraceType::Inline {
                    "common_lookup_elements"
                } else {
                    "self.common_lookup_elements"
                };
                code.append(quote! {
                    $("\n")
                    $(relation_name.clone())_sum_$(relation_offset) = $(lookup_elements).combine_qm31(
                        [
                            $(felts.join(",\n"))
                        ].span(),
                    );
                    numerator_$(relation_offset) = $(parse_var(air_fn, multiplicity, &mut relation_offset));
                });
                relation_offset += 1;
            }
        }
    }

    code
}

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
            if vars.len() == 1 {
                parse_var(air_fn, &vars[0], relation_offset)
            } else {
                let vars =
                    vars.iter().map(|v| parse_var(air_fn, v, relation_offset)).collect::<Vec<_>>();
                format!("[{}]", vars.join(", "))
            }
        }
        CompiledAirVar::ExternalState(col_id) => col_id.to_lowercase(),
        CompiledAirVar::PublicParam(name) => name.clone(),
        CompiledAirVar::Enabler => "enabler".to_string(),
        v => unimplemented!("Unsupported variable type: {v:?}"),
    }
}

fn gen_evaluate_call(
    air_fn: &CompiledAirFn,
    id: &str,
    args: &[CompiledAirVar],
    relation_offset: &mut usize,
) -> String {
    let mut arg_str =
        args.iter().map(|arg| parse_var(air_fn, arg, relation_offset)).collect::<Vec<_>>();

    let inline_fn = id.trim_end_matches(&format!("::{CONSTRAINT_EVAL_FUNCTION_NAME}"));
    let (relations, params, external_states) = air_fn.inline_calls.get(inline_fn).unwrap();
    if air_fn.r#type == TraceType::Inline {
        arg_str.push("common_lookup_elements".to_string());
    } else {
        arg_str.push("self.common_lookup_elements".to_string());
    }
    for param in params {
        arg_str.push(parse_var(
            air_fn,
            &CompiledAirVar::PublicParam(param.clone()),
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
        arg_str.push(format!("ref {}_sum_{}", relation.to_case(Case::Snake), *relation_offset));
        arg_str.push(format!("ref numerator_{}", *relation_offset));
        *relation_offset += 1;
    }

    arg_str.push("ref sum".to_string());
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
