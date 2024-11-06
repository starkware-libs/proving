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

use crate::components::narrowfib_num_steps_20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationElements(LookupElements<2>);
impl RelationElements {
    pub fn draw(channel: &mut impl Channel) -> Self {
        Self(LookupElements::<2>::draw(channel))
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
    pub narrowfib_num_steps_20_lookup_elements: narrowfib_num_steps_20::RelationElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_size = self.n_calls.next_power_of_two().ilog2();
        let interaction_0_log_sizes = vec![log_size; 17];
        let interaction_1_log_sizes = vec![log_size; SECURE_EXTENSION_DEGREE * 3];
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
        let M31_1 = E::F::from(M31::from(1));
        let [is_first] = eval.next_interaction_mask(2, [0]);
        let mut logup = LogupAtRow::<E>::new(1, self.interaction_claim.claimed_sum, None, is_first);
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
        let frac = Fraction::new(
            E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                M31_1.clone(),
                col0.clone(),
                narrowfib_num_steps_20_output_col1.clone(),
                narrowfib_num_steps_20_output_col2.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                narrowfib_num_steps_20_output_col1.clone(),
                narrowfib_num_steps_20_output_col2.clone(),
                narrowfib_num_steps_20_output_col3.clone(),
                narrowfib_num_steps_20_output_col4.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                narrowfib_num_steps_20_output_col3.clone(),
                narrowfib_num_steps_20_output_col4.clone(),
                narrowfib_num_steps_20_output_col5.clone(),
                narrowfib_num_steps_20_output_col6.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                narrowfib_num_steps_20_output_col5.clone(),
                narrowfib_num_steps_20_output_col6.clone(),
                narrowfib_num_steps_20_output_col7.clone(),
                narrowfib_num_steps_20_output_col8.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                narrowfib_num_steps_20_output_col7.clone(),
                narrowfib_num_steps_20_output_col8.clone(),
                narrowfib_num_steps_20_output_col9.clone(),
                narrowfib_num_steps_20_output_col10.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                narrowfib_num_steps_20_output_col9.clone(),
                narrowfib_num_steps_20_output_col10.clone(),
                narrowfib_num_steps_20_output_col11.clone(),
                narrowfib_num_steps_20_output_col12.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                narrowfib_num_steps_20_output_col11.clone(),
                narrowfib_num_steps_20_output_col12.clone(),
                narrowfib_num_steps_20_output_col13.clone(),
                narrowfib_num_steps_20_output_col14.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.narrowfib_num_steps_20_lookup_elements.combine(&[
                narrowfib_num_steps_20_output_col13.clone(),
                narrowfib_num_steps_20_output_col14.clone(),
                narrowfib_num_steps_20_output_col15.clone(),
                narrowfib_num_steps_20_output_col16.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        logup.finalize(&mut eval);

        eval
    }
}
