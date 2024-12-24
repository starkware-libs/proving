use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, LookupTerm, TraceGenStep, UseOrYield,
};
use convert_case::{Case, Casing};
use genco::lang::{rust, Rust};
use genco::quote;
use itertools::{chain, Itertools};

use super::framework_gen::seek_consts;
use super::utils::{block_doc, unique_relation_calls};

// TODO(Ohad): Refactor. build a 'auto-gen' struct from the lists, and have it generate the code.
pub fn generate_simd_claim_provers(lists: &CompiledAirFn) -> rust::Tokens {
    let configs = generate_configs(lists);
    let imports_code = generate_imports_code(&lists.deductions);
    let typedefs = generate_input_output_typedefs(lists);
    let n_trace_cols = generate_n_trace_columns(lists);
    let lookup_data_code = generate_lookup_data_struct(&lists.deductions);
    let sub_components_inputs = generate_sub_components_inputs_struct(&lists.deductions);
    let claim_generator_code = generate_claim_generator_struct();
    let claim_generator_impl_code = generate_claim_generator_impl(&lists.deductions);
    let claim_prover_code = generate_claim_prover_struct();
    let claim_prover_impl = generate_claim_prover_impl(&lists.deductions);
    let write_trace_code = generate_simd_write_trace_code(lists);
    quote! {
        $(configs)
        $(imports_code)
        $['\n']
        $(typedefs)
        $(n_trace_cols)
        $['\n']
        $(claim_generator_code)
        $(claim_generator_impl_code)
        $['\n']
        $(sub_components_inputs)
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

const INPUTS_SUFFIX: &str = "_inputs";
const STATE_SUFFIX: &str = "_state";

// Generates the body of the write_trace function.
fn generate_simd_write_trace_body_code(
    lists: &CompiledAirFn,
    const_names: &HashMap<(String, String), String>,
) -> rust::Tokens {
    let mut write_trace_body = rust::Tokens::new();
    let mut offset = 0;
    let mut add_inputs_offsets = HashMap::new();
    for deduction in &lists.deductions {
        if let TraceGenStep::LookupAddInput { fn_name, .. } = deduction {
            add_inputs_offsets.insert(fn_name, 0);
        }
    }

    let mut relation_data_offsets = HashMap::new();
    for relation in unique_relation_calls(&lists.deductions) {
        relation_data_offsets.insert(relation, 0);
    }

    for deduction in &lists.deductions {
        match deduction {
            TraceGenStep::Deduction(expr) => {
                let name = lists.state_names[offset].clone();
                write_trace_body.append(quote! {
                    let $(name.clone()) = $(simd_parse_air_var(expr,const_names));
                    trace[$(offset)].data[row_index] = $(name);
                });
                offset += 1;
            }
            TraceGenStep::Intermediate(name, expr) => {
                write_trace_body.extend(quote! {
                    let $(name) = $(simd_parse_air_var(expr,const_names));
                });
            }
            TraceGenStep::LookupCall {
                fn_name,
                input,
                output_name,
            } => {
                let input = simd_parse_air_var(input, const_names);
                if let Some(output_name) = output_name {
                    let delimiter = if is_stateful(fn_name) {
                        STATE_SUFFIX.to_owned() + "."
                    } else {
                        "::".to_owned()
                    };
                    write_trace_body.extend(quote! {
                            let $(output_name) = $(fn_name)$(delimiter)deduce_output(
                                $(input)
                            );
                    });
                }
            }
            TraceGenStep::StartBlock(msg) => {
                write_trace_body.extend(block_doc(msg));
            }
            TraceGenStep::EndBlock => {
                write_trace_body.extend(quote!(
                    $['\n']
                ));
            }
            TraceGenStep::LookupTerm(LookupTerm {
                relation_name,
                felts,
                ..
            }) => {
                let offset = relation_data_offsets.get_mut(relation_name).unwrap();
                let felts = felts
                    .iter()
                    .map(|felt| simd_parse_air_var(felt, const_names))
                    .join(", ");
                let felts = &felts;
                let collect_felts = quote! {
                    // TODO(Ohad): change this to not vec.
                    lookup_data.$(relation_name.to_case(Case::Snake))_$(*offset).push([$(felts)]);
                };
                write_trace_body.extend(collect_felts);
                *offset += 1;
            }
            TraceGenStep::LookupAddInput { fn_name, input } => {
                let offset = add_inputs_offsets.get_mut(fn_name).unwrap();
                write_trace_body.extend(quote! {
                    sub_components$INPUTS_SUFFIX
                        .$(fn_name)$INPUTS_SUFFIX[$(offset.to_string())]
                        .extend($(simd_parse_air_var(input, const_names)).unpack());
                });
                *offset += 1;
            }
        }
        write_trace_body.extend(quote!(
            $("\n")
        ));
    }
    write_trace_body
}

// Removes trailing zeroes from a comma-separated sequence of M31 elements.
// Used to reduce 0 multiplications in the extension field.
pub fn remove_trailing_zeroes(mut felts: Vec<String>) -> Vec<String> {
    while felts
        .last()
        .is_some_and(|f| f.eq("M31_0") || f.eq("M31_0.clone()"))
    {
        felts.pop();
    }
    felts
}

#[allow(dead_code)]
fn generate_simd_write_trace_code(lists: &CompiledAirFn) -> rust::Tokens {
    let contains_deductions = !lists.state_names.is_empty();
    if !contains_deductions {
        return quote! {
        pub fn write_trace_simd() {
            unimplemented!()
        }};
    }

    // Declare constants.
    let mut constants_def_code = quote! {};
    let constants = deduction_consts(&lists.deductions);
    let mut const_names = HashMap::new();
    for (ty, val) in constants.into_iter() {
        let name = format!("{ty}_{val}");
        const_names.insert((ty.clone(), val.clone()), name.clone());
        constants_def_code.extend(quote! {
            let $(name) = $(packed_name(&ty))::broadcast($(ty)::from($(val)));
        });
    }

    let mut code = rust::Tokens::new();
    code.extend(quote! {
        #[allow(clippy::useless_conversion)]
        // TODO(Ohad): attempt to remove this.
        #[allow(unused_variables)]
        #[allow(clippy::double_parens)]
        #[allow(non_snake_case)]
        pub fn write_trace_simd(
            inputs: $(vec_of_type("PackedInputType")),
            $(generate_stateful_component_params(&lists.deductions))
        ) -> ([BaseColumn; N_TRACE_COLUMNS],
            SubComponentInputs,
            LookupData) {
            const N_TRACE_COLUMNS: usize = $(lists.state_names.len());
            let mut trace: [_ ;N_TRACE_COLUMNS]= std::array::from_fn
                    (|_| Col::<SimdBackend, M31>::zeros(inputs.len() * N_LANES));

            let mut lookup_data = LookupData::with_capacity(inputs.len());
            #[allow(unused_mut)]
            let mut sub_components_inputs = SubComponentInputs::with_capacity(inputs.len());

            $(constants_def_code)

            inputs.into_iter()
                .enumerate().for_each(|(row_index, $(&lists.name)_input)| {
                $(generate_simd_write_trace_body_code(lists,&const_names))
            });

            (trace, sub_components_inputs, lookup_data)
        }
        $['\n']
    });
    code
}

fn deduction_consts(deductions: &[TraceGenStep]) -> Vec<(String, String)> {
    deductions
        .iter()
        .fold(HashSet::new(), |mut const_defs, deductions| {
            match deductions {
                TraceGenStep::Deduction(expr, ..) => {
                    const_defs.extend(seek_consts(expr));
                }
                TraceGenStep::Intermediate(_, expr) => {
                    const_defs.extend(seek_consts(expr));
                }
                TraceGenStep::LookupTerm(LookupTerm {
                    relation_name: _,
                    felts,
                    ..
                }) => const_defs.extend(felts.iter().flat_map(seek_consts)),
                TraceGenStep::LookupCall {
                    fn_name: _, input, ..
                } => {
                    const_defs.extend(seek_consts(input));
                }
                TraceGenStep::StartBlock(_) => {}
                TraceGenStep::EndBlock => {}
                // TODO
                TraceGenStep::LookupAddInput { .. } => {}
            };
            const_defs
        })
        .into_iter()
        .sorted()
        .collect()
}

fn generate_input_output_typedefs(lists: &CompiledAirFn) -> rust::Tokens {
    quote! {
        pub type InputType = $(air_var_type(&lists.input, &mut |ty| quote!($ty)));
        pub type PackedInputType = $(air_var_type(&lists.input, &mut |ty| quote!(Packed$ty)));
    }
}

fn generate_n_trace_columns(lists: &CompiledAirFn) -> rust::Tokens {
    quote!(const N_TRACE_COLUMNS: usize = $(lists.state_names.len());)
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

        pub struct InteractionClaimGenerator {
            pub n_calls: usize,
            pub lookup_data: LookupData,
        }
    }
}

fn generate_claim_generator_impl(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        impl ClaimGenerator {
            pub fn new(inputs: Vec<InputType>) -> Self {
                Self { inputs }
            }

            pub fn write_trace<MC: MerkleChannel>(
                mut self,
                tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
                $(generate_sub_component_params(deductions))
            ) -> (Claim, InteractionClaimGenerator)
            where
                SimdBackend: BackendForChannel<MC>
            {
                $(write_trace_body_simd(deductions))
            }

            pub fn add_inputs(
                &mut self,
                inputs: &[InputType],
            ) {
                $(add_inputs_simd_body())
            }
        }
    }
}

