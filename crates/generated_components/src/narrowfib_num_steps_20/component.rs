#![allow(non_camel_case_types)]
#![allow(unused_imports)]
use num_traits::One;
use stwo_prover::constraint_framework::logup::{LogupAtRow, LookupElements};
use stwo_prover::constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval};
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::channel::Channel;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::secure_column::SECURE_EXTENSION_DEGREE;
use stwo_prover::core::lookups::utils::Fraction;
use stwo_prover::core::pcs::TreeVec;

use crate::{narrowfib_num_steps_20, LOGUP_BATCH_SIZE};

pub type ComponentLookupElements = LookupElements<4>;

pub struct NarrowFib_num_steps_20Eval {
    pub claim: Claim,
    pub interaction_claim: InteractionClaim,
    pub narrowfib_num_steps_20_lookup_elements: narrowfib_num_steps_20::ComponentLookupElements,
}

#[derive(Copy, Clone)]
pub struct Claim {
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_size = self.n_calls.next_power_of_two().ilog2();
        let interaction_0_log_sizes = vec![log_size; 22];
        let interaction_1_log_sizes = vec![log_size; SECURE_EXTENSION_DEGREE * 5];
        TreeVec::new(vec![interaction_0_log_sizes, interaction_1_log_sizes])
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.n_calls as u64);
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

#[allow(non_snake_case)]
pub type NarrowFib_num_steps_20Component = FrameworkComponent<NarrowFib_num_steps_20Eval>;

impl FrameworkEval for NarrowFib_num_steps_20Eval {
    fn log_size(&self) -> u32 {
        self.claim.n_calls.next_power_of_two().ilog2()
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let [is_first] = eval.next_interaction_mask(2, [0]);
        let mut logup = LogupAtRow::<E>::new(1, self.interaction_claim.claimed_sum, None, is_first);
        let trace_row: [_; 22] = std::array::from_fn(|_| eval.next_trace_mask());
        eval.add_constraint(
            (trace_row[2].clone()
                - ((trace_row[0].clone() * trace_row[0].clone())
                    + (trace_row[1].clone() * trace_row[1].clone()))),
        );
        eval.add_constraint(
            (trace_row[3].clone()
                - ((trace_row[1].clone() * trace_row[1].clone())
                    + (trace_row[2].clone() * trace_row[2].clone()))),
        );
        eval.add_constraint(
            (trace_row[4].clone()
                - ((trace_row[2].clone() * trace_row[2].clone())
                    + (trace_row[3].clone() * trace_row[3].clone()))),
        );
        eval.add_constraint(
            (trace_row[5].clone()
                - ((trace_row[3].clone() * trace_row[3].clone())
                    + (trace_row[4].clone() * trace_row[4].clone()))),
        );
        eval.add_constraint(
            (trace_row[6].clone()
                - ((trace_row[4].clone() * trace_row[4].clone())
                    + (trace_row[5].clone() * trace_row[5].clone()))),
        );
        eval.add_constraint(
            (trace_row[7].clone()
                - ((trace_row[5].clone() * trace_row[5].clone())
                    + (trace_row[6].clone() * trace_row[6].clone()))),
        );
        eval.add_constraint(
            (trace_row[8].clone()
                - ((trace_row[6].clone() * trace_row[6].clone())
                    + (trace_row[7].clone() * trace_row[7].clone()))),
        );
        eval.add_constraint(
            (trace_row[9].clone()
                - ((trace_row[7].clone() * trace_row[7].clone())
                    + (trace_row[8].clone() * trace_row[8].clone()))),
        );
        eval.add_constraint(
            (trace_row[10].clone()
                - ((trace_row[8].clone() * trace_row[8].clone())
                    + (trace_row[9].clone() * trace_row[9].clone()))),
        );
        eval.add_constraint(
            (trace_row[11].clone()
                - ((trace_row[9].clone() * trace_row[9].clone())
                    + (trace_row[10].clone() * trace_row[10].clone()))),
        );
        eval.add_constraint(
            (trace_row[12].clone()
                - ((trace_row[10].clone() * trace_row[10].clone())
                    + (trace_row[11].clone() * trace_row[11].clone()))),
        );
        eval.add_constraint(
            (trace_row[13].clone()
                - ((trace_row[11].clone() * trace_row[11].clone())
                    + (trace_row[12].clone() * trace_row[12].clone()))),
        );
        eval.add_constraint(
            (trace_row[14].clone()
                - ((trace_row[12].clone() * trace_row[12].clone())
                    + (trace_row[13].clone() * trace_row[13].clone()))),
        );
        eval.add_constraint(
            (trace_row[15].clone()
                - ((trace_row[13].clone() * trace_row[13].clone())
                    + (trace_row[14].clone() * trace_row[14].clone()))),
        );
        eval.add_constraint(
            (trace_row[16].clone()
                - ((trace_row[14].clone() * trace_row[14].clone())
                    + (trace_row[15].clone() * trace_row[15].clone()))),
        );
        eval.add_constraint(
            (trace_row[17].clone()
                - ((trace_row[15].clone() * trace_row[15].clone())
                    + (trace_row[16].clone() * trace_row[16].clone()))),
        );
        eval.add_constraint(
            (trace_row[18].clone()
                - ((trace_row[16].clone() * trace_row[16].clone())
                    + (trace_row[17].clone() * trace_row[17].clone()))),
        );
        eval.add_constraint(
            (trace_row[19].clone()
                - ((trace_row[17].clone() * trace_row[17].clone())
                    + (trace_row[18].clone() * trace_row[18].clone()))),
        );
        eval.add_constraint(
            (trace_row[20].clone()
                - ((trace_row[18].clone() * trace_row[18].clone())
                    + (trace_row[19].clone() * trace_row[19].clone()))),
        );
        eval.add_constraint(
            (trace_row[21].clone()
                - ((trace_row[19].clone() * trace_row[19].clone())
                    + (trace_row[20].clone() * trace_row[20].clone()))),
        );
        let frac = Fraction::new(
            -E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                trace_row[0].clone(),
                trace_row[1].clone(),
                trace_row[20].clone(),
                trace_row[21].clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        logup.finalize(&mut eval);

        eval
    }
}
