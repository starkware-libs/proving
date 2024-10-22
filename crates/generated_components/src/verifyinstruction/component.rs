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

use crate::{
    memoryaddresstoid, memoryidtobig, rangecheck_n_2_bits_4_3, rangecheck_n_3_bits_7_2_5,
    verifyinstruction, LOGUP_BATCH_SIZE,
};

pub type ComponentLookupElements = LookupElements<3>;

pub struct VerifyInstructionEval {
    pub claim: Claim,
    pub interaction_claim: InteractionClaim,
    pub memoryaddresstoid_lookup_elements: memoryaddresstoid::ComponentLookupElements,
    pub memoryidtobig_lookup_elements: memoryidtobig::ComponentLookupElements,
    pub rangecheck_n_2_bits_4_3_lookup_elements: rangecheck_n_2_bits_4_3::ComponentLookupElements,
    pub rangecheck_n_3_bits_7_2_5_lookup_elements:
        rangecheck_n_3_bits_7_2_5::ComponentLookupElements,
    pub verifyinstruction_lookup_elements: verifyinstruction::ComponentLookupElements,
}

#[derive(Copy, Clone)]
pub struct Claim {
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_size = self.n_calls.next_power_of_two().ilog2();
        let interaction_0_log_sizes = vec![log_size; 28];
        let interaction_1_log_sizes = vec![log_size; SECURE_EXTENSION_DEGREE * 8];
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
pub type VerifyInstructionComponent = FrameworkComponent<VerifyInstructionEval>;

impl FrameworkEval for VerifyInstructionEval {
    fn log_size(&self) -> u32 {
        self.claim.n_calls.next_power_of_two().ilog2()
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let M31_0 = E::F::from(M31::from(0));
        let M31_1 = E::F::from(M31::from(1));
        let M31_128 = E::F::from(M31::from(128));
        let M31_16 = E::F::from(M31::from(16));
        let M31_2 = E::F::from(M31::from(2));
        let M31_2048 = E::F::from(M31::from(2048));
        let M31_256 = E::F::from(M31::from(256));
        let M31_32 = E::F::from(M31::from(32));
        let M31_4 = E::F::from(M31::from(4));
        let M31_512 = E::F::from(M31::from(512));
        let M31_64 = E::F::from(M31::from(64));
        let M31_8 = E::F::from(M31::from(8));
        let M31_8192 = E::F::from(M31::from(8192));
        let [is_first] = eval.next_interaction_mask(2, [0]);
        let mut logup = LogupAtRow::<E>::new(1, self.interaction_claim.claimed_sum, None, is_first);
        let trace_row: [_; 28] = std::array::from_fn(|_| eval.next_trace_mask());
        eval.add_constraint(
            ((trace_row[19].clone() + (trace_row[20].clone() * M31_512.clone()))
                - trace_row[1].clone()),
        );
        eval.add_constraint(
            (((trace_row[21].clone() + (trace_row[22].clone() * M31_4.clone()))
                + (trace_row[23].clone() * M31_2048.clone()))
                - trace_row[2].clone()),
        );
        eval.add_constraint(
            (((trace_row[24].clone() + (trace_row[25].clone() * M31_16.clone()))
                + (trace_row[26].clone() * M31_8192.clone()))
                - trace_row[3].clone()),
        );
        let frac = Fraction::new(
            E::EF::one(),
            self.rangecheck_n_3_bits_7_2_5_lookup_elements.combine(&[
                trace_row[20].clone(),
                trace_row[21].clone(),
                trace_row[23].clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.rangecheck_n_2_bits_4_3_lookup_elements
                .combine(&[trace_row[24].clone(), trace_row[26].clone()]),
        );
        logup.write_frac(&mut eval, frac);
        eval.add_constraint((trace_row[4].clone() * (M31_1.clone() - trace_row[4].clone())));
        eval.add_constraint((trace_row[5].clone() * (M31_1.clone() - trace_row[5].clone())));
        eval.add_constraint((trace_row[6].clone() * (M31_1.clone() - trace_row[6].clone())));
        eval.add_constraint((trace_row[7].clone() * (M31_1.clone() - trace_row[7].clone())));
        eval.add_constraint((trace_row[8].clone() * (M31_1.clone() - trace_row[8].clone())));
        eval.add_constraint((trace_row[9].clone() * (M31_1.clone() - trace_row[9].clone())));
        eval.add_constraint((trace_row[10].clone() * (M31_1.clone() - trace_row[10].clone())));
        eval.add_constraint((trace_row[11].clone() * (M31_1.clone() - trace_row[11].clone())));
        eval.add_constraint((trace_row[12].clone() * (M31_1.clone() - trace_row[12].clone())));
        eval.add_constraint((trace_row[13].clone() * (M31_1.clone() - trace_row[13].clone())));
        eval.add_constraint((trace_row[14].clone() * (M31_1.clone() - trace_row[14].clone())));
        eval.add_constraint((trace_row[15].clone() * (M31_1.clone() - trace_row[15].clone())));
        eval.add_constraint((trace_row[16].clone() * (M31_1.clone() - trace_row[16].clone())));
        eval.add_constraint((trace_row[17].clone() * (M31_1.clone() - trace_row[17].clone())));
        eval.add_constraint((trace_row[18].clone() * (M31_1.clone() - trace_row[18].clone())));
        let frac = Fraction::new(
            E::EF::one(),
            self.memoryaddresstoid_lookup_elements
                .combine(&[trace_row[0].clone(), trace_row[27].clone()]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.memoryidtobig_lookup_elements.combine(&[
                trace_row[27].clone(),
                trace_row[19].clone(),
                (trace_row[20].clone() + (trace_row[21].clone() * M31_128.clone())),
                trace_row[22].clone(),
                (trace_row[23].clone() + (trace_row[24].clone() * M31_32.clone())),
                trace_row[25].clone(),
                (trace_row[26].clone()
                    + ((((((M31_0.clone() + (trace_row[4].clone() * M31_8.clone()))
                        + (trace_row[5].clone() * M31_16.clone()))
                        + (trace_row[6].clone() * M31_32.clone()))
                        + (trace_row[7].clone() * M31_64.clone()))
                        + (trace_row[8].clone() * M31_128.clone()))
                        + (trace_row[9].clone() * M31_256.clone()))),
                (((((((((M31_0.clone() + (trace_row[10].clone() * M31_1.clone()))
                    + (trace_row[11].clone() * M31_2.clone()))
                    + (trace_row[12].clone() * M31_4.clone()))
                    + (trace_row[13].clone() * M31_8.clone()))
                    + (trace_row[14].clone() * M31_16.clone()))
                    + (trace_row[15].clone() * M31_32.clone()))
                    + (trace_row[16].clone() * M31_64.clone()))
                    + (trace_row[17].clone() * M31_128.clone()))
                    + (trace_row[18].clone() * M31_256.clone())),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            -E::EF::one(),
            self.verifyinstruction_lookup_elements.combine(&[
                trace_row[0].clone(),
                trace_row[1].clone(),
                trace_row[2].clone(),
                trace_row[3].clone(),
                trace_row[4].clone(),
                trace_row[5].clone(),
                trace_row[6].clone(),
                trace_row[7].clone(),
                trace_row[8].clone(),
                trace_row[9].clone(),
                trace_row[10].clone(),
                trace_row[11].clone(),
                trace_row[12].clone(),
                trace_row[13].clone(),
                trace_row[14].clone(),
                trace_row[15].clone(),
                trace_row[16].clone(),
                trace_row[17].clone(),
                trace_row[18].clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        logup.finalize(&mut eval);

        eval
    }
}
