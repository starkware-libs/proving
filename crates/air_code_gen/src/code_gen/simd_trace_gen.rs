use air_infra::core::autogen_structs::{ProcessedAirVar, TraceGenerationStep};
use genco::lang::rust;
use genco::quote;

/// Parses a `ProcessedAirVar` into a string for the write_trace function.
pub fn simd_parse_air_var(expr: &ProcessedAirVar) -> String {
    match expr {
        ProcessedAirVar::Const(ty, val) => {
            format!(
                "{}::broadcast({}::from({}).into())",
                packed_name(ty),
                ty,
                val
            )
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
                arg_str.push_str(&simd_parse_air_var(arg));
            }
            format!("{}({})", id, arg_str)
        }
        ProcessedAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&simd_parse_air_var(arg));
            }
            format!("{}.{}({})", simd_parse_air_var(id), func, arg_str)
        }
        ProcessedAirVar::UnaryOp(op, expr) => {
            format!("{}({})", op, simd_parse_air_var(expr))
        }
        ProcessedAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({}) {} ({})",
                simd_parse_air_var(lhs),
                op,
                simd_parse_air_var(rhs)
            )
        }
        ProcessedAirVar::Tuple(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&simd_parse_air_var(expr));
            }
            format!("({})", expr_str)
        }
        ProcessedAirVar::Array(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&simd_parse_air_var(expr));
            }
            format!("[{}]", expr_str)
        }
    }
}

fn packed_name(ty: &str) -> String {
    format!("Packed{}", ty)
}

/// Outputs the code for the write_trace function.
#[allow(dead_code)]
pub fn generate_simd_write_trace_row_code(
    input: ProcessedAirVar,
    deductions: &[TraceGenerationStep],
) -> rust::Tokens {
    // Generate the parameters for the write_trace function.
    let mut write_trace_params = rust::Tokens::new();
    match input {
        ProcessedAirVar::Var(ty, id) => {
            write_trace_params.extend(quote! {
                $(id): $(packed_name(&ty))
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
                    let col$(offset) = $(simd_parse_air_var(expr));
                    dst[$(offset)][row_index] = col$(offset);
                });
                offset += 1;
            }
            TraceGenerationStep::Intermediate(name, expr) => {
                write_trace_body.extend(quote! {
                    let $(name) = $(simd_parse_air_var(expr));
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
    // TODO(Ohad): remove the `allow(clippy::useless_conversion)` when stwo and infra felt type is
    // unified.
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        #[allow(non_snake_case)]
        #[allow(clippy::useless_conversion)]
        pub fn write_trace_row(dst: &mut [Vec<PackedBaseField>], $(write_trace_params), row_index: usize) {
            $(write_trace_body)
        }
    });
    code
}

#[allow(dead_code)]
pub fn generate_simd_write_trace_code(
    component_name: &str,
    input: ProcessedAirVar,
    deductions: &[TraceGenerationStep],
) -> rust::Tokens {
    // Generate the imports for the write_trace function.
    let mut imports = rust::Tokens::new();
    imports.append(quote! {
        // TODO(Shahars): import only the necessary types.
        use std::iter::zip;

        use air_infra::core::prover_types::*;
        use crate::code_gen::packed_types::*;
        use itertools::Itertools;
        use num_traits::Zero;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::backend::simd::column::BaseFieldVec;
        use stwo_prover::core::backend::simd::m31::PackedBaseField;
        use stwo_prover::core::backend::simd::SimdBackend;
        use stwo_prover::core::fields::m31::BaseField;
        use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
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
        pub fn write_trace_simd(
            component: &$component_name,
            secrets: &[$(packed_name(&input_type))],
        ) -> Vec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
            let n_columns = component.trace_log_degree_bounds().len();
            let mut trace_values = vec![vec![PackedBaseField::zero(); secrets.len()]; n_columns];
            for (i, secret) in secrets.iter().copied().enumerate() {
                super::simd_trace::write_trace_row(&mut trace_values, secret, i);
            }
            let trace_domains = trace_values
                .iter()
                .map(|col| CanonicCoset::new((col.len() * N_LANES)
                    .checked_ilog2()
                    .expect("Input not a power of 2!")).circle_domain())
                    .collect_vec();
            zip(trace_values, trace_domains)
                .map(|(eval, trace_domain)| {
                    let length = eval.len() * N_LANES;
                    let eval = BaseFieldVec{
                        data: eval,
                        length,
                    };
                    CircleEvaluation::<SimdBackend, BaseField, BitReversedOrder>::new(
                        trace_domain,
                        eval,
                    )
                })
                .collect_vec()
        }
        $['\n']
    });
    code.extend(generate_simd_write_trace_row_code(input, deductions));
    code
}
