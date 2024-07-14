use air_infra::core::compiled_structs::{CompiledAirVar, TraceGenStep};
use genco::lang::rust;
use genco::quote;

pub fn generate_trace_writer_code(
    component_name: &str,
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
    let imports_code = generate_imports_code(component_name);
    let struct_code = generate_trace_gen_struct_code(component_name, input);
    let impl_code = generate_trace_gen_impl_code(component_name, input);
    let write_trace_code = generate_write_trace_code(component_name, input, deductions);

    let mut code = rust::Tokens::new();
    code.extend(imports_code);
    code.extend(struct_code);
    code.extend(impl_code);
    code.extend(write_trace_code);
    code
}

/// Outputs the code for the write_trace function.
#[allow(dead_code)]
pub fn generate_write_trace_row_code(
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
    // Generate the parameters for the write_trace function.
    let write_trace_params = quote! {
        $(air_var_input_name(input)): $(air_var_type(input))
    };

    // Generate the body of the write_trace function.
    let mut write_trace_body = rust::Tokens::new();
    let mut offset = 0;
    for deduction in deductions {
        match deduction {
            TraceGenStep::Deduction(expr) => {
                write_trace_body.append(quote! {
                    let col$(offset) = $(parse_air_var(expr));
                    dst[$(offset)][row_index] = col$(offset).into();
                });
                offset += 1;
            }
            TraceGenStep::Intermediate(name, expr) => {
                write_trace_body.extend(quote! {
                    let $(name) = $(parse_air_var(expr));
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
        fn write_trace_row(dst: &mut [Vec<BaseField>], $(write_trace_params), row_index: usize) {
            $(write_trace_body)
        }
        $['\n']
    });
    code
}

#[allow(dead_code)]
pub fn generate_write_trace_code(
    component_name: &str,
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
    let input_type = parse_inputs_cpu_type(input);

    // Generate the final write_trace function.
    let mut code = rust::Tokens::new();
    code.extend(quote! {
            #[allow(clippy::ptr_arg)]
            pub fn write_trace_cpu(
                component: &$component_name,
                secrets: &$input_type,
            ) -> Vec<CpuCircleEvaluation<BaseField, BitReversedOrder>> {
                let n_trace_columns = component.trace_log_degree_bounds()[0].len();
                let mut trace_values = vec![vec![BaseField::zero(); secrets.len()]; n_trace_columns];
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

pub fn generate_trace_gen_struct_code(
    component_name: &str,
    input: &CompiledAirVar,
) -> rust::Tokens {
    let input_ty = parse_inputs_cpu_type(input);
    let struct_name = trace_gen_struct_name(component_name, "Cpu");
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
    let struct_name = trace_gen_struct_name(component_name, "Cpu");
    let inputs_ty = parse_inputs_cpu_type(input);
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        impl TraceGenerator<CpuBackend> for $(&struct_name) {
            type Component = $component_name;
            type Inputs = $(&inputs_ty);

            fn write_trace(
                component_id: &str,
                registry: &mut ComponentGenerationRegistry,
            ) -> Vec<CpuCircleEvaluation<Felt, BitReversedOrder>> {
                let generator = registry.get_generator::<$(&struct_name)>(component_id);
                write_trace_cpu(&generator.component(), &generator.inputs)
            }

            fn add_inputs(
                &mut self,
                inputs: &Self::Inputs,
            ) {
                $(add_inputs_body());
            }

            // TODO(Ohad): extend this to support non-power of 2 inputs.
            fn component(&self) -> $component_name {
                $(&to_component_body(component_name))
            }
        }
        $['\n']
    });
    code
}

// TODO(Ohad): add logic.
fn add_inputs_body() -> rust::Tokens {
    quote! {
        self.inputs.extend(inputs);
    }
}

// TODO(Ohad): add logic.
fn to_component_body(component_name: &str) -> rust::Tokens {
    quote! {
        $component_name {
            log_n_instances : self.inputs.len().checked_ilog2().expect("Input not a power of 2!"),
        }
    }
}

fn generate_imports_code(component_name: &str) -> rust::Tokens {
    quote! {
        use air_infra::core::prover_types::*;
        use itertools::Itertools;
        use num_traits::Zero;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
        use stwo_prover::core::backend::CpuBackend;
        use stwo_prover::core::fields::m31::BaseField;
        use stwo_prover::core::poly::circle::CanonicCoset;
        use stwo_prover::core::poly::BitReversedOrder;
        use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
        use stwo_prover::trace_generation::{ComponentGen, TraceGenerator};
        use super::component::$component_name;
        $['\n']
    }
}

fn air_var_type(expr: &CompiledAirVar) -> String {
    match expr {
        CompiledAirVar::Const(ty, _) => ty.to_string(),
        CompiledAirVar::Var(ty, _) => ty.to_string(),
        CompiledAirVar::State(_) => "Felt".to_string(),
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
            let left_type = air_var_type(&tuple[0]);
            let right_type = air_var_type(&tuple[1]);
            format!("({}, {})", left_type, right_type)
        }
        CompiledAirVar::Array(arr) => {
            let ty = air_var_type(&arr[0]);
            let len = arr.len();
            format!("[{};{}]", ty, len)
        }
    }
}

// Parses the collection type of the input.
// E.g. for a Felt input, it will return Vec<Felt>.
fn parse_inputs_cpu_type(inputs_var: &CompiledAirVar) -> String {
    format!("Vec<{}>", air_var_type(inputs_var))
}

// Use only when we need the name of the input variable.
pub fn air_var_input_name(input_expr: &CompiledAirVar) -> String {
    match input_expr {
        CompiledAirVar::Var(_, id) => id.to_string(),
        CompiledAirVar::Tuple(_) => {
            panic!("Tuple not supported yet.")
        }
        CompiledAirVar::Array(arr) => air_var_input_name(&arr[0])
            .split('[')
            .next()
            .unwrap()
            .to_string(),

        _ => panic!("Only variables, tuples, arrays are supported as input."),
    }
}

/// Parses a `CompiledAirVar` into a string for the write_trace function.
pub fn parse_air_var(expr: &CompiledAirVar) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => {
            format!("{}::from({})", ty, val)
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
                arg_str.push_str(&parse_air_var(arg));
            }
            format!("{}({})", id, arg_str)
        }
        CompiledAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_air_var(arg));
            }
            format!("{}.{}({})", parse_air_var(id), func, arg_str)
        }
        CompiledAirVar::UnaryOp(op, expr) => {
            format!("{}({})", op, parse_air_var(expr))
        }
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            format!("({}) {} ({})", parse_air_var(lhs), op, parse_air_var(rhs))
        }
        CompiledAirVar::Tuple(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&parse_air_var(expr));
            }
            format!("({})", expr_str)
        }
        CompiledAirVar::Array(exprs) => {
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
