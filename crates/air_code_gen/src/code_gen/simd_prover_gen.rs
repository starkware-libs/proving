use std::collections::HashMap;

use air_infra::core::compiled_structs::{CompiledAirFn, CompiledAirVar, TraceGenStep};
use genco::lang::rust;
use genco::quote;

use super::trace_gen::{air_var_input_name, generate_lookup_data_struct};
use super::utils::{n_trace_cells, unique_deduction_function_calls};
use crate::code_gen::trace_gen::generate_sub_component_imports;

// TODO(Ohad): Refactor. build a 'auto-gen' struct from the lists, and have it generate the code.
pub fn generate_simd_claim_provers(lists: &CompiledAirFn) -> rust::Tokens {
    let imports_code = generate_imports_code(&lists.deductions);
    let typedefs = generate_input_output_typedefs(lists);
    let lookup_data_code = generate_lookup_data_struct(&lists.deductions);
    let claim_generator_code = generate_claim_generator_struct();
    let claim_generator_impl_code = generate_claim_generator_impl(&lists.deductions);
    let claim_prover_code = generate_claim_prover_struct();
    let claim_prover_impl = generate_claim_prover_impl(&lists.deductions);
    let write_trace_code = generate_simd_write_trace_code(lists);

    quote! {
        $(imports_code)
        $['\n']
        $(typedefs)
        $['\n']
        $(claim_generator_code)
        $(claim_generator_impl_code)
        $['\n']
        $(write_trace_code)
        $['\n']
        $(lookup_data_code)
        $['\n']
        $(claim_prover_code)
        $(claim_prover_impl)
        $['\n']
    }
}

/// Outputs the code for the write_trace function.
#[allow(dead_code)]
fn generate_simd_write_trace_row_code(lists: &CompiledAirFn) -> rust::Tokens {
    // Generate the parameters for the write_trace_row function.
    let mut write_trace_row_params = quote! {
        dst: &mut [Col<SimdBackend, M31>],
        $(air_var_input_name(&lists.input)): InputType,
        row_index: usize,
        lookup_data: &mut LookupData,
    };
    write_trace_row_params.extend(generate_stateful_component_params(&lists.deductions));

    // Generate the body of the write_trace function.
    let mut write_trace_body = rust::Tokens::new();
    let mut offset = 0;
    let mut function_call_multiplicitiy = HashMap::new();
    for fn_call in unique_deduction_function_calls(&lists.deductions) {
        function_call_multiplicitiy.insert(fn_call, 0);
    }

    for deduction in &lists.deductions {
        match deduction {
            TraceGenStep::Deduction(expr) => {
                write_trace_body.append(quote! {
                    let col$(offset) = $(simd_parse_air_var(expr));
                    dst[$(offset)].data[row_index] = col$(offset);
                });
                offset += 1;
            }
            TraceGenStep::Intermediate(name, expr) => {
                write_trace_body.extend(quote! {
                    let $(name) = $(simd_parse_air_var(expr));
                });
            }
            TraceGenStep::Lookup {
                fn_name,
                input,
                output_name,
            } => {
                let multiplicity = function_call_multiplicitiy.get_mut(fn_name).unwrap();
                let input = simd_parse_air_var(input);
                write_trace_body.extend(quote! {
                    lookup_data
                        .$(&fn_name.to_lowercase())_inputs[$(multiplicity.to_string())]
                        .push($(&input).into());
                    let $(output_name) = $(fn_name.to_lowercase())::deduce_output(
                        $(input).into()
                    );
                    lookup_data
                        .$(fn_name.to_lowercase())_outputs[$(multiplicity.to_string())]
                        .push($(output_name).into());
                });
                *multiplicity += 1;
            }
            // TODO: Implement.
            TraceGenStep::AccessExternalColumn {
                fn_name: _,
                output_name: _,
            } => (),
        }
    }

    // Insert input..output values into lookup_data.
    write_trace_body.extend(quote! {
        $['\n']
        lookup_data.self_inputs.push($(air_var_input_name(&lists.input)));
        lookup_data.self_outputs.push($(simd_parse_air_var(&lists.output)).into());
    });

    // Generate the final write_trace_row function.
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        #[allow(clippy::useless_conversion)]
        fn write_trace_row(
            #[allow(unused_variables)]
            $(write_trace_row_params)){
            $(write_trace_body)
        }
    });
    code
}

