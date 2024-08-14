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

use super::component::BitUnpack_e0b35c6b3a8afa3d;

impl ComponentProver<SimdBackend> for BitUnpack_e0b35c6b3a8afa3d {
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
            let tmp_3 = (trace_evals[0].data[i]
                - (trace_evals[1].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[12]);
            *numer += random_coeff
                * (tmp_3 * (tmp_3 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_5 = (trace_evals[1].data[i]
                - (trace_evals[2].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[11]);
            *numer += random_coeff
                * (tmp_5 * (tmp_5 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_7 = (trace_evals[2].data[i]
                - (trace_evals[3].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[10]);
            *numer += random_coeff
                * (tmp_7 * (tmp_7 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_9 = (trace_evals[3].data[i]
                - (trace_evals[4].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[9]);
            *numer += random_coeff
                * (tmp_9 * (tmp_9 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_11 = (trace_evals[4].data[i]
                - (trace_evals[5].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[8]);
            *numer += random_coeff
                * (tmp_11
                    * (tmp_11 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_13 = (trace_evals[5].data[i]
                - (trace_evals[6].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[7]);
            *numer += random_coeff
                * (tmp_13
                    * (tmp_13 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_15 = (trace_evals[6].data[i]
                - (trace_evals[7].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[6]);
            *numer += random_coeff
                * (tmp_15
                    * (tmp_15 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_17 = (trace_evals[7].data[i]
                - (trace_evals[8].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[5]);
            *numer += random_coeff
                * (tmp_17
                    * (tmp_17 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_19 = (trace_evals[8].data[i]
                - (trace_evals[9].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[4]);
            *numer += random_coeff
                * (tmp_19
                    * (tmp_19 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_21 = (trace_evals[9].data[i]
                - (trace_evals[10].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[3]);
            *numer += random_coeff
                * (tmp_21
                    * (tmp_21 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_23 = (trace_evals[10].data[i]
                - (trace_evals[11].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[2]);
            *numer += random_coeff
                * (tmp_23
                    * (tmp_23 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let tmp_25 = (trace_evals[11].data[i]
                - (trace_evals[12].data[i]
                    * PackedBaseField::broadcast(BaseField::from_u32_unchecked(2))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[1]);
            *numer += random_coeff
                * (tmp_25
                    * (tmp_25 - PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[0]);
            *numer += random_coeff * (trace_evals[12].data[i]);
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
