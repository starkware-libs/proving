mod claim_generator;
mod interaction_claim_generator;
use std::collections::{HashMap, HashSet};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledTraceGenIntermediate, LookupTerm, PaddingType, TraceGenStep, TraceType,
};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use indexmap::IndexMap;
use interaction_claim_generator::interaction_prover_struct;
use itertools::Itertools;

use super::parse::seek_consts;
use crate::code_gen::utils::is_const_size_component;

pub enum Mode {
    NoInputs,
    Inputs, // TODO(Gali): Unite with PackedInputs.
    PackedInputs,
    Mults,
}

pub struct RustProverGen {
    pub air_fn: CompiledAirFn,
    pub constants: Vec<(String, String)>,
    pub add_input_mults: IndexMap<String, usize>,
    pub lookup_terms: Vec<LookupTerm>,
    pub mode: Mode,
}
impl RustProverGen {
    pub fn new(air_fn: CompiledAirFn) -> Self {
        let mode = match air_fn.r#type {
            TraceType::Builtin => Mode::NoInputs,
            TraceType::ChainRound => Mode::PackedInputs,
            TraceType::Component => {
                if air_fn.padding_type == PaddingType::Multiplicity {
                    Mode::Mults
                } else {
                    Mode::PackedInputs
                }
            }
            TraceType::Opcode | TraceType::Memory => Mode::Inputs,
            TraceType::Const | TraceType::Inline => {
                panic!("Not generating code for Const/Inline components.")
            }
        };

        let constants = deduction_consts(&air_fn.deductions);
        // TODO(AnatG): Put air_body.get_lookup_n_rows in compiled air.
        let add_input_mults = add_inputs_mults(&air_fn.deductions);
        let lookup_terms = filter_lookup_terms(&air_fn.deductions);

        Self {
            air_fn,
            mode,
            add_input_mults,
            constants,
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
        let interaction_struct = interaction_prover_struct(&self.air_fn);
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
        let (_name, ty, packed_ty) = &self.air_fn.prover_input;
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
        if self.air_fn.name.contains("generic_opcode") {
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
                quote! {
                    $(component_name): [$component_name::PackedInputType; $mult],
                }
            })
            .collect_vec();

        quote! {
            type SubComponentInputs = Vec<SubComponentInputsPerRow>;

            #[allow(clippy::uninit_vec)]
            unsafe fn uninitialized_sub_component_inputs(log_n_packed_rows: u32) -> SubComponentInputs {
                let mut vec: SubComponentInputs = Vec::with_capacity(1 << log_n_packed_rows);
                vec.set_len(1 << log_n_packed_rows);
                vec
            }

            struct SubComponentInputsPerRow {
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
        if self.air_fn.padding_type == PaddingType::Multiplicity {
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
        self.air_fn
            .deduction_lookups
            .iter()
            .for_each(|(fn_name, _)| {
                sub_component_imports.extend(quote! {
                    use crate::witness::components::$(fn_name.to_case(Case::Snake));
                })
            });
        if is_const_size_component(&self.air_fn) {
            sub_component_imports
                .extend(quote! {use cairo_air::components::$(&self.air_fn.name)::LOG_SIZE;});
        }

        quote! {
            use crate::witness::prelude::*;
            use cairo_air::components::$(&self.air_fn.name)::{Claim, InteractionClaim, N_TRACE_COLUMNS};
            $(sub_component_imports)
        }
    }

    fn contains_sub_components(&self) -> bool {
        !self.add_input_mults.is_empty()
    }
}

/// Builds the IndexMap of the number of inputs for each sub-component, meaning how many inputs
/// should be added to each sub-component per row in the trace.
fn add_inputs_mults(deductions: &[TraceGenStep]) -> IndexMap<String, usize> {
    let mut add_input_mults = IndexMap::new();
    for deduction in deductions {
        if let TraceGenStep::LookupAddInput { relation_name, .. } = deduction {
            add_input_mults
                .entry(relation_name.to_case(Case::Snake))
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
                TraceGenStep::Intermediate(CompiledTraceGenIntermediate {
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

fn packed_name(ty: &str) -> String {
    format!("Packed{}", ty)
}

fn vec_of_type(ty: &str) -> String {
    format!("Vec<{}>", ty)
}
