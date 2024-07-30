#![allow(unused_imports)]
use stwo_prover::core::air::accumulation::DomainEvaluationAccumulator;
use stwo_prover::core::air::{Component, ComponentProver, ComponentTrace};
use stwo_prover::core::backend::simd::column::{BaseFieldVec, SecureFieldVec};
use stwo_prover::core::backend::simd::m31::PackedBaseField;
use stwo_prover::core::backend::simd::qm31::PackedSecureField;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{Column, ColumnOps};
use stwo_prover::core::constraints::coset_vanishing;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::fields::FieldOps;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::InteractionElements;

use super::component::NarrowFib_1ddf31c88316e62f;

impl ComponentProver<SimdBackend> for NarrowFib_1ddf31c88316e62f {
    #[allow(unused_parens)]
    fn evaluate_constraint_quotients_on_domain(
        &self,
        trace: &ComponentTrace<'_, SimdBackend>,
        evaluation_accumulator: &mut DomainEvaluationAccumulator<SimdBackend>,
        _interaction_elements: &InteractionElements,
    ) {
        // Numerator computation.
        let trace_evals = &trace.evals[0];
        let mut numerators = SecureFieldVec::zeros(1 << (self.max_constraint_log_degree_bound()));
        let [accum] = evaluation_accumulator
            .columns([(self.max_constraint_log_degree_bound(), self.n_constraints())]);
        let random_coeff_powers = &accum.random_coeff_powers;
        for (i, numer) in numerators.data.iter_mut().enumerate() {
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[19]);
            *numer += random_coeff
                * (trace_evals[2].data[i]
                    - ((trace_evals[0].data[i] * trace_evals[0].data[i])
                        + (trace_evals[1].data[i] * trace_evals[1].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[18]);
            *numer += random_coeff
                * (trace_evals[3].data[i]
                    - ((trace_evals[1].data[i] * trace_evals[1].data[i])
                        + (trace_evals[2].data[i] * trace_evals[2].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[17]);
            *numer += random_coeff
                * (trace_evals[4].data[i]
                    - ((trace_evals[2].data[i] * trace_evals[2].data[i])
                        + (trace_evals[3].data[i] * trace_evals[3].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[16]);
            *numer += random_coeff
                * (trace_evals[5].data[i]
                    - ((trace_evals[3].data[i] * trace_evals[3].data[i])
                        + (trace_evals[4].data[i] * trace_evals[4].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[15]);
            *numer += random_coeff
                * (trace_evals[6].data[i]
                    - ((trace_evals[4].data[i] * trace_evals[4].data[i])
                        + (trace_evals[5].data[i] * trace_evals[5].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[14]);
            *numer += random_coeff
                * (trace_evals[7].data[i]
                    - ((trace_evals[5].data[i] * trace_evals[5].data[i])
                        + (trace_evals[6].data[i] * trace_evals[6].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[13]);
            *numer += random_coeff
                * (trace_evals[8].data[i]
                    - ((trace_evals[6].data[i] * trace_evals[6].data[i])
                        + (trace_evals[7].data[i] * trace_evals[7].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[12]);
            *numer += random_coeff
                * (trace_evals[9].data[i]
                    - ((trace_evals[7].data[i] * trace_evals[7].data[i])
                        + (trace_evals[8].data[i] * trace_evals[8].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[11]);
            *numer += random_coeff
                * (trace_evals[10].data[i]
                    - ((trace_evals[8].data[i] * trace_evals[8].data[i])
                        + (trace_evals[9].data[i] * trace_evals[9].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[10]);
            *numer += random_coeff
                * (trace_evals[11].data[i]
                    - ((trace_evals[9].data[i] * trace_evals[9].data[i])
                        + (trace_evals[10].data[i] * trace_evals[10].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[9]);
            *numer += random_coeff
                * (trace_evals[12].data[i]
                    - ((trace_evals[10].data[i] * trace_evals[10].data[i])
                        + (trace_evals[11].data[i] * trace_evals[11].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[8]);
            *numer += random_coeff
                * (trace_evals[13].data[i]
                    - ((trace_evals[11].data[i] * trace_evals[11].data[i])
                        + (trace_evals[12].data[i] * trace_evals[12].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[7]);
            *numer += random_coeff
                * (trace_evals[14].data[i]
                    - ((trace_evals[12].data[i] * trace_evals[12].data[i])
                        + (trace_evals[13].data[i] * trace_evals[13].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[6]);
            *numer += random_coeff
                * (trace_evals[15].data[i]
                    - ((trace_evals[13].data[i] * trace_evals[13].data[i])
                        + (trace_evals[14].data[i] * trace_evals[14].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[5]);
            *numer += random_coeff
                * (trace_evals[16].data[i]
                    - ((trace_evals[14].data[i] * trace_evals[14].data[i])
                        + (trace_evals[15].data[i] * trace_evals[15].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[4]);
            *numer += random_coeff
                * (trace_evals[17].data[i]
                    - ((trace_evals[15].data[i] * trace_evals[15].data[i])
                        + (trace_evals[16].data[i] * trace_evals[16].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[3]);
            *numer += random_coeff
                * (trace_evals[18].data[i]
                    - ((trace_evals[16].data[i] * trace_evals[16].data[i])
                        + (trace_evals[17].data[i] * trace_evals[17].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[2]);
            *numer += random_coeff
                * (trace_evals[19].data[i]
                    - ((trace_evals[17].data[i] * trace_evals[17].data[i])
                        + (trace_evals[18].data[i] * trace_evals[18].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[1]);
            *numer += random_coeff
                * (trace_evals[20].data[i]
                    - ((trace_evals[18].data[i] * trace_evals[18].data[i])
                        + (trace_evals[19].data[i] * trace_evals[19].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[0]);
            *numer += random_coeff
                * (trace_evals[21].data[i]
                    - ((trace_evals[19].data[i] * trace_evals[19].data[i])
                        + (trace_evals[20].data[i] * trace_evals[20].data[i])));
        }

        // Denominator computation.
        let zero_domain = CanonicCoset::new(self.log_n_instances).coset;
        let eval_domain = CanonicCoset::new(self.max_constraint_log_degree_bound()).circle_domain();
        let mut denoms =
            BaseFieldVec::from_iter(eval_domain.iter().map(|p| coset_vanishing(zero_domain, p)));
        <SimdBackend as ColumnOps<BaseField>>::bit_reverse_column(&mut denoms);
        let mut denom_inverses = BaseFieldVec::zeros(denoms.len());
        <SimdBackend as FieldOps<BaseField>>::batch_inverse(&denoms, &mut denom_inverses);

        // Accumulate constraints.
        for (i, (num, denom)) in numerators
            .data
            .iter()
            .zip(denom_inverses.data.iter())
            .enumerate()
        {
            unsafe { accum.col.set_packed(i, *num * *denom) };
        }
    }
}
