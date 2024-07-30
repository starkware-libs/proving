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
            let constraint_tmp_3 = (trace_evals[0].values.at(i)
                - (trace_evals[1].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[12]
                * (constraint_tmp_3 * (constraint_tmp_3 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_5 = (trace_evals[1].values.at(i)
                - (trace_evals[2].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[11]
                * (constraint_tmp_5 * (constraint_tmp_5 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_7 = (trace_evals[2].values.at(i)
                - (trace_evals[3].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[10]
                * (constraint_tmp_7 * (constraint_tmp_7 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_9 = (trace_evals[3].values.at(i)
                - (trace_evals[4].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[9]
                * (constraint_tmp_9 * (constraint_tmp_9 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_11 = (trace_evals[4].values.at(i)
                - (trace_evals[5].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[8]
                * (constraint_tmp_11 * (constraint_tmp_11 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_13 = (trace_evals[5].values.at(i)
                - (trace_evals[6].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[7]
                * (constraint_tmp_13 * (constraint_tmp_13 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_15 = (trace_evals[6].values.at(i)
                - (trace_evals[7].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[6]
                * (constraint_tmp_15 * (constraint_tmp_15 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_17 = (trace_evals[7].values.at(i)
                - (trace_evals[8].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[5]
                * (constraint_tmp_17 * (constraint_tmp_17 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_19 = (trace_evals[8].values.at(i)
                - (trace_evals[9].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[4]
                * (constraint_tmp_19 * (constraint_tmp_19 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_21 = (trace_evals[9].values.at(i)
                - (trace_evals[10].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[3]
                * (constraint_tmp_21 * (constraint_tmp_21 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_23 = (trace_evals[10].values.at(i)
                - (trace_evals[11].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[2]
                * (constraint_tmp_23 * (constraint_tmp_23 - BaseField::from_u32_unchecked(1)));
            let constraint_tmp_25 = (trace_evals[11].values.at(i)
                - (trace_evals[12].values.at(i) * BaseField::from_u32_unchecked(2)));
            *numer += accum.random_coeff_powers[1]
                * (constraint_tmp_25 * (constraint_tmp_25 - BaseField::from_u32_unchecked(1)));
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
