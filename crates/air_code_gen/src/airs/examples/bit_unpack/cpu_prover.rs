#![allow(unused_imports)]
use num_traits::identities::Zero;
use stwo_prover::core::air::accumulation::DomainEvaluationAccumulator;
use stwo_prover::core::air::{Component, ComponentProver, ComponentTrace};
use stwo_prover::core::backend::{Column, CpuBackend};
use stwo_prover::core::constraints::coset_vanishing;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::utils::bit_reverse;
use stwo_prover::core::InteractionElements;

use super::component::BitUnpack_e0b35c6b3a8afa3d;

impl ComponentProver<CpuBackend> for BitUnpack_e0b35c6b3a8afa3d {
    #[allow(unused_parens)]
    fn evaluate_constraint_quotients_on_domain(
        &self,
        trace: &ComponentTrace<'_, CpuBackend>,
        evaluation_accumulator: &mut DomainEvaluationAccumulator<CpuBackend>,
        _interaction_elements: &InteractionElements,
    ) {
        // Numerator computation.
        let trace_evals = &trace.evals[0];
        let mut numerators =
            vec![SecureField::zero(); 1 << (self.max_constraint_log_degree_bound())];
        let [mut accum] = evaluation_accumulator
            .columns([(self.max_constraint_log_degree_bound(), self.n_constraints())]);
        for (i, numer) in numerators.iter_mut().enumerate() {
            let tmp_3 = (trace_evals[0].values.at(i)
                - (trace_evals[1].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[12]
                * (tmp_3 * (tmp_3 - BaseField::from_u32_unchecked(1)));
            let tmp_5 = (trace_evals[1].values.at(i)
                - (trace_evals[2].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[11]
                * (tmp_5 * (tmp_5 - BaseField::from_u32_unchecked(1)));
            let tmp_7 = (trace_evals[2].values.at(i)
                - (trace_evals[3].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[10]
                * (tmp_7 * (tmp_7 - BaseField::from_u32_unchecked(1)));
            let tmp_9 = (trace_evals[3].values.at(i)
                - (trace_evals[4].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer +=
                accum.random_coeff_powers[9] * (tmp_9 * (tmp_9 - BaseField::from_u32_unchecked(1)));
            let tmp_11 = (trace_evals[4].values.at(i)
                - (trace_evals[5].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[8]
                * (tmp_11 * (tmp_11 - BaseField::from_u32_unchecked(1)));
            let tmp_13 = (trace_evals[5].values.at(i)
                - (trace_evals[6].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[7]
                * (tmp_13 * (tmp_13 - BaseField::from_u32_unchecked(1)));
            let tmp_15 = (trace_evals[6].values.at(i)
                - (trace_evals[7].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[6]
                * (tmp_15 * (tmp_15 - BaseField::from_u32_unchecked(1)));
            let tmp_17 = (trace_evals[7].values.at(i)
                - (trace_evals[8].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[5]
                * (tmp_17 * (tmp_17 - BaseField::from_u32_unchecked(1)));
            let tmp_19 = (trace_evals[8].values.at(i)
                - (trace_evals[9].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[4]
                * (tmp_19 * (tmp_19 - BaseField::from_u32_unchecked(1)));
            let tmp_21 = (trace_evals[9].values.at(i)
                - (trace_evals[10].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[3]
                * (tmp_21 * (tmp_21 - BaseField::from_u32_unchecked(1)));
            let tmp_23 = (trace_evals[10].values.at(i)
                - (trace_evals[11].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[2]
                * (tmp_23 * (tmp_23 - BaseField::from_u32_unchecked(1)));
            let tmp_25 = (trace_evals[11].values.at(i)
                - (trace_evals[12].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[1]
                * (tmp_25 * (tmp_25 - BaseField::from_u32_unchecked(1)));
            *numer += accum.random_coeff_powers[0] * (trace_evals[12].values.at(i));
        }

        // Denominator computation.
        let zero_domain = CanonicCoset::new(self.log_n_instances).coset;
        let eval_domain = CanonicCoset::new(self.max_constraint_log_degree_bound()).circle_domain();
        let mut denoms = vec![];
        for point in eval_domain.iter() {
            denoms.push(coset_vanishing(zero_domain, point));
        }
        bit_reverse(&mut denoms);
        let mut denom_inverses =
            vec![BaseField::zero(); 1 << (self.max_constraint_log_degree_bound())];
        BaseField::batch_inverse(&denoms, &mut denom_inverses);

        // Accumulate constraints.
        for (i, (num, denom)) in numerators.iter().zip(denom_inverses.iter()).enumerate() {
            accum.accumulate(i, *num * *denom);
        }
    }
}