fn unique_add_input_calls(deductions: &[TraceGenStep]) -> Vec<String> {
    deductions
        .iter()
        .filter_map(|d| {
            if let TraceGenStep::LookupAddInput { fn_name, .. } = d {
                Some(fn_name.to_string())
            } else {
                None
            }
        })
        .sorted()
        .dedup()
        .collect()
}

fn unique_function_calls(deductions: &[TraceGenStep]) -> Vec<String> {
    deductions
        .iter()
        .filter_map(|d| {
            if let TraceGenStep::LookupCall { fn_name, .. } = d {
                Some(fn_name.to_string())
            } else {
                None
            }
        })
        .sorted()
        .dedup()
        .collect()
}

fn generate_sub_component_params(deductions: &[TraceGenStep]) -> rust::Tokens {
    // write_trace is responsible for generating the trace calling `add_inputs` on sub_components.
    // Collect all the unique function and add_input calls.
    let mut context = chain![
        unique_function_calls(deductions),
        unique_add_input_calls(deductions)
    ]
    .collect_vec();
    context.sort_by_key(|a| a.clone());
    context.dedup();

    let mut params = rust::Tokens::new();
    for fn_name in context {
        params.extend(quote! {
            $(&fn_name)$STATE_SUFFIX: &mut $(fn_name)::ClaimGenerator,
        });
    }
    params
}

