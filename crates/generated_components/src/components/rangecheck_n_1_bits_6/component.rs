#![allow(non_camel_case_types)]
#![allow(unused_imports)]
use std::ops::{Mul, Sub};

use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use stwo_prover::constraint_framework::logup::{LogupAtRow, LookupElements};
use stwo_prover::constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval};
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::channel::Channel;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::secure_column::SECURE_EXTENSION_DEGREE;
use stwo_prover::core::lookups::utils::Fraction;
use stwo_prover::core::pcs::TreeVec;

use crate::components::rangecheck_n_1_bits_6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationElements(LookupElements<1>);
impl RelationElements {
    pub fn draw(channel: &mut impl Channel) -> Self {
        Self(LookupElements::<1>::draw(channel))
    }
    pub fn combine<F: Clone, EF>(&self, values: &[F]) -> EF
    where
        EF: Clone + Zero + From<F> + From<SecureField> + Mul<F, Output = EF> + Sub<EF, Output = EF>,
    {
        self.0.combine(values)
    }
}

pub struct Eval {
    pub claim: Claim,
    pub interaction_claim: InteractionClaim,
    pub rangecheck_n_1_bits_6_lookup_elements: rangecheck_n_1_bits_6::RelationElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_size = self.n_calls.next_power_of_two().ilog2();
        let interaction_0_log_sizes = vec![log_size; 0];
        let interaction_1_log_sizes = vec![log_size; SECURE_EXTENSION_DEGREE * 2];
        TreeVec::new(vec![interaction_0_log_sizes, interaction_1_log_sizes])
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.n_calls as u64);
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct InteractionClaim {
    pub claimed_sum: SecureField,
}
impl InteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.claimed_sum]);
    }
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.claim.n_calls.next_power_of_two().ilog2()
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let [is_first] = eval.next_interaction_mask(2, [0]);
        let mut logup = LogupAtRow::<E>::new(1, self.interaction_claim.claimed_sum, None, is_first);
        let frac = Fraction::new(
            -E::EF::one(),
            self.rangecheck_n_1_bits_6_lookup_elements
                .combine(&[todo!()]),
        );
        logup.write_frac(&mut eval, frac);
        logup.finalize(&mut eval);

        eval
    }
}
