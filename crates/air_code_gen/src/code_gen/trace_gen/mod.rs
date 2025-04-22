mod claim_generator;
mod interaction_claim_generator;
use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, CompiledIntermediate, LookupTerm, PaddingType, TraceGenStep,
    TraceType,
};
use compiled_casm_air::public_params::PublicParam;
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use indexmap::IndexMap;
use interaction_claim_generator::interaction_prover_struct;
use itertools::Itertools;

use super::parse::seek_consts;
use crate::code_gen::parse::is_const_size_component;

pub enum Mode {
    NoInputs,
    Inputs, // TODO(Gali): Unite with PackedInputs.
    PackedInputs,
    Mults,
}

pub struct RustProverGen {
    pub lists: CompiledAirFn,
    pub public_params: Vec<PublicParam>,
    pub write_trace_context: Vec<String>,
    pub constants: Vec<(String, String)>,
    pub relation_calls: Vec<String>,
    pub add_input_mults: IndexMap<String, usize>,
    pub lookup_terms: Vec<LookupTerm>,
    pub mode: Mode,
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

fn packed_name(ty: &str) -> String {
    format!("Packed{}", ty)
}

fn vec_of_type(ty: &str) -> String {
    format!("Vec<{}>", ty)
}
