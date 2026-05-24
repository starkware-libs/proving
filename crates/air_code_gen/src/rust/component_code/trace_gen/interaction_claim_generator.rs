use air_common::UseOrYield;
use air_compile::compiled_structs::CompiledAirFn;
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

use super::RustProverGen;
use crate::utils::is_const_size_component;

impl RustProverGen {
    // TODO(Gali): Consider uniting def and impl functions.
    pub fn generate_interaction_impl(&self) -> rust::Tokens {
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
                    common_lookup_elements: &relations::CommonLookupElements
                ) -> (Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>, InteractionClaim)
                {
                    let mut logup_gen = unsafe { LogupTraceGenerator::uninitialized($log_size) };

                    $(self.generate_write_interaction_trace_body())
                    let (trace, claimed_sum) = logup_gen.finalize_last();

                    (trace, InteractionClaim {
                        claimed_sum,
                    },)
                }
            }
        }
    }

    fn generate_write_interaction_trace_body(&self) -> rust::Tokens {
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
        let mut offset = 0;
        for (term0, term1) in pairs {
            code.extend(quote!());
            let relation0 = &term0.relation_name;
            let relation1 = &term1.relation_name;
            let relation_0_snake_case = &relation0.to_case(Case::Snake);
            let relation_1_snake_case = &relation1.to_case(Case::Snake);
            let term0_multiplicity_index =
                self.multiplicity_indices.get(&term0.multiplicity).unwrap();
            let term1_multiplicity_index =
                self.multiplicity_indices.get(&term1.multiplicity).unwrap();

            // Projective fraction addition (with numerator +-1).
            let (numerator, denom) = (
                match (term0.use_or_yield, term1.use_or_yield) {
                    (UseOrYield::Use, UseOrYield::Use) => {
                        quote! {denom0 * *mult1 + denom1 * *mult0}
                    }
                    (UseOrYield::Use, UseOrYield::Yield) => {
                        quote! {denom1 * *mult0 - denom0 * *mult1}
                    }
                    (UseOrYield::Yield, UseOrYield::Use) => {
                        quote! {denom0 * *mult1 - denom1 * *mult0}
                    }
                    (UseOrYield::Yield, UseOrYield::Yield) => {
                        quote! {-(denom0 * *mult1 + denom1 * *mult0)}
                    }
                },
                "denom0 * denom1",
            );
            let (for_each, mults) = (
                quote! { (
                    writer,
                    values0,
                    values1,
                    mult0,
                    mult1
                ) },
                quote! {
                    , &self.lookup_data.mults_$(*term0_multiplicity_index),
                    &self.lookup_data.mults_$(*term1_multiplicity_index)
                },
            );
            code.extend(quote! {
                let mut col_gen = logup_gen.new_col();
                (col_gen.par_iter_mut(),
                &self.lookup_data.$(relation_0_snake_case)_$(offset),
                &self.lookup_data.$(relation_1_snake_case)_$(offset + 1)
                $mults)
                    .into_par_iter().for_each(|$for_each| {
                        let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                        let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                        writer.write_frac($(numerator), $(denom));
                    });
                col_gen.finalize_col();
                $['\n']
            });

            offset += 2;
        }

        // Handle odd remainder.
        if let Some(term) = remainder {
            let sign = match term.use_or_yield {
                UseOrYield::Use => "",
                UseOrYield::Yield => "-",
            };
            let multiplicity_index = self
                .multiplicity_indices
                .get(&term.multiplicity)
                .unwrap_or_else(|| panic!("Missing multiplicity {}", term.multiplicity));
            let (for_each, mults) = (
                quote! { (writer, values, mult) },
                quote! {, self.lookup_data.mults_$(*multiplicity_index)},
            );
            code.extend(quote! {
                    $['\n']$("//")$(format!("Sum last logup term."))
                    let mut col_gen = logup_gen.new_col();
                    (
                        col_gen.par_iter_mut(),
                        &self.lookup_data.$(term.relation_name.to_case(Case::Snake))_$(offset)
                        $mults
                    )
                        .into_par_iter().for_each(|$for_each| {
                        let denom =
                            common_lookup_elements.combine(values);
                        writer.write_frac(
                            ($(sign)mult).into(),
                            denom
                        );
                    });
                    col_gen.finalize_col();
                    $['\n']
            });
        }
        code
    }
}

pub fn interaction_prover_struct(air_fn: &CompiledAirFn) -> rust::Tokens {
    // Opcodes mask is determined by the number of "real" instances.
    let mut interaction_claim_fields = quote! {};
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
