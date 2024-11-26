#![allow(non_camel_case_types)]
#![allow(unused_imports)]
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use stwo_prover::constraint_framework::logup::{LogupAtRow, LogupSums, LookupElements};
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};
use stwo_prover::core::backend::simd::m31::LOG_N_LANES;
use stwo_prover::core::channel::Channel;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::secure_column::SECURE_EXTENSION_DEGREE;
use stwo_prover::core::pcs::TreeVec;

use crate::components::narrowfib_num_steps_20;

stwo_prover::relation!(RelationElements, 4);

pub struct Eval {
    pub claim: Claim,
    pub narrowfib_num_steps_20_lookup_elements: narrowfib_num_steps_20::RelationElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_size = std::cmp::max(self.n_calls.next_power_of_two().ilog2(), LOG_N_LANES);
        let trace_log_sizes = vec![log_size; 22];
        let interaction_log_sizes = vec![log_size; SECURE_EXTENSION_DEGREE * 5];
        let preprocessed_log_sizes = vec![log_size];
        TreeVec::new(vec![
            preprocessed_log_sizes,
            trace_log_sizes,
            interaction_log_sizes,
        ])
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.n_calls as u64);
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct InteractionClaim {
    pub logup_sums: LogupSums,
}
impl InteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let (total_sum, claimed_sum) = self.logup_sums;
        channel.mix_felts(&[total_sum]);
        if let Some(claimed_sum) = claimed_sum {
            channel.mix_felts(&[claimed_sum.0]);
            channel.mix_u64(claimed_sum.1 as u64);
        }
    }
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        std::cmp::max(self.claim.n_calls.next_power_of_two().ilog2(), LOG_N_LANES)
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
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

        // FibStep.

        eval.add_constraint(
            (col2.clone()
                - ((input_col0.clone() * input_col0.clone())
                    + (input_col1.clone() * input_col1.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col3.clone()
                - ((input_col1.clone() * input_col1.clone()) + (col2.clone() * col2.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col4.clone() - ((col2.clone() * col2.clone()) + (col3.clone() * col3.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col5.clone() - ((col3.clone() * col3.clone()) + (col4.clone() * col4.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col6.clone() - ((col4.clone() * col4.clone()) + (col5.clone() * col5.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col7.clone() - ((col5.clone() * col5.clone()) + (col6.clone() * col6.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col8.clone() - ((col6.clone() * col6.clone()) + (col7.clone() * col7.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col9.clone() - ((col7.clone() * col7.clone()) + (col8.clone() * col8.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col10.clone() - ((col8.clone() * col8.clone()) + (col9.clone() * col9.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col11.clone() - ((col9.clone() * col9.clone()) + (col10.clone() * col10.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col12.clone() - ((col10.clone() * col10.clone()) + (col11.clone() * col11.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col13.clone() - ((col11.clone() * col11.clone()) + (col12.clone() * col12.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col14.clone() - ((col12.clone() * col12.clone()) + (col13.clone() * col13.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col15.clone() - ((col13.clone() * col13.clone()) + (col14.clone() * col14.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col16.clone() - ((col14.clone() * col14.clone()) + (col15.clone() * col15.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col17.clone() - ((col15.clone() * col15.clone()) + (col16.clone() * col16.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col18.clone() - ((col16.clone() * col16.clone()) + (col17.clone() * col17.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col19.clone() - ((col17.clone() * col17.clone()) + (col18.clone() * col18.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col20.clone() - ((col18.clone() * col18.clone()) + (col19.clone() * col19.clone()))),
        );

        // FibStep.

        eval.add_constraint(
            (col21.clone() - ((col19.clone() * col19.clone()) + (col20.clone() * col20.clone()))),
        );

        eval.add_to_relation(&[RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            -E::EF::one(),
            &[
                input_col0.clone(),
                input_col1.clone(),
                col20.clone(),
                col21.clone(),
            ],
        )]);

        eval.finalize_logup();
        eval
    }
}
