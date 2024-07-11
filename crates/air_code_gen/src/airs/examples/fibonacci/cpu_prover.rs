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

use super::component::Fib__100;

impl ComponentProver<CpuBackend> for Fib__100 {
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
            *numer += accum.random_coeff_powers[97]
                * (trace_evals[1].values.at(i)
                    - (BaseField::from_u32_unchecked(1)
                        + (trace_evals[0].values.at(i) * trace_evals[0].values.at(i))));
            *numer += accum.random_coeff_powers[96]
                * (trace_evals[2].values.at(i)
                    - ((trace_evals[0].values.at(i) * trace_evals[0].values.at(i))
                        + (trace_evals[1].values.at(i) * trace_evals[1].values.at(i))));
            *numer += accum.random_coeff_powers[95]
                * (trace_evals[3].values.at(i)
                    - ((trace_evals[1].values.at(i) * trace_evals[1].values.at(i))
                        + (trace_evals[2].values.at(i) * trace_evals[2].values.at(i))));
            *numer += accum.random_coeff_powers[94]
                * (trace_evals[4].values.at(i)
                    - ((trace_evals[2].values.at(i) * trace_evals[2].values.at(i))
                        + (trace_evals[3].values.at(i) * trace_evals[3].values.at(i))));
            *numer += accum.random_coeff_powers[93]
                * (trace_evals[5].values.at(i)
                    - ((trace_evals[3].values.at(i) * trace_evals[3].values.at(i))
                        + (trace_evals[4].values.at(i) * trace_evals[4].values.at(i))));
            *numer += accum.random_coeff_powers[92]
                * (trace_evals[6].values.at(i)
                    - ((trace_evals[4].values.at(i) * trace_evals[4].values.at(i))
                        + (trace_evals[5].values.at(i) * trace_evals[5].values.at(i))));
            *numer += accum.random_coeff_powers[91]
                * (trace_evals[7].values.at(i)
                    - ((trace_evals[5].values.at(i) * trace_evals[5].values.at(i))
                        + (trace_evals[6].values.at(i) * trace_evals[6].values.at(i))));
            *numer += accum.random_coeff_powers[90]
                * (trace_evals[8].values.at(i)
                    - ((trace_evals[6].values.at(i) * trace_evals[6].values.at(i))
                        + (trace_evals[7].values.at(i) * trace_evals[7].values.at(i))));
            *numer += accum.random_coeff_powers[89]
                * (trace_evals[9].values.at(i)
                    - ((trace_evals[7].values.at(i) * trace_evals[7].values.at(i))
                        + (trace_evals[8].values.at(i) * trace_evals[8].values.at(i))));
            *numer += accum.random_coeff_powers[88]
                * (trace_evals[10].values.at(i)
                    - ((trace_evals[8].values.at(i) * trace_evals[8].values.at(i))
                        + (trace_evals[9].values.at(i) * trace_evals[9].values.at(i))));
            *numer += accum.random_coeff_powers[87]
                * (trace_evals[11].values.at(i)
                    - ((trace_evals[9].values.at(i) * trace_evals[9].values.at(i))
                        + (trace_evals[10].values.at(i) * trace_evals[10].values.at(i))));
            *numer += accum.random_coeff_powers[86]
                * (trace_evals[12].values.at(i)
                    - ((trace_evals[10].values.at(i) * trace_evals[10].values.at(i))
                        + (trace_evals[11].values.at(i) * trace_evals[11].values.at(i))));
            *numer += accum.random_coeff_powers[85]
                * (trace_evals[13].values.at(i)
                    - ((trace_evals[11].values.at(i) * trace_evals[11].values.at(i))
                        + (trace_evals[12].values.at(i) * trace_evals[12].values.at(i))));
            *numer += accum.random_coeff_powers[84]
                * (trace_evals[14].values.at(i)
                    - ((trace_evals[12].values.at(i) * trace_evals[12].values.at(i))
                        + (trace_evals[13].values.at(i) * trace_evals[13].values.at(i))));
            *numer += accum.random_coeff_powers[83]
                * (trace_evals[15].values.at(i)
                    - ((trace_evals[13].values.at(i) * trace_evals[13].values.at(i))
                        + (trace_evals[14].values.at(i) * trace_evals[14].values.at(i))));
            *numer += accum.random_coeff_powers[82]
                * (trace_evals[16].values.at(i)
                    - ((trace_evals[14].values.at(i) * trace_evals[14].values.at(i))
                        + (trace_evals[15].values.at(i) * trace_evals[15].values.at(i))));
            *numer += accum.random_coeff_powers[81]
                * (trace_evals[17].values.at(i)
                    - ((trace_evals[15].values.at(i) * trace_evals[15].values.at(i))
                        + (trace_evals[16].values.at(i) * trace_evals[16].values.at(i))));
            *numer += accum.random_coeff_powers[80]
                * (trace_evals[18].values.at(i)
                    - ((trace_evals[16].values.at(i) * trace_evals[16].values.at(i))
                        + (trace_evals[17].values.at(i) * trace_evals[17].values.at(i))));
            *numer += accum.random_coeff_powers[79]
                * (trace_evals[19].values.at(i)
                    - ((trace_evals[17].values.at(i) * trace_evals[17].values.at(i))
                        + (trace_evals[18].values.at(i) * trace_evals[18].values.at(i))));
            *numer += accum.random_coeff_powers[78]
                * (trace_evals[20].values.at(i)
                    - ((trace_evals[18].values.at(i) * trace_evals[18].values.at(i))
                        + (trace_evals[19].values.at(i) * trace_evals[19].values.at(i))));
            *numer += accum.random_coeff_powers[77]
                * (trace_evals[21].values.at(i)
                    - ((trace_evals[19].values.at(i) * trace_evals[19].values.at(i))
                        + (trace_evals[20].values.at(i) * trace_evals[20].values.at(i))));
            *numer += accum.random_coeff_powers[76]
                * (trace_evals[22].values.at(i)
                    - ((trace_evals[20].values.at(i) * trace_evals[20].values.at(i))
                        + (trace_evals[21].values.at(i) * trace_evals[21].values.at(i))));
            *numer += accum.random_coeff_powers[75]
                * (trace_evals[23].values.at(i)
                    - ((trace_evals[21].values.at(i) * trace_evals[21].values.at(i))
                        + (trace_evals[22].values.at(i) * trace_evals[22].values.at(i))));
            *numer += accum.random_coeff_powers[74]
                * (trace_evals[24].values.at(i)
                    - ((trace_evals[22].values.at(i) * trace_evals[22].values.at(i))
                        + (trace_evals[23].values.at(i) * trace_evals[23].values.at(i))));
            *numer += accum.random_coeff_powers[73]
                * (trace_evals[25].values.at(i)
                    - ((trace_evals[23].values.at(i) * trace_evals[23].values.at(i))
                        + (trace_evals[24].values.at(i) * trace_evals[24].values.at(i))));
            *numer += accum.random_coeff_powers[72]
                * (trace_evals[26].values.at(i)
                    - ((trace_evals[24].values.at(i) * trace_evals[24].values.at(i))
                        + (trace_evals[25].values.at(i) * trace_evals[25].values.at(i))));
            *numer += accum.random_coeff_powers[71]
                * (trace_evals[27].values.at(i)
                    - ((trace_evals[25].values.at(i) * trace_evals[25].values.at(i))
                        + (trace_evals[26].values.at(i) * trace_evals[26].values.at(i))));
            *numer += accum.random_coeff_powers[70]
                * (trace_evals[28].values.at(i)
                    - ((trace_evals[26].values.at(i) * trace_evals[26].values.at(i))
                        + (trace_evals[27].values.at(i) * trace_evals[27].values.at(i))));
            *numer += accum.random_coeff_powers[69]
                * (trace_evals[29].values.at(i)
                    - ((trace_evals[27].values.at(i) * trace_evals[27].values.at(i))
                        + (trace_evals[28].values.at(i) * trace_evals[28].values.at(i))));
            *numer += accum.random_coeff_powers[68]
                * (trace_evals[30].values.at(i)
                    - ((trace_evals[28].values.at(i) * trace_evals[28].values.at(i))
                        + (trace_evals[29].values.at(i) * trace_evals[29].values.at(i))));
            *numer += accum.random_coeff_powers[67]
                * (trace_evals[31].values.at(i)
                    - ((trace_evals[29].values.at(i) * trace_evals[29].values.at(i))
                        + (trace_evals[30].values.at(i) * trace_evals[30].values.at(i))));
            *numer += accum.random_coeff_powers[66]
                * (trace_evals[32].values.at(i)
                    - ((trace_evals[30].values.at(i) * trace_evals[30].values.at(i))
                        + (trace_evals[31].values.at(i) * trace_evals[31].values.at(i))));
            *numer += accum.random_coeff_powers[65]
                * (trace_evals[33].values.at(i)
                    - ((trace_evals[31].values.at(i) * trace_evals[31].values.at(i))
                        + (trace_evals[32].values.at(i) * trace_evals[32].values.at(i))));
            *numer += accum.random_coeff_powers[64]
                * (trace_evals[34].values.at(i)
                    - ((trace_evals[32].values.at(i) * trace_evals[32].values.at(i))
                        + (trace_evals[33].values.at(i) * trace_evals[33].values.at(i))));
            *numer += accum.random_coeff_powers[63]
                * (trace_evals[35].values.at(i)
                    - ((trace_evals[33].values.at(i) * trace_evals[33].values.at(i))
                        + (trace_evals[34].values.at(i) * trace_evals[34].values.at(i))));
            *numer += accum.random_coeff_powers[62]
                * (trace_evals[36].values.at(i)
                    - ((trace_evals[34].values.at(i) * trace_evals[34].values.at(i))
                        + (trace_evals[35].values.at(i) * trace_evals[35].values.at(i))));
            *numer += accum.random_coeff_powers[61]
                * (trace_evals[37].values.at(i)
                    - ((trace_evals[35].values.at(i) * trace_evals[35].values.at(i))
                        + (trace_evals[36].values.at(i) * trace_evals[36].values.at(i))));
            *numer += accum.random_coeff_powers[60]
                * (trace_evals[38].values.at(i)
                    - ((trace_evals[36].values.at(i) * trace_evals[36].values.at(i))
                        + (trace_evals[37].values.at(i) * trace_evals[37].values.at(i))));
            *numer += accum.random_coeff_powers[59]
                * (trace_evals[39].values.at(i)
                    - ((trace_evals[37].values.at(i) * trace_evals[37].values.at(i))
                        + (trace_evals[38].values.at(i) * trace_evals[38].values.at(i))));
            *numer += accum.random_coeff_powers[58]
                * (trace_evals[40].values.at(i)
                    - ((trace_evals[38].values.at(i) * trace_evals[38].values.at(i))
                        + (trace_evals[39].values.at(i) * trace_evals[39].values.at(i))));
            *numer += accum.random_coeff_powers[57]
                * (trace_evals[41].values.at(i)
                    - ((trace_evals[39].values.at(i) * trace_evals[39].values.at(i))
                        + (trace_evals[40].values.at(i) * trace_evals[40].values.at(i))));
            *numer += accum.random_coeff_powers[56]
                * (trace_evals[42].values.at(i)
                    - ((trace_evals[40].values.at(i) * trace_evals[40].values.at(i))
                        + (trace_evals[41].values.at(i) * trace_evals[41].values.at(i))));
            *numer += accum.random_coeff_powers[55]
                * (trace_evals[43].values.at(i)
                    - ((trace_evals[41].values.at(i) * trace_evals[41].values.at(i))
                        + (trace_evals[42].values.at(i) * trace_evals[42].values.at(i))));
            *numer += accum.random_coeff_powers[54]
                * (trace_evals[44].values.at(i)
                    - ((trace_evals[42].values.at(i) * trace_evals[42].values.at(i))
                        + (trace_evals[43].values.at(i) * trace_evals[43].values.at(i))));
            *numer += accum.random_coeff_powers[53]
                * (trace_evals[45].values.at(i)
                    - ((trace_evals[43].values.at(i) * trace_evals[43].values.at(i))
                        + (trace_evals[44].values.at(i) * trace_evals[44].values.at(i))));
            *numer += accum.random_coeff_powers[52]
                * (trace_evals[46].values.at(i)
                    - ((trace_evals[44].values.at(i) * trace_evals[44].values.at(i))
                        + (trace_evals[45].values.at(i) * trace_evals[45].values.at(i))));
            *numer += accum.random_coeff_powers[51]
                * (trace_evals[47].values.at(i)
                    - ((trace_evals[45].values.at(i) * trace_evals[45].values.at(i))
                        + (trace_evals[46].values.at(i) * trace_evals[46].values.at(i))));
            *numer += accum.random_coeff_powers[50]
                * (trace_evals[48].values.at(i)
                    - ((trace_evals[46].values.at(i) * trace_evals[46].values.at(i))
                        + (trace_evals[47].values.at(i) * trace_evals[47].values.at(i))));
            *numer += accum.random_coeff_powers[49]
                * (trace_evals[49].values.at(i)
                    - ((trace_evals[47].values.at(i) * trace_evals[47].values.at(i))
                        + (trace_evals[48].values.at(i) * trace_evals[48].values.at(i))));
            *numer += accum.random_coeff_powers[48]
                * (trace_evals[50].values.at(i)
                    - ((trace_evals[48].values.at(i) * trace_evals[48].values.at(i))
                        + (trace_evals[49].values.at(i) * trace_evals[49].values.at(i))));
            *numer += accum.random_coeff_powers[47]
                * (trace_evals[51].values.at(i)
                    - ((trace_evals[49].values.at(i) * trace_evals[49].values.at(i))
                        + (trace_evals[50].values.at(i) * trace_evals[50].values.at(i))));
            *numer += accum.random_coeff_powers[46]
                * (trace_evals[52].values.at(i)
                    - ((trace_evals[50].values.at(i) * trace_evals[50].values.at(i))
                        + (trace_evals[51].values.at(i) * trace_evals[51].values.at(i))));
            *numer += accum.random_coeff_powers[45]
                * (trace_evals[53].values.at(i)
                    - ((trace_evals[51].values.at(i) * trace_evals[51].values.at(i))
                        + (trace_evals[52].values.at(i) * trace_evals[52].values.at(i))));
            *numer += accum.random_coeff_powers[44]
                * (trace_evals[54].values.at(i)
                    - ((trace_evals[52].values.at(i) * trace_evals[52].values.at(i))
                        + (trace_evals[53].values.at(i) * trace_evals[53].values.at(i))));
            *numer += accum.random_coeff_powers[43]
                * (trace_evals[55].values.at(i)
                    - ((trace_evals[53].values.at(i) * trace_evals[53].values.at(i))
                        + (trace_evals[54].values.at(i) * trace_evals[54].values.at(i))));
            *numer += accum.random_coeff_powers[42]
                * (trace_evals[56].values.at(i)
                    - ((trace_evals[54].values.at(i) * trace_evals[54].values.at(i))
                        + (trace_evals[55].values.at(i) * trace_evals[55].values.at(i))));
            *numer += accum.random_coeff_powers[41]
                * (trace_evals[57].values.at(i)
                    - ((trace_evals[55].values.at(i) * trace_evals[55].values.at(i))
                        + (trace_evals[56].values.at(i) * trace_evals[56].values.at(i))));
            *numer += accum.random_coeff_powers[40]
                * (trace_evals[58].values.at(i)
                    - ((trace_evals[56].values.at(i) * trace_evals[56].values.at(i))
                        + (trace_evals[57].values.at(i) * trace_evals[57].values.at(i))));
            *numer += accum.random_coeff_powers[39]
                * (trace_evals[59].values.at(i)
                    - ((trace_evals[57].values.at(i) * trace_evals[57].values.at(i))
                        + (trace_evals[58].values.at(i) * trace_evals[58].values.at(i))));
            *numer += accum.random_coeff_powers[38]
                * (trace_evals[60].values.at(i)
                    - ((trace_evals[58].values.at(i) * trace_evals[58].values.at(i))
                        + (trace_evals[59].values.at(i) * trace_evals[59].values.at(i))));
            *numer += accum.random_coeff_powers[37]
                * (trace_evals[61].values.at(i)
                    - ((trace_evals[59].values.at(i) * trace_evals[59].values.at(i))
                        + (trace_evals[60].values.at(i) * trace_evals[60].values.at(i))));
            *numer += accum.random_coeff_powers[36]
                * (trace_evals[62].values.at(i)
                    - ((trace_evals[60].values.at(i) * trace_evals[60].values.at(i))
                        + (trace_evals[61].values.at(i) * trace_evals[61].values.at(i))));
            *numer += accum.random_coeff_powers[35]
                * (trace_evals[63].values.at(i)
                    - ((trace_evals[61].values.at(i) * trace_evals[61].values.at(i))
                        + (trace_evals[62].values.at(i) * trace_evals[62].values.at(i))));
            *numer += accum.random_coeff_powers[34]
                * (trace_evals[64].values.at(i)
                    - ((trace_evals[62].values.at(i) * trace_evals[62].values.at(i))
                        + (trace_evals[63].values.at(i) * trace_evals[63].values.at(i))));
            *numer += accum.random_coeff_powers[33]
                * (trace_evals[65].values.at(i)
                    - ((trace_evals[63].values.at(i) * trace_evals[63].values.at(i))
                        + (trace_evals[64].values.at(i) * trace_evals[64].values.at(i))));
            *numer += accum.random_coeff_powers[32]
                * (trace_evals[66].values.at(i)
                    - ((trace_evals[64].values.at(i) * trace_evals[64].values.at(i))
                        + (trace_evals[65].values.at(i) * trace_evals[65].values.at(i))));
            *numer += accum.random_coeff_powers[31]
                * (trace_evals[67].values.at(i)
                    - ((trace_evals[65].values.at(i) * trace_evals[65].values.at(i))
                        + (trace_evals[66].values.at(i) * trace_evals[66].values.at(i))));
            *numer += accum.random_coeff_powers[30]
                * (trace_evals[68].values.at(i)
                    - ((trace_evals[66].values.at(i) * trace_evals[66].values.at(i))
                        + (trace_evals[67].values.at(i) * trace_evals[67].values.at(i))));
            *numer += accum.random_coeff_powers[29]
                * (trace_evals[69].values.at(i)
                    - ((trace_evals[67].values.at(i) * trace_evals[67].values.at(i))
                        + (trace_evals[68].values.at(i) * trace_evals[68].values.at(i))));
            *numer += accum.random_coeff_powers[28]
                * (trace_evals[70].values.at(i)
                    - ((trace_evals[68].values.at(i) * trace_evals[68].values.at(i))
                        + (trace_evals[69].values.at(i) * trace_evals[69].values.at(i))));
            *numer += accum.random_coeff_powers[27]
                * (trace_evals[71].values.at(i)
                    - ((trace_evals[69].values.at(i) * trace_evals[69].values.at(i))
                        + (trace_evals[70].values.at(i) * trace_evals[70].values.at(i))));
            *numer += accum.random_coeff_powers[26]
                * (trace_evals[72].values.at(i)
                    - ((trace_evals[70].values.at(i) * trace_evals[70].values.at(i))
                        + (trace_evals[71].values.at(i) * trace_evals[71].values.at(i))));
            *numer += accum.random_coeff_powers[25]
                * (trace_evals[73].values.at(i)
                    - ((trace_evals[71].values.at(i) * trace_evals[71].values.at(i))
                        + (trace_evals[72].values.at(i) * trace_evals[72].values.at(i))));
            *numer += accum.random_coeff_powers[24]
                * (trace_evals[74].values.at(i)
                    - ((trace_evals[72].values.at(i) * trace_evals[72].values.at(i))
                        + (trace_evals[73].values.at(i) * trace_evals[73].values.at(i))));
            *numer += accum.random_coeff_powers[23]
                * (trace_evals[75].values.at(i)
                    - ((trace_evals[73].values.at(i) * trace_evals[73].values.at(i))
                        + (trace_evals[74].values.at(i) * trace_evals[74].values.at(i))));
            *numer += accum.random_coeff_powers[22]
                * (trace_evals[76].values.at(i)
                    - ((trace_evals[74].values.at(i) * trace_evals[74].values.at(i))
                        + (trace_evals[75].values.at(i) * trace_evals[75].values.at(i))));
            *numer += accum.random_coeff_powers[21]
                * (trace_evals[77].values.at(i)
                    - ((trace_evals[75].values.at(i) * trace_evals[75].values.at(i))
                        + (trace_evals[76].values.at(i) * trace_evals[76].values.at(i))));
            *numer += accum.random_coeff_powers[20]
                * (trace_evals[78].values.at(i)
                    - ((trace_evals[76].values.at(i) * trace_evals[76].values.at(i))
                        + (trace_evals[77].values.at(i) * trace_evals[77].values.at(i))));
            *numer += accum.random_coeff_powers[19]
                * (trace_evals[79].values.at(i)
                    - ((trace_evals[77].values.at(i) * trace_evals[77].values.at(i))
                        + (trace_evals[78].values.at(i) * trace_evals[78].values.at(i))));
            *numer += accum.random_coeff_powers[18]
                * (trace_evals[80].values.at(i)
                    - ((trace_evals[78].values.at(i) * trace_evals[78].values.at(i))
                        + (trace_evals[79].values.at(i) * trace_evals[79].values.at(i))));
            *numer += accum.random_coeff_powers[17]
                * (trace_evals[81].values.at(i)
                    - ((trace_evals[79].values.at(i) * trace_evals[79].values.at(i))
                        + (trace_evals[80].values.at(i) * trace_evals[80].values.at(i))));
            *numer += accum.random_coeff_powers[16]
                * (trace_evals[82].values.at(i)
                    - ((trace_evals[80].values.at(i) * trace_evals[80].values.at(i))
                        + (trace_evals[81].values.at(i) * trace_evals[81].values.at(i))));
            *numer += accum.random_coeff_powers[15]
                * (trace_evals[83].values.at(i)
                    - ((trace_evals[81].values.at(i) * trace_evals[81].values.at(i))
                        + (trace_evals[82].values.at(i) * trace_evals[82].values.at(i))));
            *numer += accum.random_coeff_powers[14]
                * (trace_evals[84].values.at(i)
                    - ((trace_evals[82].values.at(i) * trace_evals[82].values.at(i))
                        + (trace_evals[83].values.at(i) * trace_evals[83].values.at(i))));
            *numer += accum.random_coeff_powers[13]
                * (trace_evals[85].values.at(i)
                    - ((trace_evals[83].values.at(i) * trace_evals[83].values.at(i))
                        + (trace_evals[84].values.at(i) * trace_evals[84].values.at(i))));
            *numer += accum.random_coeff_powers[12]
                * (trace_evals[86].values.at(i)
                    - ((trace_evals[84].values.at(i) * trace_evals[84].values.at(i))
                        + (trace_evals[85].values.at(i) * trace_evals[85].values.at(i))));
            *numer += accum.random_coeff_powers[11]
                * (trace_evals[87].values.at(i)
                    - ((trace_evals[85].values.at(i) * trace_evals[85].values.at(i))
                        + (trace_evals[86].values.at(i) * trace_evals[86].values.at(i))));
            *numer += accum.random_coeff_powers[10]
                * (trace_evals[88].values.at(i)
                    - ((trace_evals[86].values.at(i) * trace_evals[86].values.at(i))
                        + (trace_evals[87].values.at(i) * trace_evals[87].values.at(i))));
            *numer += accum.random_coeff_powers[9]
                * (trace_evals[89].values.at(i)
                    - ((trace_evals[87].values.at(i) * trace_evals[87].values.at(i))
                        + (trace_evals[88].values.at(i) * trace_evals[88].values.at(i))));
            *numer += accum.random_coeff_powers[8]
                * (trace_evals[90].values.at(i)
                    - ((trace_evals[88].values.at(i) * trace_evals[88].values.at(i))
                        + (trace_evals[89].values.at(i) * trace_evals[89].values.at(i))));
            *numer += accum.random_coeff_powers[7]
                * (trace_evals[91].values.at(i)
                    - ((trace_evals[89].values.at(i) * trace_evals[89].values.at(i))
                        + (trace_evals[90].values.at(i) * trace_evals[90].values.at(i))));
            *numer += accum.random_coeff_powers[6]
                * (trace_evals[92].values.at(i)
                    - ((trace_evals[90].values.at(i) * trace_evals[90].values.at(i))
                        + (trace_evals[91].values.at(i) * trace_evals[91].values.at(i))));
            *numer += accum.random_coeff_powers[5]
                * (trace_evals[93].values.at(i)
                    - ((trace_evals[91].values.at(i) * trace_evals[91].values.at(i))
                        + (trace_evals[92].values.at(i) * trace_evals[92].values.at(i))));
            *numer += accum.random_coeff_powers[4]
                * (trace_evals[94].values.at(i)
                    - ((trace_evals[92].values.at(i) * trace_evals[92].values.at(i))
                        + (trace_evals[93].values.at(i) * trace_evals[93].values.at(i))));
            *numer += accum.random_coeff_powers[3]
                * (trace_evals[95].values.at(i)
                    - ((trace_evals[93].values.at(i) * trace_evals[93].values.at(i))
                        + (trace_evals[94].values.at(i) * trace_evals[94].values.at(i))));
            *numer += accum.random_coeff_powers[2]
                * (trace_evals[96].values.at(i)
                    - ((trace_evals[94].values.at(i) * trace_evals[94].values.at(i))
                        + (trace_evals[95].values.at(i) * trace_evals[95].values.at(i))));
            *numer += accum.random_coeff_powers[1]
                * (trace_evals[97].values.at(i)
                    - ((trace_evals[95].values.at(i) * trace_evals[95].values.at(i))
                        + (trace_evals[96].values.at(i) * trace_evals[96].values.at(i))));
            *numer += accum.random_coeff_powers[0]
                * (trace_evals[98].values.at(i)
                    - ((trace_evals[96].values.at(i) * trace_evals[96].values.at(i))
                        + (trace_evals[97].values.at(i) * trace_evals[97].values.at(i))));
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
