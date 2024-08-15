#![allow(non_camel_case_types)]
#![allow(unused_imports)]
use num_traits::One;
use stwo_prover::constraint_framework::logup::{LogupAtRow, LookupElements};
use stwo_prover::constraint_framework::{EvalAtRow, FrameworkComponent};
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::channel::Channel;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::secure_column::SECURE_EXTENSION_DEGREE;
use stwo_prover::core::pcs::TreeVec;

use crate::airs::examples::{NarrowFib_1ddf31c88316e62f, LOGUP_BATCH_SIZE};

pub type ComponentLookupElements = LookupElements<2>;

pub struct WideFib_d7cf24d545e710f9Component {
    pub claim: Claim,
    pub interaction_claim: InteractionClaim,
    pub self_lookup_elements: ComponentLookupElements,
    pub narrowfib_1ddf31c88316e62f_lookup_elements:
        NarrowFib_1ddf31c88316e62f::ComponentLookupElements,
}

#[derive(Copy, Clone)]
pub struct Claim {
    pub log_size: u32,
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let interaction_0_log_sizes = vec![self.log_size; 17];
        let interaction_1_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 3];
        TreeVec::new(vec![interaction_0_log_sizes, interaction_1_log_sizes])
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_nonce(self.log_size as u64);
        channel.mix_nonce(self.n_calls as u64);
    }
}

pub struct InteractionClaim {
    pub claimed_sum: SecureField,
}
impl InteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.claimed_sum]);
    }
}

impl FrameworkComponent for WideFib_d7cf24d545e710f9Component {
    fn log_size(&self) -> u32 {
        self.claim.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let mut logup = LogupAtRow::<LOGUP_BATCH_SIZE, E>::new(
            1,
            self.interaction_claim.claimed_sum,
            self.claim.log_size,
        );
        let trace_row: [_; 17] = std::array::from_fn(|_| eval.next_trace_mask());
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[
                E::F::from(M31::from(1)),
                trace_row[0],
                trace_row[1],
                trace_row[2],
            ],
            &self.narrowfib_1ddf31c88316e62f_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[1], trace_row[2], trace_row[3], trace_row[4]],
            &self.narrowfib_1ddf31c88316e62f_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[3], trace_row[4], trace_row[5], trace_row[6]],
            &self.narrowfib_1ddf31c88316e62f_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[5], trace_row[6], trace_row[7], trace_row[8]],
            &self.narrowfib_1ddf31c88316e62f_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[7], trace_row[8], trace_row[9], trace_row[10]],
            &self.narrowfib_1ddf31c88316e62f_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[9], trace_row[10], trace_row[11], trace_row[12]],
            &self.narrowfib_1ddf31c88316e62f_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[11], trace_row[12], trace_row[13], trace_row[14]],
            &self.narrowfib_1ddf31c88316e62f_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[13], trace_row[14], trace_row[15], trace_row[16]],
            &self.narrowfib_1ddf31c88316e62f_lookup_elements,
        );

        logup.push_lookup(
            &mut eval,
            -E::EF::one(),
            &[trace_row[0], trace_row[16]],
            &self.self_lookup_elements,
        );
        logup.finalize(&mut eval);

        eval
    }
}