#[allow(dead_code)]
fn generate_simd_write_trace_code(lists: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        pub fn write_trace_simd(
            inputs: $(vec_of_type("InputType")),
            $(generate_stateful_component_params(&lists.deductions))
        ) -> (Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>, LookupData) {
            let n_trace_columns = $(n_trace_cells(&lists.deductions));
            let mut trace_values = (0..n_trace_columns)
                .map(|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES))
                .collect_vec();
            let mut sub_components_inputs = LookupData::with_capacity(inputs.len());
            inputs.into_iter().enumerate().for_each(|(i, input)| {
                write_trace_row(
                    &mut trace_values,
                    input,
                    i,
                    &mut sub_components_inputs,
                );
            });

            let trace = trace_values
                .into_iter()
                .map(|eval| {
                 // TODO(Ohad): Support non-power of 2 inputs.
                    let domain = CanonicCoset::new(
                        eval.len()
                        .checked_ilog2()
                    .expect("Input is not a power of 2!"),
                )
            .circle_domain();
            CircleEvaluation::<SimdBackend, M31, BitReversedOrder>::new(domain, eval)
        })
        .collect_vec();

    (trace, sub_components_inputs)
        }
        $['\n']
    });
    code.extend(generate_simd_write_trace_row_code(lists));
    code
}

fn generate_input_output_typedefs(lists: &CompiledAirFn) -> rust::Tokens {
    let input_ty = generate_input_output_type(lists.input_num_of_felts);
    let output_ty = generate_input_output_type(lists.output_felts.len());
    quote! {
        pub type InputType = $(input_ty);
        pub type OutputType = $(output_ty);
    }
}

fn generate_claim_generator_struct() -> rust::Tokens {
    quote! {
        #[derive(Default)]
        pub struct ClaimGenerator {
            pub inputs: $(vec_of_type("InputType")),
        }
    }
}

fn generate_claim_prover_struct() -> rust::Tokens {
    quote! {

        pub struct ClaimProver {
            pub claim: Claim,
            pub lookup_data: LookupData,
        }
    }
}

fn generate_claim_generator_impl(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        impl ClaimGenerator {
            pub fn write_trace(
                self,
                tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
                $(generate_sub_component_params(deductions))
            ) -> ClaimProver {
                $(write_trace_body_simd(deductions))
            }

            pub fn add_inputs(
                &mut self,
                inputs: &[InputType],
            ) {
                $(add_inputs_simd_body());
            }
        }
    }
}

fn generate_sub_component_params(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut params = rust::Tokens::new();
    for fn_name in unique_deduction_function_calls(deductions).iter() {
        let fn_name = fn_name.to_lowercase();
        params.extend(quote! {
            $(&fn_name)_state: &mut $(fn_name)::ClaimGenerator,
        });
    }
    params
}

// If the component calls for memory, it needs to be passed to the write_trace function.
fn generate_stateful_component_params(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut params = rust::Tokens::new();
    for fn_name in unique_deduction_function_calls(deductions).iter() {
        // TODO(Ohad): get information about which function is stateful.
        let mut stateful = false;
        if fn_name.to_lowercase().contains("mem") {
            stateful = true;
        }

        if stateful {
            let fn_name = fn_name.to_lowercase();
            params.extend(quote! {
                $(&fn_name)_state: &mut $(fn_name)::ClaimGenerator,
            });
        }
    }
    params
}

fn generate_stateful_component_args(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut args = quote!(self.inputs);
    for fn_name in unique_deduction_function_calls(deductions).iter() {
        // TODO(Ohad): get information about which function is stateful.
        let mut stateful = false;
        if fn_name.to_lowercase().contains("mem") {
            stateful = true;
        }

        if stateful {
            args.extend(quote! {
                ,$(fn_name.to_lowercase())_state
            });
        }
    }
    args
}

fn generate_sub_component_add_inputs(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut statement = rust::Tokens::new();
    for fn_name in unique_deduction_function_calls(deductions).iter() {
        statement.extend(quote! {
            lookup_data.$(fn_name.to_lowercase())_inputs.iter().for_each(|inputs| {
                $(fn_name.to_lowercase())_state.add_inputs(inputs);
            });
        })
    }
    statement
}

fn write_trace_body_simd(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        let len = self.inputs.len();
        let (trace, lookup_data) =
                write_trace_simd($(generate_stateful_component_args(deductions)));
        $(generate_sub_component_add_inputs(deductions));

        tree_builder.extend_evals(trace);
        let claim = Claim {
            log_size: len.ilog2() + LOG_N_LANES,
            n_calls: len * N_LANES,
        };

        ClaimProver {
            claim,
            lookup_data,
        }
    }
}

