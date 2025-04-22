use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, CompiledIntermediate, LookupTerm, PaddingType, TraceGenStep,
    TraceType, UseOrYield,
};
use compiled_casm_air::public_params::PublicParam;
use convert_case::{Case, Casing};
use genco::lang::{rust, Rust};
use genco::quote;
use indexmap::IndexMap;
use itertools::Itertools;

use super::parse::{is_masked_relation, seek_consts};
use super::utils::{block_doc, get_variable_name, replace_generics_with_turbofish};
use crate::code_gen::parse::is_const_size_component;

pub enum Mode {
    NoInputs,
    Inputs, // TODO(Gali): Unite with PackedInputs.
    PackedInputs,
    Mults,
}

pub struct RustProverGen {
    lists: CompiledAirFn,
    public_params: Vec<PublicParam>,
    write_trace_context: Vec<String>,
    constants: Vec<(String, String)>,
    relation_calls: Vec<String>,
    add_input_mults: IndexMap<String, usize>,
    lookup_terms: Vec<LookupTerm>,
    mode: Mode,
}
impl RustProverGen {
    pub fn new(lists: CompiledAirFn) -> Self {
        let supported_paddings = [
            PaddingType::None,
            PaddingType::Enabler,
            PaddingType::Multiplicity,
        ];
        assert!(
            supported_paddings.contains(&lists.padding_type),
            "unsupported padding type"
        );

        let mode = match lists.r#type {
            TraceType::Builtin | TraceType::Const => Mode::NoInputs,
            TraceType::ChainRound => Mode::PackedInputs,
            TraceType::Component => {
                if lists.padding_type == PaddingType::Multiplicity {
                    Mode::Mults
                } else {
                    Mode::PackedInputs
                }
            }
            TraceType::Opcode | TraceType::Memory | TraceType::Inline => Mode::Inputs,
        };

        let public_params = lists.public_params.iter().cloned().collect_vec();
        let write_trace_context = context(&lists.deductions);
        let constants = deduction_consts(&lists.deductions);
        let add_input_mults = add_inputs_mults(&lists.deductions);
        let lookup_terms = filter_lookup_terms(&lists.deductions);
        let relation_calls = lists.lookup_names.keys().cloned().collect::<Vec<_>>();