// TODO(Ohad): get that information from the air infra.
fn is_stateful(fn_name: &str) -> bool {
    fn_name.contains("mem")
}

// Generates the parameters for `write_trace_simd` function.
fn generate_stateful_component_params(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut params = rust::Tokens::new();
    // Does not need call add_inputs.
    for fn_name in unique_function_calls(deductions) {
        // TODO(Ohad): get information about which function is stateful.
        if is_stateful(&fn_name) {
            params.extend(quote! {
                $(&fn_name)$STATE_SUFFIX: &mut $(fn_name)::ClaimGenerator,
            });
        }
    }
    params
}

// Generates the arguments for `write_trace_simd` function.
fn generate_stateful_component_args(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut args = rust::Tokens::new();
    for fn_name in unique_function_calls(deductions) {
        // TODO(Ohad): get information about which function is stateful.
        if is_stateful(&fn_name) {
            args.extend(quote! {
                $(fn_name)$STATE_SUFFIX,
            });
        }
    }
    args
}

fn generate_sub_component_add_inputs(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut statement = rust::Tokens::new();
    for fn_name in unique_add_input_calls(deductions).iter() {
        statement.extend(quote! {
            sub_components$INPUTS_SUFFIX.$(fn_name)$INPUTS_SUFFIX.iter().for_each(|inputs| {
                $(fn_name)$STATE_SUFFIX.add_inputs(&inputs[..n_calls]);
            });
        })
    }
    statement
}

