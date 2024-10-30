#![allow(non_camel_case_types)]
#![allow(unused_imports)]
use num_traits::One;
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

use crate::{narrowfib_num_steps_20, LOGUP_BATCH_SIZE};

pub type RelationElements = LookupElements<4>;

pub struct Eval {
    pub claim: Claim,
    pub interaction_claim: InteractionClaim,
    pub narrowfib_num_steps_20_lookup_elements: narrowfib_num_steps_20::RelationElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
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
        let input_col0 = eval.next_trace_mask();
        let input_col1 = eval.next_trace_mask();
        let col2 = eval.next_trace_mask();
        let col3 = eval.next_trace_mask();
        let col4 = eval.next_trace_mask();
        let col5 = eval.next_trace_mask();
        let col6 = eval.next_trace_mask();
        let col7 = eval.next_trace_mask();
        let col8 = eval.next_trace_mask();
        let col9 = eval.next_trace_mask();
        let col10 = eval.next_trace_mask();
        let col11 = eval.next_trace_mask();
        let col12 = eval.next_trace_mask();
        let col13 = eval.next_trace_mask();
        let col14 = eval.next_trace_mask();
        let col15 = eval.next_trace_mask();
        let col16 = eval.next_trace_mask();
        let col17 = eval.next_trace_mask();
        let col18 = eval.next_trace_mask();
        let col19 = eval.next_trace_mask();
        let col20 = eval.next_trace_mask();
        let col21 = eval.next_trace_mask();
        eval.add_constraint(
            (col2.clone()
                - ((input_col0.clone() * input_col0.clone())
                    + (input_col1.clone() * input_col1.clone()))),
        );
        eval.add_constraint(
            (col3.clone()
                - ((input_col1.clone() * input_col1.clone()) + (col2.clone() * col2.clone()))),
        );
        eval.add_constraint(
            (col4.clone() - ((col2.clone() * col2.clone()) + (col3.clone() * col3.clone()))),
        );
        eval.add_constraint(
            (col5.clone() - ((col3.clone() * col3.clone()) + (col4.clone() * col4.clone()))),
        );
        eval.add_constraint(
            (col6.clone() - ((col4.clone() * col4.clone()) + (col5.clone() * col5.clone()))),
        );
        eval.add_constraint(
            (col7.clone() - ((col5.clone() * col5.clone()) + (col6.clone() * col6.clone()))),
        );
        eval.add_constraint(
            (col8.clone() - ((col6.clone() * col6.clone()) + (col7.clone() * col7.clone()))),
        );
        eval.add_constraint(
            (col9.clone() - ((col7.clone() * col7.clone()) + (col8.clone() * col8.clone()))),
        );
        eval.add_constraint(
            (col10.clone() - ((col8.clone() * col8.clone()) + (col9.clone() * col9.clone()))),
        );
        eval.add_constraint(
            (col11.clone() - ((col9.clone() * col9.clone()) + (col10.clone() * col10.clone()))),
        );
        eval.add_constraint(
            (col12.clone() - ((col10.clone() * col10.clone()) + (col11.clone() * col11.clone()))),
        );
        eval.add_constraint(
            (col13.clone() - ((col11.clone() * col11.clone()) + (col12.clone() * col12.clone()))),
        );
        eval.add_constraint(
            (col14.clone() - ((col12.clone() * col12.clone()) + (col13.clone() * col13.clone()))),
        );
        eval.add_constraint(
            (col15.clone() - ((col13.clone() * col13.clone()) + (col14.clone() * col14.clone()))),
        );
        eval.add_constraint(
            (col16.clone() - ((col14.clone() * col14.clone()) + (col15.clone() * col15.clone()))),
        );
        eval.add_constraint(
            (col17.clone() - ((col15.clone() * col15.clone()) + (col16.clone() * col16.clone()))),
        );
        eval.add_constraint(
            (col18.clone() - ((col16.clone() * col16.clone()) + (col17.clone() * col17.clone()))),
        );
        eval.add_constraint(
            (col19.clone() - ((col17.clone() * col17.clone()) + (col18.clone() * col18.clone()))),
        );
        eval.add_constraint(
            (col20.clone() - ((col18.clone() * col18.clone()) + (col19.clone() * col19.clone()))),
        );
        eval.add_constraint(
            (col21.clone() - ((col19.clone() * col19.clone()) + (col20.clone() * col20.clone()))),
        );
        let frac = Fraction::new(
            -E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                input_col0.clone(),
                input_col1.clone(),
                col20.clone(),
                col21.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        logup.finalize(&mut eval);

        eval
    }
}
