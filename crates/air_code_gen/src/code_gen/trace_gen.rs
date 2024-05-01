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
pub fn gen_write_trace_code(
    input: ProcessedAirVar,
    deductions: &[TraceGenerationStep],
) -> rust::Tokens {
    // Generate the imports for the write_trace function.
    let mut imports = rust::Tokens::new();
    imports.append(quote! {
        // TODO(Shahars): import only the necessary types.
        use air_infra::core::prover_types::*;
        use stwo_prover::core::fields::m31::BaseField;
    });

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
        $(imports)
        $['\n']
        #[allow(non_snake_case)]
        pub fn write_trace_row(dst: &mut [Vec<BaseField>], $(write_trace_params), row_index: usize) {
            $(write_trace_body)
        }
    });
    code
}
