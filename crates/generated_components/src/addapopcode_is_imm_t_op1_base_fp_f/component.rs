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

use crate::{memoryaddresstoid, memoryidtobig, opcodes, verifyinstruction, LOGUP_BATCH_SIZE};

pub type ComponentLookupElements = LookupElements<4>;

pub struct AddApOpcode_is_imm_t_op1_base_fp_fEval {
    pub claim: Claim,
    pub interaction_claim: InteractionClaim,
    pub memoryaddresstoid_lookup_elements: memoryaddresstoid::ComponentLookupElements,
    pub memoryidtobig_lookup_elements: memoryidtobig::ComponentLookupElements,
    pub verifyinstruction_lookup_elements: verifyinstruction::ComponentLookupElements,
    pub opcodes_lookup_elements: opcodes::ComponentLookupElements,
}

#[derive(Copy, Clone)]
pub struct Claim {
    pub n_calls: usize,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_size = self.n_calls.next_power_of_two().ilog2();
        let interaction_0_log_sizes = vec![log_size; 9];
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
pub type AddApOpcode_is_imm_t_op1_base_fp_fComponent =
    FrameworkComponent<AddApOpcode_is_imm_t_op1_base_fp_fEval>;

impl FrameworkEval for AddApOpcode_is_imm_t_op1_base_fp_fEval {
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
        let M31_134217728 = E::F::from(M31::from(134217728));
        let M31_136 = E::F::from(M31::from(136));
        let M31_2 = E::F::from(M31::from(2));
        let M31_256 = E::F::from(M31::from(256));
        let M31_262144 = E::F::from(M31::from(262144));
        let M31_32767 = E::F::from(M31::from(32767));
        let M31_32769 = E::F::from(M31::from(32769));
        let M31_511 = E::F::from(M31::from(511));
        let M31_512 = E::F::from(M31::from(512));
        let [is_first] = eval.next_interaction_mask(2, [0]);
        let mut logup = LogupAtRow::<E>::new(1, self.interaction_claim.claimed_sum, None, is_first);
        let trace_row: [_; 9] = std::array::from_fn(|_| eval.next_trace_mask());
        let frac = Fraction::new(
            E::EF::one(),
            self.verifyinstruction_lookup_elements.combine(&[
                trace_row[0].clone(),
                M31_32767.clone(),
                M31_32767.clone(),
                M31_32769.clone(),
                M31_1.clone(),
                M31_1.clone(),
                M31_1.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_1.clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.memoryaddresstoid_lookup_elements
                .combine(&[(trace_row[0].clone() + M31_1.clone()), trace_row[3].clone()]),
        );
        logup.write_frac(&mut eval, frac);
        eval.add_constraint((trace_row[4].clone() * (trace_row[4].clone() - M31_1.clone())));
        eval.add_constraint((trace_row[5].clone() * (trace_row[5].clone() - M31_1.clone())));
        eval.add_constraint(
            ((M31_1.clone() * trace_row[5].clone()) * (trace_row[4].clone() - M31_1.clone())),
        );
        let frac = Fraction::new(
            E::EF::one(),
            self.memoryidtobig_lookup_elements.combine(&[
                trace_row[3].clone(),
                trace_row[6].clone(),
                trace_row[7].clone(),
                trace_row[8].clone(),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                (trace_row[5].clone() * M31_511.clone()),
                ((M31_136.clone() * trace_row[4].clone()) - trace_row[5].clone()),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                M31_0.clone(),
                (trace_row[4].clone() * M31_256.clone()),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            E::EF::one(),
            self.opcodes_lookup_elements.combine(&[
                trace_row[0].clone(),
                trace_row[1].clone(),
                trace_row[2].clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        let frac = Fraction::new(
            -E::EF::one(),
            self.opcodes_lookup_elements.combine(&[
                (trace_row[0].clone() + M31_2.clone()),
                (trace_row[1].clone()
                    + ((((trace_row[8].clone() * M31_262144.clone())
                        + ((trace_row[7].clone() * M31_512.clone()) + trace_row[6].clone()))
                        - trace_row[4].clone())
                        - (M31_134217728.clone() * trace_row[5].clone()))),
                trace_row[2].clone(),
            ]),
        );
        logup.write_frac(&mut eval, frac);
        logup.finalize(&mut eval);

        eval
    }
}
