use air_compile::compiled_structs::CompiledAirFn;
use genco::lang::rust;
use genco::quote;

use crate::cairo::utils::get_log_size;
use crate::utils::is_const_size_component;

pub fn gen_claim_struct(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    code.append(quote! {
        #[derive(Drop, Serde, Copy)]
        pub struct Claim {
            $(get_claim_members(air_fn))
        }

        pub impl ClaimImpl of ClaimTrait<Claim> {
            fn log_sizes(self: @Claim) -> TreeArray<Span<u32>> {
                let log_size = $(get_log_size(air_fn, true));
                let preprocessed_log_sizes = array![log_size].span();
                let trace_log_sizes = [log_size; N_TRACE_COLUMNS].span();
                let interaction_log_sizes = [log_size; N_INTERACTION_COLUMNS].span();
                array![preprocessed_log_sizes, trace_log_sizes, interaction_log_sizes]
            }

            fn mix_into(self: @Claim, ref channel: Channel) {
                $(gen_mix_into(air_fn))
            }

            fn accumulate_relation_uses(self: @Claim, ref relation_uses: RelationUsesDict) {
                $(get_accumulate_relation_uses(air_fn))
            }
        }
    });
    code
}

fn get_claim_members(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut members = rust::Tokens::new();
    if !is_const_size_component(air_fn) {
        members.append(quote! { pub log_size: u32, });
    };
    members
}

fn gen_mix_into(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    if !is_const_size_component(air_fn) {
        code.append(quote! {
            channel.mix_u64(($(get_log_size(air_fn, true))).into());
        });
    }
    code
}

pub fn get_accumulate_relation_uses(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    if !is_const_size_component(air_fn) {
        // If it's not a constant size component, it is guaranteed that `Claim` contains field
        // `log_size` and that `RELATION_USES_PER_ROW` is defined and in scope.
        code.append(quote! {
            accumulate_relation_uses(ref relation_uses, RELATION_USES_PER_ROW.span(), *self.log_size);
        });
    }
    code
}