// TODO(Ohad): Padding.
fn write_trace_body_simd(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        let n_calls = self.inputs.len();
        assert_ne!(n_calls, 0);
        let size = std::cmp::max(n_calls.next_power_of_two(), N_LANES);
        let need_padding = n_calls != size;

        if need_padding {
            self.inputs.resize(size, *self.inputs.first().unwrap());
            bit_reverse_coset_to_circle_domain_order(&mut self.inputs);
        }

        let packed_inputs = pack_values(&self.inputs);
        let (trace, mut sub_components_inputs, lookup_data) =
                write_trace_simd(packed_inputs, $(generate_stateful_component_args(deductions)));

        if need_padding {
            sub_components_inputs.bit_reverse_coset_to_circle_domain_order();
        }
        $(generate_sub_component_add_inputs(deductions))

        tree_builder.extend_evals(
            trace
                .into_iter()
                .map(|eval| {
                    let domain = CanonicCoset::new(
                        eval.len()
                            .checked_ilog2()
                            .expect("Input is not a power of 2!"),
                    )
                    .circle_domain();
                    CircleEvaluation::<SimdBackend, M31, BitReversedOrder>::new(domain, eval)
                })
                .collect_vec(),
        );

        (
        Claim {
            n_calls
        },
        InteractionClaimGenerator {
            n_calls,
            lookup_data,
        },
        )
    }
}

// TODO(Ohad): add logic.
fn add_inputs_simd_body() -> rust::Tokens {
    quote! {
        self.inputs.extend(inputs);
    }
}

pub fn generate_sub_components_inputs_struct(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut members_code = quote! {};
    let mut initialization_code = quote! {};
    let mut bit_reverse_code = quote! {};

    let mut add_inputs_offsets = HashMap::new();
    for deduction in deductions {
        if let TraceGenStep::LookupAddInput { fn_name, .. } = deduction {
            let offset = add_inputs_offsets.entry(fn_name).or_insert(0);
            *offset += 1;
        }
    }

    for (&fn_name, &offset) in add_inputs_offsets.iter().sorted_by(|a, b| a.0.cmp(b.0)) {
        members_code.extend(quote! {
            pub $(fn_name.clone())$INPUTS_SUFFIX: [Vec<$(fn_name.clone())::InputType>; $(offset)],
        });
        let inner_vecs = (0..offset)
            .map(|_| quote! {Vec::with_capacity(capacity),})
            .collect_vec();
        initialization_code.extend(quote!($(fn_name.clone())$INPUTS_SUFFIX: [$(inner_vecs)],));
        bit_reverse_code.extend(quote! {
            self.$(fn_name)$INPUTS_SUFFIX
                .iter_mut()
                .for_each(|vec| bit_reverse_coset_to_circle_domain_order(vec));
        });
    }

    quote! {
        pub struct SubComponentInputs
        {$(members_code)}
        impl SubComponentInputs {
            #[allow(unused_variables)]
            fn with_capacity(capacity: usize) -> Self {
                Self {$(initialization_code)}
            }

            fn bit_reverse_coset_to_circle_domain_order(&mut self) {
                $(bit_reverse_code)
            }
        }
    }
}

