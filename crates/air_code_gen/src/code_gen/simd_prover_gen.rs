use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, Intermediate, LookupTerm, TraceGenStep, UseOrYield,
};
use compiled_casm_air::public_params::PublicParam;
use convert_case::{Case, Casing};
use genco::lang::{rust, Rust};
use genco::quote;
use itertools::Itertools;

use super::parse::seek_consts;
use super::utils::{
    block_doc, get_const_name, replace_generics_with_turbofish, unique_relation_calls,
};
use crate::code_gen::SUPPORTED_PREPROCESSED_COLUMNS;

pub enum Mode {
    Opcode,
    Builtin,
    View,
}

pub struct RustProverGen {
    lists: CompiledAirFn,
    public_params: Vec<PublicParam>,
    constants: Vec<(String, String)>,
    relation_calls: Vec<String>,
    lookup_terms: Vec<LookupTerm>,
    mode: Mode,
}
impl RustProverGen {
    pub fn new(lists: CompiledAirFn) -> Self {
        // TODO(Ohad): replace this predicate.
        let is_builtin = lists.name.contains("builtin");
        let is_opcode = lists.name.contains("opcode");

        // TODO(Gali): handle mults column.
        let mode = match (is_builtin, is_opcode) {
            (true, false) => Mode::Builtin,
            (false, true) => Mode::Opcode,
            (false, false) => {
                assert!(contains_inputs(&lists));
                Mode::View
            }
            (true, true) => panic!("unsupported mode"),
        };

        let public_params = lists.public_params.iter().cloned().collect_vec();
        let constants = deduction_consts(&lists.deductions);
        let relation_calls = unique_relation_calls(&lists.deductions);
        let lookup_terms = filter_lookup_terms(&lists.deductions);

        Self {
            lists,
            mode,
            public_params,
            constants,
            relation_calls,
            lookup_terms,
        }
    }

    pub fn generate_simd_claim_prover(&self) -> rust::Tokens {
        let attributes = self.attributes();
        let imports_code = generate_imports_code(&self.lists.deductions);
        let typedefs = self.generate_input_output_typedefs();
        let n_trace_cols = generate_n_trace_columns(&self.lists);
        let lookup_data_code = self.generate_lookup_data_struct();
        let claim_generator_code = self.generate_claim_generator_struct();
        let claim_generator_impl_code = self.generate_claim_generator_impl();
        let interaction_struct = interaction_prover_struct();
        let interaction_impl = self.generate_interaction_impl();
        let write_trace_code = self.generate_simd_write_trace_code();
        quote! {
            $(attributes)
            $(imports_code)
            $['\n']
            $(typedefs)
            $(n_trace_cols)
            $['\n']
            $(claim_generator_code)
            $(claim_generator_impl_code)
            $['\n']
            $(write_trace_code)
            $['\n']
            $(lookup_data_code)
            $['\n']
            $(interaction_struct)
            $(interaction_impl)
            $['\n']
        }
    }

    fn generate_input_output_typedefs(&self) -> rust::Tokens {
        match self.mode {
            // Builtins have no inputs.
            Mode::Builtin => quote!(),
            Mode::Opcode | Mode::View => {
                let input = &self.lists.input;
                quote! {
                    pub type InputType = $(air_var_type(input, &mut |ty| quote!($ty)));
                    pub type PackedInputType = $(air_var_type(input, &mut |ty| quote!(Packed$ty)));
                }
            }
        }
    }

