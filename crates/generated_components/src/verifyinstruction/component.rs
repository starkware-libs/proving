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
use stwo_prover::core::pcs::TreeVec;

use crate::{
    memory_k_m31_v_felt252, memory_k_m31_v_m31, rangecheck_n_2_bits_4_3, rangecheck_n_3_bits_7_2_5,
    verifyinstruction, LOGUP_BATCH_SIZE,
};

pub type ComponentLookupElements = LookupElements<3>;

pub struct VerifyInstructionEval {
    pub claim: Claim,
    pub interaction_claim: InteractionClaim,
    pub memory_k_m31_v_felt252_lookup_elements: memory_k_m31_v_felt252::ComponentLookupElements,
    pub memory_k_m31_v_m31_lookup_elements: memory_k_m31_v_m31::ComponentLookupElements,
    pub rangecheck_n_2_bits_4_3_lookup_elements: rangecheck_n_2_bits_4_3::ComponentLookupElements,
    pub rangecheck_n_3_bits_7_2_5_lookup_elements:
        rangecheck_n_3_bits_7_2_5::ComponentLookupElements,
    pub verifyinstruction_lookup_elements: verifyinstruction::ComponentLookupElements,
}

#[derive(Copy, Clone)]
pub struct Claim {
    pub log_size: u32,
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let interaction_0_log_sizes = vec![self.log_size; 28];
        let interaction_1_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 8];
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

#[allow(non_snake_case)]
pub type VerifyInstructionComponent = FrameworkComponent<VerifyInstructionEval>;

impl FrameworkEval for VerifyInstructionEval {
    fn log_size(&self) -> u32 {
        self.claim.log_size
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
        let mut logup = LogupAtRow::<LOGUP_BATCH_SIZE, E>::new(
            1,
            self.interaction_claim.claimed_sum,
            self.claim.log_size,
        );
        let trace_row: [_; 28] = std::array::from_fn(|_| eval.next_trace_mask());
        eval.add_constraint(((trace_row[19] + (trace_row[20] * M31_512)) - trace_row[1]));
        eval.add_constraint(
            (((trace_row[21] + (trace_row[22] * M31_4)) + (trace_row[23] * M31_2048))
                - trace_row[2]),
        );
        eval.add_constraint(
            (((trace_row[24] + (trace_row[25] * M31_16)) + (trace_row[26] * M31_8192))
                - trace_row[3]),
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[20], trace_row[21], trace_row[23]],
            &self.rangecheck_n_3_bits_7_2_5_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[24], trace_row[26]],
            &self.rangecheck_n_2_bits_4_3_lookup_elements,
        );
        eval.add_constraint((trace_row[4] * (M31_1 - trace_row[4])));
        eval.add_constraint((trace_row[5] * (M31_1 - trace_row[5])));
        eval.add_constraint((trace_row[6] * (M31_1 - trace_row[6])));
        eval.add_constraint((trace_row[7] * (M31_1 - trace_row[7])));
        eval.add_constraint((trace_row[8] * (M31_1 - trace_row[8])));
        eval.add_constraint((trace_row[9] * (M31_1 - trace_row[9])));
        eval.add_constraint((trace_row[10] * (M31_1 - trace_row[10])));
        eval.add_constraint((trace_row[11] * (M31_1 - trace_row[11])));
        eval.add_constraint((trace_row[12] * (M31_1 - trace_row[12])));
        eval.add_constraint((trace_row[13] * (M31_1 - trace_row[13])));
        eval.add_constraint((trace_row[14] * (M31_1 - trace_row[14])));
        eval.add_constraint((trace_row[15] * (M31_1 - trace_row[15])));
        eval.add_constraint((trace_row[16] * (M31_1 - trace_row[16])));
        eval.add_constraint((trace_row[17] * (M31_1 - trace_row[17])));
        eval.add_constraint((trace_row[18] * (M31_1 - trace_row[18])));
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[trace_row[0], trace_row[27]],
            &self.memory_k_m31_v_m31_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            E::EF::one(),
            &[
                trace_row[27],
                trace_row[19],
                (trace_row[20] + (trace_row[21] * M31_128)),
                trace_row[22],
                (trace_row[23] + (trace_row[24] * M31_32)),
                trace_row[25],
                (trace_row[26]
                    + ((((((M31_0 + (trace_row[4] * M31_8)) + (trace_row[5] * M31_16))
                        + (trace_row[6] * M31_32))
                        + (trace_row[7] * M31_64))
                        + (trace_row[8] * M31_128))
                        + (trace_row[9] * M31_256))),
                (((((((((M31_0 + (trace_row[10] * M31_1)) + (trace_row[11] * M31_2))
                    + (trace_row[12] * M31_4))
                    + (trace_row[13] * M31_8))
                    + (trace_row[14] * M31_16))
                    + (trace_row[15] * M31_32))
                    + (trace_row[16] * M31_64))
                    + (trace_row[17] * M31_128))
                    + (trace_row[18] * M31_256)),
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
                M31_0,
            ],
            &self.memory_k_m31_v_felt252_lookup_elements,
        );
        logup.push_lookup(
            &mut eval,
            -E::EF::one(),
            &[
                trace_row[0],
                trace_row[1],
                trace_row[2],
                trace_row[3],
                trace_row[4],
                trace_row[5],
                trace_row[6],
                trace_row[7],
                trace_row[8],
                trace_row[9],
                trace_row[10],
                trace_row[11],
                trace_row[12],
                trace_row[13],
                trace_row[14],
                trace_row[15],
                trace_row[16],
                trace_row[17],
                trace_row[18],
            ],
            &self.verifyinstruction_lookup_elements,
        );
        logup.finalize(&mut eval);

        eval
    }
}