pub fn generate_lookup_data_struct(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut members_code = quote! {};
    let mut initialization_code = quote! {};

    let mut relation_offsets = HashMap::new();
    for deduction in deductions {
        if let TraceGenStep::LookupTerm(LookupTerm {
            relation_name,
            felts,
            ..
        }) = deduction
        {
            let offset = relation_offsets
                .entry((relation_name, felts.len()))
                .or_insert(0);
            *offset += 1;
        }
    }

    for (&(relation_name, width), &n_relation_terms) in
        relation_offsets.iter().sorted_by(|a, b| a.0 .0.cmp(b.0 .0))
    {
        let relation_name = relation_name.to_case(Case::Snake);
        for offset in 0..n_relation_terms {
            let member_name = format!("{relation_name}_{offset}");
            members_code.extend(quote! {
                $(&member_name): Vec<[PackedM31; $width]>,
            });
            initialization_code.extend(quote!($(member_name): Vec::with_capacity(capacity),));
        }
    }

    quote! {
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

fn generate_claim_prover_impl(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut lookup_elements = quote! {};
    for relation_name in unique_relation_calls(deductions).iter() {
        lookup_elements.extend(quote! {
            $(relation_name.to_case(Case::Snake)):
                    &relations::$(relation_name),
        });
    }
    quote! {
        impl InteractionClaimGenerator {
            // TODO(Ohad): use partial sums.
            pub fn write_interaction_trace<MC: MerkleChannel>(
                self,
                tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
                $(lookup_elements)
            ) -> InteractionClaim
            where
                SimdBackend: BackendForChannel<MC>
            {
                let log_size = std::cmp::max(self.n_calls.next_power_of_two().ilog2(), LOG_N_LANES);
                let mut logup_gen = LogupTraceGenerator::new(log_size);

                $(generate_write_interaction_trace_body(deductions))

                let (trace, total_sum, claimed_sum) = if self.n_calls == 1 << log_size {
                    let (trace, claimed_sum) = logup_gen.finalize_last();
                    (trace, claimed_sum, None)
                } else {
                    let (trace, [total_sum, claimed_sum]) =
                        logup_gen.finalize_at([(1 << log_size) - 1, self.n_calls - 1]);
                    (trace, total_sum, Some((claimed_sum, self.n_calls - 1)))
                };
                tree_builder.extend_evals(trace);

                InteractionClaim {
                    logup_sums: (total_sum,claimed_sum)
                }
            }
        }
    }
}

fn generate_write_interaction_trace_body(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut relation_data_offsets = HashMap::new();
    for relation in unique_relation_calls(deductions) {
        relation_data_offsets.insert(relation, 0);
    }
    let mut code = rust::Tokens::new();
    let mut lookup_terms = deductions
        .iter()
        .filter_map(|d| {
            if let TraceGenStep::LookupTerm(lookup_data) = d {
                Some(lookup_data)
            } else {
                None
            }
        })
        .collect_vec();

    // Batching logup in pairs. `finalize_logup_in_pairs` assumes that the first 2N terms are
    // batched in pairs, and the remainder term is not batched.
    let remainder = match lookup_terms.len() % 2 {
        0 => None,
        1 => lookup_terms.pop(),
        _ => unreachable!(),
    };
    let pairs = lookup_terms.iter().tuples();

    if lookup_terms.len() >= 2 {
        code.extend(quote!($['\n']$("//")$(format!("Sum logup terms in pairs."))$("\n")));
    }
    for (term0, term1) in pairs {
        code.extend(quote!());
        let relation0 = &term0.relation_name;
        let relation1 = &term1.relation_name;
        let relation_0_snake_case = &relation0.to_case(Case::Snake);
        let relation_1_snake_case = &relation1.to_case(Case::Snake);

        let relation0_offset = relation_data_offsets.get_mut(relation0).unwrap();
        let term0_offset = *relation0_offset;
        *relation0_offset += 1;

        let relation1_offset = relation_data_offsets.get_mut(relation1).unwrap();
        let term1_offset = *relation1_offset;
        *relation1_offset += 1;

        // Projective fraction addition (with numerator +-1).
        let (numerator, denom) = (
            match (term0.use_or_yield, term1.use_or_yield) {
                (UseOrYield::Use, UseOrYield::Use) => "denom0 + denom1",
                (UseOrYield::Use, UseOrYield::Yield) => "denom1 - denom0",
                (UseOrYield::Yield, UseOrYield::Use) => "denom - denom1",
                (UseOrYield::Yield, UseOrYield::Yield) => "-(denom0 + denom1)",
            },
            "denom0 * denom1",
        );
        code.extend(quote! {
            let mut col_gen = logup_gen.new_col();
            for (i, (values0, values1)) in zip(
                &self.lookup_data
                            .$(relation_0_snake_case)_$(term0_offset),
                &self.lookup_data
                            .$(relation_1_snake_case)_$(term1_offset),
            )
            .enumerate()
            {
                let denom0: PackedQM31 = $(relation_0_snake_case).combine(values0);
                let denom1: PackedQM31 = $(relation_1_snake_case).combine(values1);
                col_gen.write_frac(i,$(numerator), $(denom));
            }
            col_gen.finalize_col();
            $['\n']
        });
    }

    // Handle odd remainder.
    if let Some(LookupTerm {
        relation_name,
        felts: _,
        use_or_yield,
    }) = remainder
    {
        let term_offset = relation_data_offsets.get_mut(relation_name).unwrap();
        let sign = match use_or_yield {
            UseOrYield::Use => "",
            UseOrYield::Yield => "-",
        };
        code.extend(quote! {
                $['\n']$("//")$(format!("Sum last logup term."))
                let mut col_gen = logup_gen.new_col();
                for (i, values) in self.lookup_data
                    .$(relation_name.to_case(Case::Snake))_$(*term_offset).iter().enumerate() {
                    let denom =
                        $(&relation_name.to_case(Case::Snake)).combine(values);
                    col_gen.write_frac(i, $(sign)PackedQM31::one(), denom);
                }
                col_gen.finalize_col();
                $['\n']
        });
        *term_offset += 1;
    }
    code
}

pub fn generate_sub_component_imports(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    let mut seen_functions = HashSet::new();
    for deduction in deductions {
        match deduction {
            TraceGenStep::LookupTerm(..) => {}
            TraceGenStep::LookupCall { fn_name, .. } => {
                if seen_functions.insert(fn_name) {
                    code.extend(quote! {
                        use crate::components::$(fn_name);
                    });
                }
            }
            TraceGenStep::StartBlock(_) => {}
            TraceGenStep::EndBlock => {}
            TraceGenStep::Deduction(..) => {}
            TraceGenStep::Intermediate(..) => {}
            // TODO
            TraceGenStep::LookupAddInput { fn_name, .. } => {
                if seen_functions.insert(fn_name) {
                    code.extend(quote! {
                        use crate::components::$(fn_name);
                    });
                }
            }
        }
    }
    code
}

fn generate_configs(lists: &CompiledAirFn) -> rust::Tokens {
    let mut configs = quote! {};
    if lists.name.contains("generic_opcode") {
        configs.extend(quote! {
            #![cfg_attr(rustfmt, rustfmt_skip)]
        });
    };
    configs.append(quote!(#![allow(unused_parens)]));
    configs
}

fn generate_imports_code(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        #![allow(unused_imports)]
        use std::iter::zip;

        use itertools::{chain, zip_eq, Itertools};
        use num_traits::{One, Zero};
        use prover_types::cpu::*;
        use prover_types::simd::*;
        use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
        use stwo_prover::constraint_framework::Relation;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::backend::BackendForChannel;
        use stwo_prover::core::backend::{Col, Column};
        use stwo_prover::core::backend::simd::column::BaseColumn;
        use stwo_prover::core::backend::simd::conversion::Unpack;
        use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
        use stwo_prover::core::backend::simd::qm31::PackedQM31;
        use stwo_prover::core::backend::simd::SimdBackend;
        use stwo_prover::core::channel::{Channel, MerkleChannel};
        use stwo_prover::core::fields::m31::M31;
        use stwo_prover::core::pcs::TreeBuilder;
        use stwo_prover::core::poly::BitReversedOrder;
        use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
        use stwo_prover::core::utils::bit_reverse_coset_to_circle_domain_order;

        use super::component::{Claim, InteractionClaim};
        use crate::components::pack_values;
        use crate::relations;
        $(generate_sub_component_imports(deductions))
    }
}

/// Parses a `CompiledAirVar` into a string for the write_trace function.
fn simd_parse_air_var(
    expr: &CompiledAirVar,
    constant_names: &HashMap<(String, String), String>,
) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => match ty.as_str() {
            // "usize" is used as index.
            // TODO(Ohad): ask anatg about this.
            "usize" => val.to_string(),
            _ => constant_names[&(ty.clone(), val.clone())].clone(),
        },
        CompiledAirVar::Var(_, id) => id.clone(),
        CompiledAirVar::State(name) => name.clone(),
        CompiledAirVar::StaticCall(id, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&simd_parse_air_var(arg, constant_names));
            }
            format!("Packed{}({})", id, arg_str)
        }
        CompiledAirVar::MethodCall(id, func, args) => {
            let func = if func == "as_felt" { "as_m31" } else { func };
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&simd_parse_air_var(arg, constant_names));
            }
            format!(
                "{}.{}({})",
                simd_parse_air_var(id, constant_names),
                func,
                arg_str
            )
        }
        CompiledAirVar::UnaryOp(op, expr) => {
            format!("{}({})", op, simd_parse_air_var(expr, constant_names))
        }
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            let non_native_div = op == "/"
                && air_var_type(lhs, &mut |ty| quote!($ty))
                    .to_string()
                    .unwrap()
                    == "M31";
            let op = if non_native_div { ".div" } else { op };
            format!(
                "(({}) {} ({}))",
                simd_parse_air_var(lhs, constant_names),
                op,
                simd_parse_air_var(rhs, constant_names)
            )
        }
        CompiledAirVar::Tuple(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&simd_parse_air_var(expr, constant_names));
            }
            format!("({})", expr_str)
        }
        CompiledAirVar::Array(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&simd_parse_air_var(expr, constant_names));
            }
            format!("[{}]", expr_str)
        }
        CompiledAirVar::Struct { r#type, fields } => {
            let members_code = fields
                .iter()
                .map(|(name, expr)| {
                    format!("{}: {}", name, simd_parse_air_var(expr, constant_names))
                })
                .collect::<Vec<_>>()
                .join(", ");
            let quote: genco::Tokens<Rust> = quote! {
                $(r#type) {
                    $(members_code),
                }
            };
            quote.to_string().unwrap()
        }
        CompiledAirVar::ExternalState(_name, _i) => "todo!()".to_string(),
        CompiledAirVar::PublicParam(_) => todo!(),
    }
}