    fn attributes(&self) -> rust::Tokens {
        let mut attributes = quote! {};
        attributes.append(quote!(#![allow(unused_parens)]));
        attributes.append(quote! { #![allow(unused_imports)] });
        if self.lists.name.contains("generic_opcode") {
            attributes.extend(quote! {
                #![cfg_attr(rustfmt, rustfmt_skip)]
            });
        };
        // TODO(Gali): Remove allow dead code.
        if self.lists.state_names.is_empty() {
            attributes.append(quote! { #![allow(dead_code)] });
        };

        attributes
    }

    fn generate_claim_generator_struct(&self) -> rust::Tokens {
        let mut claim_generator_fields = match self.mode {
            Mode::View | Mode::Opcode => quote! { pub inputs: $(vec_of_type("InputType")), },
            _ => quote! { pub log_size: u32, },
        };
        // TODO(Gali): Get the types of the public params from air_infra.
        for public_param in &self.public_params {
            claim_generator_fields.extend(quote! { pub $(public_param.name()): u32, });
        }
        quote! {
            #[derive(Default)]
            pub struct ClaimGenerator {
                $(claim_generator_fields)
            }
        }
    }

    fn generate_claim_generator_impl(&self) -> rust::Tokens {
        let (
            mut claim_generator_fields,
            mut claim_generator_parameters,
            add_inputs_code,
            self_param,
        ) = match self.mode {
            Mode::View => (
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
            ),
            Mode::Builtin => (
                quote! { log_size, },
                quote! { log_size: u32, },
                quote! {},
                quote! {self, },
            ),
            Mode::Opcode => (
                quote! { inputs, },
                quote! { inputs: Vec<InputType>, },
                quote! {},
                quote! {mut self, },
            ),
        };
        for public_param in &self.public_params {
            claim_generator_fields.extend(quote! { $(public_param.name()), });
            claim_generator_parameters.extend(quote! { $(public_param.name()): u32, });
        }
        quote! {
            impl ClaimGenerator {
                pub fn new($(claim_generator_parameters)) -> Self {
                    Self { $(claim_generator_fields) }
                }

                pub fn write_trace<MC: MerkleChannel>(
                    $(self_param)
                    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
                    $(generate_sub_component_params_and_args(&self.lists.deductions).0)
                ) -> (Claim, InteractionClaimGenerator)
                where
                    SimdBackend: BackendForChannel<MC>
                {
                    $(write_trace_body_simd(&self.lists))
                }

                $(add_inputs_code)
            }
        }
    }

    fn generate_lookup_data_struct(&self) -> rust::Tokens {
        let mut members_code = quote! {};

        let mut relation_offsets = HashMap::new();
        for LookupTerm {
            relation_name,
            felts,
            ..
        } in &self.lookup_terms
        {
            let offset = relation_offsets
                .entry((relation_name, felts.len()))
                .or_insert(0);
            *offset += 1;
        }

        for ((relation_name, width), &n_relation_terms) in
            relation_offsets.iter().sorted_by(|a, b| a.0 .0.cmp(b.0 .0))
        {
            let relation_name = relation_name.to_case(Case::Snake);
            for offset in 0..n_relation_terms {
                let member_name = format!("{relation_name}_{offset}");
                members_code.extend(quote! {
                    $(&member_name): Vec<[PackedM31; $(*width)]>,
                });
            }
        }

        quote! {
            #[derive(Uninitialized,IterMut, ParIterMut)]
            struct LookupData
            {$(members_code)}
        }
    }

    fn generate_simd_write_trace_code(&self) -> rust::Tokens {
        let contains_state_names = !self.lists.state_names.is_empty();
        if !contains_state_names {
            return quote! {
                fn write_trace_simd(
                    $(generate_write_trace_simd_params(&self.lists))
                ) -> (ComponentTrace<N_TRACE_COLUMNS>,
                    LookupData) {
                unimplemented!()
            }};
        }

        // declare constants.
        let mut constants_def_code = quote! {};
        let constants = deduction_consts(&self.lists.deductions);
        for (ty, val) in constants.into_iter() {
            let name = get_const_name(&ty, &val);
            constants_def_code.extend(quote! {
                let $(name) = $(replace_generics_with_turbofish(&packed_name(&ty)))::broadcast(
                    $(replace_generics_with_turbofish(&ty))::from($(val))
                );
            });
        }

        let (log_size_code, zip_inputs, for_each_variables) = match self.mode {
            Mode::Builtin => (
                quote! {
                    let log_n_packed_rows = log_size - LOG_N_LANES;
                },
                quote! {},
                quote! { (row_index, row) },
            ),
            _ => (
                quote! {
                    let log_n_packed_rows = inputs.len().ilog2();
                    let log_size = log_n_packed_rows + LOG_N_LANES;
                },
                quote! { .zip(inputs.into_par_iter()) },
                quote! { ((row_index, row), $(&self.lists.name)_input) },
            ),
        };

        let mut code = rust::Tokens::new();
        code.extend(quote! {
            // TODO(Ohad): attempt to remove this.
            #[allow(clippy::useless_conversion)]
            #[allow(unused_variables)]
            #[allow(clippy::double_parens)]
            #[allow(non_snake_case)]
            fn write_trace_simd(
                $(generate_write_trace_simd_params(&self.lists))
            ) -> (ComponentTrace<N_TRACE_COLUMNS>,
                LookupData) {
                $(log_size_code)
                let (mut trace, mut lookup_data) = unsafe {
                    (
                        ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
                        LookupData::uninitialized(log_n_packed_rows),
                    )
                };

                $(constants_def_code)

                trace
                .par_iter_mut()
                .enumerate()
                $(zip_inputs)
                .zip(lookup_data.par_iter_mut())
                .for_each(
                    |($(for_each_variables), lookup_data)| {
                        $(self.write_trace_lambda())
                    });

                (trace, lookup_data)
            }
            $['\n']
        });
        code
    }

    // Generates the body of the write_trace function.
    fn write_trace_lambda(&self) -> rust::Tokens {
        let const_names = &self
            .constants
            .iter()
            .map(|(ty, value)| ((ty.clone(), value.clone()), get_const_name(ty, value)))
            .collect_vec();
        let mut write_trace_body = rust::Tokens::new();
        let mut offset = 0;
        let mut add_inputs_offsets = HashMap::new();
        for deduction in &self.lists.deductions {
            if let TraceGenStep::LookupAddInput { fn_name, .. } = deduction {
                add_inputs_offsets.insert(fn_name, 0);
            }
        }
        for (name, _) in &self.lists.external_states {
            assert!(
                SUPPORTED_PREPROCESSED_COLUMNS.contains(&name.as_str()),
                "unsupported {name}"
            );
            write_trace_body.append(quote! {
                let $(&name.to_lowercase()) = $name::new(log_size).packed_at(row_index);
            });
        }

        let mut relation_data_offsets = HashMap::new();
        for relation in unique_relation_calls(&self.lists.deductions) {
            relation_data_offsets.insert(relation, 0);
        }

        let mut add_inputs_lambda = rust::Tokens::new();
        for deduction in &self.lists.deductions {
            match deduction {
                TraceGenStep::Deduction(expr) => {
                    let name = self.lists.state_names[offset].clone();
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
                            $(fn_name)_state.add_inputs(
                                &$(fn_name)$(INPUTS_SUFFIX)_$(offset.to_string())
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
        write_trace_body.extend(add_inputs_lambda);

        write_trace_body
    }

    fn generate_interaction_impl(&self) -> rust::Tokens {
        let lookup_elements = self
            .relation_calls
            .iter()
            .map(|relation_name| {
                quote! {
                    $(relation_name.to_case(Case::Snake)): &relations::$(relation_name),
                }
            })
            .fold(rust::Tokens::new(), |mut tokens, next| {
                tokens.extend(next);
                tokens
            });
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
                    let mut logup_gen = LogupTraceGenerator::new(self.log_size);

                    $(self.generate_write_interaction_trace_body())
                    let (trace, claimed_sum) = logup_gen.finalize_last();
                    tree_builder.extend_evals(trace);

                    InteractionClaim {
                        claimed_sum,
                    }
                }
            }
        }
    }

    fn generate_write_interaction_trace_body(&self) -> rust::Tokens {
        let mut relation_data_offsets = HashMap::new();
        for relation in &self.relation_calls {
            relation_data_offsets.insert(relation.clone(), 0);
        }
        let mut code = rust::Tokens::new();
        let mut lookup_terms = self.lookup_terms.clone();

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
            let term_offset = relation_data_offsets.get_mut(&relation_name).unwrap();
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
}

const INPUTS_SUFFIX: &str = "_inputs";
const STATE_SUFFIX: &str = "_state";

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

fn generate_n_trace_columns(lists: &CompiledAirFn) -> rust::Tokens {
    quote!(const N_TRACE_COLUMNS: usize = $(lists.state_names.len());)
}

fn interaction_prover_struct() -> rust::Tokens {
    quote! {

        pub struct InteractionClaimGenerator {
            log_size: u32,
            lookup_data: LookupData,
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

fn generate_sub_component_params_and_args(
    deductions: &[TraceGenStep],
) -> (rust::Tokens, rust::Tokens) {
    // write_trace_simd is responsible for generating the trace and calling `add_inputs` on
    // sub_components.
    // Collect all the unique function and add_input calls.
    let mut context = unique_add_input_calls(deductions);
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

// Generates the parameters for `write_trace_simd` function.
fn generate_write_trace_simd_params(lists: &CompiledAirFn) -> rust::Tokens {
    let mut params = if contains_inputs(lists) {
        quote! { n_rows: usize, inputs: $(vec_of_type("PackedInputType")), }
    } else {
        quote! { log_size: u32, }
    };
    for public_param in &lists.public_params {
        params.extend(quote! { $(public_param.name()): u32, });
    }
    params.extend(generate_sub_component_params_and_args(&lists.deductions).0);
    params
}

// Generates the arguments for `write_trace_simd` function.
fn generate_write_trace_simd_args(lists: &CompiledAirFn) -> rust::Tokens {
    let mut args = if contains_inputs(lists) {
        quote! { n_rows, packed_inputs, }
    } else {
        quote! { log_size, }
    };
    for public_param in &lists.public_params {
        args.extend(quote! { self.$(public_param.name()), });
    }
    args.extend(generate_sub_component_params_and_args(&lists.deductions).1);
    args
}

fn write_trace_body_simd(lists: &CompiledAirFn) -> rust::Tokens {
    let mut claim_fields = quote! {log_size,};
    for public_param in &lists.public_params {
        claim_fields.extend(quote! {
            $(public_param.name()): self.$(public_param.name()),
        });
    }

    let init_code = if contains_inputs(lists) {
        quote! {
            let n_rows = self.inputs.len();
            assert_ne!(n_rows, 0);
            let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
            let need_padding = n_rows != size;
            let log_size = size.ilog2();

            if need_padding {
                self.inputs.resize(size, *self.inputs.first().unwrap());
                bit_reverse_coset_to_circle_domain_order(&mut self.inputs);
            }

            let packed_inputs = pack_values(&self.inputs);
        }
    } else {
        quote! {
           let log_size = self.log_size;
        }
    };
    quote! {
        $(init_code)

        let (trace, lookup_data) =
                write_trace_simd($(generate_write_trace_simd_args(lists)));

        tree_builder.extend_evals(trace.to_evals());

        (
        Claim {
            $(claim_fields)
        },
        InteractionClaimGenerator {
            log_size,
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

pub fn generate_sub_component_imports(deductions: &[TraceGenStep]) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    let mut seen_functions = HashSet::new();
    for deduction in deductions {
        match deduction {
            TraceGenStep::LookupTerm(..) => {}
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

fn generate_imports_code(deductions: &[TraceGenStep]) -> rust::Tokens {
    quote! {
        use crate::components::prelude::proving::*;
        use super::component::{Claim, InteractionClaim};
        $(generate_sub_component_imports(deductions))
    }
}

/// Parses a `CompiledAirVar` into a string for the write_trace function.
fn simd_parse_air_var(
    expr: &CompiledAirVar,
    constant_names: &[((String, String), String)],
) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => match ty.as_str() {
            // "usize" is used as index.
            // TODO(Ohad): ask anatg about this.
            "usize" => val.to_string(),
            _ => constant_names
                .iter()
                .find(|((t, v), _)| t == ty && v == val)
                .map(|(_, name)| name.clone())
                .unwrap(),
        },
        CompiledAirVar::Var(_, id) => id.clone(),
        CompiledAirVar::State(name) => name.clone(),
        CompiledAirVar::StaticCall(id, args) => {
            // TODO(Ohad): get that information from the air infra.
            if id.starts_with("Memory") {
                let mut id = id.to_case(Case::Snake);
                id = id.replace("::", &format!("{STATE_SUFFIX}."));
                let input = simd_parse_air_var(&args[0], constant_names);
                return format!("{}({})", id, input);
            }

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
        CompiledAirVar::ExternalState(name, ..) => name.to_lowercase(),
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
        CompiledAirVar::ExternalState { .. } => append_type_prefix("M31"),
        _ => unimplemented!(),
    }
}

fn packed_name(ty: &str) -> String {
    format!("Packed{}", ty)
}

fn vec_of_type(ty: &str) -> String {
    format!("Vec<{}>", ty)
}

fn contains_inputs(lists: &CompiledAirFn) -> bool {
    // No inputs is defined by an empty tuple.
    if let CompiledAirVar::Tuple(inputs) = &lists.input {
        !inputs.is_empty()
    } else {
        true
    }
}

fn filter_lookup_terms(deductions: &[TraceGenStep]) -> Vec<LookupTerm> {
    deductions
        .iter()
        .filter_map(|d| {
            if let TraceGenStep::LookupTerm(lookup_data) = d {
                Some(lookup_data.clone())
            } else {
                None
            }
        })
        .collect()
}
