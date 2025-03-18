use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::fields::FieldExpOps;

pub mod component;
pub mod prover;

pub use component::{Claim, InteractionClaim};
pub use prover::{ClaimGenerator, InputType, InteractionClaimGenerator, PackedInputType};

pub fn deduce_output(input: [PackedM31; 2]) -> [PackedM31; 2] {
    let mut state = input;
    for _ in 0..20 {
        let next = [state[1], state[0].square() + state[1].square()];
        state = next;
    }
    state
}
