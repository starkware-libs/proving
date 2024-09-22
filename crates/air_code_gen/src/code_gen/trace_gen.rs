use core::panic;
use std::collections::{HashMap, HashSet};

use air_infra::core::compiled_structs::{CompiledAirVar, TraceGenStep};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

pub fn generate_trace_writer_code(
    component_name: &str,
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
    let imports_code = generate_imports_code(component_name, deductions);
    let struct_code = generate_trace_gen_struct_code(component_name, input);
    let impl_code = generate_trace_gen_impl_code(component_name, input, deductions);
    let write_trace_code = generate_cpu_write_trace_code(component_name, input, deductions);

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
                fn_name,
                input,
                output_name,
            } => {
                write_trace_body.extend(quote! {
                    returned_inputs.$(fn_name)_inputs.push($(parse_air_var(input)));
                });
                if let Some(output_name) = output_name {
                    write_trace_body.extend(quote! {
                        let $(output_name) = $(fn_name)CpuTraceGenerator::deduce_output($(parse_air_var(input)));
                    });
                }
            }
        }
    }

    // Generate the final write_trace_row function.
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        #[allow(non_snake_case)]
        #[allow(clippy::useless_conversion)]
        #[allow(clippy::type_complexity)]
        fn write_trace_row(
            dst: &mut [Vec<M31>],
            $(write_trace_params),
            row_index: usize,
            #[allow(unused_variables)]
            returned_inputs: &mut ReturnedInputs) {
            $(write_trace_body)
        }
        $['\n']
    });
    code
}

// Generates the final write_trace function.
pub fn generate_cpu_write_trace_code(
    component_name: &str,
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
    let input_type = parse_inputs_cpu_type(input);

    let mut code = rust::Tokens::new();
    code.extend(quote! {
            $(generate_lookup_data_struct(deductions))

            #[allow(clippy::ptr_arg)]
            #[allow(clippy::type_complexity)]
            #[allow(clippy::let_unit_value)]
            pub fn write_trace_cpu(
                component: &$component_name,
                secrets: &$input_type,
            ) -> (Vec<CpuCircleEvaluation<M31, BitReversedOrder>>, ReturnedInputs)
                {
                let n_trace_columns = component.trace_log_degree_bounds()[0].len();
                let mut trace_values = vec![vec![M31::zero(); secrets.len()]; n_trace_columns];
                let mut sub_components_inputs = ReturnedInputs::with_capacity(secrets.len());
                secrets
                    .iter()
                    .enumerate()
                    .for_each(|(i, secret)| {
                        write_trace_row(&mut trace_values, *secret, i, &mut sub_components_inputs)
                    });

                // TODO(Ohad): make this a function. support non-power of 2 inputs.
                let trace = trace_values.into_iter()
                .map(|eval| {
                    let domain = CanonicCoset::new(eval.len().checked_ilog2().expect("Input not a power of 2!"))
                    .circle_domain();
                    CpuCircleEvaluation::<M31, BitReversedOrder>::new(domain, eval)
                })
                .collect_vec();

            (trace, sub_components_inputs)
        }
        $['\n']
    });
    code.extend(generate_write_trace_row_code(input, deductions));
    code
}

fn generate_write_trace_trait_body(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    code.extend(quote! {let generator = registry.get_generator::<Self>(component_id);});
    code.extend(quote! {
        #[allow(unused_variables)]
        let (trace, sub_component_inputs) =
            write_trace_cpu(&generator.component(), &generator.inputs);
    });

    let mut seen_functions = HashSet::new();
    for fn_name in deductions.iter().filter_map(|d| match d {
        TraceGenStep::Lookup { fn_name, .. } => {
            if seen_functions.insert(fn_name) {
                Some(fn_name)
            } else {
                None
            }
        }
        _ => None,
    }) {
        let component_id = format!("\"{}\"", fn_name);
        code.extend(quote! {
            registry.get_generator_mut::<$(fn_name)CpuTraceGenerator>($component_id)
                    .add_inputs(&sub_component_inputs.$(fn_name)_inputs);
        });
    }

    code.extend(quote! {
        trace
    });

    code
}

