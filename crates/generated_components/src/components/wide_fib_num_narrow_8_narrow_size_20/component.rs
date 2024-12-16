#![allow(non_camel_case_types)]
#![allow(unused_imports)]
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use stwo_cairo_serialize::CairoSerialize;
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

use crate::relations;

pub struct Eval {
    pub claim: Claim,
    pub narrowfib_num_steps_20_lookup_elements: relations::NarrowFib_num_steps_20,
}

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
pub struct Claim {
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_size = std::cmp::max(self.n_calls.next_power_of_two().ilog2(), LOG_N_LANES);
        let trace_log_sizes = vec![log_size; 17];
        let interaction_log_sizes = vec![log_size; SECURE_EXTENSION_DEGREE * 8];
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

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
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
        let M31_1 = E::F::from(M31::from(1));
        let col0 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col1 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col2 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col3 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col4 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col5 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col6 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col7 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col8 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col9 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col10 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col11 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col12 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col13 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col14 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col15 = eval.next_trace_mask();
        let narrowfib_num_steps_20_output_col16 = eval.next_trace_mask();
        eval.add_to_relation(RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            E::EF::one(),
            &[
                M31_1.clone(),
                col0.clone(),
                narrowfib_num_steps_20_output_col1.clone(),
                narrowfib_num_steps_20_output_col2.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            E::EF::one(),
            &[
                narrowfib_num_steps_20_output_col1.clone(),
                narrowfib_num_steps_20_output_col2.clone(),
                narrowfib_num_steps_20_output_col3.clone(),
                narrowfib_num_steps_20_output_col4.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            E::EF::one(),
            &[
                narrowfib_num_steps_20_output_col3.clone(),
                narrowfib_num_steps_20_output_col4.clone(),
                narrowfib_num_steps_20_output_col5.clone(),
                narrowfib_num_steps_20_output_col6.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            E::EF::one(),
            &[
                narrowfib_num_steps_20_output_col5.clone(),
                narrowfib_num_steps_20_output_col6.clone(),
                narrowfib_num_steps_20_output_col7.clone(),
                narrowfib_num_steps_20_output_col8.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            E::EF::one(),
            &[
                narrowfib_num_steps_20_output_col7.clone(),
                narrowfib_num_steps_20_output_col8.clone(),
                narrowfib_num_steps_20_output_col9.clone(),
                narrowfib_num_steps_20_output_col10.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            E::EF::one(),
            &[
                narrowfib_num_steps_20_output_col9.clone(),
                narrowfib_num_steps_20_output_col10.clone(),
                narrowfib_num_steps_20_output_col11.clone(),
                narrowfib_num_steps_20_output_col12.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            E::EF::one(),
            &[
                narrowfib_num_steps_20_output_col11.clone(),
                narrowfib_num_steps_20_output_col12.clone(),
                narrowfib_num_steps_20_output_col13.clone(),
                narrowfib_num_steps_20_output_col14.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.narrowfib_num_steps_20_lookup_elements,
            E::EF::one(),
            &[
                narrowfib_num_steps_20_output_col13.clone(),
                narrowfib_num_steps_20_output_col14.clone(),
                narrowfib_num_steps_20_output_col15.clone(),
                narrowfib_num_steps_20_output_col16.clone(),
            ],
        ));

        eval.finalize_logup();
        eval
    }
}
