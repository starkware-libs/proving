use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, Intermediate, LookupTerm, TraceGenStep, UseOrYield,
};
use convert_case::{Case, Casing};
use genco::lang::{rust, Rust};
use genco::quote;
use itertools::{chain, Itertools};

use super::parse::{
    get_external_states_from_lookup_terms, get_public_params_from_lookup_terms, seek_consts,
};
use super::utils::{block_doc, contains_inputs, unique_relation_calls};
use crate::code_gen::SUPPORTED_PREPROCESSED_COLUMNS;

// TODO(Ohad): Refactor. build a 'auto-gen' struct from the lists, and have it generate the code.
pub fn generate_simd_claim_provers(lists: &CompiledAirFn) -> rust::Tokens {
    let contains_inputs = contains_inputs(lists);
    let public_params = get_public_params_from_lookup_terms(&lists.constraints);
    let configs = generate_configs(lists);
    let imports_code = generate_imports_code(&lists.deductions);
    let typedefs = if contains_inputs {
        generate_input_output_typedefs(lists)
    } else {
        quote! {}
    };
    let n_trace_cols = generate_n_trace_columns(lists);
    let lookup_data_code = generate_lookup_data_struct(&lists.deductions);
    let sub_components_inputs = generate_sub_components_inputs_struct(&lists.deductions);
    let claim_generator_code = generate_claim_generator_struct(&public_params, contains_inputs);
    let claim_generator_impl_code = generate_claim_generator_impl(lists, &public_params);
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
    // TODO(Gali): Get the variables of the PreprocessedColumn from air_infra.
    let external_states = get_external_states_from_lookup_terms(&lists.constraints);
    for (name, _) in external_states {
        assert!(
            SUPPORTED_PREPROCESSED_COLUMNS.contains(&name.as_str()),
            "unsupported {name}"
        );
        write_trace_body.append(quote! {
            let $(&name.to_lowercase()) = PreprocessedColumn::$name(log_size).packed_at(row_index);
        });
    }

    let mut relation_data_offsets = HashMap::new();
    for relation in unique_relation_calls(&lists.deductions) {
        relation_data_offsets.insert(relation, 0);
    }

    let mut add_inputs_lambda = rust::Tokens::new();
    for deduction in &lists.deductions {
        match deduction {
            TraceGenStep::Deduction(expr) => {
                let name = lists.state_names[offset].clone();
                write_trace_body.append(quote! {
                    let $(name.clone()) = $(simd_parse_air_var(expr,const_names));
                    *row[$(offset)] = $(name);
                });
                offset += 1;
            }
            TraceGenStep::Intermediate(Intermediate {
                name,
                r#type: _,
                var,
            }) => {
                write_trace_body.extend(quote! {
                    let $(name) = $(simd_parse_air_var(var,const_names));
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
                    *lookup_data.$(relation_name.to_case(Case::Snake))_$(*offset) = [$(felts)];
                };
                write_trace_body.extend(collect_felts);
                *offset += 1;
            }
            TraceGenStep::LookupAddInput { fn_name, input } => {
                let offset = add_inputs_offsets.get_mut(fn_name).unwrap();
                if input != &CompiledAirVar::Tuple(vec![]) {
                    write_trace_body.extend(quote! {
                        let $(fn_name)$(INPUTS_SUFFIX)_$(offset.to_string()) =
                            $(simd_parse_air_var(input, const_names)).unpack();

                    });
                    add_inputs_lambda.extend(quote! {
                        $(fn_name)_state.add_input(
                            &$(fn_name)$(INPUTS_SUFFIX)_$(offset.to_string())[i]
                        );
                    });
                }
                *offset += 1;
            }
        }
    }
    write_trace_body.extend(quote!(
        $['\n']$("// Add sub-components inputs.\n")
    ));
    write_trace_body.extend(quote! {
        #[allow(clippy::needless_range_loop)]
            for i in 0..N_LANES {
                if bit_reverse_index(
                    coset_index_to_circle_domain_index(row_index * N_LANES + i, log_size),
                    log_size,
                ) < n_rows
                {
                    $(add_inputs_lambda)
                }
            }

    });

    write_trace_body
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
    let (log_size_code, zip_inputs, for_each_variables) = if contains_inputs(lists) {
        (
            quote! {
                let log_n_packed_rows = inputs.len().ilog2();
                let log_size = log_n_packed_rows + LOG_N_LANES;
            },
            quote! { .zip(inputs.into_par_iter()) },
            quote! { ((row_index, row), $(&lists.name)_input) },
        )
    } else {
        (
            quote! {
                let log_size = n_rows.next_power_of_two().ilog2();
                let log_n_packed_rows = log_size - LOG_N_LANES;
            },
            quote! {},
            quote! { (row_index, row) },
        )
    };
    let mut code = rust::Tokens::new();
    code.extend(quote! {
        // TODO(Ohad): attempt to remove this.
        #[allow(clippy::useless_conversion)]
        #[allow(unused_variables)]
        #[allow(clippy::double_parens)]
        #[allow(non_snake_case)]
        fn write_trace_simd(
            $(generate_write_trace_simd_params(lists))
        ) -> (ComponentTrace<N_TRACE_COLUMNS>,
            SubComponentInputs,
            LookupData) {
            $(log_size_code)
            let (mut trace, mut lookup_data, mut sub_components_inputs) = unsafe {
                (
                    ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
                    LookupData::uninitialized(log_n_packed_rows),
                    SubComponentInputs::uninitialized(log_size),
                )
            };

            $(constants_def_code)

            trace
            .par_iter_mut()
            .enumerate()
            $(zip_inputs)
            .zip(lookup_data.par_iter_mut())
            .zip(sub_components_inputs.par_iter_mut().chunks(N_LANES))
            .for_each(
                |(($(for_each_variables), lookup_data), mut sub_components_inputs)| {
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
                TraceGenStep::Intermediate(Intermediate {
                    name: _,
                    r#type: _,
                    var,
                }) => {
                    const_defs.extend(seek_consts(var));
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

fn generate_claim_generator_struct(
    public_params: &[String],
    contains_inputs: bool,
) -> rust::Tokens {
    let mut claim_generator_fields = if contains_inputs {
        quote! { pub inputs: $(vec_of_type("InputType")), }
    } else {
        quote! { pub n_rows: usize, }
    };
    // TODO(Gali): Get the types of the public params from air_infra.
    for public_param in public_params {
        claim_generator_fields.extend(quote! { pub $(public_param): u32, });
    }
    quote! {
        #[derive(Default)]
        pub struct ClaimGenerator {
            $(claim_generator_fields)
        }
    }
}

fn generate_claim_prover_struct() -> rust::Tokens {
    quote! {

        pub struct InteractionClaimGenerator {
            n_rows: usize,
            lookup_data: LookupData,
        }
    }
}

fn generate_claim_generator_impl(lists: &CompiledAirFn, public_params: &[String]) -> rust::Tokens {
    let (mut claim_generator_fields, mut claim_generator_parameters, add_inputs_code, self_param) =
        if contains_inputs(lists) {
            (
                quote! { inputs, },
                quote! { inputs: Vec<InputType>, },
                quote! {
                    pub fn add_input(&self, input: &InputType,) {
                        $(add_input_simd_body())
                    }

                    // TODO(Ohad): consider removing this.
                    pub fn add_inputs (&self, inputs: &[InputType]) {
                        for input in inputs {
                            self.add_input(input);
                        }
                    }
                },
                quote! {mut self, },
            )
        } else {
            (
                quote! { n_rows, },
                quote! { n_rows: usize, },
                quote! {},
                quote! {self, },
            )
        };
    for public_param in public_params {
        claim_generator_fields.extend(quote! { $(public_param), });
        claim_generator_parameters.extend(quote! { $(public_param): u32, });
    }
    quote! {
        impl ClaimGenerator {
            pub fn new($(claim_generator_parameters)) -> Self {
                Self { $(claim_generator_fields) }
            }

            pub fn write_trace<MC: MerkleChannel>(
                $(self_param)
                tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
                $(generate_sub_component_params_and_args(&lists.deductions).0)
            ) -> (Claim, InteractionClaimGenerator)
            where
                SimdBackend: BackendForChannel<MC>
            {
                $(write_trace_body_simd(lists, public_params))
            }

            $(add_inputs_code)
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

fn generate_sub_component_params_and_args(
    deductions: &[TraceGenStep],
) -> (rust::Tokens, rust::Tokens) {
    // write_trace_simd is responsible for generating the trace and calling `add_inputs` on
    // sub_components.
    // Collect all the unique function and add_input calls.
    let mut context = chain![
        unique_function_calls(deductions),
        unique_add_input_calls(deductions)
    ]
    .collect_vec();
    context.sort_by_key(|a| a.clone());
    context.dedup();

    let mut params = rust::Tokens::new();
    let mut args = rust::Tokens::new();
    for fn_name in &context {
        params.extend(quote! {
            $(fn_name)$STATE_SUFFIX: &$(fn_name)::ClaimGenerator,
        });
        args.extend(quote! {
            $(fn_name)$STATE_SUFFIX,
        });
    }
    (params, args)
}

// TODO(Ohad): get that information from the air infra.
fn is_stateful(fn_name: &str) -> bool {
    fn_name.contains("mem")
}

// Generates the parameters for `write_trace_simd` function.
fn generate_write_trace_simd_params(lists: &CompiledAirFn) -> rust::Tokens {
    let mut params = quote! { n_rows: usize, };
    if contains_inputs(lists) {
        params.extend(quote! { inputs: $(vec_of_type("PackedInputType")), });
    }
    params.extend(generate_sub_component_params_and_args(&lists.deductions).0);
    for public_param in get_public_params_from_lookup_terms(&lists.constraints) {
        params.extend(quote! { $(public_param): u32, });
    }
    params
}

// Generates the arguments for `write_trace_simd` function.
fn generate_write_trace_simd_args(lists: &CompiledAirFn) -> rust::Tokens {
    let mut args = quote! { n_rows, };
    if contains_inputs(lists) {
        args.extend(quote! { packed_inputs, });
    }
    args.extend(generate_sub_component_params_and_args(&lists.deductions).1);
    for public_param in get_public_params_from_lookup_terms(&lists.constraints) {
        args.extend(quote! { self.$(public_param), });
    }
    args
}

fn write_trace_body_simd(lists: &CompiledAirFn, public_params: &[String]) -> rust::Tokens {
    let mut claim_fields = quote! {n_rows,};
    for public_param in public_params {
        claim_fields.extend(quote! {
            $(public_param): self.$(public_param),
        });
    }

    let (n_rows_init_code, inputs_code) = if contains_inputs(lists) {
        (
            quote! { let n_rows = self.inputs.len(); },
            quote! {
                if need_padding {
                    self.inputs.resize(size, *self.inputs.first().unwrap());
                    bit_reverse_coset_to_circle_domain_order(&mut self.inputs);
                }

                let packed_inputs = pack_values(&self.inputs);
            },
        )
    } else {
        (quote! { let n_rows = self.n_rows; }, quote! {})
    };
    quote! {
        $(n_rows_init_code)
        assert_ne!(n_rows, 0);
        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let need_padding = n_rows != size;

        $(inputs_code)
        let (trace, mut sub_components_inputs, lookup_data) =
                write_trace_simd($(generate_write_trace_simd_args(lists)));


        tree_builder.extend_evals(trace.to_evals());

        (
        Claim {
            $(claim_fields)
        },
        InteractionClaimGenerator {
            n_rows,
            lookup_data,
        },
        )
    }
}

// TODO(Ohad): add logic.
fn add_input_simd_body() -> rust::Tokens {
    quote! {
        unimplemented!("Implement manually");
    }
}

pub fn generate_sub_components_inputs_struct(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut members_code = quote! {};

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
    }

    quote! {
        #[derive(SubComponentInputs,Uninitialized,IterMut, ParIterMut)]
        pub struct SubComponentInputs
        {$(members_code)}
    }
}

pub fn generate_lookup_data_struct(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut members_code = quote! {};

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
        }
    }

    quote! {
        #[derive(Uninitialized,IterMut, ParIterMut)]
        struct LookupData
        {$(members_code)}
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
                let log_size = std::cmp::max(self.n_rows.next_power_of_two().ilog2(), LOG_N_LANES);
                let mut logup_gen = LogupTraceGenerator::new(log_size);

                $(generate_write_interaction_trace_body(deductions))

                let (trace, total_sum, claimed_sum) = if self.n_rows == 1 << log_size {
                    let (trace, claimed_sum) = logup_gen.finalize_last();
                    (trace, claimed_sum, None)
                } else {
                    let (trace, [total_sum, claimed_sum]) =
                        logup_gen.finalize_at([(1 << log_size) - 1, self.n_rows - 1]);
                    (trace, total_sum, Some((claimed_sum, self.n_rows - 1)))
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

        use air_structs_derive::SubComponentInputs;
        use itertools::{chain, zip_eq, Itertools};
        use num_traits::{One, Zero};
        use prover_types::cpu::*;
        use prover_types::simd::*;
        use rayon::iter::{
            IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
        };
        use stwo_air_utils::trace::component_trace::ComponentTrace;
        use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
        use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
        use stwo_prover::constraint_framework::preprocessed_columns::PreprocessedColumn;
        use stwo_prover::constraint_framework::Relation;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::backend::simd::column::BaseColumn;
        use stwo_prover::core::backend::simd::conversion::Unpack;
        use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
        use stwo_prover::core::backend::simd::qm31::PackedQM31;
        use stwo_prover::core::backend::simd::SimdBackend;
        use stwo_prover::core::backend::{BackendForChannel, Col, Column};
        use stwo_prover::core::channel::{Channel, MerkleChannel};
        use stwo_prover::core::fields::m31::M31;
        use stwo_prover::core::fields::FieldExpOps;
        use stwo_prover::core::pcs::TreeBuilder;
        use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
        use stwo_prover::core::poly::BitReversedOrder;
        use stwo_prover::core::utils::bit_reverse_index;
        use stwo_prover::core::utils::coset_index_to_circle_domain_index;
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
            if op == "inverse" {
                return format!("({}).inverse()", simd_parse_air_var(expr, constant_names));
            }
            format!("{}({})", op, simd_parse_air_var(expr, constant_names))
        }
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
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
        CompiledAirVar::ExternalState(name, _) => name.to_lowercase(),
        CompiledAirVar::PublicParam(public_param) => {
            format!("PackedM31::broadcast(M31::from({public_param}))")
        }
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
        CompiledAirVar::ExternalState(..) => append_type_prefix("M31"),
        _ => unimplemented!(),
    }
}

fn packed_name(ty: &str) -> String {
    format!("Packed{}", ty)
}

fn vec_of_type(ty: &str) -> String {
    format!("Vec<{}>", ty)
}
