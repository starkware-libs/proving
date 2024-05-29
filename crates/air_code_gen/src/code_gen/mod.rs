use genco::lang::rust;
use genco::quote;

pub mod component_gen;
pub mod cpu_prover_gen;
pub mod packed_types;
pub mod simd_prover_gen;
pub mod simd_trace_gen;
pub mod test_utils_gen;
pub mod trace_gen;
pub mod utils;

pub fn generate_prover_component(
    component_name: &str,
    backend_type: &str,
    imports_code: rust::Tokens,
    numerator_code: rust::Tokens,
    denomerator_code: rust::Tokens,
    accumulation_code: rust::Tokens,
) -> rust::Tokens {
    quote! {
        $(imports_code)
        $['\n']
        impl ComponentProver<$(backend_type)> for $(component_name) {
            #[allow(unused_parens)]
            fn evaluate_constraint_quotients_on_domain(
                &self,
                trace: &ComponentTrace<'_, $(backend_type)>,
                evaluation_accumulator: &mut DomainEvaluationAccumulator<$(backend_type)>,
            ) {
                $("// Numerator computation.")
                $(numerator_code)
                $("\n// Denominator computation.")
                $(denomerator_code)
                $("\n// Accumulate constraints.")
                $(accumulation_code)
            }
        }
    }
}
