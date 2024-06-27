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

use super::component::NarrowFib__20;

impl ComponentProver<CpuBackend> for NarrowFib__20 {
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
            *numer += accum.random_coeff_powers[19]
                * (trace_evals[2].values.at(i)
                    - ((trace_evals[0].values.at(i) * trace_evals[0].values.at(i))
                        + (trace_evals[1].values.at(i) * trace_evals[1].values.at(i))));
            *numer += accum.random_coeff_powers[18]
                * (trace_evals[3].values.at(i)
                    - ((trace_evals[1].values.at(i) * trace_evals[1].values.at(i))
                        + (trace_evals[2].values.at(i) * trace_evals[2].values.at(i))));
            *numer += accum.random_coeff_powers[17]
                * (trace_evals[4].values.at(i)
                    - ((trace_evals[2].values.at(i) * trace_evals[2].values.at(i))
                        + (trace_evals[3].values.at(i) * trace_evals[3].values.at(i))));
            *numer += accum.random_coeff_powers[16]
                * (trace_evals[5].values.at(i)
                    - ((trace_evals[3].values.at(i) * trace_evals[3].values.at(i))
                        + (trace_evals[4].values.at(i) * trace_evals[4].values.at(i))));
            *numer += accum.random_coeff_powers[15]
                * (trace_evals[6].values.at(i)
                    - ((trace_evals[4].values.at(i) * trace_evals[4].values.at(i))
                        + (trace_evals[5].values.at(i) * trace_evals[5].values.at(i))));
            *numer += accum.random_coeff_powers[14]
                * (trace_evals[7].values.at(i)
                    - ((trace_evals[5].values.at(i) * trace_evals[5].values.at(i))
                        + (trace_evals[6].values.at(i) * trace_evals[6].values.at(i))));
            *numer += accum.random_coeff_powers[13]
                * (trace_evals[8].values.at(i)
                    - ((trace_evals[6].values.at(i) * trace_evals[6].values.at(i))
                        + (trace_evals[7].values.at(i) * trace_evals[7].values.at(i))));
            *numer += accum.random_coeff_powers[12]
                * (trace_evals[9].values.at(i)
                    - ((trace_evals[7].values.at(i) * trace_evals[7].values.at(i))
                        + (trace_evals[8].values.at(i) * trace_evals[8].values.at(i))));
            *numer += accum.random_coeff_powers[11]
                * (trace_evals[10].values.at(i)
                    - ((trace_evals[8].values.at(i) * trace_evals[8].values.at(i))
                        + (trace_evals[9].values.at(i) * trace_evals[9].values.at(i))));
            *numer += accum.random_coeff_powers[10]
                * (trace_evals[11].values.at(i)
                    - ((trace_evals[9].values.at(i) * trace_evals[9].values.at(i))
                        + (trace_evals[10].values.at(i) * trace_evals[10].values.at(i))));
            *numer += accum.random_coeff_powers[9]
                * (trace_evals[12].values.at(i)
                    - ((trace_evals[10].values.at(i) * trace_evals[10].values.at(i))
                        + (trace_evals[11].values.at(i) * trace_evals[11].values.at(i))));
            *numer += accum.random_coeff_powers[8]
                * (trace_evals[13].values.at(i)
                    - ((trace_evals[11].values.at(i) * trace_evals[11].values.at(i))
                        + (trace_evals[12].values.at(i) * trace_evals[12].values.at(i))));
            *numer += accum.random_coeff_powers[7]
                * (trace_evals[14].values.at(i)
                    - ((trace_evals[12].values.at(i) * trace_evals[12].values.at(i))
                        + (trace_evals[13].values.at(i) * trace_evals[13].values.at(i))));
            *numer += accum.random_coeff_powers[6]
                * (trace_evals[15].values.at(i)
                    - ((trace_evals[13].values.at(i) * trace_evals[13].values.at(i))
                        + (trace_evals[14].values.at(i) * trace_evals[14].values.at(i))));
            *numer += accum.random_coeff_powers[5]
                * (trace_evals[16].values.at(i)
                    - ((trace_evals[14].values.at(i) * trace_evals[14].values.at(i))
                        + (trace_evals[15].values.at(i) * trace_evals[15].values.at(i))));
            *numer += accum.random_coeff_powers[4]
                * (trace_evals[17].values.at(i)
                    - ((trace_evals[15].values.at(i) * trace_evals[15].values.at(i))
                        + (trace_evals[16].values.at(i) * trace_evals[16].values.at(i))));
            *numer += accum.random_coeff_powers[3]
                * (trace_evals[18].values.at(i)
                    - ((trace_evals[16].values.at(i) * trace_evals[16].values.at(i))
                        + (trace_evals[17].values.at(i) * trace_evals[17].values.at(i))));
            *numer += accum.random_coeff_powers[2]
                * (trace_evals[19].values.at(i)
                    - ((trace_evals[17].values.at(i) * trace_evals[17].values.at(i))
                        + (trace_evals[18].values.at(i) * trace_evals[18].values.at(i))));
            *numer += accum.random_coeff_powers[1]
                * (trace_evals[20].values.at(i)
                    - ((trace_evals[18].values.at(i) * trace_evals[18].values.at(i))
                        + (trace_evals[19].values.at(i) * trace_evals[19].values.at(i))));
            *numer += accum.random_coeff_powers[0]
                * (trace_evals[21].values.at(i)
                    - ((trace_evals[19].values.at(i) * trace_evals[19].values.at(i))
                        + (trace_evals[20].values.at(i) * trace_evals[20].values.at(i))));
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
