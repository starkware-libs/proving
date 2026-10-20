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
            fn mix_into(self: @Claim, ref channel: Channel) {
                $(gen_mix_into(air_fn))
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
