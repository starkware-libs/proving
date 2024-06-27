use air_infra::core::autogen_structs::{ProcessedAirVar, TraceGenerationStep};
use genco::lang::rust;
use genco::quote;

/// Parses a `ProcessedAirVar` into a string for the write_trace function.
pub fn parse_air_var(expr: &ProcessedAirVar) -> String {
    match expr {
        ProcessedAirVar::Const(ty, val) => {
            format!("{}::from({})", ty, val)
        }
        ProcessedAirVar::Var(_, id) => id.to_string(),
        ProcessedAirVar::State(index) => {
            format!("col{}", index)
        }
        ProcessedAirVar::StaticCall(id, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_air_var(arg));
            }
            format!("{}({})", id, arg_str)
        }
        ProcessedAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_air_var(arg));
            }
            format!("{}.{}({})", parse_air_var(id), func, arg_str)
        }
        ProcessedAirVar::UnaryOp(op, expr) => {
            format!("{}({})", op, parse_air_var(expr))
        }
        ProcessedAirVar::BinaryOp(lhs, op, rhs) => {
            format!("({}) {} ({})", parse_air_var(lhs), op, parse_air_var(rhs))
        }
        ProcessedAirVar::Tuple(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&parse_air_var(expr));
            }
            format!("({})", expr_str)
        }
        ProcessedAirVar::Array(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&parse_air_var(expr));
            }
            format!("[{}]", expr_str)
        }
    }
}

/// Outputs the code for the write_trace function.
#[allow(dead_code)]
pub fn generate_write_trace_row_code(
    input: ProcessedAirVar,
    deductions: &[TraceGenerationStep],
) -> rust::Tokens {
    // Generate the parameters for the write_trace function.
    let mut write_trace_params = rust::Tokens::new();
    match input {
        ProcessedAirVar::Var(ty, id) => {
            write_trace_params.extend(quote! {
                $(id): $(ty)
            });
        }
        _ => panic!("Expected input to be a variable."),
    }

    // Generate the body of the write_trace function.
    let mut write_trace_body = rust::Tokens::new();
    let mut offset = 0;
    for deduction in deductions {
        match deduction {
            TraceGenerationStep::Deduction(expr) => {
                write_trace_body.append(quote! {
                    let col$(offset) = $(parse_air_var(expr));
                    dst[$(offset)][row_index] = col$(offset).into();
                });
                offset += 1;
            }
            TraceGenerationStep::Intermediate(name, expr) => {
                write_trace_body.extend(quote! {
                    let $(name) = $(parse_air_var(expr));
                });
            }
            TraceGenerationStep::Lookup {
                fn_name: _,
                input: _,
                output_name: _,
            } => todo!(),
        }
    }

    // Generate the final write_trace function.
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        #[allow(non_snake_case)]
        #[allow(clippy::useless_conversion)]
        pub fn write_trace_row(dst: &mut [Vec<BaseField>], $(write_trace_params), row_index: usize) {
            $(write_trace_body)
        }
    });
    code
}

#[allow(dead_code)]
pub fn generate_write_trace_code(
    component_name: &str,
    input: ProcessedAirVar,
    deductions: &[TraceGenerationStep],
) -> rust::Tokens {
    // Generate the imports for the write_trace function.
    let mut imports = rust::Tokens::new();
    imports.append(quote! {
        // TODO(Shahars): import only the necessary types.
        use air_infra::core::prover_types::*;
        use stwo_prover::core::{backend::cpu::CpuCircleEvaluation, fields::m31::BaseField, poly::{circle::CanonicCoset, BitReversedOrder}};
        use itertools::Itertools;
        use num_traits::Zero;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
        use stwo_prover::core::fields::m31::BaseField;
        use stwo_prover::core::poly::circle::CanonicCoset;
        use stwo_prover::core::poly::BitReversedOrder;

        use super::component::$component_name;

    });

    let input_type = match input {
        ProcessedAirVar::Var(ref ty, _) => ty,
        _ => panic!("Expected input to be a variable."),
    }
    .to_owned();

    // Generate the final write_trace function.
    let mut code = rust::Tokens::new();
    code.extend(quote! {
            $(imports)
            $['\n']
            pub fn write_trace(
                component: &$component_name,
                secrets: &[$input_type],
            ) -> Vec<CpuCircleEvaluation<BaseField, BitReversedOrder>> {
                let n_columns = component.trace_log_degree_bounds()[0].len();
                let mut trace_values = vec![vec![BaseField::zero(); secrets.len()]; n_columns];
                for (i, secret) in secrets.iter().enumerate() {
                    write_trace_row(&mut trace_values, *secret, i);
                }

                // TODO(Ohad): make this a function. support non-power of 2 inputs.
                let trace_domains = trace_values
                .iter()
                .map(|col| CanonicCoset::new(col.len().checked_ilog2().expect("Input not a power of 2!")).circle_domain())
                .collect_vec();
            std::iter::zip(trace_values, trace_domains)
            .map(|(eval, trace_domain)| {
                CpuCircleEvaluation::<BaseField, BitReversedOrder>::new(trace_domain, eval)
            })
            .collect_vec()
        }
        $['\n']
    });
    code.extend(generate_write_trace_row_code(input, deductions));
    code
}
