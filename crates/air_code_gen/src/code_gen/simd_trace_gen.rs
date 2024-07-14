use air_infra::core::compiled_structs::{CompiledAirVar, TraceGenStep};
use genco::lang::rust;
use genco::quote;

use super::trace_gen::air_var_input_name;

pub fn generate_simd_trace_writer_code(
    component_name: &str,
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
    let imports_code = generate_imports_code(component_name);
    let struct_code = generate_simd_trace_gen_struct_code(component_name, input);
    let impl_code = generate_trace_gen_impl_code(component_name, input);
    let write_trace_code = generate_simd_write_trace_code(component_name, input, deductions);

    let mut code = rust::Tokens::new();
    code.extend(imports_code);
    code.extend(struct_code);
    code.extend(impl_code);
    code.extend(write_trace_code);
    code
}

/// Outputs the code for the write_trace function.
#[allow(dead_code)]
pub fn generate_simd_write_trace_row_code(
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
    // Generate the parameters for the write_trace function.
    let write_trace_params = quote! {
        $(air_var_input_name(input)): $(air_var_type_simd(input))
    };

    // Generate the body of the write_trace function.
    let mut write_trace_body = rust::Tokens::new();
    let mut offset = 0;
    for deduction in deductions {
        match deduction {
            TraceGenStep::Deduction(expr) => {
                write_trace_body.append(quote! {
                    let col$(offset) = $(simd_parse_air_var(expr));
                    dst[$(offset)][row_index] = col$(offset);
                });
                offset += 1;
            }
            TraceGenStep::Intermediate(name, expr) => {
                write_trace_body.extend(quote! {
                    let $(name) = $(simd_parse_air_var(expr));
                });
            }
            TraceGenStep::Lookup {
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
        pub fn write_trace_row(dst: &mut [Vec<PackedBaseField>], $(write_trace_params), row_index: usize) {
            $(write_trace_body)
        }
    });
    code
}

#[allow(dead_code)]
pub fn generate_simd_write_trace_code(
    component_name: &str,
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
    let input_type = parse_inputs_simd_type(input);

    // Generate the final write_trace function.
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        #[allow(clippy::ptr_arg)]
        pub fn write_trace_simd(
            component: &$component_name,
            secrets: &$(&input_type),
        ) -> Vec<CircleEvaluation<SimdBackend, Felt, BitReversedOrder>> {
            let n_trace_columns = component.trace_log_degree_bounds()[0].len();
            let mut trace_values = vec![vec![PackedBaseField::zero(); secrets.len()]; n_trace_columns];
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
                    CircleEvaluation::<SimdBackend, Felt, BitReversedOrder>::new(
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

pub fn generate_simd_trace_gen_struct_code(
    component_name: &str,
    input: &CompiledAirVar,
) -> rust::Tokens {
    let input_ty = parse_inputs_simd_type(input);
    let struct_name = trace_gen_struct_name(component_name, "Simd");
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        #[allow(non_camel_case_types)]
        #[derive(Default)]
        pub struct $(&struct_name) {
            pub inputs: $input_ty,
        }
        impl ComponentGen for $struct_name {}
        $['\n']
    });
    code
}

fn trace_gen_struct_name(component_name: &str, backend: &str) -> String {
    format!("{}{}TraceGenerator", component_name, backend)
}

pub fn generate_trace_gen_impl_code(component_name: &str, input: &CompiledAirVar) -> rust::Tokens {
    let struct_name = trace_gen_struct_name(component_name, "Simd");
    let inputs_ty = parse_inputs_simd_type(input);
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        impl TraceGenerator<SimdBackend> for $(&struct_name) {
            type Component = $component_name;
            type Inputs = $(&inputs_ty);

            fn write_trace(
                component_id: &str,
                registry: &mut ComponentGenerationRegistry,
            ) -> Vec<CircleEvaluation<SimdBackend, Felt, BitReversedOrder>> {
                let generator = registry.get_generator::<$(&struct_name)>(component_id);
                write_trace_simd(&generator.component(), &generator.inputs)
            }

            fn add_inputs(
                &mut self,
                inputs: &Self::Inputs,
            ) {
                $(add_inputs_simd_body());
            }

            // TODO(Ohad): extend this to support non-power of 2 inputs.
            fn component(&self) -> $component_name {
                $(to_component_simd_body(component_name))
            }
        }
        $['\n']
    });
    code
}