pub fn generate_lookup_data_struct(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut members_code = quote! {
        pub self_inputs: Vec<InputType>,
        pub self_outputs: Vec<OutputType>,
    };
    let mut initialization_code = quote! {
        self_inputs: Vec::with_capacity(capacity),
        self_outputs: Vec::with_capacity(capacity),
    };

    let mut function_call_multiplicity = HashMap::new();
    for deduction in deductions {
        if let TraceGenStep::Lookup { fn_name, .. } = deduction {
            let multiplicity = function_call_multiplicity.entry(fn_name).or_insert(0);
            *multiplicity += 1;
        }
    }

    for (fn_name, multiplicity) in function_call_multiplicity {
        let fn_name = fn_name.to_lowercase();
        members_code.extend(quote! {
            pub $(&fn_name)_inputs: [Vec<$(&fn_name)::InputType>; $(multiplicity)],
            pub $(&fn_name)_outputs: [Vec<$(&fn_name)::OutputType>; $(multiplicity)],
        });
        let inner_vecs = (0..multiplicity)
            .map(|_| quote! {Vec::with_capacity(capacity),})
            .collect_vec();
        initialization_code.extend(quote!($(&fn_name)_inputs: [$(inner_vecs.clone())],));
        initialization_code.extend(quote!($(&fn_name)_outputs: [$(inner_vecs)],));
    }

    quote! {
        #[allow(non_snake_case)]
        pub struct LookupData
        {$(members_code)}
        impl LookupData {
            #[allow(unused_variables)]
            fn with_capacity(capacity: usize) -> Self {
                Self {$(initialization_code)}

            }
        }
    }
}

fn generate_trace_gen_struct_code(component_name: &str, input: &CompiledAirVar) -> rust::Tokens {
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

fn generate_trace_gen_impl_code(
    component_name: &str,
    input: &CompiledAirVar,
    deductions: &[TraceGenStep],
) -> rust::Tokens {
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
            ) -> Vec<CpuCircleEvaluation<M31, BitReversedOrder>> {
                $(generate_write_trace_trait_body(deductions))
            }

            fn add_inputs(
                &mut self,
                inputs: &Self::Inputs,
            ) {
                $(add_inputs_body())
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

fn generate_imports_code(component_name: &str, deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        #![allow(unused_imports)]
        use air_infra::core::prover_types::*;
        use itertools::Itertools;
        use num_traits::Zero;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
        use stwo_prover::core::backend::CpuBackend;
        use stwo_prover::core::fields::m31::M31;
        use stwo_prover::core::poly::circle::CanonicCoset;
        use stwo_prover::core::poly::BitReversedOrder;
        use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
        use stwo_prover::trace_generation::{ComponentGen, TraceGenerator};

        use super::component::$component_name;
        $(generate_sub_component_imports(deductions))
        $['\n']
    }
}

pub fn generate_sub_component_imports(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    let mut seen_functions = HashSet::new();
    for deduction in deductions {
        if let TraceGenStep::Lookup { fn_name, .. } = deduction {
            if seen_functions.insert(fn_name) {
                code.extend(quote! {
                    use crate::$(fn_name.to_lowercase());
                });
            }
        }
    }
    code
}

fn air_var_type(expr: &CompiledAirVar) -> String {
    match expr {
        CompiledAirVar::Const(ty, _) => ty.to_string(),
        CompiledAirVar::Var(ty, _) => ty.to_string(),
        CompiledAirVar::State(_) => "M31".to_string(),
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
        CompiledAirVar::Struct { .. } => {
            todo!()
        }
        CompiledAirVar::ExternalState(..) => {
            todo!()
        }
    }
}

// Parses the collection type of the input.
// E.g. for a M31 input, it will return Vec<M31>.
fn parse_inputs_cpu_type(inputs_var: &CompiledAirVar) -> String {
    format!("Vec<{}>", air_var_type(inputs_var))
}

// Use only when we need the name of the input variable.
pub fn air_var_input_name(input_expr: &CompiledAirVar) -> String {
    match input_expr {
        CompiledAirVar::Var(_, id) => id.to_string().to_lowercase(),
        CompiledAirVar::Tuple(_) => {
            panic!("Tuple not supported yet.")
        }
        CompiledAirVar::Array(arr) => air_var_input_name(&arr[0])
            .split('[')
            .next()
            .unwrap()
            .to_string(),

        _ => panic!("Only Var and Array are supported."),
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
            let func = if func == "as_felt" { "as_m31" } else { func };
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
        CompiledAirVar::Struct { .. } => {
            todo!()
        }
        CompiledAirVar::ExternalState(..) => {
            todo!()
        }
    }
}

pub fn contains_lookup(deductions: &[TraceGenStep]) -> bool {
    for deduction in deductions {
        if let TraceGenStep::Lookup { .. } = deduction {
            return true;
        }
    }
    false
}