        Self {
            lists,
            mode,
            public_params,
            write_trace_context,
            add_input_mults,
            constants,
            relation_calls,
            lookup_terms,
        }
    }

    pub fn generate_witness_code(&self) -> rust::Tokens {
        let attributes = self.attributes();
        let imports_code = self.generate_imports_code();
        let typedefs = self.generate_input_output_typedefs();
        let lookup_data_code = self.generate_lookup_data_struct();
        let sub_component_inputs_struct = self.generate_sub_component_inputs_struct();
        let claim_generator_code = self.generate_claim_generator_struct();
        let claim_generator_impl_code = self.generate_claim_generator_impl();
        let interaction_struct = interaction_prover_struct(&self.lists);
        let interaction_impl = self.generate_interaction_impl();
        let write_trace_code = self.generate_simd_write_trace_code();
        quote! {
            $(attributes)
            $(imports_code)
            $['\n']
            $(typedefs)
            $['\n']
            $(claim_generator_code)
            $(claim_generator_impl_code)
            $['\n']
            $sub_component_inputs_struct
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
        let (_name, ty, packed_ty) = &self.lists.prover_input;
        match self.mode {
            Mode::NoInputs => quote!(),
            Mode::PackedInputs => {
                quote! {
                    pub type PackedInputType = $packed_ty;
                }
            }
            Mode::Inputs | Mode::Mults => {
                quote! {
                    pub type InputType = $ty;
                    pub type PackedInputType = $packed_ty;
                }
            }
        }
    }

    fn attributes(&self) -> rust::Tokens {
        let mut attributes = quote! {};
        attributes.append(quote!(#![allow(unused_parens)]));
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

    fn generate_sub_component_inputs_struct(&self) -> rust::Tokens {
        if !self.contains_sub_components() {
            return quote! {};
        }
        let members = self
            .add_input_mults
            .iter()
            .map(|(component_name, &mult)| {
                let component_name = component_name.to_lowercase();
                quote! {
                    $(&component_name): [Vec<$component_name::PackedInputType>; $mult],
                }
            })
            .collect_vec();

        quote! {
            #[derive(Uninitialized, IterMut, ParIterMut)]
            struct SubComponentInputs {
                $members
            }
        }
    }

    fn generate_claim_generator_struct(&self) -> rust::Tokens {
        let mut claim_generator_fields = match self.mode {
            Mode::NoInputs => quote! { pub log_size: u32, },
            Mode::PackedInputs => {
                quote! { pub packed_inputs: $(vec_of_type("PackedInputType")), }
            }
            Mode::Inputs => quote! { pub inputs: $(vec_of_type("InputType")), },
            Mode::Mults => quote! { pub mults: AtomicMultiplicityColumn, },
        };
        // TODO(Gali): Get the types of the public params from air_infra.
        for public_param in &self.public_params {
            claim_generator_fields.extend(quote! { pub $(public_param.name()): u32, });
        }
        let derive_default = if self.lists.padding_type == PaddingType::Multiplicity {
            quote! {}
        } else {
            quote! { #[derive(Default)] }
        };
        quote! {
            $derive_default
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
            Mode::NoInputs => (
                quote! { log_size, },
                quote! { log_size: u32, },
                quote! {},
                quote! {self, },
            ),
            Mode::Inputs => (
                quote! { inputs, },
                quote! { inputs: $(vec_of_type("InputType")), },
                quote! {},
                quote! {mut self, },
            ),
            Mode::PackedInputs => (
                quote! { packed_inputs, },
                quote! { packed_inputs: $(vec_of_type("PackedInputType")), },
                quote! {
                    pub fn add_packed_inputs(&mut self, inputs: &[PackedInputType]) {
                        self.packed_inputs.extend(inputs);
                    }
                },
                quote! {mut self, },
            ),
            Mode::Mults => (
                quote! { mults: AtomicMultiplicityColumn::new(1 << LOG_SIZE), },
                quote! {},
                quote! {
                pub fn add_input(&self, _input: &InputType) {
                    todo!() // Implement manually
                }

                pub fn add_packed_inputs(&self, packed_inputs: &[PackedInputType]) {
                    packed_inputs.into_par_iter().for_each(|packed_input| {
                        packed_input.unpack().into_iter().for_each(|input| {
                            self.add_input(&input);
                        });
                    });
                }},
                quote! {self, },
            ),
        };
        let new_without_default = if self.lists.padding_type == PaddingType::Multiplicity {
            quote! {#[allow(clippy::new_without_default)]}
        } else {
            quote! {}
        };
        for public_param in &self.public_params {
            claim_generator_fields.extend(quote! { $(public_param.name()), });
            claim_generator_parameters.extend(quote! { $(public_param.name()): u32, });
        }
        quote! {
            impl ClaimGenerator {
                $new_without_default
                pub fn new($(claim_generator_parameters)) -> Self {
                    Self { $(claim_generator_fields) }
                }

                pub fn write_trace(
                    $(self_param)
                    tree_builder: &mut impl TreeBuilder<SimdBackend>,
                    $(write_trace_params(&self.write_trace_context))
                ) -> (Claim, InteractionClaimGenerator)
                {
                    $(self.write_trace_body_simd())
                }

                $(add_inputs_code)
            }
        }
    }

    fn write_trace_body_simd(&self) -> rust::Tokens {
        let mut claim_fields = if is_const_size_component(&self.lists) {
            quote! {}
        } else {
            quote! {log_size,}
        };
        for public_param in &self.public_params {
            claim_fields.extend(quote! {
                $(public_param.name()): self.$(public_param.name()),
            });
        }

        let init_code = match self.mode {
            Mode::NoInputs => quote! {
               let log_size = self.log_size;
            },
            Mode::PackedInputs => quote! {
                assert!(!self.packed_inputs.is_empty());
                let n_vec_rows = self.packed_inputs.len();
                let n_rows = n_vec_rows * N_LANES;
                let packed_size = n_vec_rows.next_power_of_two();
                let log_size = packed_size.ilog2() + LOG_N_LANES;
                self.packed_inputs.resize(packed_size, *self.packed_inputs.first().unwrap());
            },
            Mode::Inputs => quote! {
                let n_rows = self.inputs.len();
                assert_ne!(n_rows, 0);
                let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
                let log_size = size.ilog2();
                self.inputs.resize(size, *self.inputs.first().unwrap());
                let packed_inputs = pack_values(&self.inputs);
            },
            Mode::Mults => quote! {
                let mults = self.mults.into_simd_vec();
            },
        };
        let mut interaction_claim_fields = match self.lists.padding_type {
            PaddingType::Enabler => quote! { n_rows, },
            _ => quote! {},
        };
        if !is_const_size_component(&self.lists) {
            interaction_claim_fields.extend(quote! { log_size, });
        }

        let sub_component_inputs = if self.contains_sub_components() {
            quote! { sub_component_inputs }
        } else {
            quote! {}
        };
        let add_inputs = self
            .add_input_mults
            .iter()
            .map(|(component_name, ..)| {
                let component_name = component_name.to_lowercase();
                quote! { sub_component_inputs.$(&component_name).iter().for_each(|inputs| {
                    $component_name$STATE_SUFFIX.add_packed_inputs(inputs);
                });}
            })
            .collect_vec();
        quote! {
            $(init_code)

            let (trace, lookup_data, $sub_component_inputs) =
                    write_trace_simd($(self.generate_write_trace_simd_args()));
            $add_inputs
            tree_builder.extend_evals(trace.to_evals());

            (
            Claim {
                $(claim_fields)
            },
            InteractionClaimGenerator {
                $(interaction_claim_fields)
                lookup_data,
            },
            )
        }
    }

    // Generates the parameters for `write_trace_simd` function.
    fn generate_write_trace_simd_params(&self) -> rust::Tokens {
        let mut params = match self.mode {
            Mode::NoInputs => quote! { log_size: u32, },
            Mode::PackedInputs | Mode::Inputs => {
                quote! { inputs: $(vec_of_type("PackedInputType")), }
            }
            Mode::Mults => quote! { mults: $(vec_of_type("PackedM31")), },
        };
        if self.lists.padding_type == PaddingType::Enabler {
            params.extend(quote! { n_rows: usize, })
        }
        for public_param in &self.public_params {
            params.extend(quote! { $(public_param.name()): u32, });
        }
        params.extend(write_trace_params(&self.write_trace_context));
        params
    }

    // Generates the arguments for `write_trace_simd` function.
    fn generate_write_trace_simd_args(&self) -> rust::Tokens {
        let mut args = match self.mode {
            Mode::NoInputs => quote! { log_size, },
            Mode::PackedInputs => quote! { self.packed_inputs, },
            Mode::Inputs => quote! { packed_inputs, },
            Mode::Mults => quote! { mults, },
        };
        if self.lists.padding_type == PaddingType::Enabler {
            args.extend(quote! { n_rows, })
        }
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
        if self.lists.padding_type == PaddingType::Multiplicity {
            members_code.extend(quote! { mults: $(vec_of_type("PackedM31")), })
        };

        quote! {
            #[derive(Uninitialized,IterMut, ParIterMut)]
            struct LookupData
            {$(members_code)}
        }
    }

    fn generate_simd_write_trace_code(&self) -> rust::Tokens {
        let mut return_tuple = (
            quote! { ComponentTrace<N_TRACE_COLUMNS>, LookupData, },
            quote! {trace, lookup_data, },
        );
        if self.contains_sub_components() {
            return_tuple.0.extend(quote! {SubComponentInputs, });
            return_tuple.1.extend(quote! {sub_component_inputs, });
        };
        let contains_state_names = !self.lists.state_names.is_empty();
        if !contains_state_names {
            return quote! {
                fn write_trace_simd(
                    $(self.generate_write_trace_simd_params())
                ) -> ($(return_tuple.0)) {
                unimplemented!()
            }};
        }

        // declare constants.
        let mut constants_def_code = quote! {};
        let constants = deduction_consts(&self.lists.deductions);
        for (ty, val) in constants.into_iter() {
            let name = get_variable_name(&ty, &val);
            constants_def_code.extend(quote! {
                let $(name) = $(replace_generics_with_turbofish(&packed_name(&ty)))::broadcast(
                    $(replace_generics_with_turbofish(&ty))::from($(val))
                );
            });
        }

        let log_size = if is_const_size_component(&self.lists) {
            quote! { LOG_SIZE }
        } else {
            quote! {log_size}
        };

        let mut preprocessed_def_code = quote! {};
        for (name, args) in &self.lists.external_states {
            // Seq is the only preprocessed column that is of unfixed size.
            if name == "Seq" {
                preprocessed_def_code.extend(quote! {
                    let seq = Seq::new($(&log_size));
                });
            } else {
                preprocessed_def_code.extend(quote! {
                    let $(&get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())) = $name::new($(args.join(", ")));
                });
            }
        }

        let prelude_code = match self.mode {
            Mode::NoInputs | Mode::Mults => quote! {
            let log_n_packed_rows = $(&log_size) - LOG_N_LANES;
            },
            _ => quote! {
            let log_n_packed_rows = inputs.len().ilog2();
            let log_size = log_n_packed_rows + LOG_N_LANES;
            },
        };

        let mut init_code = (
            quote! { mut trace, mut lookup_data,},
            quote! {
                ComponentTrace::<N_TRACE_COLUMNS>::uninitialized($log_size),
                LookupData::uninitialized(log_n_packed_rows),
            },
        );

        let mut lambda_producer = (
            quote! {
                trace.par_iter_mut(),
                lookup_data.par_iter_mut(),
            },
            quote! {mut row, lookup_data, },
        );

        if self.contains_sub_components() {
            init_code.0.extend(quote! { mut sub_component_inputs, });
            init_code.1.extend(quote! {
                SubComponentInputs::uninitialized(log_n_packed_rows),
            });
            lambda_producer.0.extend(quote! {
                sub_component_inputs.par_iter_mut(),
            });
            lambda_producer.1.extend(quote! {
                sub_component_inputs,
            });
        };

        match self.mode {
            Mode::NoInputs | Mode::Mults => {}
            _ => {
                lambda_producer.0.extend(quote! {
                   inputs.into_par_iter(),
                });
                lambda_producer
                    .1
                    .extend(quote! { $(&self.lists.name)_input, });
            }
        }

        let padding = match self.lists.padding_type {
            PaddingType::Enabler => quote!(let enabler_col = Enabler::new(n_rows);),
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
            ) -> ($(return_tuple.0)) {
                $(prelude_code)
                let ($(init_code.0)) = unsafe {
                    ($(init_code.1))
                };

                $(constants_def_code)
                $(preprocessed_def_code)
                $(padding)

                ($(lambda_producer.0))
                .into_par_iter()
                .enumerate()
                .for_each(
                    |(row_index,($(lambda_producer.1)))| {
                        $(self.write_trace_lambda())
                    });

                ($(return_tuple.1))
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
            .map(|(ty, value)| ((ty.clone(), value.clone()), get_variable_name(ty, value)))
            .collect_vec();
        let mut write_trace_body = rust::Tokens::new();
        let mut offset = 0;
        let mut add_inputs_offsets = HashMap::new();
        for deduction in &self.lists.deductions {
            if let TraceGenStep::LookupAddInput { fn_name, .. } = deduction {
                add_inputs_offsets.insert(fn_name, 0);
            }
        }
        for (name, args) in &self.lists.external_states {
            if name == "Seq" {
                write_trace_body.append(quote! {
                    let $(&name.to_lowercase()) = $(&name.to_lowercase()).packed_at(row_index);
                });
            } else {
                write_trace_body.append(quote! {
                let $(&get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())) = $(&get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())).packed_at(row_index);
                });
            }
        }

        let mut relation_data_offsets = HashMap::new();
        for relation in &self.relation_calls {
            relation_data_offsets.insert(relation, 0);
        }
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
                            *sub_component_inputs.$(fn_name)[$(offset.to_string())] =
                                $(simd_parse_air_var(input, const_names));

                        });
                    }
                    *offset += 1;
                }
            }
        }

        // Padding code.
        write_trace_body.extend(match self.lists.padding_type {
            PaddingType::Enabler => quote! {
                *row[$(offset)] = enabler_col.packed_at(row_index);
            },
            _ => quote!(),
        });

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

        let padding = match self.lists.padding_type {
            PaddingType::Enabler => quote! {let enabler_col = Enabler::new(self.n_rows);},
            _ => quote! {},
        };
        let log_size = if is_const_size_component(&self.lists) {
            quote! {LOG_SIZE}
        } else {
            quote! {self.log_size}
        };
        quote! {
            impl InteractionClaimGenerator {
                // TODO(Ohad): use partial sums.
                pub fn write_interaction_trace(
                    self,
                    tree_builder: &mut impl TreeBuilder<SimdBackend>,
                    $(lookup_elements)
                ) -> InteractionClaim
                {
                    $(padding)
                    let mut logup_gen = LogupTraceGenerator::new($log_size);

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
            let masked_denom_0 = quote! {denom0 $(mask_relation(&self.lists, relation1))};
            let masked_denom_1 = quote! {denom1 $(mask_relation(&self.lists, relation0))};

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
                        quote! {$masked_denom_0 + $masked_denom_1}
                    }
                    (UseOrYield::Use, UseOrYield::Yield) => {
                        quote! {$masked_denom_1 - $masked_denom_0}
                    }
                    (UseOrYield::Yield, UseOrYield::Use) => {
                        quote! {$masked_denom_0 - $masked_denom_1}
                    }
                    (UseOrYield::Yield, UseOrYield::Yield) => {
                        quote! {-($masked_denom_0 + $masked_denom_1)}
                    }
                },
                "denom0 * denom1",
            );
            let (for_each, enumerate) = if self.lists.padding_type == PaddingType::Enabler
                && (is_masked_relation(&self.lists, relation0)
                    || is_masked_relation(&self.lists, relation1))
            {
                (
                    quote! { (i, (writer, values0, values1)) },
                    quote! {.enumerate()},
                )
            } else {
                (quote! { (writer, values0, values1)}, quote! {})
            };
            code.extend(quote! {
                let mut col_gen = logup_gen.new_col();
                (col_gen.par_iter_mut(),
                &self.lookup_data.$(relation_0_snake_case)_$(term0_offset),
                &self.lookup_data.$(relation_1_snake_case)_$(term1_offset))
                .into_par_iter()$enumerate.for_each(|$for_each| {
                let denom0: PackedQM31 = $(relation_0_snake_case).combine(values0);
                let denom1: PackedQM31 = $(relation_1_snake_case).combine(values1);
                writer.write_frac($(numerator), $(denom));
                });
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
            let (for_each, enumerate) = if self.lists.padding_type == PaddingType::Enabler
                && is_masked_relation(&self.lists, &relation_name)
            {
                (quote! { (i, (writer, values)) }, quote! {.enumerate()})
            } else {
                (quote! { (writer, values)}, quote! {})
            };
            code.extend(quote! {
                    $['\n']$("//")$(format!("Sum last logup term."))
                    let mut col_gen = logup_gen.new_col();
                    (col_gen.par_iter_mut(),
                        &self.lookup_data
                        .$(relation_name.to_case(Case::Snake))_$(*term_offset))
                        .into_par_iter()$enumerate.for_each(|$for_each| {
                        let denom =
                            $(&relation_name.to_case(Case::Snake)).combine(values);
                        writer.write_frac(
                            $(sign)PackedQM31::one()$(mask_relation(&self.lists, &relation_name)),
                            denom
                        );
                    });
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
                use crate::witness::components::$(fn_name);
            })
        });
        if is_const_size_component(&self.lists) {
            sub_component_imports
                .extend(quote! {use cairo_air::components::$(&self.lists.name)::LOG_SIZE;});
        }
        quote! {
            use crate::witness::prelude::*;
            use cairo_air::components::$(&self.lists.name)::{Claim, InteractionClaim, N_TRACE_COLUMNS};
            $(sub_component_imports)
        }
    }

    fn contains_sub_components(&self) -> bool {
        !self.add_input_mults.is_empty()
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

fn interaction_prover_struct(lists: &CompiledAirFn) -> rust::Tokens {
    // Opcodes mask is determined by the number of "real" instances.
    // Both log_size and n_rows is needed because padding might not be to the next power of 2.
    let mut interaction_claim_fields = match lists.padding_type {
        PaddingType::Enabler => quote! { n_rows: usize, },
        _ => quote! {},
    };
    if !is_const_size_component(lists) {
        interaction_claim_fields.extend(quote! { log_size: u32, });
    }

    quote! {
        pub struct InteractionClaimGenerator {
            $(interaction_claim_fields)
            lookup_data: LookupData,
        }
    }
}

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
                .unwrap_or_else(|| {
                    let name = get_variable_name(ty, val);
                    panic!("const_{}", name)
                }),
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
            let id = id
                .replace("from_felt252", "from_packed_felt252")
                .replace("from_biguint", "from_packed_biguint");
            if id.ends_with("from_packed_felt252_array") {
                return format!("Packed{}(&{})", id, arg_str);
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
            if exprs.is_empty() {
                return "()".to_string();
            }
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
                $(packed_name(r#type)) {
                    $(members_code),
                }
            };
            quote.to_string().unwrap()
        }
        CompiledAirVar::ExternalState(name, args) => {
            if name == "Seq" {
                name.to_lowercase()
            } else {
                let args = &args.join("_");
                get_variable_name(name.to_lowercase().as_str(), args.as_str())
            }
        }
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

/// Determines if a relation is masked in the interaction trace and returns the proper mask.
pub fn mask_relation(lists: &CompiledAirFn, relation_name: &str) -> rust::Tokens {
    let is_masked = is_masked_relation(lists, relation_name);
    match lists.padding_type {
        PaddingType::Enabler if is_masked => quote! { * enabler_col.packed_at(i)},
        _ => quote! {},
    }
}

/// Builds the IndexMap of the number of inputs for each sub-component, meaning how many inputs
/// should be added to each sub-component per row in the trace.
fn add_inputs_mults(deductions: &[TraceGenStep]) -> IndexMap<String, usize> {
    let mut add_input_mults = IndexMap::new();
    for deduction in deductions {
        if let TraceGenStep::LookupAddInput { fn_name, .. } = deduction {
            add_input_mults
                .entry(fn_name.clone())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
    }
    add_input_mults
}
