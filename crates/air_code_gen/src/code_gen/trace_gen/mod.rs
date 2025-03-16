use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, CompiledIntermediate, LookupTerm, TraceGenStep, UseOrYield,
};
use compiled_casm_air::public_params::PublicParam;
use convert_case::{Case, Casing};
use genco::lang::{rust, Rust};
use genco::quote;
use itertools::Itertools;

use super::parse::seek_consts;
use super::utils::{block_doc, get_const_name, replace_generics_with_turbofish};
use crate::code_gen::SUPPORTED_PREPROCESSED_COLUMNS;

pub enum Mode {
    Opcode,
    Builtin,
    View,
}

pub struct RustProverGen {
    lists: CompiledAirFn,
    n_state_cells: usize,
    public_params: Vec<PublicParam>,
    write_trace_context: Vec<String>,
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

        let n_state_cells = match mode {
            Mode::Opcode => lists.state_names.len() + 1,
            _ => lists.state_names.len(),
        };

        let public_params = lists.public_params.iter().cloned().collect_vec();
        let write_trace_context = context(&lists.deductions);
        let constants = deduction_consts(&lists.deductions);
        let lookup_terms = filter_lookup_terms(&lists.deductions);
        let relation_calls = unique_relation_calls(&lookup_terms);

        Self {
            lists,
            mode,
            public_params,
            n_state_cells,
            write_trace_context,
            constants,
            relation_calls,
            lookup_terms,
        }
    }

    pub fn generate_simd_claim_prover(&self) -> rust::Tokens {
        let attributes = self.attributes();
        let imports_code = self.generate_imports_code();
        let typedefs = self.generate_input_output_typedefs();
        let n_trace_cols = self.generate_n_trace_columns();
        let lookup_data_code = self.generate_lookup_data_struct();
        let claim_generator_code = self.generate_claim_generator_struct();
        let claim_generator_impl_code = self.generate_claim_generator_impl();
        let interaction_struct = interaction_prover_struct(&self.mode);
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
                let (_name, ty, packed_ty) = &self.lists.prover_input;
                quote! {
                    pub type InputType = $ty;
                    pub type PackedInputType = $packed_ty;
                }
            }
        }
    }

    fn generate_n_trace_columns(&self) -> rust::Tokens {
        // Opcodes relation gets masked with an "Enabler" column.
        quote!(const N_TRACE_COLUMNS: usize = $(self.n_state_cells);)
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
                    $(write_trace_params(&self.write_trace_context))
                ) -> (Claim, InteractionClaimGenerator)
                where
                    SimdBackend: BackendForChannel<MC>
                {
                    $(self.write_trace_body_simd())
                }

                $(add_inputs_code)
            }
        }
    }

    fn write_trace_body_simd(&self) -> rust::Tokens {
        let mut claim_fields = quote! {log_size,};
        for public_param in &self.public_params {
            claim_fields.extend(quote! {
                $(public_param.name()): self.$(public_param.name()),
            });
        }

        let init_code = match self.mode {
            Mode::Opcode | Mode::View => quote! {
                let n_rows = self.inputs.len();
                assert_ne!(n_rows, 0);
                let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
                let log_size = size.ilog2();
                self.inputs.resize(size, *self.inputs.first().unwrap());
                let packed_inputs = pack_values(&self.inputs);
            },
            _ => quote! {
               let log_size = self.log_size;
            },
        };
        let n_rows = match self.mode {
            Mode::Opcode => quote! { n_rows, },
            _ => quote! {},
        };
        quote! {
            $(init_code)

            let (trace, lookup_data) =
                    write_trace_simd($(self.generate_write_trace_simd_args()));

            tree_builder.extend_evals(trace.to_evals());

            (
            Claim {
                $(claim_fields)
            },
            InteractionClaimGenerator {
                $(n_rows)
                log_size,
                lookup_data,
            },
            )
        }
    }

    // Generates the parameters for `write_trace_simd` function.
    fn generate_write_trace_simd_params(&self) -> rust::Tokens {
        let mut params = match self.mode {
            Mode::Opcode | Mode::View => {
                quote! { n_rows: usize, inputs: $(vec_of_type("PackedInputType")), }
            }
            _ => quote! { log_size: u32, },
        };
        for public_param in &self.public_params {
            params.extend(quote! { $(public_param.name()): u32, });
        }
        params.extend(write_trace_params(&self.write_trace_context));
        params
    }

    // Generates the arguments for `write_trace_simd` function.
    fn generate_write_trace_simd_args(&self) -> rust::Tokens {
        let mut args = match self.mode {
            Mode::Opcode | Mode::View => quote! { n_rows, packed_inputs, },
            _ => quote! { log_size, },
        };
        for public_param in &self.public_params {
            args.extend(quote! { self.$(public_param.name()), });
        }
        args.extend(write_trace_args(&self.write_trace_context));
        args
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
                    $(self.generate_write_trace_simd_params())
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

        let prelude_code = match self.mode {
            Mode::Builtin => quote! {
            let log_n_packed_rows = log_size - LOG_N_LANES;
            },
            _ => quote! {
            let log_n_packed_rows = inputs.len().ilog2();
            let log_size = log_n_packed_rows + LOG_N_LANES;
            },
        };

        let init_code = (
            quote! { mut trace, mut lookup_data},
            quote! {
                ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
                LookupData::uninitialized(log_n_packed_rows),
            },
        );

        let mut lambda_producer = (
            quote! {
                trace.par_iter_mut(),
                lookup_data.par_iter_mut(),
            },
            quote! {mut row, lookup_data,},
        );

        match self.mode {
            Mode::Opcode | Mode::View => {
                lambda_producer.0.extend(quote! {
                   inputs.into_par_iter(),
                });
                lambda_producer
                    .1
                    .extend(quote! { $(&self.lists.name)_input });
            }
            Mode::Builtin => {}
        }

        let opcode_mask = match self.mode {
            Mode::Opcode => quote!(let padding = Enabler::new(n_rows);),
            _ => quote!(),
        };

        let mut code = rust::Tokens::new();
        code.extend(quote! {
            // TODO(Ohad): attempt to remove this.
            #[allow(clippy::useless_conversion)]
            #[allow(unused_variables)]
            #[allow(clippy::double_parens)]
            #[allow(non_snake_case)]
            fn write_trace_simd(
                $(self.generate_write_trace_simd_params())
            ) -> (ComponentTrace<N_TRACE_COLUMNS>,
                LookupData) {
                $(prelude_code)
                let ($(init_code.0)) = unsafe {
                    ($(init_code.1))
                };

                $(constants_def_code)

                $(opcode_mask)

                ($(lambda_producer.0))
                .into_par_iter()
                .enumerate()
                .for_each(
                    |(row_index,($(lambda_producer.1)))| {
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
        for relation in &self.relation_calls {
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
                TraceGenStep::Intermediate(CompiledIntermediate {
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

        // Padding code.
        write_trace_body.extend(match self.mode {
            Mode::Opcode => quote! {
                *row[$(offset)] = padding.packed_at(row_index);
            },
            _ => quote!(),
        });

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

        let padding = if let Mode::Opcode = self.mode {
            quote!(let padding_col = Enabler::new(self.n_rows);)
        } else {
            quote!()
        };
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
                    $(padding)

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
            let masked_denom_0 = "denom0".to_owned() + mask_opcode_relation(relation1);
            let masked_denom_1 = "denom1".to_owned() + mask_opcode_relation(relation0);

            let relation0_offset = relation_data_offsets.get_mut(relation0).unwrap();
            let term0_offset = *relation0_offset;
            *relation0_offset += 1;

            let relation1_offset = relation_data_offsets.get_mut(relation1).unwrap();
            let term1_offset = *relation1_offset;
            *relation1_offset += 1;

            // Projective fraction addition (with numerator +-1).
            let (numerator, denom) = (
                match (term0.use_or_yield, term1.use_or_yield) {
                    (UseOrYield::Use, UseOrYield::Use) => {
                        format!("{masked_denom_0} + {masked_denom_1}")
                    }
                    (UseOrYield::Use, UseOrYield::Yield) => {
                        format!("{masked_denom_1} - {masked_denom_0}")
                    }
                    (UseOrYield::Yield, UseOrYield::Use) => {
                        format!("{masked_denom_0} - {masked_denom_1}")
                    }
                    (UseOrYield::Yield, UseOrYield::Yield) => {
                        format!("-({masked_denom_0}+ {masked_denom_1})")
                    }
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
                        col_gen.write_frac(
                            i, $(sign)PackedQM31::one()$(mask_opcode_relation(&relation_name)),
                            denom
                        );
                    }
                    col_gen.finalize_col();
                    $['\n']
            });
            *term_offset += 1;
        }
        code
    }

    fn generate_imports_code(&self) -> rust::Tokens {
        let mut sub_component_imports = rust::Tokens::new();
        self.write_trace_context.iter().for_each(|fn_name| {
            sub_component_imports.extend(quote! {
                use crate::components::$(fn_name);
            })
        });
        quote! {
            use crate::components::prelude::proving::*;
            use super::component::{Claim, InteractionClaim};
            $(sub_component_imports)
        }
    }
}

fn deduction_consts(deductions: &[TraceGenStep]) -> Vec<(String, String)> {
    deductions
        .iter()
        .fold(HashSet::new(), |mut const_defs, deductions| {
            match deductions {
                TraceGenStep::Deduction(expr, ..) => {
                    const_defs.extend(seek_consts(expr));
                }
                TraceGenStep::Intermediate(CompiledIntermediate {
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

fn interaction_prover_struct(mode: &Mode) -> rust::Tokens {
    // Opcodes mask is determined by the number of "real" instances.
    // Both log_size and n_rows is needed because padding might not be to the next power of 2.
    let n_rows = match mode {
        Mode::Opcode => quote! { n_rows: usize, },
        _ => quote! {},
    };
    quote! {
        pub struct InteractionClaimGenerator {
            $(n_rows)
            log_size: u32,
            lookup_data: LookupData,
        }
    }
}

const INPUTS_SUFFIX: &str = "_inputs";
const STATE_SUFFIX: &str = "_state";
fn write_trace_params(context: &[String]) -> rust::Tokens {
    let mut params = rust::Tokens::new();
    for fn_name in context {
        params.extend(quote! {
            $(fn_name)$STATE_SUFFIX: &$(fn_name)::ClaimGenerator,
        });
    }
    params
}

fn write_trace_args(context: &[String]) -> rust::Tokens {
    let mut args = rust::Tokens::new();
    for fn_name in context {
        args.extend(quote! {
            $(fn_name)$STATE_SUFFIX,
        });
    }
    args
}

// TODO(Ohad): add logic.
fn add_input_simd_body() -> rust::Tokens {
    quote! {
        unimplemented!("Implement manually");
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

fn packed_name(ty: &str) -> String {
    format!("Packed{}", ty)
}

fn vec_of_type(ty: &str) -> String {
    format!("Vec<{}>", ty)
}

fn contains_inputs(lists: &CompiledAirFn) -> bool {
    // No inputs is defined by an empty tuple.
    lists.prover_input.1 != "()"
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

// Returns the context of the write_trace function.
// e.g. opcodes needs `memory_address_to_id`.
fn context(deductions: &[TraceGenStep]) -> Vec<String> {
    deductions
        .iter()
        .filter_map(|d| match d {
            TraceGenStep::Deduction(CompiledAirVar::StaticCall(fn_name, ..))
            | TraceGenStep::Intermediate(CompiledIntermediate {
                var: CompiledAirVar::StaticCall(fn_name, ..),
                ..
            }) => {
                if fn_name.starts_with("Memory") {
                    Some(fn_name.split("::").next().unwrap().to_case(Case::Snake))
                } else {
                    None
                }
            }
            TraceGenStep::LookupAddInput { fn_name, .. } => Some(fn_name.to_string()),
            _ => None,
        })
        .sorted()
        .dedup()
        .collect()
}

fn unique_relation_calls(lookup_terms: &[LookupTerm]) -> Vec<String> {
    lookup_terms
        .iter()
        .map(|lookup_term| lookup_term.relation_name.clone())
        .sorted()
        .dedup()
        .collect()
}

pub fn mask_opcode_relation(relation_name: &str) -> &str {
    let is_opcode_relation = relation_name.eq("Opcodes");
    if is_opcode_relation {
        " * padding_col.packed_at(i)"
    } else {
        ""
    }
}