// TODO(Ohad): add logic.
fn add_inputs_simd_body() -> rust::Tokens {
    quote! {
        self.inputs.extend(inputs);
    }
}

// TODO(Ohad): add logic.
fn to_component_simd_body(component_name: &str) -> rust::Tokens {
    quote! {
        $component_name {
            log_n_instances : self.inputs.len().checked_ilog2().
                            expect("Input not a power of 2!")
                            + LOG_N_LANES,
        }
    }
}

fn generate_imports_code(component_name: &str) -> rust::Tokens {
    quote! {
        #![allow(unused_imports)]
        use std::iter::zip;

        use air_infra::core::prover_types::*;
        use itertools::Itertools;
        use num_traits::Zero;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::backend::simd::column::BaseFieldVec;
        use stwo_prover::core::backend::simd::m31::PackedBaseField;
        use stwo_prover::core::backend::simd::SimdBackend;
        use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
        use stwo_prover::core::poly::BitReversedOrder;
        use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
        use stwo_prover::trace_generation::{ComponentGen, TraceGenerator};

        use crate::code_gen::packed_types::*;
        use super::component::$component_name;
        $['\n']
    }
}

/// Parses a `CompiledAirVar` into a string for the write_trace function.
pub fn simd_parse_air_var(expr: &CompiledAirVar) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => {
            format!(
                "{}::broadcast({}::from({}).into())",
                packed_name(ty),
                ty,
                val
            )
        }
        CompiledAirVar::Var(_, id) => id.to_string(),
        CompiledAirVar::State(index) => {
            format!("col{}", index)
        }
        CompiledAirVar::StaticCall(id, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&simd_parse_air_var(arg));
            }
            format!("{}({})", id, arg_str)
        }
        CompiledAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&simd_parse_air_var(arg));
            }
            format!("{}.{}({})", simd_parse_air_var(id), func, arg_str)
        }
        CompiledAirVar::UnaryOp(op, expr) => {
            format!("{}({})", op, simd_parse_air_var(expr))
        }
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({}) {} ({})",
                simd_parse_air_var(lhs),
                op,
                simd_parse_air_var(rhs)
            )
        }
        CompiledAirVar::Tuple(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&simd_parse_air_var(expr));
            }
            format!("({})", expr_str)
        }
        CompiledAirVar::Array(exprs) => {
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

// Parses the collection type of the input.
// E.g. for a Felt input, it will return Vec<Felt>.
fn parse_inputs_simd_type(inputs_var: &CompiledAirVar) -> String {
    format!("Vec<{}>", air_var_type_simd(inputs_var))
}

pub fn air_var_type_simd(expr: &CompiledAirVar) -> String {
    match expr {
        CompiledAirVar::Const(ty, _) => packed_name(ty),
        CompiledAirVar::Var(ty, _) => packed_name(ty),
        CompiledAirVar::State(_) => packed_name("Felt"),
        CompiledAirVar::StaticCall(..) => {
            panic!("StaticCall not supported yet.")
        }
        CompiledAirVar::MethodCall(..) => {
            panic!("MethodCall not supported yet.")
        }
        CompiledAirVar::UnaryOp(..) => {
            panic!("UnaryOp not supported yet.")
        }
        CompiledAirVar::BinaryOp(..) => {
            panic!("BinaryOp not supported yet.")
        }
        CompiledAirVar::Tuple(tuple) => {
            let left_type = air_var_type_simd(&tuple[0]);
            let right_type = air_var_type_simd(&tuple[1]);
            format!("({}, {})", left_type, right_type)
        }
        CompiledAirVar::Array(arr) => {
            let ty = air_var_type_simd(&arr[0]);
            let len = arr.len();
            format!("[{};{}]", ty, len)
        }
    }
}
