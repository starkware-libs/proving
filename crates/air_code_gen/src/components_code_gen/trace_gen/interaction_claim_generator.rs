use std::collections::HashMap;

use compiled_casm_air::compiled_structs::{CompiledAirFn, LookupTerm, PaddingType, UseOrYield};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

use super::RustProverGen;
use crate::utils::{
    is_const_size_component, relation_multiplicity_index, relations_used_or_yielded,
};

impl RustProverGen {
    // TODO(Gali): Consider uniting def and impl functions.
    pub fn generate_interaction_impl(&self) -> rust::Tokens {
        let padding = match self.air_fn.padding_type {
            PaddingType::Enabler => quote! {let enabler_col = Enabler::new(self.n_rows);},
            _ => quote! {},
        };
        let log_size = if is_const_size_component(&self.air_fn) {
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
                    common_lookup_elements: &relations::CommonLookupElements
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
        for relation in relations_used_or_yielded(&self.air_fn) {
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
            let masked_denom_0 = quote! {denom0 $(mask_relation(&self.air_fn, relation1))};
            let masked_denom_1 = quote! {denom1 $(mask_relation(&self.air_fn, relation0))};

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
            let is_masked0 = relation_multiplicity_index(&self.air_fn, relation0);
            let is_masked1 = relation_multiplicity_index(&self.air_fn, relation1);
            let (for_each, enumerate, mults) = match self.air_fn.padding_type {
                PaddingType::Multiplicity if (is_masked0.is_some() && is_masked1.is_some()) => (
                    quote! { (
                        writer,
                        values0,
                        values1,
                        mults_$(is_masked0.unwrap()),
                        mults_$(is_masked1.unwrap())
                    ) },
                    quote! {},
                    quote! {
                        , self.lookup_data.mults_$(is_masked0.unwrap()),
                        self.lookup_data.mults_$(is_masked1.unwrap())
                    },
                ),
                PaddingType::Multiplicity if is_masked0.is_some() => (
                    quote! { (
                        writer,
                        values0,
                        values1,
                        mults_$(is_masked0.unwrap()),
                    ) },
                    quote! {},
                    quote! {
                        , self.lookup_data.mults_$(is_masked0.unwrap())
                    },
                ),
                PaddingType::Multiplicity if is_masked1.is_some() => (
                    quote! { (
                        writer,
                        values0,
                        values1,
                        mults_$(is_masked1.unwrap()),
                    ) },
                    quote! {},
                    quote! {
                        , self.lookup_data.mults_$(is_masked1.unwrap())
                    },
                ),
                PaddingType::Enabler if (is_masked0.is_some() || is_masked1.is_some()) => (
                    quote! { (i, (writer, values0, values1)) },
                    quote! {.enumerate()},
                    quote! {},
                ),
                _ => (quote! { (writer, values0, values1)}, quote! {}, quote! {}),
            };
            code.extend(quote! {
                let mut col_gen = logup_gen.new_col();
                (col_gen.par_iter_mut(),
                &self.lookup_data.$(relation_0_snake_case)_$(term0_offset),
                &self.lookup_data.$(relation_1_snake_case)_$(term1_offset)
                $mults)
                    .into_par_iter()$enumerate.for_each(|$for_each| {
                        let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                        let denom1: PackedQM31 = common_lookup_elements.combine(values1);
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
            let is_masked = relation_multiplicity_index(&self.air_fn, &relation_name);
            let (for_each, enumerate, mults) = match self.air_fn.padding_type {
                PaddingType::Multiplicity if is_masked.is_some() => (
                    quote! { (writer, values, mults_$(is_masked.unwrap())) },
                    quote! {},
                    quote! {, self.lookup_data.mults_$(is_masked.unwrap())},
                ),
                PaddingType::Enabler if is_masked.is_some() => (
                    quote! { (i, (writer, values)) },
                    quote! {.enumerate()},
                    quote! {},
                ),
                _ => (quote! { (writer, values)}, quote! {}, quote! {}),
            };
            code.extend(quote! {
                    $['\n']$("//")$(format!("Sum last logup term."))
                    let mut col_gen = logup_gen.new_col();
                    (
                        col_gen.par_iter_mut(),
                        &self.lookup_data.$(relation_name.to_case(Case::Snake))_$(*term_offset)
                        $mults
                    )
                        .into_par_iter()$enumerate.for_each(|$for_each| {
                        let denom =
                            common_lookup_elements.combine(values);
                        writer.write_frac(
                            $(sign)PackedQM31::one()$(mask_relation(&self.air_fn, &relation_name)),
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
}

pub fn interaction_prover_struct(air_fn: &CompiledAirFn) -> rust::Tokens {
    // Opcodes mask is determined by the number of "real" instances.
    // Both log_size and n_rows is needed because padding might not be to the next power of 2.
    let mut interaction_claim_fields = match air_fn.padding_type {
        PaddingType::Enabler => quote! { n_rows: usize, },
        _ => quote! {},
    };
    if !is_const_size_component(air_fn) {
        interaction_claim_fields.extend(quote! { log_size: u32, });
    }

    quote! {
        pub struct InteractionClaimGenerator {
            $(interaction_claim_fields)
            lookup_data: LookupData,
        }
    }
}

/// Determines if a relation is masked in the interaction trace and returns the proper mask.
pub fn mask_relation(air_fn: &CompiledAirFn, relation_name: &str) -> rust::Tokens {
    let is_masked = relation_multiplicity_index(air_fn, relation_name);
    match air_fn.padding_type {
        PaddingType::Enabler if is_masked.is_some() => quote! { * enabler_col.packed_at(i)},
        PaddingType::Multiplicity if is_masked.is_some() => {
            quote! { * mults_$(is_masked.unwrap()) }
        }
        _ => quote! {},
    }
}
