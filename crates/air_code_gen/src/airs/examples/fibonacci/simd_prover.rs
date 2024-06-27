#![allow(unused_variables)]
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

use super::component::Fib__100;

impl ComponentProver<SimdBackend> for Fib__100 {
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
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[97]);
            *numer += random_coeff
                * (trace_evals[1].data[i]
                    - ((PackedBaseField::broadcast(BaseField::from_u32_unchecked(1))
                        * PackedBaseField::broadcast(BaseField::from_u32_unchecked(1)))
                        + (trace_evals[0].data[i] * trace_evals[0].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[96]);
            *numer += random_coeff
                * (trace_evals[2].data[i]
                    - ((trace_evals[0].data[i] * trace_evals[0].data[i])
                        + (trace_evals[1].data[i] * trace_evals[1].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[95]);
            *numer += random_coeff
                * (trace_evals[3].data[i]
                    - ((trace_evals[1].data[i] * trace_evals[1].data[i])
                        + (trace_evals[2].data[i] * trace_evals[2].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[94]);
            *numer += random_coeff
                * (trace_evals[4].data[i]
                    - ((trace_evals[2].data[i] * trace_evals[2].data[i])
                        + (trace_evals[3].data[i] * trace_evals[3].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[93]);
            *numer += random_coeff
                * (trace_evals[5].data[i]
                    - ((trace_evals[3].data[i] * trace_evals[3].data[i])
                        + (trace_evals[4].data[i] * trace_evals[4].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[92]);
            *numer += random_coeff
                * (trace_evals[6].data[i]
                    - ((trace_evals[4].data[i] * trace_evals[4].data[i])
                        + (trace_evals[5].data[i] * trace_evals[5].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[91]);
            *numer += random_coeff
                * (trace_evals[7].data[i]
                    - ((trace_evals[5].data[i] * trace_evals[5].data[i])
                        + (trace_evals[6].data[i] * trace_evals[6].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[90]);
            *numer += random_coeff
                * (trace_evals[8].data[i]
                    - ((trace_evals[6].data[i] * trace_evals[6].data[i])
                        + (trace_evals[7].data[i] * trace_evals[7].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[89]);
            *numer += random_coeff
                * (trace_evals[9].data[i]
                    - ((trace_evals[7].data[i] * trace_evals[7].data[i])
                        + (trace_evals[8].data[i] * trace_evals[8].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[88]);
            *numer += random_coeff
                * (trace_evals[10].data[i]
                    - ((trace_evals[8].data[i] * trace_evals[8].data[i])
                        + (trace_evals[9].data[i] * trace_evals[9].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[87]);
            *numer += random_coeff
                * (trace_evals[11].data[i]
                    - ((trace_evals[9].data[i] * trace_evals[9].data[i])
                        + (trace_evals[10].data[i] * trace_evals[10].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[86]);
            *numer += random_coeff
                * (trace_evals[12].data[i]
                    - ((trace_evals[10].data[i] * trace_evals[10].data[i])
                        + (trace_evals[11].data[i] * trace_evals[11].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[85]);
            *numer += random_coeff
                * (trace_evals[13].data[i]
                    - ((trace_evals[11].data[i] * trace_evals[11].data[i])
                        + (trace_evals[12].data[i] * trace_evals[12].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[84]);
            *numer += random_coeff
                * (trace_evals[14].data[i]
                    - ((trace_evals[12].data[i] * trace_evals[12].data[i])
                        + (trace_evals[13].data[i] * trace_evals[13].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[83]);
            *numer += random_coeff
                * (trace_evals[15].data[i]
                    - ((trace_evals[13].data[i] * trace_evals[13].data[i])
                        + (trace_evals[14].data[i] * trace_evals[14].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[82]);
            *numer += random_coeff
                * (trace_evals[16].data[i]
                    - ((trace_evals[14].data[i] * trace_evals[14].data[i])
                        + (trace_evals[15].data[i] * trace_evals[15].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[81]);
            *numer += random_coeff
                * (trace_evals[17].data[i]
                    - ((trace_evals[15].data[i] * trace_evals[15].data[i])
                        + (trace_evals[16].data[i] * trace_evals[16].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[80]);
            *numer += random_coeff
                * (trace_evals[18].data[i]
                    - ((trace_evals[16].data[i] * trace_evals[16].data[i])
                        + (trace_evals[17].data[i] * trace_evals[17].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[79]);
            *numer += random_coeff
                * (trace_evals[19].data[i]
                    - ((trace_evals[17].data[i] * trace_evals[17].data[i])
                        + (trace_evals[18].data[i] * trace_evals[18].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[78]);
            *numer += random_coeff
                * (trace_evals[20].data[i]
                    - ((trace_evals[18].data[i] * trace_evals[18].data[i])
                        + (trace_evals[19].data[i] * trace_evals[19].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[77]);
            *numer += random_coeff
                * (trace_evals[21].data[i]
                    - ((trace_evals[19].data[i] * trace_evals[19].data[i])
                        + (trace_evals[20].data[i] * trace_evals[20].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[76]);
            *numer += random_coeff
                * (trace_evals[22].data[i]
                    - ((trace_evals[20].data[i] * trace_evals[20].data[i])
                        + (trace_evals[21].data[i] * trace_evals[21].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[75]);
            *numer += random_coeff
                * (trace_evals[23].data[i]
                    - ((trace_evals[21].data[i] * trace_evals[21].data[i])
                        + (trace_evals[22].data[i] * trace_evals[22].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[74]);
            *numer += random_coeff
                * (trace_evals[24].data[i]
                    - ((trace_evals[22].data[i] * trace_evals[22].data[i])
                        + (trace_evals[23].data[i] * trace_evals[23].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[73]);
            *numer += random_coeff
                * (trace_evals[25].data[i]
                    - ((trace_evals[23].data[i] * trace_evals[23].data[i])
                        + (trace_evals[24].data[i] * trace_evals[24].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[72]);
            *numer += random_coeff
                * (trace_evals[26].data[i]
                    - ((trace_evals[24].data[i] * trace_evals[24].data[i])
                        + (trace_evals[25].data[i] * trace_evals[25].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[71]);
            *numer += random_coeff
                * (trace_evals[27].data[i]
                    - ((trace_evals[25].data[i] * trace_evals[25].data[i])
                        + (trace_evals[26].data[i] * trace_evals[26].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[70]);
            *numer += random_coeff
                * (trace_evals[28].data[i]
                    - ((trace_evals[26].data[i] * trace_evals[26].data[i])
                        + (trace_evals[27].data[i] * trace_evals[27].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[69]);
            *numer += random_coeff
                * (trace_evals[29].data[i]
                    - ((trace_evals[27].data[i] * trace_evals[27].data[i])
                        + (trace_evals[28].data[i] * trace_evals[28].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[68]);
            *numer += random_coeff
                * (trace_evals[30].data[i]
                    - ((trace_evals[28].data[i] * trace_evals[28].data[i])
                        + (trace_evals[29].data[i] * trace_evals[29].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[67]);
            *numer += random_coeff
                * (trace_evals[31].data[i]
                    - ((trace_evals[29].data[i] * trace_evals[29].data[i])
                        + (trace_evals[30].data[i] * trace_evals[30].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[66]);
            *numer += random_coeff
                * (trace_evals[32].data[i]
                    - ((trace_evals[30].data[i] * trace_evals[30].data[i])
                        + (trace_evals[31].data[i] * trace_evals[31].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[65]);
            *numer += random_coeff
                * (trace_evals[33].data[i]
                    - ((trace_evals[31].data[i] * trace_evals[31].data[i])
                        + (trace_evals[32].data[i] * trace_evals[32].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[64]);
            *numer += random_coeff
                * (trace_evals[34].data[i]
                    - ((trace_evals[32].data[i] * trace_evals[32].data[i])
                        + (trace_evals[33].data[i] * trace_evals[33].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[63]);
            *numer += random_coeff
                * (trace_evals[35].data[i]
                    - ((trace_evals[33].data[i] * trace_evals[33].data[i])
                        + (trace_evals[34].data[i] * trace_evals[34].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[62]);
            *numer += random_coeff
                * (trace_evals[36].data[i]
                    - ((trace_evals[34].data[i] * trace_evals[34].data[i])
                        + (trace_evals[35].data[i] * trace_evals[35].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[61]);
            *numer += random_coeff
                * (trace_evals[37].data[i]
                    - ((trace_evals[35].data[i] * trace_evals[35].data[i])
                        + (trace_evals[36].data[i] * trace_evals[36].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[60]);
            *numer += random_coeff
                * (trace_evals[38].data[i]
                    - ((trace_evals[36].data[i] * trace_evals[36].data[i])
                        + (trace_evals[37].data[i] * trace_evals[37].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[59]);
            *numer += random_coeff
                * (trace_evals[39].data[i]
                    - ((trace_evals[37].data[i] * trace_evals[37].data[i])
                        + (trace_evals[38].data[i] * trace_evals[38].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[58]);
            *numer += random_coeff
                * (trace_evals[40].data[i]
                    - ((trace_evals[38].data[i] * trace_evals[38].data[i])
                        + (trace_evals[39].data[i] * trace_evals[39].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[57]);
            *numer += random_coeff
                * (trace_evals[41].data[i]
                    - ((trace_evals[39].data[i] * trace_evals[39].data[i])
                        + (trace_evals[40].data[i] * trace_evals[40].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[56]);
            *numer += random_coeff
                * (trace_evals[42].data[i]
                    - ((trace_evals[40].data[i] * trace_evals[40].data[i])
                        + (trace_evals[41].data[i] * trace_evals[41].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[55]);
            *numer += random_coeff
                * (trace_evals[43].data[i]
                    - ((trace_evals[41].data[i] * trace_evals[41].data[i])
                        + (trace_evals[42].data[i] * trace_evals[42].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[54]);
            *numer += random_coeff
                * (trace_evals[44].data[i]
                    - ((trace_evals[42].data[i] * trace_evals[42].data[i])
                        + (trace_evals[43].data[i] * trace_evals[43].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[53]);
            *numer += random_coeff
                * (trace_evals[45].data[i]
                    - ((trace_evals[43].data[i] * trace_evals[43].data[i])
                        + (trace_evals[44].data[i] * trace_evals[44].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[52]);
            *numer += random_coeff
                * (trace_evals[46].data[i]
                    - ((trace_evals[44].data[i] * trace_evals[44].data[i])
                        + (trace_evals[45].data[i] * trace_evals[45].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[51]);
            *numer += random_coeff
                * (trace_evals[47].data[i]
                    - ((trace_evals[45].data[i] * trace_evals[45].data[i])
                        + (trace_evals[46].data[i] * trace_evals[46].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[50]);
            *numer += random_coeff
                * (trace_evals[48].data[i]
                    - ((trace_evals[46].data[i] * trace_evals[46].data[i])
                        + (trace_evals[47].data[i] * trace_evals[47].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[49]);
            *numer += random_coeff
                * (trace_evals[49].data[i]
                    - ((trace_evals[47].data[i] * trace_evals[47].data[i])
                        + (trace_evals[48].data[i] * trace_evals[48].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[48]);
            *numer += random_coeff
                * (trace_evals[50].data[i]
                    - ((trace_evals[48].data[i] * trace_evals[48].data[i])
                        + (trace_evals[49].data[i] * trace_evals[49].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[47]);
            *numer += random_coeff
                * (trace_evals[51].data[i]
                    - ((trace_evals[49].data[i] * trace_evals[49].data[i])
                        + (trace_evals[50].data[i] * trace_evals[50].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[46]);
            *numer += random_coeff
                * (trace_evals[52].data[i]
                    - ((trace_evals[50].data[i] * trace_evals[50].data[i])
                        + (trace_evals[51].data[i] * trace_evals[51].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[45]);
            *numer += random_coeff
                * (trace_evals[53].data[i]
                    - ((trace_evals[51].data[i] * trace_evals[51].data[i])
                        + (trace_evals[52].data[i] * trace_evals[52].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[44]);
            *numer += random_coeff
                * (trace_evals[54].data[i]
                    - ((trace_evals[52].data[i] * trace_evals[52].data[i])
                        + (trace_evals[53].data[i] * trace_evals[53].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[43]);
            *numer += random_coeff
                * (trace_evals[55].data[i]
                    - ((trace_evals[53].data[i] * trace_evals[53].data[i])
                        + (trace_evals[54].data[i] * trace_evals[54].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[42]);
            *numer += random_coeff
                * (trace_evals[56].data[i]
                    - ((trace_evals[54].data[i] * trace_evals[54].data[i])
                        + (trace_evals[55].data[i] * trace_evals[55].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[41]);
            *numer += random_coeff
                * (trace_evals[57].data[i]
                    - ((trace_evals[55].data[i] * trace_evals[55].data[i])
                        + (trace_evals[56].data[i] * trace_evals[56].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[40]);
            *numer += random_coeff
                * (trace_evals[58].data[i]
                    - ((trace_evals[56].data[i] * trace_evals[56].data[i])
                        + (trace_evals[57].data[i] * trace_evals[57].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[39]);
            *numer += random_coeff
                * (trace_evals[59].data[i]
                    - ((trace_evals[57].data[i] * trace_evals[57].data[i])
                        + (trace_evals[58].data[i] * trace_evals[58].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[38]);
            *numer += random_coeff
                * (trace_evals[60].data[i]
                    - ((trace_evals[58].data[i] * trace_evals[58].data[i])
                        + (trace_evals[59].data[i] * trace_evals[59].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[37]);
            *numer += random_coeff
                * (trace_evals[61].data[i]
                    - ((trace_evals[59].data[i] * trace_evals[59].data[i])
                        + (trace_evals[60].data[i] * trace_evals[60].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[36]);
            *numer += random_coeff
                * (trace_evals[62].data[i]
                    - ((trace_evals[60].data[i] * trace_evals[60].data[i])
                        + (trace_evals[61].data[i] * trace_evals[61].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[35]);
            *numer += random_coeff
                * (trace_evals[63].data[i]
                    - ((trace_evals[61].data[i] * trace_evals[61].data[i])
                        + (trace_evals[62].data[i] * trace_evals[62].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[34]);
            *numer += random_coeff
                * (trace_evals[64].data[i]
                    - ((trace_evals[62].data[i] * trace_evals[62].data[i])
                        + (trace_evals[63].data[i] * trace_evals[63].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[33]);
            *numer += random_coeff
                * (trace_evals[65].data[i]
                    - ((trace_evals[63].data[i] * trace_evals[63].data[i])
                        + (trace_evals[64].data[i] * trace_evals[64].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[32]);
            *numer += random_coeff
                * (trace_evals[66].data[i]
                    - ((trace_evals[64].data[i] * trace_evals[64].data[i])
                        + (trace_evals[65].data[i] * trace_evals[65].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[31]);
            *numer += random_coeff
                * (trace_evals[67].data[i]
                    - ((trace_evals[65].data[i] * trace_evals[65].data[i])
                        + (trace_evals[66].data[i] * trace_evals[66].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[30]);
            *numer += random_coeff
                * (trace_evals[68].data[i]
                    - ((trace_evals[66].data[i] * trace_evals[66].data[i])
                        + (trace_evals[67].data[i] * trace_evals[67].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[29]);
            *numer += random_coeff
                * (trace_evals[69].data[i]
                    - ((trace_evals[67].data[i] * trace_evals[67].data[i])
                        + (trace_evals[68].data[i] * trace_evals[68].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[28]);
            *numer += random_coeff
                * (trace_evals[70].data[i]
                    - ((trace_evals[68].data[i] * trace_evals[68].data[i])
                        + (trace_evals[69].data[i] * trace_evals[69].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[27]);
            *numer += random_coeff
                * (trace_evals[71].data[i]
                    - ((trace_evals[69].data[i] * trace_evals[69].data[i])
                        + (trace_evals[70].data[i] * trace_evals[70].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[26]);
            *numer += random_coeff
                * (trace_evals[72].data[i]
                    - ((trace_evals[70].data[i] * trace_evals[70].data[i])
                        + (trace_evals[71].data[i] * trace_evals[71].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[25]);
            *numer += random_coeff
                * (trace_evals[73].data[i]
                    - ((trace_evals[71].data[i] * trace_evals[71].data[i])
                        + (trace_evals[72].data[i] * trace_evals[72].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[24]);
            *numer += random_coeff
                * (trace_evals[74].data[i]
                    - ((trace_evals[72].data[i] * trace_evals[72].data[i])
                        + (trace_evals[73].data[i] * trace_evals[73].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[23]);
            *numer += random_coeff
                * (trace_evals[75].data[i]
                    - ((trace_evals[73].data[i] * trace_evals[73].data[i])
                        + (trace_evals[74].data[i] * trace_evals[74].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[22]);
            *numer += random_coeff
                * (trace_evals[76].data[i]
                    - ((trace_evals[74].data[i] * trace_evals[74].data[i])
                        + (trace_evals[75].data[i] * trace_evals[75].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[21]);
            *numer += random_coeff
                * (trace_evals[77].data[i]
                    - ((trace_evals[75].data[i] * trace_evals[75].data[i])
                        + (trace_evals[76].data[i] * trace_evals[76].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[20]);
            *numer += random_coeff
                * (trace_evals[78].data[i]
                    - ((trace_evals[76].data[i] * trace_evals[76].data[i])
                        + (trace_evals[77].data[i] * trace_evals[77].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[19]);
            *numer += random_coeff
                * (trace_evals[79].data[i]
                    - ((trace_evals[77].data[i] * trace_evals[77].data[i])
                        + (trace_evals[78].data[i] * trace_evals[78].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[18]);
            *numer += random_coeff
                * (trace_evals[80].data[i]
                    - ((trace_evals[78].data[i] * trace_evals[78].data[i])
                        + (trace_evals[79].data[i] * trace_evals[79].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[17]);
            *numer += random_coeff
                * (trace_evals[81].data[i]
                    - ((trace_evals[79].data[i] * trace_evals[79].data[i])
                        + (trace_evals[80].data[i] * trace_evals[80].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[16]);
            *numer += random_coeff
                * (trace_evals[82].data[i]
                    - ((trace_evals[80].data[i] * trace_evals[80].data[i])
                        + (trace_evals[81].data[i] * trace_evals[81].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[15]);
            *numer += random_coeff
                * (trace_evals[83].data[i]
                    - ((trace_evals[81].data[i] * trace_evals[81].data[i])
                        + (trace_evals[82].data[i] * trace_evals[82].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[14]);
            *numer += random_coeff
                * (trace_evals[84].data[i]
                    - ((trace_evals[82].data[i] * trace_evals[82].data[i])
                        + (trace_evals[83].data[i] * trace_evals[83].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[13]);
            *numer += random_coeff
                * (trace_evals[85].data[i]
                    - ((trace_evals[83].data[i] * trace_evals[83].data[i])
                        + (trace_evals[84].data[i] * trace_evals[84].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[12]);
            *numer += random_coeff
                * (trace_evals[86].data[i]
                    - ((trace_evals[84].data[i] * trace_evals[84].data[i])
                        + (trace_evals[85].data[i] * trace_evals[85].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[11]);
            *numer += random_coeff
                * (trace_evals[87].data[i]
                    - ((trace_evals[85].data[i] * trace_evals[85].data[i])
                        + (trace_evals[86].data[i] * trace_evals[86].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[10]);
            *numer += random_coeff
                * (trace_evals[88].data[i]
                    - ((trace_evals[86].data[i] * trace_evals[86].data[i])
                        + (trace_evals[87].data[i] * trace_evals[87].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[9]);
            *numer += random_coeff
                * (trace_evals[89].data[i]
                    - ((trace_evals[87].data[i] * trace_evals[87].data[i])
                        + (trace_evals[88].data[i] * trace_evals[88].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[8]);
            *numer += random_coeff
                * (trace_evals[90].data[i]
                    - ((trace_evals[88].data[i] * trace_evals[88].data[i])
                        + (trace_evals[89].data[i] * trace_evals[89].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[7]);
            *numer += random_coeff
                * (trace_evals[91].data[i]
                    - ((trace_evals[89].data[i] * trace_evals[89].data[i])
                        + (trace_evals[90].data[i] * trace_evals[90].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[6]);
            *numer += random_coeff
                * (trace_evals[92].data[i]
                    - ((trace_evals[90].data[i] * trace_evals[90].data[i])
                        + (trace_evals[91].data[i] * trace_evals[91].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[5]);
            *numer += random_coeff
                * (trace_evals[93].data[i]
                    - ((trace_evals[91].data[i] * trace_evals[91].data[i])
                        + (trace_evals[92].data[i] * trace_evals[92].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[4]);
            *numer += random_coeff
                * (trace_evals[94].data[i]
                    - ((trace_evals[92].data[i] * trace_evals[92].data[i])
                        + (trace_evals[93].data[i] * trace_evals[93].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[3]);
            *numer += random_coeff
                * (trace_evals[95].data[i]
                    - ((trace_evals[93].data[i] * trace_evals[93].data[i])
                        + (trace_evals[94].data[i] * trace_evals[94].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[2]);
            *numer += random_coeff
                * (trace_evals[96].data[i]
                    - ((trace_evals[94].data[i] * trace_evals[94].data[i])
                        + (trace_evals[95].data[i] * trace_evals[95].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[1]);
            *numer += random_coeff
                * (trace_evals[97].data[i]
                    - ((trace_evals[95].data[i] * trace_evals[95].data[i])
                        + (trace_evals[96].data[i] * trace_evals[96].data[i])));
            let random_coeff = PackedSecureField::broadcast(random_coeff_powers[0]);
            *numer += random_coeff
                * (trace_evals[98].data[i]
                    - ((trace_evals[96].data[i] * trace_evals[96].data[i])
                        + (trace_evals[97].data[i] * trace_evals[97].data[i])));
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
