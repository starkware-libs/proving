use compiled_casm_air::compiled_structs::CompiledAirFn;
use genco::lang::rust;
use genco::quote;

use super::utils::{get_log_size, is_const_size_component, n_logup_columns};

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
                let interaction_log_sizes = [log_size; $(n_logup_columns(air_fn))].span();
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
    for public_param in &air_fn.public_params {
        members.append(quote! {
            pub $(public_param.name()): u32,
        });
    }
    members
}

fn gen_mix_into(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    if !is_const_size_component(air_fn) {
        code.append(quote! {
            channel.mix_u64(($(get_log_size(air_fn, true))).into());
        });
    }
    for param in &air_fn.public_params {
        code.append(quote! {
            channel.mix_u64((*self.$(param.name())).into());
        });
    }
    code
}

pub fn get_accumulate_relation_uses(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    if !is_const_size_component(air_fn) {
        // If it's not a constant size component, it is guaranteed that `Claim` constains field
        // `log_size` and that `RELATION_USES_PER_ROW` is defined and in scope.
        code.append(quote! {
            accumulate_relation_uses(ref relation_uses, RELATION_USES_PER_ROW.span(), *self.log_size);
        });
    } else {
        code.append(quote! {()});
    }
    code
}

pub fn gen_interaction_claim_struct() -> rust::Tokens {
    quote! {
        #[derive(Drop, Serde, Copy)]
        pub struct InteractionClaim {
            pub claimed_sum: QM31,
        }

        #[generate_trait]
        pub impl InteractionClaimImpl of InteractionClaimTrait {
            fn mix_into(self: @InteractionClaim, ref channel: Channel) {
                channel.mix_felts([*self.claimed_sum].span());
            }
        }
    }
}