pub fn air_var_type<F>(expr: &CompiledAirVar, append_type_prefix: &mut F) -> rust::Tokens
where
    F: FnMut(&str) -> rust::Tokens,
{
    match expr {
        CompiledAirVar::Const(ty, _) => append_type_prefix(ty),
        CompiledAirVar::Var(ty, _) => append_type_prefix(ty),
        CompiledAirVar::State(_) => append_type_prefix("M31"),
        CompiledAirVar::Tuple(tuple) => {
            let member_types = tuple
                .iter()
                .map(|var| air_var_type(var, append_type_prefix))
                .fold(rust::Tokens::new(), |mut member_types, t| {
                    member_types.append(quote!($t,));
                    member_types
                });
            quote!(($member_types))
        }
        CompiledAirVar::Array(arr) => {
            let ty = air_var_type(&arr[0], append_type_prefix);
            let len = arr.len();
            quote!([$ty; $len])
        }
        CompiledAirVar::Struct { r#type, fields } => {
            let members_code = fields
                .iter()
                .map(|(name, expr)| quote!($name: $(air_var_type(expr,append_type_prefix))))
                .fold(rust::Tokens::new(), |mut members_code, t| {
                    members_code.append(quote!($t,));
                    members_code
                });
            quote! {
                $(r#type) {
                    $(members_code),
                }
            }
        }
        _ => unimplemented!(),
    }
}

fn packed_name(ty: &str) -> String {
    format!("Packed{}", ty)
}

fn vec_of_type(ty: &str) -> String {
    format!("Vec<{}>", ty)
}