// TODO(Ohad): add logic.
fn add_inputs_simd_body() -> rust::Tokens {
    quote! {
        self.inputs.extend(inputs);
    }
}

fn generate_claim_prover_impl(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut lookup_elements = quote! {self_lookup_elements: &ComponentLookupElements,};
    for fn_name in unique_deduction_function_calls(deductions).iter() {
        lookup_elements.extend(quote! {
            $(fn_name.to_lowercase())_lookup_elements: &$(fn_name.to_lowercase())::ComponentLookupElements,
        });
    }
    quote! {
        impl ClaimProver {
            // TODO(Ohad): Batch in pairs.
            pub fn write_interaction_trace(
                self,
                tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sMerkleChannel>,
                $(lookup_elements)
            ) -> InteractionClaim {
                let log_size = self.claim.log_size;
                let mut logup_gen = LogupTraceGenerator::new(log_size);

                $(generate_write_interaction_trace_body(deductions))

                // Self lookup.
                let mut col_gen = logup_gen.new_col();
                for (vec_row, (input, output)) in
                    zip_eq(self.lookup_data.self_inputs, self.lookup_data.self_outputs).enumerate()
                {
                    let lookup_values = input.concat(&output);
                    let denom = self_lookup_elements.combine(lookup_values.as_ref());
                    col_gen.write_frac(vec_row, PackedQM31::one(), denom);
                }
                col_gen.finalize_col();

                let (trace, claimed_sum) = logup_gen.finalize();
                tree_builder.extend_evals(trace);

                InteractionClaim { claimed_sum }
            }
        }
    }
}

fn generate_write_interaction_trace_body(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for fn_name in unique_deduction_function_calls(deductions) {
        code.extend(quote! {
            for (inputs, outputs) in zip_eq(
                self.lookup_data.$(&fn_name.to_lowercase())_inputs,
                self.lookup_data.$(&fn_name.to_lowercase())_outputs,
            ) {
                let mut col_gen = logup_gen.new_col();
                for (i, (input, output)) in zip_eq(inputs, outputs).enumerate() {
                    let lookup_values = input.concat(&output);
                    let denom = $(&fn_name.to_lowercase())_lookup_elements
                        .combine(lookup_values.as_ref());
                    col_gen.write_frac(i, PackedQM31::one(), denom);
                }
                col_gen.finalize_col();
            }
        });
    }

    code
}
fn generate_imports_code(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        #![allow(unused_imports)]
        use air_infra::core::prover_types::*;
        use itertools::{chain, zip_eq, Itertools};
        use num_traits::{One, Zero};
        use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
        use stwo_prover::core::backend::simd::qm31::PackedQM31;
        use stwo_prover::core::backend::simd::SimdBackend;
        use stwo_prover::core::backend::{Col, Column};
        use stwo_prover::core::fields::m31::M31;
        use stwo_prover::core::pcs::TreeBuilder;
        use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
        use stwo_prover::core::poly::BitReversedOrder;
        use stwo_prover::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};
        use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;

        use super::component::{Claim, ComponentLookupElements, InteractionClaim};
        use crate::AirFnIO;
        $(generate_sub_component_imports(deductions))
    }
}

/// Parses a `CompiledAirVar` into a string for the write_trace function.
fn simd_parse_air_var(expr: &CompiledAirVar) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => {
            format!(
                "{}::broadcast({}::from({}).into())",
                packed_name(ty),
                ty,
                val
            )
        }
        CompiledAirVar::Var(_, id) => {
            let id = id.to_lowercase();
            // Air infra expects M31, Input types are AirFnIO<_>.
            // TODO(Ohad): this is a hack.
            // it's possible to mark the input variables not in the name.
            match id.contains("_input") {
                true => id + ".into()",
                false => id,
            }
        }
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
            let func = if func == "as_felt" { "as_m31" } else { func };
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
        CompiledAirVar::Struct { .. } => {
            todo!()
        }
    }
}

fn packed_name(ty: &str) -> String {
    format!("Packed{}", ty)
}

fn vec_of_type(ty: &str) -> String {
    format!("Vec<{}>", ty)
}

pub fn generate_input_output_type(n: usize) -> String {
    format!("AirFnIO<{}>", n)
}
