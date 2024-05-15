use num_traits::identities::Zero;
use stwo_prover::core::air::accumulation::DomainEvaluationAccumulator;
use stwo_prover::core::air::{AirProver, Component, ComponentProver, ComponentTrace};
use stwo_prover::core::backend::{CPUBackend, Column};
use stwo_prover::core::constraints::coset_vanishing;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::utils::bit_reverse;

use super::component::{Fib__1000, Fib__1000TestAIR};

impl AirProver<CPUBackend> for Fib__1000TestAIR {
    fn prover_components(&self) -> Vec<&dyn ComponentProver<CPUBackend>> {
        vec![&self.component]
    }
}

impl ComponentProver<CPUBackend> for Fib__1000 {
    fn evaluate_constraint_quotients_on_domain(
        &self,
        trace: &ComponentTrace<'_, CPUBackend>,
        evaluation_accumulator: &mut DomainEvaluationAccumulator<CPUBackend>,
    ) {
        // Numerator computation.
        let trace_evals = &trace.evals;
        let mut numerators =
            vec![SecureField::zero(); 1 << (self.max_constraint_log_degree_bound())];
        let [mut accum] = evaluation_accumulator
            .columns([(self.max_constraint_log_degree_bound(), self.n_constraints())]);
        for (i, numer) in numerators.iter_mut().enumerate() {
            *numer += accum.random_coeff_powers[997]
                * (trace_evals[1].values.at(i)
                    - ((BaseField::from_u32_unchecked(1) * BaseField::from_u32_unchecked(1))
                        + (trace_evals[0].values.at(i) * trace_evals[0].values.at(i))));
            *numer += accum.random_coeff_powers[996]
                * (trace_evals[2].values.at(i)
                    - ((trace_evals[0].values.at(i) * trace_evals[0].values.at(i))
                        + (trace_evals[1].values.at(i) * trace_evals[1].values.at(i))));
            *numer += accum.random_coeff_powers[995]
                * (trace_evals[3].values.at(i)
                    - ((trace_evals[1].values.at(i) * trace_evals[1].values.at(i))
                        + (trace_evals[2].values.at(i) * trace_evals[2].values.at(i))));
            *numer += accum.random_coeff_powers[994]
                * (trace_evals[4].values.at(i)
                    - ((trace_evals[2].values.at(i) * trace_evals[2].values.at(i))
                        + (trace_evals[3].values.at(i) * trace_evals[3].values.at(i))));
            *numer += accum.random_coeff_powers[993]
                * (trace_evals[5].values.at(i)
                    - ((trace_evals[3].values.at(i) * trace_evals[3].values.at(i))
                        + (trace_evals[4].values.at(i) * trace_evals[4].values.at(i))));
            *numer += accum.random_coeff_powers[992]
                * (trace_evals[6].values.at(i)
                    - ((trace_evals[4].values.at(i) * trace_evals[4].values.at(i))
                        + (trace_evals[5].values.at(i) * trace_evals[5].values.at(i))));
            *numer += accum.random_coeff_powers[991]
                * (trace_evals[7].values.at(i)
                    - ((trace_evals[5].values.at(i) * trace_evals[5].values.at(i))
                        + (trace_evals[6].values.at(i) * trace_evals[6].values.at(i))));
            *numer += accum.random_coeff_powers[990]
                * (trace_evals[8].values.at(i)
                    - ((trace_evals[6].values.at(i) * trace_evals[6].values.at(i))
                        + (trace_evals[7].values.at(i) * trace_evals[7].values.at(i))));
            *numer += accum.random_coeff_powers[989]
                * (trace_evals[9].values.at(i)
                    - ((trace_evals[7].values.at(i) * trace_evals[7].values.at(i))
                        + (trace_evals[8].values.at(i) * trace_evals[8].values.at(i))));
            *numer += accum.random_coeff_powers[988]
                * (trace_evals[10].values.at(i)
                    - ((trace_evals[8].values.at(i) * trace_evals[8].values.at(i))
                        + (trace_evals[9].values.at(i) * trace_evals[9].values.at(i))));
            *numer += accum.random_coeff_powers[987]
                * (trace_evals[11].values.at(i)
                    - ((trace_evals[9].values.at(i) * trace_evals[9].values.at(i))
                        + (trace_evals[10].values.at(i) * trace_evals[10].values.at(i))));
            *numer += accum.random_coeff_powers[986]
                * (trace_evals[12].values.at(i)
                    - ((trace_evals[10].values.at(i) * trace_evals[10].values.at(i))
                        + (trace_evals[11].values.at(i) * trace_evals[11].values.at(i))));
            *numer += accum.random_coeff_powers[985]
                * (trace_evals[13].values.at(i)
                    - ((trace_evals[11].values.at(i) * trace_evals[11].values.at(i))
                        + (trace_evals[12].values.at(i) * trace_evals[12].values.at(i))));
            *numer += accum.random_coeff_powers[984]
                * (trace_evals[14].values.at(i)
                    - ((trace_evals[12].values.at(i) * trace_evals[12].values.at(i))
                        + (trace_evals[13].values.at(i) * trace_evals[13].values.at(i))));
            *numer += accum.random_coeff_powers[983]
                * (trace_evals[15].values.at(i)
                    - ((trace_evals[13].values.at(i) * trace_evals[13].values.at(i))
                        + (trace_evals[14].values.at(i) * trace_evals[14].values.at(i))));
            *numer += accum.random_coeff_powers[982]
                * (trace_evals[16].values.at(i)
                    - ((trace_evals[14].values.at(i) * trace_evals[14].values.at(i))
                        + (trace_evals[15].values.at(i) * trace_evals[15].values.at(i))));
            *numer += accum.random_coeff_powers[981]
                * (trace_evals[17].values.at(i)
                    - ((trace_evals[15].values.at(i) * trace_evals[15].values.at(i))
                        + (trace_evals[16].values.at(i) * trace_evals[16].values.at(i))));
            *numer += accum.random_coeff_powers[980]
                * (trace_evals[18].values.at(i)
                    - ((trace_evals[16].values.at(i) * trace_evals[16].values.at(i))
                        + (trace_evals[17].values.at(i) * trace_evals[17].values.at(i))));
            *numer += accum.random_coeff_powers[979]
                * (trace_evals[19].values.at(i)
                    - ((trace_evals[17].values.at(i) * trace_evals[17].values.at(i))
                        + (trace_evals[18].values.at(i) * trace_evals[18].values.at(i))));
            *numer += accum.random_coeff_powers[978]
                * (trace_evals[20].values.at(i)
                    - ((trace_evals[18].values.at(i) * trace_evals[18].values.at(i))
                        + (trace_evals[19].values.at(i) * trace_evals[19].values.at(i))));
            *numer += accum.random_coeff_powers[977]
                * (trace_evals[21].values.at(i)
                    - ((trace_evals[19].values.at(i) * trace_evals[19].values.at(i))
                        + (trace_evals[20].values.at(i) * trace_evals[20].values.at(i))));
            *numer += accum.random_coeff_powers[976]
                * (trace_evals[22].values.at(i)
                    - ((trace_evals[20].values.at(i) * trace_evals[20].values.at(i))
                        + (trace_evals[21].values.at(i) * trace_evals[21].values.at(i))));
            *numer += accum.random_coeff_powers[975]
                * (trace_evals[23].values.at(i)
                    - ((trace_evals[21].values.at(i) * trace_evals[21].values.at(i))
                        + (trace_evals[22].values.at(i) * trace_evals[22].values.at(i))));
            *numer += accum.random_coeff_powers[974]
                * (trace_evals[24].values.at(i)
                    - ((trace_evals[22].values.at(i) * trace_evals[22].values.at(i))
                        + (trace_evals[23].values.at(i) * trace_evals[23].values.at(i))));
            *numer += accum.random_coeff_powers[973]
                * (trace_evals[25].values.at(i)
                    - ((trace_evals[23].values.at(i) * trace_evals[23].values.at(i))
                        + (trace_evals[24].values.at(i) * trace_evals[24].values.at(i))));
            *numer += accum.random_coeff_powers[972]
                * (trace_evals[26].values.at(i)
                    - ((trace_evals[24].values.at(i) * trace_evals[24].values.at(i))
                        + (trace_evals[25].values.at(i) * trace_evals[25].values.at(i))));
            *numer += accum.random_coeff_powers[971]
                * (trace_evals[27].values.at(i)
                    - ((trace_evals[25].values.at(i) * trace_evals[25].values.at(i))
                        + (trace_evals[26].values.at(i) * trace_evals[26].values.at(i))));
            *numer += accum.random_coeff_powers[970]
                * (trace_evals[28].values.at(i)
                    - ((trace_evals[26].values.at(i) * trace_evals[26].values.at(i))
                        + (trace_evals[27].values.at(i) * trace_evals[27].values.at(i))));
            *numer += accum.random_coeff_powers[969]
                * (trace_evals[29].values.at(i)
                    - ((trace_evals[27].values.at(i) * trace_evals[27].values.at(i))
                        + (trace_evals[28].values.at(i) * trace_evals[28].values.at(i))));
            *numer += accum.random_coeff_powers[968]
                * (trace_evals[30].values.at(i)
                    - ((trace_evals[28].values.at(i) * trace_evals[28].values.at(i))
                        + (trace_evals[29].values.at(i) * trace_evals[29].values.at(i))));
            *numer += accum.random_coeff_powers[967]
                * (trace_evals[31].values.at(i)
                    - ((trace_evals[29].values.at(i) * trace_evals[29].values.at(i))
                        + (trace_evals[30].values.at(i) * trace_evals[30].values.at(i))));
            *numer += accum.random_coeff_powers[966]
                * (trace_evals[32].values.at(i)
                    - ((trace_evals[30].values.at(i) * trace_evals[30].values.at(i))
                        + (trace_evals[31].values.at(i) * trace_evals[31].values.at(i))));
            *numer += accum.random_coeff_powers[965]
                * (trace_evals[33].values.at(i)
                    - ((trace_evals[31].values.at(i) * trace_evals[31].values.at(i))
                        + (trace_evals[32].values.at(i) * trace_evals[32].values.at(i))));
            *numer += accum.random_coeff_powers[964]
                * (trace_evals[34].values.at(i)
                    - ((trace_evals[32].values.at(i) * trace_evals[32].values.at(i))
                        + (trace_evals[33].values.at(i) * trace_evals[33].values.at(i))));
            *numer += accum.random_coeff_powers[963]
                * (trace_evals[35].values.at(i)
                    - ((trace_evals[33].values.at(i) * trace_evals[33].values.at(i))
                        + (trace_evals[34].values.at(i) * trace_evals[34].values.at(i))));
            *numer += accum.random_coeff_powers[962]
                * (trace_evals[36].values.at(i)
                    - ((trace_evals[34].values.at(i) * trace_evals[34].values.at(i))
                        + (trace_evals[35].values.at(i) * trace_evals[35].values.at(i))));
            *numer += accum.random_coeff_powers[961]
                * (trace_evals[37].values.at(i)
                    - ((trace_evals[35].values.at(i) * trace_evals[35].values.at(i))
                        + (trace_evals[36].values.at(i) * trace_evals[36].values.at(i))));
            *numer += accum.random_coeff_powers[960]
                * (trace_evals[38].values.at(i)
                    - ((trace_evals[36].values.at(i) * trace_evals[36].values.at(i))
                        + (trace_evals[37].values.at(i) * trace_evals[37].values.at(i))));
            *numer += accum.random_coeff_powers[959]
                * (trace_evals[39].values.at(i)
                    - ((trace_evals[37].values.at(i) * trace_evals[37].values.at(i))
                        + (trace_evals[38].values.at(i) * trace_evals[38].values.at(i))));
            *numer += accum.random_coeff_powers[958]
                * (trace_evals[40].values.at(i)
                    - ((trace_evals[38].values.at(i) * trace_evals[38].values.at(i))
                        + (trace_evals[39].values.at(i) * trace_evals[39].values.at(i))));
            *numer += accum.random_coeff_powers[957]
                * (trace_evals[41].values.at(i)
                    - ((trace_evals[39].values.at(i) * trace_evals[39].values.at(i))
                        + (trace_evals[40].values.at(i) * trace_evals[40].values.at(i))));
            *numer += accum.random_coeff_powers[956]
                * (trace_evals[42].values.at(i)
                    - ((trace_evals[40].values.at(i) * trace_evals[40].values.at(i))
                        + (trace_evals[41].values.at(i) * trace_evals[41].values.at(i))));
            *numer += accum.random_coeff_powers[955]
                * (trace_evals[43].values.at(i)
                    - ((trace_evals[41].values.at(i) * trace_evals[41].values.at(i))
                        + (trace_evals[42].values.at(i) * trace_evals[42].values.at(i))));
            *numer += accum.random_coeff_powers[954]
                * (trace_evals[44].values.at(i)
                    - ((trace_evals[42].values.at(i) * trace_evals[42].values.at(i))
                        + (trace_evals[43].values.at(i) * trace_evals[43].values.at(i))));
            *numer += accum.random_coeff_powers[953]
                * (trace_evals[45].values.at(i)
                    - ((trace_evals[43].values.at(i) * trace_evals[43].values.at(i))
                        + (trace_evals[44].values.at(i) * trace_evals[44].values.at(i))));
            *numer += accum.random_coeff_powers[952]
                * (trace_evals[46].values.at(i)
                    - ((trace_evals[44].values.at(i) * trace_evals[44].values.at(i))
                        + (trace_evals[45].values.at(i) * trace_evals[45].values.at(i))));
            *numer += accum.random_coeff_powers[951]
                * (trace_evals[47].values.at(i)
                    - ((trace_evals[45].values.at(i) * trace_evals[45].values.at(i))
                        + (trace_evals[46].values.at(i) * trace_evals[46].values.at(i))));
            *numer += accum.random_coeff_powers[950]
                * (trace_evals[48].values.at(i)
                    - ((trace_evals[46].values.at(i) * trace_evals[46].values.at(i))
                        + (trace_evals[47].values.at(i) * trace_evals[47].values.at(i))));
            *numer += accum.random_coeff_powers[949]
                * (trace_evals[49].values.at(i)
                    - ((trace_evals[47].values.at(i) * trace_evals[47].values.at(i))
                        + (trace_evals[48].values.at(i) * trace_evals[48].values.at(i))));
            *numer += accum.random_coeff_powers[948]
                * (trace_evals[50].values.at(i)
                    - ((trace_evals[48].values.at(i) * trace_evals[48].values.at(i))
                        + (trace_evals[49].values.at(i) * trace_evals[49].values.at(i))));
            *numer += accum.random_coeff_powers[947]
                * (trace_evals[51].values.at(i)
                    - ((trace_evals[49].values.at(i) * trace_evals[49].values.at(i))
                        + (trace_evals[50].values.at(i) * trace_evals[50].values.at(i))));
            *numer += accum.random_coeff_powers[946]
                * (trace_evals[52].values.at(i)
                    - ((trace_evals[50].values.at(i) * trace_evals[50].values.at(i))
                        + (trace_evals[51].values.at(i) * trace_evals[51].values.at(i))));
            *numer += accum.random_coeff_powers[945]
                * (trace_evals[53].values.at(i)
                    - ((trace_evals[51].values.at(i) * trace_evals[51].values.at(i))
                        + (trace_evals[52].values.at(i) * trace_evals[52].values.at(i))));
            *numer += accum.random_coeff_powers[944]
                * (trace_evals[54].values.at(i)
                    - ((trace_evals[52].values.at(i) * trace_evals[52].values.at(i))
                        + (trace_evals[53].values.at(i) * trace_evals[53].values.at(i))));
            *numer += accum.random_coeff_powers[943]
                * (trace_evals[55].values.at(i)
                    - ((trace_evals[53].values.at(i) * trace_evals[53].values.at(i))
                        + (trace_evals[54].values.at(i) * trace_evals[54].values.at(i))));
            *numer += accum.random_coeff_powers[942]
                * (trace_evals[56].values.at(i)
                    - ((trace_evals[54].values.at(i) * trace_evals[54].values.at(i))
                        + (trace_evals[55].values.at(i) * trace_evals[55].values.at(i))));
            *numer += accum.random_coeff_powers[941]
                * (trace_evals[57].values.at(i)
                    - ((trace_evals[55].values.at(i) * trace_evals[55].values.at(i))
                        + (trace_evals[56].values.at(i) * trace_evals[56].values.at(i))));
            *numer += accum.random_coeff_powers[940]
                * (trace_evals[58].values.at(i)
                    - ((trace_evals[56].values.at(i) * trace_evals[56].values.at(i))
                        + (trace_evals[57].values.at(i) * trace_evals[57].values.at(i))));
            *numer += accum.random_coeff_powers[939]
                * (trace_evals[59].values.at(i)
                    - ((trace_evals[57].values.at(i) * trace_evals[57].values.at(i))
                        + (trace_evals[58].values.at(i) * trace_evals[58].values.at(i))));
            *numer += accum.random_coeff_powers[938]
                * (trace_evals[60].values.at(i)
                    - ((trace_evals[58].values.at(i) * trace_evals[58].values.at(i))
                        + (trace_evals[59].values.at(i) * trace_evals[59].values.at(i))));
            *numer += accum.random_coeff_powers[937]
                * (trace_evals[61].values.at(i)
                    - ((trace_evals[59].values.at(i) * trace_evals[59].values.at(i))
                        + (trace_evals[60].values.at(i) * trace_evals[60].values.at(i))));
            *numer += accum.random_coeff_powers[936]
                * (trace_evals[62].values.at(i)
                    - ((trace_evals[60].values.at(i) * trace_evals[60].values.at(i))
                        + (trace_evals[61].values.at(i) * trace_evals[61].values.at(i))));
            *numer += accum.random_coeff_powers[935]
                * (trace_evals[63].values.at(i)
                    - ((trace_evals[61].values.at(i) * trace_evals[61].values.at(i))
                        + (trace_evals[62].values.at(i) * trace_evals[62].values.at(i))));
            *numer += accum.random_coeff_powers[934]
                * (trace_evals[64].values.at(i)
                    - ((trace_evals[62].values.at(i) * trace_evals[62].values.at(i))
                        + (trace_evals[63].values.at(i) * trace_evals[63].values.at(i))));
            *numer += accum.random_coeff_powers[933]
                * (trace_evals[65].values.at(i)
                    - ((trace_evals[63].values.at(i) * trace_evals[63].values.at(i))
                        + (trace_evals[64].values.at(i) * trace_evals[64].values.at(i))));
            *numer += accum.random_coeff_powers[932]
                * (trace_evals[66].values.at(i)
                    - ((trace_evals[64].values.at(i) * trace_evals[64].values.at(i))
                        + (trace_evals[65].values.at(i) * trace_evals[65].values.at(i))));
            *numer += accum.random_coeff_powers[931]
                * (trace_evals[67].values.at(i)
                    - ((trace_evals[65].values.at(i) * trace_evals[65].values.at(i))
                        + (trace_evals[66].values.at(i) * trace_evals[66].values.at(i))));
            *numer += accum.random_coeff_powers[930]
                * (trace_evals[68].values.at(i)
                    - ((trace_evals[66].values.at(i) * trace_evals[66].values.at(i))
                        + (trace_evals[67].values.at(i) * trace_evals[67].values.at(i))));
            *numer += accum.random_coeff_powers[929]
                * (trace_evals[69].values.at(i)
                    - ((trace_evals[67].values.at(i) * trace_evals[67].values.at(i))
                        + (trace_evals[68].values.at(i) * trace_evals[68].values.at(i))));
            *numer += accum.random_coeff_powers[928]
                * (trace_evals[70].values.at(i)
                    - ((trace_evals[68].values.at(i) * trace_evals[68].values.at(i))
                        + (trace_evals[69].values.at(i) * trace_evals[69].values.at(i))));
            *numer += accum.random_coeff_powers[927]
                * (trace_evals[71].values.at(i)
                    - ((trace_evals[69].values.at(i) * trace_evals[69].values.at(i))
                        + (trace_evals[70].values.at(i) * trace_evals[70].values.at(i))));
            *numer += accum.random_coeff_powers[926]
                * (trace_evals[72].values.at(i)
                    - ((trace_evals[70].values.at(i) * trace_evals[70].values.at(i))
                        + (trace_evals[71].values.at(i) * trace_evals[71].values.at(i))));
            *numer += accum.random_coeff_powers[925]
                * (trace_evals[73].values.at(i)
                    - ((trace_evals[71].values.at(i) * trace_evals[71].values.at(i))
                        + (trace_evals[72].values.at(i) * trace_evals[72].values.at(i))));
            *numer += accum.random_coeff_powers[924]
                * (trace_evals[74].values.at(i)
                    - ((trace_evals[72].values.at(i) * trace_evals[72].values.at(i))
                        + (trace_evals[73].values.at(i) * trace_evals[73].values.at(i))));
            *numer += accum.random_coeff_powers[923]
                * (trace_evals[75].values.at(i)
                    - ((trace_evals[73].values.at(i) * trace_evals[73].values.at(i))
                        + (trace_evals[74].values.at(i) * trace_evals[74].values.at(i))));
            *numer += accum.random_coeff_powers[922]
                * (trace_evals[76].values.at(i)
                    - ((trace_evals[74].values.at(i) * trace_evals[74].values.at(i))
                        + (trace_evals[75].values.at(i) * trace_evals[75].values.at(i))));
            *numer += accum.random_coeff_powers[921]
                * (trace_evals[77].values.at(i)
                    - ((trace_evals[75].values.at(i) * trace_evals[75].values.at(i))
                        + (trace_evals[76].values.at(i) * trace_evals[76].values.at(i))));
            *numer += accum.random_coeff_powers[920]
                * (trace_evals[78].values.at(i)
                    - ((trace_evals[76].values.at(i) * trace_evals[76].values.at(i))
                        + (trace_evals[77].values.at(i) * trace_evals[77].values.at(i))));
            *numer += accum.random_coeff_powers[919]
                * (trace_evals[79].values.at(i)
                    - ((trace_evals[77].values.at(i) * trace_evals[77].values.at(i))
                        + (trace_evals[78].values.at(i) * trace_evals[78].values.at(i))));
            *numer += accum.random_coeff_powers[918]
                * (trace_evals[80].values.at(i)
                    - ((trace_evals[78].values.at(i) * trace_evals[78].values.at(i))
                        + (trace_evals[79].values.at(i) * trace_evals[79].values.at(i))));
            *numer += accum.random_coeff_powers[917]
                * (trace_evals[81].values.at(i)
                    - ((trace_evals[79].values.at(i) * trace_evals[79].values.at(i))
                        + (trace_evals[80].values.at(i) * trace_evals[80].values.at(i))));
            *numer += accum.random_coeff_powers[916]
                * (trace_evals[82].values.at(i)
                    - ((trace_evals[80].values.at(i) * trace_evals[80].values.at(i))
                        + (trace_evals[81].values.at(i) * trace_evals[81].values.at(i))));
            *numer += accum.random_coeff_powers[915]
                * (trace_evals[83].values.at(i)
                    - ((trace_evals[81].values.at(i) * trace_evals[81].values.at(i))
                        + (trace_evals[82].values.at(i) * trace_evals[82].values.at(i))));
            *numer += accum.random_coeff_powers[914]
                * (trace_evals[84].values.at(i)
                    - ((trace_evals[82].values.at(i) * trace_evals[82].values.at(i))
                        + (trace_evals[83].values.at(i) * trace_evals[83].values.at(i))));
            *numer += accum.random_coeff_powers[913]
                * (trace_evals[85].values.at(i)
                    - ((trace_evals[83].values.at(i) * trace_evals[83].values.at(i))
                        + (trace_evals[84].values.at(i) * trace_evals[84].values.at(i))));
            *numer += accum.random_coeff_powers[912]
                * (trace_evals[86].values.at(i)
                    - ((trace_evals[84].values.at(i) * trace_evals[84].values.at(i))
                        + (trace_evals[85].values.at(i) * trace_evals[85].values.at(i))));
            *numer += accum.random_coeff_powers[911]
                * (trace_evals[87].values.at(i)
                    - ((trace_evals[85].values.at(i) * trace_evals[85].values.at(i))
                        + (trace_evals[86].values.at(i) * trace_evals[86].values.at(i))));
            *numer += accum.random_coeff_powers[910]
                * (trace_evals[88].values.at(i)
                    - ((trace_evals[86].values.at(i) * trace_evals[86].values.at(i))
                        + (trace_evals[87].values.at(i) * trace_evals[87].values.at(i))));
            *numer += accum.random_coeff_powers[909]
                * (trace_evals[89].values.at(i)
                    - ((trace_evals[87].values.at(i) * trace_evals[87].values.at(i))
                        + (trace_evals[88].values.at(i) * trace_evals[88].values.at(i))));
            *numer += accum.random_coeff_powers[908]
                * (trace_evals[90].values.at(i)
                    - ((trace_evals[88].values.at(i) * trace_evals[88].values.at(i))
                        + (trace_evals[89].values.at(i) * trace_evals[89].values.at(i))));
            *numer += accum.random_coeff_powers[907]
                * (trace_evals[91].values.at(i)
                    - ((trace_evals[89].values.at(i) * trace_evals[89].values.at(i))
                        + (trace_evals[90].values.at(i) * trace_evals[90].values.at(i))));
            *numer += accum.random_coeff_powers[906]
                * (trace_evals[92].values.at(i)
                    - ((trace_evals[90].values.at(i) * trace_evals[90].values.at(i))
                        + (trace_evals[91].values.at(i) * trace_evals[91].values.at(i))));
            *numer += accum.random_coeff_powers[905]
                * (trace_evals[93].values.at(i)
                    - ((trace_evals[91].values.at(i) * trace_evals[91].values.at(i))
                        + (trace_evals[92].values.at(i) * trace_evals[92].values.at(i))));
            *numer += accum.random_coeff_powers[904]
                * (trace_evals[94].values.at(i)
                    - ((trace_evals[92].values.at(i) * trace_evals[92].values.at(i))
                        + (trace_evals[93].values.at(i) * trace_evals[93].values.at(i))));
            *numer += accum.random_coeff_powers[903]
                * (trace_evals[95].values.at(i)
                    - ((trace_evals[93].values.at(i) * trace_evals[93].values.at(i))
                        + (trace_evals[94].values.at(i) * trace_evals[94].values.at(i))));
            *numer += accum.random_coeff_powers[902]
                * (trace_evals[96].values.at(i)
                    - ((trace_evals[94].values.at(i) * trace_evals[94].values.at(i))
                        + (trace_evals[95].values.at(i) * trace_evals[95].values.at(i))));
            *numer += accum.random_coeff_powers[901]
                * (trace_evals[97].values.at(i)
                    - ((trace_evals[95].values.at(i) * trace_evals[95].values.at(i))
                        + (trace_evals[96].values.at(i) * trace_evals[96].values.at(i))));
            *numer += accum.random_coeff_powers[900]
                * (trace_evals[98].values.at(i)
                    - ((trace_evals[96].values.at(i) * trace_evals[96].values.at(i))
                        + (trace_evals[97].values.at(i) * trace_evals[97].values.at(i))));
            *numer += accum.random_coeff_powers[899]
                * (trace_evals[99].values.at(i)
                    - ((trace_evals[97].values.at(i) * trace_evals[97].values.at(i))
                        + (trace_evals[98].values.at(i) * trace_evals[98].values.at(i))));
            *numer += accum.random_coeff_powers[898]
                * (trace_evals[100].values.at(i)
                    - ((trace_evals[98].values.at(i) * trace_evals[98].values.at(i))
                        + (trace_evals[99].values.at(i) * trace_evals[99].values.at(i))));
            *numer += accum.random_coeff_powers[897]
                * (trace_evals[101].values.at(i)
                    - ((trace_evals[99].values.at(i) * trace_evals[99].values.at(i))
                        + (trace_evals[100].values.at(i) * trace_evals[100].values.at(i))));
            *numer += accum.random_coeff_powers[896]
                * (trace_evals[102].values.at(i)
                    - ((trace_evals[100].values.at(i) * trace_evals[100].values.at(i))
                        + (trace_evals[101].values.at(i) * trace_evals[101].values.at(i))));
            *numer += accum.random_coeff_powers[895]
                * (trace_evals[103].values.at(i)
                    - ((trace_evals[101].values.at(i) * trace_evals[101].values.at(i))
                        + (trace_evals[102].values.at(i) * trace_evals[102].values.at(i))));
            *numer += accum.random_coeff_powers[894]
                * (trace_evals[104].values.at(i)
                    - ((trace_evals[102].values.at(i) * trace_evals[102].values.at(i))
                        + (trace_evals[103].values.at(i) * trace_evals[103].values.at(i))));
            *numer += accum.random_coeff_powers[893]
                * (trace_evals[105].values.at(i)
                    - ((trace_evals[103].values.at(i) * trace_evals[103].values.at(i))
                        + (trace_evals[104].values.at(i) * trace_evals[104].values.at(i))));
            *numer += accum.random_coeff_powers[892]
                * (trace_evals[106].values.at(i)
                    - ((trace_evals[104].values.at(i) * trace_evals[104].values.at(i))
                        + (trace_evals[105].values.at(i) * trace_evals[105].values.at(i))));
            *numer += accum.random_coeff_powers[891]
                * (trace_evals[107].values.at(i)
                    - ((trace_evals[105].values.at(i) * trace_evals[105].values.at(i))
                        + (trace_evals[106].values.at(i) * trace_evals[106].values.at(i))));
            *numer += accum.random_coeff_powers[890]
                * (trace_evals[108].values.at(i)
                    - ((trace_evals[106].values.at(i) * trace_evals[106].values.at(i))
                        + (trace_evals[107].values.at(i) * trace_evals[107].values.at(i))));
            *numer += accum.random_coeff_powers[889]
                * (trace_evals[109].values.at(i)
                    - ((trace_evals[107].values.at(i) * trace_evals[107].values.at(i))
                        + (trace_evals[108].values.at(i) * trace_evals[108].values.at(i))));
            *numer += accum.random_coeff_powers[888]
                * (trace_evals[110].values.at(i)
                    - ((trace_evals[108].values.at(i) * trace_evals[108].values.at(i))
                        + (trace_evals[109].values.at(i) * trace_evals[109].values.at(i))));
            *numer += accum.random_coeff_powers[887]
                * (trace_evals[111].values.at(i)
                    - ((trace_evals[109].values.at(i) * trace_evals[109].values.at(i))
                        + (trace_evals[110].values.at(i) * trace_evals[110].values.at(i))));
            *numer += accum.random_coeff_powers[886]
                * (trace_evals[112].values.at(i)
                    - ((trace_evals[110].values.at(i) * trace_evals[110].values.at(i))
                        + (trace_evals[111].values.at(i) * trace_evals[111].values.at(i))));
            *numer += accum.random_coeff_powers[885]
                * (trace_evals[113].values.at(i)
                    - ((trace_evals[111].values.at(i) * trace_evals[111].values.at(i))
                        + (trace_evals[112].values.at(i) * trace_evals[112].values.at(i))));
            *numer += accum.random_coeff_powers[884]
                * (trace_evals[114].values.at(i)
                    - ((trace_evals[112].values.at(i) * trace_evals[112].values.at(i))
                        + (trace_evals[113].values.at(i) * trace_evals[113].values.at(i))));
            *numer += accum.random_coeff_powers[883]
                * (trace_evals[115].values.at(i)
                    - ((trace_evals[113].values.at(i) * trace_evals[113].values.at(i))
                        + (trace_evals[114].values.at(i) * trace_evals[114].values.at(i))));
            *numer += accum.random_coeff_powers[882]
                * (trace_evals[116].values.at(i)
                    - ((trace_evals[114].values.at(i) * trace_evals[114].values.at(i))
                        + (trace_evals[115].values.at(i) * trace_evals[115].values.at(i))));
            *numer += accum.random_coeff_powers[881]
                * (trace_evals[117].values.at(i)
                    - ((trace_evals[115].values.at(i) * trace_evals[115].values.at(i))
                        + (trace_evals[116].values.at(i) * trace_evals[116].values.at(i))));
            *numer += accum.random_coeff_powers[880]
                * (trace_evals[118].values.at(i)
                    - ((trace_evals[116].values.at(i) * trace_evals[116].values.at(i))
                        + (trace_evals[117].values.at(i) * trace_evals[117].values.at(i))));
            *numer += accum.random_coeff_powers[879]
                * (trace_evals[119].values.at(i)
                    - ((trace_evals[117].values.at(i) * trace_evals[117].values.at(i))
                        + (trace_evals[118].values.at(i) * trace_evals[118].values.at(i))));
            *numer += accum.random_coeff_powers[878]
                * (trace_evals[120].values.at(i)
                    - ((trace_evals[118].values.at(i) * trace_evals[118].values.at(i))
                        + (trace_evals[119].values.at(i) * trace_evals[119].values.at(i))));
            *numer += accum.random_coeff_powers[877]
                * (trace_evals[121].values.at(i)
                    - ((trace_evals[119].values.at(i) * trace_evals[119].values.at(i))
                        + (trace_evals[120].values.at(i) * trace_evals[120].values.at(i))));
            *numer += accum.random_coeff_powers[876]
                * (trace_evals[122].values.at(i)
                    - ((trace_evals[120].values.at(i) * trace_evals[120].values.at(i))
                        + (trace_evals[121].values.at(i) * trace_evals[121].values.at(i))));
            *numer += accum.random_coeff_powers[875]
                * (trace_evals[123].values.at(i)
                    - ((trace_evals[121].values.at(i) * trace_evals[121].values.at(i))
                        + (trace_evals[122].values.at(i) * trace_evals[122].values.at(i))));
            *numer += accum.random_coeff_powers[874]
                * (trace_evals[124].values.at(i)
                    - ((trace_evals[122].values.at(i) * trace_evals[122].values.at(i))
                        + (trace_evals[123].values.at(i) * trace_evals[123].values.at(i))));
            *numer += accum.random_coeff_powers[873]
                * (trace_evals[125].values.at(i)
                    - ((trace_evals[123].values.at(i) * trace_evals[123].values.at(i))
                        + (trace_evals[124].values.at(i) * trace_evals[124].values.at(i))));
            *numer += accum.random_coeff_powers[872]
                * (trace_evals[126].values.at(i)
                    - ((trace_evals[124].values.at(i) * trace_evals[124].values.at(i))
                        + (trace_evals[125].values.at(i) * trace_evals[125].values.at(i))));
            *numer += accum.random_coeff_powers[871]
                * (trace_evals[127].values.at(i)
                    - ((trace_evals[125].values.at(i) * trace_evals[125].values.at(i))
                        + (trace_evals[126].values.at(i) * trace_evals[126].values.at(i))));
            *numer += accum.random_coeff_powers[870]
                * (trace_evals[128].values.at(i)
                    - ((trace_evals[126].values.at(i) * trace_evals[126].values.at(i))
                        + (trace_evals[127].values.at(i) * trace_evals[127].values.at(i))));
            *numer += accum.random_coeff_powers[869]
                * (trace_evals[129].values.at(i)
                    - ((trace_evals[127].values.at(i) * trace_evals[127].values.at(i))
                        + (trace_evals[128].values.at(i) * trace_evals[128].values.at(i))));
            *numer += accum.random_coeff_powers[868]
                * (trace_evals[130].values.at(i)
                    - ((trace_evals[128].values.at(i) * trace_evals[128].values.at(i))
                        + (trace_evals[129].values.at(i) * trace_evals[129].values.at(i))));
            *numer += accum.random_coeff_powers[867]
                * (trace_evals[131].values.at(i)
                    - ((trace_evals[129].values.at(i) * trace_evals[129].values.at(i))
                        + (trace_evals[130].values.at(i) * trace_evals[130].values.at(i))));
            *numer += accum.random_coeff_powers[866]
                * (trace_evals[132].values.at(i)
                    - ((trace_evals[130].values.at(i) * trace_evals[130].values.at(i))
                        + (trace_evals[131].values.at(i) * trace_evals[131].values.at(i))));
            *numer += accum.random_coeff_powers[865]
                * (trace_evals[133].values.at(i)
                    - ((trace_evals[131].values.at(i) * trace_evals[131].values.at(i))
                        + (trace_evals[132].values.at(i) * trace_evals[132].values.at(i))));
            *numer += accum.random_coeff_powers[864]
                * (trace_evals[134].values.at(i)
                    - ((trace_evals[132].values.at(i) * trace_evals[132].values.at(i))
                        + (trace_evals[133].values.at(i) * trace_evals[133].values.at(i))));
            *numer += accum.random_coeff_powers[863]
                * (trace_evals[135].values.at(i)
                    - ((trace_evals[133].values.at(i) * trace_evals[133].values.at(i))
                        + (trace_evals[134].values.at(i) * trace_evals[134].values.at(i))));
            *numer += accum.random_coeff_powers[862]
                * (trace_evals[136].values.at(i)
                    - ((trace_evals[134].values.at(i) * trace_evals[134].values.at(i))
                        + (trace_evals[135].values.at(i) * trace_evals[135].values.at(i))));
            *numer += accum.random_coeff_powers[861]
                * (trace_evals[137].values.at(i)
                    - ((trace_evals[135].values.at(i) * trace_evals[135].values.at(i))
                        + (trace_evals[136].values.at(i) * trace_evals[136].values.at(i))));
            *numer += accum.random_coeff_powers[860]
                * (trace_evals[138].values.at(i)
                    - ((trace_evals[136].values.at(i) * trace_evals[136].values.at(i))
                        + (trace_evals[137].values.at(i) * trace_evals[137].values.at(i))));
            *numer += accum.random_coeff_powers[859]
                * (trace_evals[139].values.at(i)
                    - ((trace_evals[137].values.at(i) * trace_evals[137].values.at(i))
                        + (trace_evals[138].values.at(i) * trace_evals[138].values.at(i))));
            *numer += accum.random_coeff_powers[858]
                * (trace_evals[140].values.at(i)
                    - ((trace_evals[138].values.at(i) * trace_evals[138].values.at(i))
                        + (trace_evals[139].values.at(i) * trace_evals[139].values.at(i))));
            *numer += accum.random_coeff_powers[857]
                * (trace_evals[141].values.at(i)
                    - ((trace_evals[139].values.at(i) * trace_evals[139].values.at(i))
                        + (trace_evals[140].values.at(i) * trace_evals[140].values.at(i))));
            *numer += accum.random_coeff_powers[856]
                * (trace_evals[142].values.at(i)
                    - ((trace_evals[140].values.at(i) * trace_evals[140].values.at(i))
                        + (trace_evals[141].values.at(i) * trace_evals[141].values.at(i))));
            *numer += accum.random_coeff_powers[855]
                * (trace_evals[143].values.at(i)
                    - ((trace_evals[141].values.at(i) * trace_evals[141].values.at(i))
                        + (trace_evals[142].values.at(i) * trace_evals[142].values.at(i))));
            *numer += accum.random_coeff_powers[854]
                * (trace_evals[144].values.at(i)
                    - ((trace_evals[142].values.at(i) * trace_evals[142].values.at(i))
                        + (trace_evals[143].values.at(i) * trace_evals[143].values.at(i))));
            *numer += accum.random_coeff_powers[853]
                * (trace_evals[145].values.at(i)
                    - ((trace_evals[143].values.at(i) * trace_evals[143].values.at(i))
                        + (trace_evals[144].values.at(i) * trace_evals[144].values.at(i))));
            *numer += accum.random_coeff_powers[852]
                * (trace_evals[146].values.at(i)
                    - ((trace_evals[144].values.at(i) * trace_evals[144].values.at(i))
                        + (trace_evals[145].values.at(i) * trace_evals[145].values.at(i))));
            *numer += accum.random_coeff_powers[851]
                * (trace_evals[147].values.at(i)
                    - ((trace_evals[145].values.at(i) * trace_evals[145].values.at(i))
                        + (trace_evals[146].values.at(i) * trace_evals[146].values.at(i))));
            *numer += accum.random_coeff_powers[850]
                * (trace_evals[148].values.at(i)
                    - ((trace_evals[146].values.at(i) * trace_evals[146].values.at(i))
                        + (trace_evals[147].values.at(i) * trace_evals[147].values.at(i))));
            *numer += accum.random_coeff_powers[849]
                * (trace_evals[149].values.at(i)
                    - ((trace_evals[147].values.at(i) * trace_evals[147].values.at(i))
                        + (trace_evals[148].values.at(i) * trace_evals[148].values.at(i))));
            *numer += accum.random_coeff_powers[848]
                * (trace_evals[150].values.at(i)
                    - ((trace_evals[148].values.at(i) * trace_evals[148].values.at(i))
                        + (trace_evals[149].values.at(i) * trace_evals[149].values.at(i))));
            *numer += accum.random_coeff_powers[847]
                * (trace_evals[151].values.at(i)
                    - ((trace_evals[149].values.at(i) * trace_evals[149].values.at(i))
                        + (trace_evals[150].values.at(i) * trace_evals[150].values.at(i))));
            *numer += accum.random_coeff_powers[846]
                * (trace_evals[152].values.at(i)
                    - ((trace_evals[150].values.at(i) * trace_evals[150].values.at(i))
                        + (trace_evals[151].values.at(i) * trace_evals[151].values.at(i))));
            *numer += accum.random_coeff_powers[845]
                * (trace_evals[153].values.at(i)
                    - ((trace_evals[151].values.at(i) * trace_evals[151].values.at(i))
                        + (trace_evals[152].values.at(i) * trace_evals[152].values.at(i))));
            *numer += accum.random_coeff_powers[844]
                * (trace_evals[154].values.at(i)
                    - ((trace_evals[152].values.at(i) * trace_evals[152].values.at(i))
                        + (trace_evals[153].values.at(i) * trace_evals[153].values.at(i))));
            *numer += accum.random_coeff_powers[843]
                * (trace_evals[155].values.at(i)
                    - ((trace_evals[153].values.at(i) * trace_evals[153].values.at(i))
                        + (trace_evals[154].values.at(i) * trace_evals[154].values.at(i))));
            *numer += accum.random_coeff_powers[842]
                * (trace_evals[156].values.at(i)
                    - ((trace_evals[154].values.at(i) * trace_evals[154].values.at(i))
                        + (trace_evals[155].values.at(i) * trace_evals[155].values.at(i))));
            *numer += accum.random_coeff_powers[841]
                * (trace_evals[157].values.at(i)
                    - ((trace_evals[155].values.at(i) * trace_evals[155].values.at(i))
                        + (trace_evals[156].values.at(i) * trace_evals[156].values.at(i))));
            *numer += accum.random_coeff_powers[840]
                * (trace_evals[158].values.at(i)
                    - ((trace_evals[156].values.at(i) * trace_evals[156].values.at(i))
                        + (trace_evals[157].values.at(i) * trace_evals[157].values.at(i))));
            *numer += accum.random_coeff_powers[839]
                * (trace_evals[159].values.at(i)
                    - ((trace_evals[157].values.at(i) * trace_evals[157].values.at(i))
                        + (trace_evals[158].values.at(i) * trace_evals[158].values.at(i))));
            *numer += accum.random_coeff_powers[838]
                * (trace_evals[160].values.at(i)
                    - ((trace_evals[158].values.at(i) * trace_evals[158].values.at(i))
                        + (trace_evals[159].values.at(i) * trace_evals[159].values.at(i))));
            *numer += accum.random_coeff_powers[837]
                * (trace_evals[161].values.at(i)
                    - ((trace_evals[159].values.at(i) * trace_evals[159].values.at(i))
                        + (trace_evals[160].values.at(i) * trace_evals[160].values.at(i))));
            *numer += accum.random_coeff_powers[836]
                * (trace_evals[162].values.at(i)
                    - ((trace_evals[160].values.at(i) * trace_evals[160].values.at(i))
                        + (trace_evals[161].values.at(i) * trace_evals[161].values.at(i))));
            *numer += accum.random_coeff_powers[835]
                * (trace_evals[163].values.at(i)
                    - ((trace_evals[161].values.at(i) * trace_evals[161].values.at(i))
                        + (trace_evals[162].values.at(i) * trace_evals[162].values.at(i))));
            *numer += accum.random_coeff_powers[834]
                * (trace_evals[164].values.at(i)
                    - ((trace_evals[162].values.at(i) * trace_evals[162].values.at(i))
                        + (trace_evals[163].values.at(i) * trace_evals[163].values.at(i))));
            *numer += accum.random_coeff_powers[833]
                * (trace_evals[165].values.at(i)
                    - ((trace_evals[163].values.at(i) * trace_evals[163].values.at(i))
                        + (trace_evals[164].values.at(i) * trace_evals[164].values.at(i))));
            *numer += accum.random_coeff_powers[832]
                * (trace_evals[166].values.at(i)
                    - ((trace_evals[164].values.at(i) * trace_evals[164].values.at(i))
                        + (trace_evals[165].values.at(i) * trace_evals[165].values.at(i))));
            *numer += accum.random_coeff_powers[831]
                * (trace_evals[167].values.at(i)
                    - ((trace_evals[165].values.at(i) * trace_evals[165].values.at(i))
                        + (trace_evals[166].values.at(i) * trace_evals[166].values.at(i))));
            *numer += accum.random_coeff_powers[830]
                * (trace_evals[168].values.at(i)
                    - ((trace_evals[166].values.at(i) * trace_evals[166].values.at(i))
                        + (trace_evals[167].values.at(i) * trace_evals[167].values.at(i))));
            *numer += accum.random_coeff_powers[829]
                * (trace_evals[169].values.at(i)
                    - ((trace_evals[167].values.at(i) * trace_evals[167].values.at(i))
                        + (trace_evals[168].values.at(i) * trace_evals[168].values.at(i))));
            *numer += accum.random_coeff_powers[828]
                * (trace_evals[170].values.at(i)
                    - ((trace_evals[168].values.at(i) * trace_evals[168].values.at(i))
                        + (trace_evals[169].values.at(i) * trace_evals[169].values.at(i))));
            *numer += accum.random_coeff_powers[827]
                * (trace_evals[171].values.at(i)
                    - ((trace_evals[169].values.at(i) * trace_evals[169].values.at(i))
                        + (trace_evals[170].values.at(i) * trace_evals[170].values.at(i))));
            *numer += accum.random_coeff_powers[826]
                * (trace_evals[172].values.at(i)
                    - ((trace_evals[170].values.at(i) * trace_evals[170].values.at(i))
                        + (trace_evals[171].values.at(i) * trace_evals[171].values.at(i))));
            *numer += accum.random_coeff_powers[825]
                * (trace_evals[173].values.at(i)
                    - ((trace_evals[171].values.at(i) * trace_evals[171].values.at(i))
                        + (trace_evals[172].values.at(i) * trace_evals[172].values.at(i))));
            *numer += accum.random_coeff_powers[824]
                * (trace_evals[174].values.at(i)
                    - ((trace_evals[172].values.at(i) * trace_evals[172].values.at(i))
                        + (trace_evals[173].values.at(i) * trace_evals[173].values.at(i))));
            *numer += accum.random_coeff_powers[823]
                * (trace_evals[175].values.at(i)
                    - ((trace_evals[173].values.at(i) * trace_evals[173].values.at(i))
                        + (trace_evals[174].values.at(i) * trace_evals[174].values.at(i))));
            *numer += accum.random_coeff_powers[822]
                * (trace_evals[176].values.at(i)
                    - ((trace_evals[174].values.at(i) * trace_evals[174].values.at(i))
                        + (trace_evals[175].values.at(i) * trace_evals[175].values.at(i))));
            *numer += accum.random_coeff_powers[821]
                * (trace_evals[177].values.at(i)
                    - ((trace_evals[175].values.at(i) * trace_evals[175].values.at(i))
                        + (trace_evals[176].values.at(i) * trace_evals[176].values.at(i))));
            *numer += accum.random_coeff_powers[820]
                * (trace_evals[178].values.at(i)
                    - ((trace_evals[176].values.at(i) * trace_evals[176].values.at(i))
                        + (trace_evals[177].values.at(i) * trace_evals[177].values.at(i))));
            *numer += accum.random_coeff_powers[819]
                * (trace_evals[179].values.at(i)
                    - ((trace_evals[177].values.at(i) * trace_evals[177].values.at(i))
                        + (trace_evals[178].values.at(i) * trace_evals[178].values.at(i))));
            *numer += accum.random_coeff_powers[818]
                * (trace_evals[180].values.at(i)
                    - ((trace_evals[178].values.at(i) * trace_evals[178].values.at(i))
                        + (trace_evals[179].values.at(i) * trace_evals[179].values.at(i))));
            *numer += accum.random_coeff_powers[817]
                * (trace_evals[181].values.at(i)
                    - ((trace_evals[179].values.at(i) * trace_evals[179].values.at(i))
                        + (trace_evals[180].values.at(i) * trace_evals[180].values.at(i))));
            *numer += accum.random_coeff_powers[816]
                * (trace_evals[182].values.at(i)
                    - ((trace_evals[180].values.at(i) * trace_evals[180].values.at(i))
                        + (trace_evals[181].values.at(i) * trace_evals[181].values.at(i))));
            *numer += accum.random_coeff_powers[815]
                * (trace_evals[183].values.at(i)
                    - ((trace_evals[181].values.at(i) * trace_evals[181].values.at(i))
                        + (trace_evals[182].values.at(i) * trace_evals[182].values.at(i))));
            *numer += accum.random_coeff_powers[814]
                * (trace_evals[184].values.at(i)
                    - ((trace_evals[182].values.at(i) * trace_evals[182].values.at(i))
                        + (trace_evals[183].values.at(i) * trace_evals[183].values.at(i))));
            *numer += accum.random_coeff_powers[813]
                * (trace_evals[185].values.at(i)
                    - ((trace_evals[183].values.at(i) * trace_evals[183].values.at(i))
                        + (trace_evals[184].values.at(i) * trace_evals[184].values.at(i))));
            *numer += accum.random_coeff_powers[812]
                * (trace_evals[186].values.at(i)
                    - ((trace_evals[184].values.at(i) * trace_evals[184].values.at(i))
                        + (trace_evals[185].values.at(i) * trace_evals[185].values.at(i))));
            *numer += accum.random_coeff_powers[811]
                * (trace_evals[187].values.at(i)
                    - ((trace_evals[185].values.at(i) * trace_evals[185].values.at(i))
                        + (trace_evals[186].values.at(i) * trace_evals[186].values.at(i))));
            *numer += accum.random_coeff_powers[810]
                * (trace_evals[188].values.at(i)
                    - ((trace_evals[186].values.at(i) * trace_evals[186].values.at(i))
                        + (trace_evals[187].values.at(i) * trace_evals[187].values.at(i))));
            *numer += accum.random_coeff_powers[809]
                * (trace_evals[189].values.at(i)
                    - ((trace_evals[187].values.at(i) * trace_evals[187].values.at(i))
                        + (trace_evals[188].values.at(i) * trace_evals[188].values.at(i))));
            *numer += accum.random_coeff_powers[808]
                * (trace_evals[190].values.at(i)
                    - ((trace_evals[188].values.at(i) * trace_evals[188].values.at(i))
                        + (trace_evals[189].values.at(i) * trace_evals[189].values.at(i))));
            *numer += accum.random_coeff_powers[807]
                * (trace_evals[191].values.at(i)
                    - ((trace_evals[189].values.at(i) * trace_evals[189].values.at(i))
                        + (trace_evals[190].values.at(i) * trace_evals[190].values.at(i))));
            *numer += accum.random_coeff_powers[806]
                * (trace_evals[192].values.at(i)
                    - ((trace_evals[190].values.at(i) * trace_evals[190].values.at(i))
                        + (trace_evals[191].values.at(i) * trace_evals[191].values.at(i))));
            *numer += accum.random_coeff_powers[805]
                * (trace_evals[193].values.at(i)
                    - ((trace_evals[191].values.at(i) * trace_evals[191].values.at(i))
                        + (trace_evals[192].values.at(i) * trace_evals[192].values.at(i))));
            *numer += accum.random_coeff_powers[804]
                * (trace_evals[194].values.at(i)
                    - ((trace_evals[192].values.at(i) * trace_evals[192].values.at(i))
                        + (trace_evals[193].values.at(i) * trace_evals[193].values.at(i))));
            *numer += accum.random_coeff_powers[803]
                * (trace_evals[195].values.at(i)
                    - ((trace_evals[193].values.at(i) * trace_evals[193].values.at(i))
                        + (trace_evals[194].values.at(i) * trace_evals[194].values.at(i))));
            *numer += accum.random_coeff_powers[802]
                * (trace_evals[196].values.at(i)
                    - ((trace_evals[194].values.at(i) * trace_evals[194].values.at(i))
                        + (trace_evals[195].values.at(i) * trace_evals[195].values.at(i))));
            *numer += accum.random_coeff_powers[801]
                * (trace_evals[197].values.at(i)
                    - ((trace_evals[195].values.at(i) * trace_evals[195].values.at(i))
                        + (trace_evals[196].values.at(i) * trace_evals[196].values.at(i))));
            *numer += accum.random_coeff_powers[800]
                * (trace_evals[198].values.at(i)
                    - ((trace_evals[196].values.at(i) * trace_evals[196].values.at(i))
                        + (trace_evals[197].values.at(i) * trace_evals[197].values.at(i))));
            *numer += accum.random_coeff_powers[799]
                * (trace_evals[199].values.at(i)
                    - ((trace_evals[197].values.at(i) * trace_evals[197].values.at(i))
                        + (trace_evals[198].values.at(i) * trace_evals[198].values.at(i))));
            *numer += accum.random_coeff_powers[798]
                * (trace_evals[200].values.at(i)
                    - ((trace_evals[198].values.at(i) * trace_evals[198].values.at(i))
                        + (trace_evals[199].values.at(i) * trace_evals[199].values.at(i))));
            *numer += accum.random_coeff_powers[797]
                * (trace_evals[201].values.at(i)
                    - ((trace_evals[199].values.at(i) * trace_evals[199].values.at(i))
                        + (trace_evals[200].values.at(i) * trace_evals[200].values.at(i))));
            *numer += accum.random_coeff_powers[796]
                * (trace_evals[202].values.at(i)
                    - ((trace_evals[200].values.at(i) * trace_evals[200].values.at(i))
                        + (trace_evals[201].values.at(i) * trace_evals[201].values.at(i))));
            *numer += accum.random_coeff_powers[795]
                * (trace_evals[203].values.at(i)
                    - ((trace_evals[201].values.at(i) * trace_evals[201].values.at(i))
                        + (trace_evals[202].values.at(i) * trace_evals[202].values.at(i))));
            *numer += accum.random_coeff_powers[794]
                * (trace_evals[204].values.at(i)
                    - ((trace_evals[202].values.at(i) * trace_evals[202].values.at(i))
                        + (trace_evals[203].values.at(i) * trace_evals[203].values.at(i))));
            *numer += accum.random_coeff_powers[793]
                * (trace_evals[205].values.at(i)
                    - ((trace_evals[203].values.at(i) * trace_evals[203].values.at(i))
                        + (trace_evals[204].values.at(i) * trace_evals[204].values.at(i))));
            *numer += accum.random_coeff_powers[792]
                * (trace_evals[206].values.at(i)
                    - ((trace_evals[204].values.at(i) * trace_evals[204].values.at(i))
                        + (trace_evals[205].values.at(i) * trace_evals[205].values.at(i))));
            *numer += accum.random_coeff_powers[791]
                * (trace_evals[207].values.at(i)
                    - ((trace_evals[205].values.at(i) * trace_evals[205].values.at(i))
                        + (trace_evals[206].values.at(i) * trace_evals[206].values.at(i))));
            *numer += accum.random_coeff_powers[790]
                * (trace_evals[208].values.at(i)
                    - ((trace_evals[206].values.at(i) * trace_evals[206].values.at(i))
                        + (trace_evals[207].values.at(i) * trace_evals[207].values.at(i))));
            *numer += accum.random_coeff_powers[789]
                * (trace_evals[209].values.at(i)
                    - ((trace_evals[207].values.at(i) * trace_evals[207].values.at(i))
                        + (trace_evals[208].values.at(i) * trace_evals[208].values.at(i))));
            *numer += accum.random_coeff_powers[788]
                * (trace_evals[210].values.at(i)
                    - ((trace_evals[208].values.at(i) * trace_evals[208].values.at(i))
                        + (trace_evals[209].values.at(i) * trace_evals[209].values.at(i))));
            *numer += accum.random_coeff_powers[787]
                * (trace_evals[211].values.at(i)
                    - ((trace_evals[209].values.at(i) * trace_evals[209].values.at(i))
                        + (trace_evals[210].values.at(i) * trace_evals[210].values.at(i))));
            *numer += accum.random_coeff_powers[786]
                * (trace_evals[212].values.at(i)
                    - ((trace_evals[210].values.at(i) * trace_evals[210].values.at(i))
                        + (trace_evals[211].values.at(i) * trace_evals[211].values.at(i))));
            *numer += accum.random_coeff_powers[785]
                * (trace_evals[213].values.at(i)
                    - ((trace_evals[211].values.at(i) * trace_evals[211].values.at(i))
                        + (trace_evals[212].values.at(i) * trace_evals[212].values.at(i))));
            *numer += accum.random_coeff_powers[784]
                * (trace_evals[214].values.at(i)
                    - ((trace_evals[212].values.at(i) * trace_evals[212].values.at(i))
                        + (trace_evals[213].values.at(i) * trace_evals[213].values.at(i))));
            *numer += accum.random_coeff_powers[783]
                * (trace_evals[215].values.at(i)
                    - ((trace_evals[213].values.at(i) * trace_evals[213].values.at(i))
                        + (trace_evals[214].values.at(i) * trace_evals[214].values.at(i))));
            *numer += accum.random_coeff_powers[782]
                * (trace_evals[216].values.at(i)
                    - ((trace_evals[214].values.at(i) * trace_evals[214].values.at(i))
                        + (trace_evals[215].values.at(i) * trace_evals[215].values.at(i))));
            *numer += accum.random_coeff_powers[781]
                * (trace_evals[217].values.at(i)
                    - ((trace_evals[215].values.at(i) * trace_evals[215].values.at(i))
                        + (trace_evals[216].values.at(i) * trace_evals[216].values.at(i))));
            *numer += accum.random_coeff_powers[780]
                * (trace_evals[218].values.at(i)
                    - ((trace_evals[216].values.at(i) * trace_evals[216].values.at(i))
                        + (trace_evals[217].values.at(i) * trace_evals[217].values.at(i))));
            *numer += accum.random_coeff_powers[779]
                * (trace_evals[219].values.at(i)
                    - ((trace_evals[217].values.at(i) * trace_evals[217].values.at(i))
                        + (trace_evals[218].values.at(i) * trace_evals[218].values.at(i))));
            *numer += accum.random_coeff_powers[778]
                * (trace_evals[220].values.at(i)
                    - ((trace_evals[218].values.at(i) * trace_evals[218].values.at(i))
                        + (trace_evals[219].values.at(i) * trace_evals[219].values.at(i))));
            *numer += accum.random_coeff_powers[777]
                * (trace_evals[221].values.at(i)
                    - ((trace_evals[219].values.at(i) * trace_evals[219].values.at(i))
                        + (trace_evals[220].values.at(i) * trace_evals[220].values.at(i))));
            *numer += accum.random_coeff_powers[776]
                * (trace_evals[222].values.at(i)
                    - ((trace_evals[220].values.at(i) * trace_evals[220].values.at(i))
                        + (trace_evals[221].values.at(i) * trace_evals[221].values.at(i))));
            *numer += accum.random_coeff_powers[775]
                * (trace_evals[223].values.at(i)
                    - ((trace_evals[221].values.at(i) * trace_evals[221].values.at(i))
                        + (trace_evals[222].values.at(i) * trace_evals[222].values.at(i))));
            *numer += accum.random_coeff_powers[774]
                * (trace_evals[224].values.at(i)
                    - ((trace_evals[222].values.at(i) * trace_evals[222].values.at(i))
                        + (trace_evals[223].values.at(i) * trace_evals[223].values.at(i))));
            *numer += accum.random_coeff_powers[773]
                * (trace_evals[225].values.at(i)
                    - ((trace_evals[223].values.at(i) * trace_evals[223].values.at(i))
                        + (trace_evals[224].values.at(i) * trace_evals[224].values.at(i))));
            *numer += accum.random_coeff_powers[772]
                * (trace_evals[226].values.at(i)
                    - ((trace_evals[224].values.at(i) * trace_evals[224].values.at(i))
                        + (trace_evals[225].values.at(i) * trace_evals[225].values.at(i))));
            *numer += accum.random_coeff_powers[771]
                * (trace_evals[227].values.at(i)
                    - ((trace_evals[225].values.at(i) * trace_evals[225].values.at(i))
                        + (trace_evals[226].values.at(i) * trace_evals[226].values.at(i))));
            *numer += accum.random_coeff_powers[770]
                * (trace_evals[228].values.at(i)
                    - ((trace_evals[226].values.at(i) * trace_evals[226].values.at(i))
                        + (trace_evals[227].values.at(i) * trace_evals[227].values.at(i))));
            *numer += accum.random_coeff_powers[769]
                * (trace_evals[229].values.at(i)
                    - ((trace_evals[227].values.at(i) * trace_evals[227].values.at(i))
                        + (trace_evals[228].values.at(i) * trace_evals[228].values.at(i))));
            *numer += accum.random_coeff_powers[768]
                * (trace_evals[230].values.at(i)
                    - ((trace_evals[228].values.at(i) * trace_evals[228].values.at(i))
                        + (trace_evals[229].values.at(i) * trace_evals[229].values.at(i))));
            *numer += accum.random_coeff_powers[767]
                * (trace_evals[231].values.at(i)
                    - ((trace_evals[229].values.at(i) * trace_evals[229].values.at(i))
                        + (trace_evals[230].values.at(i) * trace_evals[230].values.at(i))));
            *numer += accum.random_coeff_powers[766]
                * (trace_evals[232].values.at(i)
                    - ((trace_evals[230].values.at(i) * trace_evals[230].values.at(i))
                        + (trace_evals[231].values.at(i) * trace_evals[231].values.at(i))));
            *numer += accum.random_coeff_powers[765]
                * (trace_evals[233].values.at(i)
                    - ((trace_evals[231].values.at(i) * trace_evals[231].values.at(i))
                        + (trace_evals[232].values.at(i) * trace_evals[232].values.at(i))));
            *numer += accum.random_coeff_powers[764]
                * (trace_evals[234].values.at(i)
                    - ((trace_evals[232].values.at(i) * trace_evals[232].values.at(i))
                        + (trace_evals[233].values.at(i) * trace_evals[233].values.at(i))));
            *numer += accum.random_coeff_powers[763]
                * (trace_evals[235].values.at(i)
                    - ((trace_evals[233].values.at(i) * trace_evals[233].values.at(i))
                        + (trace_evals[234].values.at(i) * trace_evals[234].values.at(i))));
            *numer += accum.random_coeff_powers[762]
                * (trace_evals[236].values.at(i)
                    - ((trace_evals[234].values.at(i) * trace_evals[234].values.at(i))
                        + (trace_evals[235].values.at(i) * trace_evals[235].values.at(i))));
            *numer += accum.random_coeff_powers[761]
                * (trace_evals[237].values.at(i)
                    - ((trace_evals[235].values.at(i) * trace_evals[235].values.at(i))
                        + (trace_evals[236].values.at(i) * trace_evals[236].values.at(i))));
            *numer += accum.random_coeff_powers[760]
                * (trace_evals[238].values.at(i)
                    - ((trace_evals[236].values.at(i) * trace_evals[236].values.at(i))
                        + (trace_evals[237].values.at(i) * trace_evals[237].values.at(i))));
            *numer += accum.random_coeff_powers[759]
                * (trace_evals[239].values.at(i)
                    - ((trace_evals[237].values.at(i) * trace_evals[237].values.at(i))
                        + (trace_evals[238].values.at(i) * trace_evals[238].values.at(i))));
            *numer += accum.random_coeff_powers[758]
                * (trace_evals[240].values.at(i)
                    - ((trace_evals[238].values.at(i) * trace_evals[238].values.at(i))
                        + (trace_evals[239].values.at(i) * trace_evals[239].values.at(i))));
            *numer += accum.random_coeff_powers[757]
                * (trace_evals[241].values.at(i)
                    - ((trace_evals[239].values.at(i) * trace_evals[239].values.at(i))
                        + (trace_evals[240].values.at(i) * trace_evals[240].values.at(i))));
            *numer += accum.random_coeff_powers[756]
                * (trace_evals[242].values.at(i)
                    - ((trace_evals[240].values.at(i) * trace_evals[240].values.at(i))
                        + (trace_evals[241].values.at(i) * trace_evals[241].values.at(i))));
            *numer += accum.random_coeff_powers[755]
                * (trace_evals[243].values.at(i)
                    - ((trace_evals[241].values.at(i) * trace_evals[241].values.at(i))
                        + (trace_evals[242].values.at(i) * trace_evals[242].values.at(i))));
            *numer += accum.random_coeff_powers[754]
                * (trace_evals[244].values.at(i)
                    - ((trace_evals[242].values.at(i) * trace_evals[242].values.at(i))
                        + (trace_evals[243].values.at(i) * trace_evals[243].values.at(i))));
            *numer += accum.random_coeff_powers[753]
                * (trace_evals[245].values.at(i)
                    - ((trace_evals[243].values.at(i) * trace_evals[243].values.at(i))
                        + (trace_evals[244].values.at(i) * trace_evals[244].values.at(i))));
            *numer += accum.random_coeff_powers[752]
                * (trace_evals[246].values.at(i)
                    - ((trace_evals[244].values.at(i) * trace_evals[244].values.at(i))
                        + (trace_evals[245].values.at(i) * trace_evals[245].values.at(i))));
            *numer += accum.random_coeff_powers[751]
                * (trace_evals[247].values.at(i)
                    - ((trace_evals[245].values.at(i) * trace_evals[245].values.at(i))
                        + (trace_evals[246].values.at(i) * trace_evals[246].values.at(i))));
            *numer += accum.random_coeff_powers[750]
                * (trace_evals[248].values.at(i)
                    - ((trace_evals[246].values.at(i) * trace_evals[246].values.at(i))
                        + (trace_evals[247].values.at(i) * trace_evals[247].values.at(i))));
            *numer += accum.random_coeff_powers[749]
                * (trace_evals[249].values.at(i)
                    - ((trace_evals[247].values.at(i) * trace_evals[247].values.at(i))
                        + (trace_evals[248].values.at(i) * trace_evals[248].values.at(i))));
            *numer += accum.random_coeff_powers[748]
                * (trace_evals[250].values.at(i)
                    - ((trace_evals[248].values.at(i) * trace_evals[248].values.at(i))
                        + (trace_evals[249].values.at(i) * trace_evals[249].values.at(i))));
            *numer += accum.random_coeff_powers[747]
                * (trace_evals[251].values.at(i)
                    - ((trace_evals[249].values.at(i) * trace_evals[249].values.at(i))
                        + (trace_evals[250].values.at(i) * trace_evals[250].values.at(i))));
            *numer += accum.random_coeff_powers[746]
                * (trace_evals[252].values.at(i)
                    - ((trace_evals[250].values.at(i) * trace_evals[250].values.at(i))
                        + (trace_evals[251].values.at(i) * trace_evals[251].values.at(i))));
            *numer += accum.random_coeff_powers[745]
                * (trace_evals[253].values.at(i)
                    - ((trace_evals[251].values.at(i) * trace_evals[251].values.at(i))
                        + (trace_evals[252].values.at(i) * trace_evals[252].values.at(i))));
            *numer += accum.random_coeff_powers[744]
                * (trace_evals[254].values.at(i)
                    - ((trace_evals[252].values.at(i) * trace_evals[252].values.at(i))
                        + (trace_evals[253].values.at(i) * trace_evals[253].values.at(i))));
            *numer += accum.random_coeff_powers[743]
                * (trace_evals[255].values.at(i)
                    - ((trace_evals[253].values.at(i) * trace_evals[253].values.at(i))
                        + (trace_evals[254].values.at(i) * trace_evals[254].values.at(i))));
            *numer += accum.random_coeff_powers[742]
                * (trace_evals[256].values.at(i)
                    - ((trace_evals[254].values.at(i) * trace_evals[254].values.at(i))
                        + (trace_evals[255].values.at(i) * trace_evals[255].values.at(i))));
            *numer += accum.random_coeff_powers[741]
                * (trace_evals[257].values.at(i)
                    - ((trace_evals[255].values.at(i) * trace_evals[255].values.at(i))
                        + (trace_evals[256].values.at(i) * trace_evals[256].values.at(i))));
            *numer += accum.random_coeff_powers[740]
                * (trace_evals[258].values.at(i)
                    - ((trace_evals[256].values.at(i) * trace_evals[256].values.at(i))
                        + (trace_evals[257].values.at(i) * trace_evals[257].values.at(i))));
            *numer += accum.random_coeff_powers[739]
                * (trace_evals[259].values.at(i)
                    - ((trace_evals[257].values.at(i) * trace_evals[257].values.at(i))
                        + (trace_evals[258].values.at(i) * trace_evals[258].values.at(i))));
            *numer += accum.random_coeff_powers[738]
                * (trace_evals[260].values.at(i)
                    - ((trace_evals[258].values.at(i) * trace_evals[258].values.at(i))
                        + (trace_evals[259].values.at(i) * trace_evals[259].values.at(i))));
            *numer += accum.random_coeff_powers[737]
                * (trace_evals[261].values.at(i)
                    - ((trace_evals[259].values.at(i) * trace_evals[259].values.at(i))
                        + (trace_evals[260].values.at(i) * trace_evals[260].values.at(i))));
            *numer += accum.random_coeff_powers[736]
                * (trace_evals[262].values.at(i)
                    - ((trace_evals[260].values.at(i) * trace_evals[260].values.at(i))
                        + (trace_evals[261].values.at(i) * trace_evals[261].values.at(i))));
            *numer += accum.random_coeff_powers[735]
                * (trace_evals[263].values.at(i)
                    - ((trace_evals[261].values.at(i) * trace_evals[261].values.at(i))
                        + (trace_evals[262].values.at(i) * trace_evals[262].values.at(i))));
            *numer += accum.random_coeff_powers[734]
                * (trace_evals[264].values.at(i)
                    - ((trace_evals[262].values.at(i) * trace_evals[262].values.at(i))
                        + (trace_evals[263].values.at(i) * trace_evals[263].values.at(i))));
            *numer += accum.random_coeff_powers[733]
                * (trace_evals[265].values.at(i)
                    - ((trace_evals[263].values.at(i) * trace_evals[263].values.at(i))
                        + (trace_evals[264].values.at(i) * trace_evals[264].values.at(i))));
            *numer += accum.random_coeff_powers[732]
                * (trace_evals[266].values.at(i)
                    - ((trace_evals[264].values.at(i) * trace_evals[264].values.at(i))
                        + (trace_evals[265].values.at(i) * trace_evals[265].values.at(i))));
            *numer += accum.random_coeff_powers[731]
                * (trace_evals[267].values.at(i)
                    - ((trace_evals[265].values.at(i) * trace_evals[265].values.at(i))
                        + (trace_evals[266].values.at(i) * trace_evals[266].values.at(i))));
            *numer += accum.random_coeff_powers[730]
                * (trace_evals[268].values.at(i)
                    - ((trace_evals[266].values.at(i) * trace_evals[266].values.at(i))
                        + (trace_evals[267].values.at(i) * trace_evals[267].values.at(i))));
            *numer += accum.random_coeff_powers[729]
                * (trace_evals[269].values.at(i)
                    - ((trace_evals[267].values.at(i) * trace_evals[267].values.at(i))
                        + (trace_evals[268].values.at(i) * trace_evals[268].values.at(i))));
            *numer += accum.random_coeff_powers[728]
                * (trace_evals[270].values.at(i)
                    - ((trace_evals[268].values.at(i) * trace_evals[268].values.at(i))
                        + (trace_evals[269].values.at(i) * trace_evals[269].values.at(i))));
            *numer += accum.random_coeff_powers[727]
                * (trace_evals[271].values.at(i)
                    - ((trace_evals[269].values.at(i) * trace_evals[269].values.at(i))
                        + (trace_evals[270].values.at(i) * trace_evals[270].values.at(i))));
            *numer += accum.random_coeff_powers[726]
                * (trace_evals[272].values.at(i)
                    - ((trace_evals[270].values.at(i) * trace_evals[270].values.at(i))
                        + (trace_evals[271].values.at(i) * trace_evals[271].values.at(i))));
            *numer += accum.random_coeff_powers[725]
                * (trace_evals[273].values.at(i)
                    - ((trace_evals[271].values.at(i) * trace_evals[271].values.at(i))
                        + (trace_evals[272].values.at(i) * trace_evals[272].values.at(i))));
            *numer += accum.random_coeff_powers[724]
                * (trace_evals[274].values.at(i)
                    - ((trace_evals[272].values.at(i) * trace_evals[272].values.at(i))
                        + (trace_evals[273].values.at(i) * trace_evals[273].values.at(i))));
            *numer += accum.random_coeff_powers[723]
                * (trace_evals[275].values.at(i)
                    - ((trace_evals[273].values.at(i) * trace_evals[273].values.at(i))
                        + (trace_evals[274].values.at(i) * trace_evals[274].values.at(i))));
            *numer += accum.random_coeff_powers[722]
                * (trace_evals[276].values.at(i)
                    - ((trace_evals[274].values.at(i) * trace_evals[274].values.at(i))
                        + (trace_evals[275].values.at(i) * trace_evals[275].values.at(i))));
            *numer += accum.random_coeff_powers[721]
                * (trace_evals[277].values.at(i)
                    - ((trace_evals[275].values.at(i) * trace_evals[275].values.at(i))
                        + (trace_evals[276].values.at(i) * trace_evals[276].values.at(i))));
            *numer += accum.random_coeff_powers[720]
                * (trace_evals[278].values.at(i)
                    - ((trace_evals[276].values.at(i) * trace_evals[276].values.at(i))
                        + (trace_evals[277].values.at(i) * trace_evals[277].values.at(i))));
            *numer += accum.random_coeff_powers[719]
                * (trace_evals[279].values.at(i)
                    - ((trace_evals[277].values.at(i) * trace_evals[277].values.at(i))
                        + (trace_evals[278].values.at(i) * trace_evals[278].values.at(i))));
            *numer += accum.random_coeff_powers[718]
                * (trace_evals[280].values.at(i)
                    - ((trace_evals[278].values.at(i) * trace_evals[278].values.at(i))
                        + (trace_evals[279].values.at(i) * trace_evals[279].values.at(i))));
            *numer += accum.random_coeff_powers[717]
                * (trace_evals[281].values.at(i)
                    - ((trace_evals[279].values.at(i) * trace_evals[279].values.at(i))
                        + (trace_evals[280].values.at(i) * trace_evals[280].values.at(i))));
            *numer += accum.random_coeff_powers[716]
                * (trace_evals[282].values.at(i)
                    - ((trace_evals[280].values.at(i) * trace_evals[280].values.at(i))
                        + (trace_evals[281].values.at(i) * trace_evals[281].values.at(i))));
            *numer += accum.random_coeff_powers[715]
                * (trace_evals[283].values.at(i)
                    - ((trace_evals[281].values.at(i) * trace_evals[281].values.at(i))
                        + (trace_evals[282].values.at(i) * trace_evals[282].values.at(i))));
            *numer += accum.random_coeff_powers[714]
                * (trace_evals[284].values.at(i)
                    - ((trace_evals[282].values.at(i) * trace_evals[282].values.at(i))
                        + (trace_evals[283].values.at(i) * trace_evals[283].values.at(i))));
            *numer += accum.random_coeff_powers[713]
                * (trace_evals[285].values.at(i)
                    - ((trace_evals[283].values.at(i) * trace_evals[283].values.at(i))
                        + (trace_evals[284].values.at(i) * trace_evals[284].values.at(i))));
            *numer += accum.random_coeff_powers[712]
                * (trace_evals[286].values.at(i)
                    - ((trace_evals[284].values.at(i) * trace_evals[284].values.at(i))
                        + (trace_evals[285].values.at(i) * trace_evals[285].values.at(i))));
            *numer += accum.random_coeff_powers[711]
                * (trace_evals[287].values.at(i)
                    - ((trace_evals[285].values.at(i) * trace_evals[285].values.at(i))
                        + (trace_evals[286].values.at(i) * trace_evals[286].values.at(i))));
            *numer += accum.random_coeff_powers[710]
                * (trace_evals[288].values.at(i)
                    - ((trace_evals[286].values.at(i) * trace_evals[286].values.at(i))
                        + (trace_evals[287].values.at(i) * trace_evals[287].values.at(i))));
            *numer += accum.random_coeff_powers[709]
                * (trace_evals[289].values.at(i)
                    - ((trace_evals[287].values.at(i) * trace_evals[287].values.at(i))
                        + (trace_evals[288].values.at(i) * trace_evals[288].values.at(i))));
            *numer += accum.random_coeff_powers[708]
                * (trace_evals[290].values.at(i)
                    - ((trace_evals[288].values.at(i) * trace_evals[288].values.at(i))
                        + (trace_evals[289].values.at(i) * trace_evals[289].values.at(i))));
            *numer += accum.random_coeff_powers[707]
                * (trace_evals[291].values.at(i)
                    - ((trace_evals[289].values.at(i) * trace_evals[289].values.at(i))
                        + (trace_evals[290].values.at(i) * trace_evals[290].values.at(i))));
            *numer += accum.random_coeff_powers[706]
                * (trace_evals[292].values.at(i)
                    - ((trace_evals[290].values.at(i) * trace_evals[290].values.at(i))
                        + (trace_evals[291].values.at(i) * trace_evals[291].values.at(i))));
            *numer += accum.random_coeff_powers[705]
                * (trace_evals[293].values.at(i)
                    - ((trace_evals[291].values.at(i) * trace_evals[291].values.at(i))
                        + (trace_evals[292].values.at(i) * trace_evals[292].values.at(i))));
            *numer += accum.random_coeff_powers[704]
                * (trace_evals[294].values.at(i)
                    - ((trace_evals[292].values.at(i) * trace_evals[292].values.at(i))
                        + (trace_evals[293].values.at(i) * trace_evals[293].values.at(i))));
            *numer += accum.random_coeff_powers[703]
                * (trace_evals[295].values.at(i)
                    - ((trace_evals[293].values.at(i) * trace_evals[293].values.at(i))
                        + (trace_evals[294].values.at(i) * trace_evals[294].values.at(i))));
            *numer += accum.random_coeff_powers[702]
                * (trace_evals[296].values.at(i)
                    - ((trace_evals[294].values.at(i) * trace_evals[294].values.at(i))
                        + (trace_evals[295].values.at(i) * trace_evals[295].values.at(i))));
            *numer += accum.random_coeff_powers[701]
                * (trace_evals[297].values.at(i)
                    - ((trace_evals[295].values.at(i) * trace_evals[295].values.at(i))
                        + (trace_evals[296].values.at(i) * trace_evals[296].values.at(i))));
            *numer += accum.random_coeff_powers[700]
                * (trace_evals[298].values.at(i)
                    - ((trace_evals[296].values.at(i) * trace_evals[296].values.at(i))
                        + (trace_evals[297].values.at(i) * trace_evals[297].values.at(i))));
            *numer += accum.random_coeff_powers[699]
                * (trace_evals[299].values.at(i)
                    - ((trace_evals[297].values.at(i) * trace_evals[297].values.at(i))
                        + (trace_evals[298].values.at(i) * trace_evals[298].values.at(i))));
            *numer += accum.random_coeff_powers[698]
                * (trace_evals[300].values.at(i)
                    - ((trace_evals[298].values.at(i) * trace_evals[298].values.at(i))
                        + (trace_evals[299].values.at(i) * trace_evals[299].values.at(i))));
            *numer += accum.random_coeff_powers[697]
                * (trace_evals[301].values.at(i)
                    - ((trace_evals[299].values.at(i) * trace_evals[299].values.at(i))
                        + (trace_evals[300].values.at(i) * trace_evals[300].values.at(i))));
            *numer += accum.random_coeff_powers[696]
                * (trace_evals[302].values.at(i)
                    - ((trace_evals[300].values.at(i) * trace_evals[300].values.at(i))
                        + (trace_evals[301].values.at(i) * trace_evals[301].values.at(i))));
            *numer += accum.random_coeff_powers[695]
                * (trace_evals[303].values.at(i)
                    - ((trace_evals[301].values.at(i) * trace_evals[301].values.at(i))
                        + (trace_evals[302].values.at(i) * trace_evals[302].values.at(i))));
            *numer += accum.random_coeff_powers[694]
                * (trace_evals[304].values.at(i)
                    - ((trace_evals[302].values.at(i) * trace_evals[302].values.at(i))
                        + (trace_evals[303].values.at(i) * trace_evals[303].values.at(i))));
            *numer += accum.random_coeff_powers[693]
                * (trace_evals[305].values.at(i)
                    - ((trace_evals[303].values.at(i) * trace_evals[303].values.at(i))
                        + (trace_evals[304].values.at(i) * trace_evals[304].values.at(i))));
            *numer += accum.random_coeff_powers[692]
                * (trace_evals[306].values.at(i)
                    - ((trace_evals[304].values.at(i) * trace_evals[304].values.at(i))
                        + (trace_evals[305].values.at(i) * trace_evals[305].values.at(i))));
            *numer += accum.random_coeff_powers[691]
                * (trace_evals[307].values.at(i)
                    - ((trace_evals[305].values.at(i) * trace_evals[305].values.at(i))
                        + (trace_evals[306].values.at(i) * trace_evals[306].values.at(i))));
            *numer += accum.random_coeff_powers[690]
                * (trace_evals[308].values.at(i)
                    - ((trace_evals[306].values.at(i) * trace_evals[306].values.at(i))
                        + (trace_evals[307].values.at(i) * trace_evals[307].values.at(i))));
            *numer += accum.random_coeff_powers[689]
                * (trace_evals[309].values.at(i)
                    - ((trace_evals[307].values.at(i) * trace_evals[307].values.at(i))
                        + (trace_evals[308].values.at(i) * trace_evals[308].values.at(i))));
            *numer += accum.random_coeff_powers[688]
                * (trace_evals[310].values.at(i)
                    - ((trace_evals[308].values.at(i) * trace_evals[308].values.at(i))
                        + (trace_evals[309].values.at(i) * trace_evals[309].values.at(i))));
            *numer += accum.random_coeff_powers[687]
                * (trace_evals[311].values.at(i)
                    - ((trace_evals[309].values.at(i) * trace_evals[309].values.at(i))
                        + (trace_evals[310].values.at(i) * trace_evals[310].values.at(i))));
            *numer += accum.random_coeff_powers[686]
                * (trace_evals[312].values.at(i)
                    - ((trace_evals[310].values.at(i) * trace_evals[310].values.at(i))
                        + (trace_evals[311].values.at(i) * trace_evals[311].values.at(i))));
            *numer += accum.random_coeff_powers[685]
                * (trace_evals[313].values.at(i)
                    - ((trace_evals[311].values.at(i) * trace_evals[311].values.at(i))
                        + (trace_evals[312].values.at(i) * trace_evals[312].values.at(i))));
            *numer += accum.random_coeff_powers[684]
                * (trace_evals[314].values.at(i)
                    - ((trace_evals[312].values.at(i) * trace_evals[312].values.at(i))
                        + (trace_evals[313].values.at(i) * trace_evals[313].values.at(i))));
            *numer += accum.random_coeff_powers[683]
                * (trace_evals[315].values.at(i)
                    - ((trace_evals[313].values.at(i) * trace_evals[313].values.at(i))
                        + (trace_evals[314].values.at(i) * trace_evals[314].values.at(i))));
            *numer += accum.random_coeff_powers[682]
                * (trace_evals[316].values.at(i)
                    - ((trace_evals[314].values.at(i) * trace_evals[314].values.at(i))
                        + (trace_evals[315].values.at(i) * trace_evals[315].values.at(i))));
            *numer += accum.random_coeff_powers[681]
                * (trace_evals[317].values.at(i)
                    - ((trace_evals[315].values.at(i) * trace_evals[315].values.at(i))
                        + (trace_evals[316].values.at(i) * trace_evals[316].values.at(i))));
            *numer += accum.random_coeff_powers[680]
                * (trace_evals[318].values.at(i)
                    - ((trace_evals[316].values.at(i) * trace_evals[316].values.at(i))
                        + (trace_evals[317].values.at(i) * trace_evals[317].values.at(i))));
            *numer += accum.random_coeff_powers[679]
                * (trace_evals[319].values.at(i)
                    - ((trace_evals[317].values.at(i) * trace_evals[317].values.at(i))
                        + (trace_evals[318].values.at(i) * trace_evals[318].values.at(i))));
            *numer += accum.random_coeff_powers[678]
                * (trace_evals[320].values.at(i)
                    - ((trace_evals[318].values.at(i) * trace_evals[318].values.at(i))
                        + (trace_evals[319].values.at(i) * trace_evals[319].values.at(i))));
            *numer += accum.random_coeff_powers[677]
                * (trace_evals[321].values.at(i)
                    - ((trace_evals[319].values.at(i) * trace_evals[319].values.at(i))
                        + (trace_evals[320].values.at(i) * trace_evals[320].values.at(i))));
            *numer += accum.random_coeff_powers[676]
                * (trace_evals[322].values.at(i)
                    - ((trace_evals[320].values.at(i) * trace_evals[320].values.at(i))
                        + (trace_evals[321].values.at(i) * trace_evals[321].values.at(i))));
            *numer += accum.random_coeff_powers[675]
                * (trace_evals[323].values.at(i)
                    - ((trace_evals[321].values.at(i) * trace_evals[321].values.at(i))
                        + (trace_evals[322].values.at(i) * trace_evals[322].values.at(i))));
            *numer += accum.random_coeff_powers[674]
                * (trace_evals[324].values.at(i)
                    - ((trace_evals[322].values.at(i) * trace_evals[322].values.at(i))
                        + (trace_evals[323].values.at(i) * trace_evals[323].values.at(i))));
            *numer += accum.random_coeff_powers[673]
                * (trace_evals[325].values.at(i)
                    - ((trace_evals[323].values.at(i) * trace_evals[323].values.at(i))
                        + (trace_evals[324].values.at(i) * trace_evals[324].values.at(i))));
            *numer += accum.random_coeff_powers[672]
                * (trace_evals[326].values.at(i)
                    - ((trace_evals[324].values.at(i) * trace_evals[324].values.at(i))
                        + (trace_evals[325].values.at(i) * trace_evals[325].values.at(i))));
            *numer += accum.random_coeff_powers[671]
                * (trace_evals[327].values.at(i)
                    - ((trace_evals[325].values.at(i) * trace_evals[325].values.at(i))
                        + (trace_evals[326].values.at(i) * trace_evals[326].values.at(i))));
            *numer += accum.random_coeff_powers[670]
                * (trace_evals[328].values.at(i)
                    - ((trace_evals[326].values.at(i) * trace_evals[326].values.at(i))
                        + (trace_evals[327].values.at(i) * trace_evals[327].values.at(i))));
            *numer += accum.random_coeff_powers[669]
                * (trace_evals[329].values.at(i)
                    - ((trace_evals[327].values.at(i) * trace_evals[327].values.at(i))
                        + (trace_evals[328].values.at(i) * trace_evals[328].values.at(i))));
            *numer += accum.random_coeff_powers[668]
                * (trace_evals[330].values.at(i)
                    - ((trace_evals[328].values.at(i) * trace_evals[328].values.at(i))
                        + (trace_evals[329].values.at(i) * trace_evals[329].values.at(i))));
            *numer += accum.random_coeff_powers[667]
                * (trace_evals[331].values.at(i)
                    - ((trace_evals[329].values.at(i) * trace_evals[329].values.at(i))
                        + (trace_evals[330].values.at(i) * trace_evals[330].values.at(i))));
            *numer += accum.random_coeff_powers[666]
                * (trace_evals[332].values.at(i)
                    - ((trace_evals[330].values.at(i) * trace_evals[330].values.at(i))
                        + (trace_evals[331].values.at(i) * trace_evals[331].values.at(i))));
            *numer += accum.random_coeff_powers[665]
                * (trace_evals[333].values.at(i)
                    - ((trace_evals[331].values.at(i) * trace_evals[331].values.at(i))
                        + (trace_evals[332].values.at(i) * trace_evals[332].values.at(i))));
            *numer += accum.random_coeff_powers[664]
                * (trace_evals[334].values.at(i)
                    - ((trace_evals[332].values.at(i) * trace_evals[332].values.at(i))
                        + (trace_evals[333].values.at(i) * trace_evals[333].values.at(i))));
            *numer += accum.random_coeff_powers[663]
                * (trace_evals[335].values.at(i)
                    - ((trace_evals[333].values.at(i) * trace_evals[333].values.at(i))
                        + (trace_evals[334].values.at(i) * trace_evals[334].values.at(i))));
            *numer += accum.random_coeff_powers[662]
                * (trace_evals[336].values.at(i)
                    - ((trace_evals[334].values.at(i) * trace_evals[334].values.at(i))
                        + (trace_evals[335].values.at(i) * trace_evals[335].values.at(i))));
            *numer += accum.random_coeff_powers[661]
                * (trace_evals[337].values.at(i)
                    - ((trace_evals[335].values.at(i) * trace_evals[335].values.at(i))
                        + (trace_evals[336].values.at(i) * trace_evals[336].values.at(i))));
            *numer += accum.random_coeff_powers[660]
                * (trace_evals[338].values.at(i)
                    - ((trace_evals[336].values.at(i) * trace_evals[336].values.at(i))
                        + (trace_evals[337].values.at(i) * trace_evals[337].values.at(i))));
            *numer += accum.random_coeff_powers[659]
                * (trace_evals[339].values.at(i)
                    - ((trace_evals[337].values.at(i) * trace_evals[337].values.at(i))
                        + (trace_evals[338].values.at(i) * trace_evals[338].values.at(i))));
            *numer += accum.random_coeff_powers[658]
                * (trace_evals[340].values.at(i)
                    - ((trace_evals[338].values.at(i) * trace_evals[338].values.at(i))
                        + (trace_evals[339].values.at(i) * trace_evals[339].values.at(i))));
            *numer += accum.random_coeff_powers[657]
                * (trace_evals[341].values.at(i)
                    - ((trace_evals[339].values.at(i) * trace_evals[339].values.at(i))
                        + (trace_evals[340].values.at(i) * trace_evals[340].values.at(i))));
            *numer += accum.random_coeff_powers[656]
                * (trace_evals[342].values.at(i)
                    - ((trace_evals[340].values.at(i) * trace_evals[340].values.at(i))
                        + (trace_evals[341].values.at(i) * trace_evals[341].values.at(i))));
            *numer += accum.random_coeff_powers[655]
                * (trace_evals[343].values.at(i)
                    - ((trace_evals[341].values.at(i) * trace_evals[341].values.at(i))
                        + (trace_evals[342].values.at(i) * trace_evals[342].values.at(i))));
            *numer += accum.random_coeff_powers[654]
                * (trace_evals[344].values.at(i)
                    - ((trace_evals[342].values.at(i) * trace_evals[342].values.at(i))
                        + (trace_evals[343].values.at(i) * trace_evals[343].values.at(i))));
            *numer += accum.random_coeff_powers[653]
                * (trace_evals[345].values.at(i)
                    - ((trace_evals[343].values.at(i) * trace_evals[343].values.at(i))
                        + (trace_evals[344].values.at(i) * trace_evals[344].values.at(i))));
            *numer += accum.random_coeff_powers[652]
                * (trace_evals[346].values.at(i)
                    - ((trace_evals[344].values.at(i) * trace_evals[344].values.at(i))
                        + (trace_evals[345].values.at(i) * trace_evals[345].values.at(i))));
            *numer += accum.random_coeff_powers[651]
                * (trace_evals[347].values.at(i)
                    - ((trace_evals[345].values.at(i) * trace_evals[345].values.at(i))
                        + (trace_evals[346].values.at(i) * trace_evals[346].values.at(i))));
            *numer += accum.random_coeff_powers[650]
                * (trace_evals[348].values.at(i)
                    - ((trace_evals[346].values.at(i) * trace_evals[346].values.at(i))
                        + (trace_evals[347].values.at(i) * trace_evals[347].values.at(i))));
            *numer += accum.random_coeff_powers[649]
                * (trace_evals[349].values.at(i)
                    - ((trace_evals[347].values.at(i) * trace_evals[347].values.at(i))
                        + (trace_evals[348].values.at(i) * trace_evals[348].values.at(i))));
            *numer += accum.random_coeff_powers[648]
                * (trace_evals[350].values.at(i)
                    - ((trace_evals[348].values.at(i) * trace_evals[348].values.at(i))
                        + (trace_evals[349].values.at(i) * trace_evals[349].values.at(i))));
            *numer += accum.random_coeff_powers[647]
                * (trace_evals[351].values.at(i)
                    - ((trace_evals[349].values.at(i) * trace_evals[349].values.at(i))
                        + (trace_evals[350].values.at(i) * trace_evals[350].values.at(i))));
            *numer += accum.random_coeff_powers[646]
                * (trace_evals[352].values.at(i)
                    - ((trace_evals[350].values.at(i) * trace_evals[350].values.at(i))
                        + (trace_evals[351].values.at(i) * trace_evals[351].values.at(i))));
            *numer += accum.random_coeff_powers[645]
                * (trace_evals[353].values.at(i)
                    - ((trace_evals[351].values.at(i) * trace_evals[351].values.at(i))
                        + (trace_evals[352].values.at(i) * trace_evals[352].values.at(i))));
            *numer += accum.random_coeff_powers[644]
                * (trace_evals[354].values.at(i)
                    - ((trace_evals[352].values.at(i) * trace_evals[352].values.at(i))
                        + (trace_evals[353].values.at(i) * trace_evals[353].values.at(i))));
            *numer += accum.random_coeff_powers[643]
                * (trace_evals[355].values.at(i)
                    - ((trace_evals[353].values.at(i) * trace_evals[353].values.at(i))
                        + (trace_evals[354].values.at(i) * trace_evals[354].values.at(i))));
            *numer += accum.random_coeff_powers[642]
                * (trace_evals[356].values.at(i)
                    - ((trace_evals[354].values.at(i) * trace_evals[354].values.at(i))
                        + (trace_evals[355].values.at(i) * trace_evals[355].values.at(i))));
            *numer += accum.random_coeff_powers[641]
                * (trace_evals[357].values.at(i)
                    - ((trace_evals[355].values.at(i) * trace_evals[355].values.at(i))
                        + (trace_evals[356].values.at(i) * trace_evals[356].values.at(i))));
            *numer += accum.random_coeff_powers[640]
                * (trace_evals[358].values.at(i)
                    - ((trace_evals[356].values.at(i) * trace_evals[356].values.at(i))
                        + (trace_evals[357].values.at(i) * trace_evals[357].values.at(i))));
            *numer += accum.random_coeff_powers[639]
                * (trace_evals[359].values.at(i)
                    - ((trace_evals[357].values.at(i) * trace_evals[357].values.at(i))
                        + (trace_evals[358].values.at(i) * trace_evals[358].values.at(i))));
            *numer += accum.random_coeff_powers[638]
                * (trace_evals[360].values.at(i)
                    - ((trace_evals[358].values.at(i) * trace_evals[358].values.at(i))
                        + (trace_evals[359].values.at(i) * trace_evals[359].values.at(i))));
            *numer += accum.random_coeff_powers[637]
                * (trace_evals[361].values.at(i)
                    - ((trace_evals[359].values.at(i) * trace_evals[359].values.at(i))
                        + (trace_evals[360].values.at(i) * trace_evals[360].values.at(i))));
            *numer += accum.random_coeff_powers[636]
                * (trace_evals[362].values.at(i)
                    - ((trace_evals[360].values.at(i) * trace_evals[360].values.at(i))
                        + (trace_evals[361].values.at(i) * trace_evals[361].values.at(i))));
            *numer += accum.random_coeff_powers[635]
                * (trace_evals[363].values.at(i)
                    - ((trace_evals[361].values.at(i) * trace_evals[361].values.at(i))
                        + (trace_evals[362].values.at(i) * trace_evals[362].values.at(i))));
            *numer += accum.random_coeff_powers[634]
                * (trace_evals[364].values.at(i)
                    - ((trace_evals[362].values.at(i) * trace_evals[362].values.at(i))
                        + (trace_evals[363].values.at(i) * trace_evals[363].values.at(i))));
            *numer += accum.random_coeff_powers[633]
                * (trace_evals[365].values.at(i)
                    - ((trace_evals[363].values.at(i) * trace_evals[363].values.at(i))
                        + (trace_evals[364].values.at(i) * trace_evals[364].values.at(i))));
            *numer += accum.random_coeff_powers[632]
                * (trace_evals[366].values.at(i)
                    - ((trace_evals[364].values.at(i) * trace_evals[364].values.at(i))
                        + (trace_evals[365].values.at(i) * trace_evals[365].values.at(i))));
            *numer += accum.random_coeff_powers[631]
                * (trace_evals[367].values.at(i)
                    - ((trace_evals[365].values.at(i) * trace_evals[365].values.at(i))
                        + (trace_evals[366].values.at(i) * trace_evals[366].values.at(i))));
            *numer += accum.random_coeff_powers[630]
                * (trace_evals[368].values.at(i)
                    - ((trace_evals[366].values.at(i) * trace_evals[366].values.at(i))
                        + (trace_evals[367].values.at(i) * trace_evals[367].values.at(i))));
            *numer += accum.random_coeff_powers[629]
                * (trace_evals[369].values.at(i)
                    - ((trace_evals[367].values.at(i) * trace_evals[367].values.at(i))
                        + (trace_evals[368].values.at(i) * trace_evals[368].values.at(i))));
            *numer += accum.random_coeff_powers[628]
                * (trace_evals[370].values.at(i)
                    - ((trace_evals[368].values.at(i) * trace_evals[368].values.at(i))
                        + (trace_evals[369].values.at(i) * trace_evals[369].values.at(i))));
            *numer += accum.random_coeff_powers[627]
                * (trace_evals[371].values.at(i)
                    - ((trace_evals[369].values.at(i) * trace_evals[369].values.at(i))
                        + (trace_evals[370].values.at(i) * trace_evals[370].values.at(i))));
            *numer += accum.random_coeff_powers[626]
                * (trace_evals[372].values.at(i)
                    - ((trace_evals[370].values.at(i) * trace_evals[370].values.at(i))
                        + (trace_evals[371].values.at(i) * trace_evals[371].values.at(i))));
            *numer += accum.random_coeff_powers[625]
                * (trace_evals[373].values.at(i)
                    - ((trace_evals[371].values.at(i) * trace_evals[371].values.at(i))
                        + (trace_evals[372].values.at(i) * trace_evals[372].values.at(i))));
            *numer += accum.random_coeff_powers[624]
                * (trace_evals[374].values.at(i)
                    - ((trace_evals[372].values.at(i) * trace_evals[372].values.at(i))
                        + (trace_evals[373].values.at(i) * trace_evals[373].values.at(i))));
            *numer += accum.random_coeff_powers[623]
                * (trace_evals[375].values.at(i)
                    - ((trace_evals[373].values.at(i) * trace_evals[373].values.at(i))
                        + (trace_evals[374].values.at(i) * trace_evals[374].values.at(i))));
            *numer += accum.random_coeff_powers[622]
                * (trace_evals[376].values.at(i)
                    - ((trace_evals[374].values.at(i) * trace_evals[374].values.at(i))
                        + (trace_evals[375].values.at(i) * trace_evals[375].values.at(i))));
            *numer += accum.random_coeff_powers[621]
                * (trace_evals[377].values.at(i)
                    - ((trace_evals[375].values.at(i) * trace_evals[375].values.at(i))
                        + (trace_evals[376].values.at(i) * trace_evals[376].values.at(i))));
            *numer += accum.random_coeff_powers[620]
                * (trace_evals[378].values.at(i)
                    - ((trace_evals[376].values.at(i) * trace_evals[376].values.at(i))
                        + (trace_evals[377].values.at(i) * trace_evals[377].values.at(i))));
            *numer += accum.random_coeff_powers[619]
                * (trace_evals[379].values.at(i)
                    - ((trace_evals[377].values.at(i) * trace_evals[377].values.at(i))
                        + (trace_evals[378].values.at(i) * trace_evals[378].values.at(i))));
            *numer += accum.random_coeff_powers[618]
                * (trace_evals[380].values.at(i)
                    - ((trace_evals[378].values.at(i) * trace_evals[378].values.at(i))
                        + (trace_evals[379].values.at(i) * trace_evals[379].values.at(i))));
            *numer += accum.random_coeff_powers[617]
                * (trace_evals[381].values.at(i)
                    - ((trace_evals[379].values.at(i) * trace_evals[379].values.at(i))
                        + (trace_evals[380].values.at(i) * trace_evals[380].values.at(i))));
            *numer += accum.random_coeff_powers[616]
                * (trace_evals[382].values.at(i)
                    - ((trace_evals[380].values.at(i) * trace_evals[380].values.at(i))
                        + (trace_evals[381].values.at(i) * trace_evals[381].values.at(i))));
            *numer += accum.random_coeff_powers[615]
                * (trace_evals[383].values.at(i)
                    - ((trace_evals[381].values.at(i) * trace_evals[381].values.at(i))
                        + (trace_evals[382].values.at(i) * trace_evals[382].values.at(i))));
            *numer += accum.random_coeff_powers[614]
                * (trace_evals[384].values.at(i)
                    - ((trace_evals[382].values.at(i) * trace_evals[382].values.at(i))
                        + (trace_evals[383].values.at(i) * trace_evals[383].values.at(i))));
            *numer += accum.random_coeff_powers[613]
                * (trace_evals[385].values.at(i)
                    - ((trace_evals[383].values.at(i) * trace_evals[383].values.at(i))
                        + (trace_evals[384].values.at(i) * trace_evals[384].values.at(i))));
            *numer += accum.random_coeff_powers[612]
                * (trace_evals[386].values.at(i)
                    - ((trace_evals[384].values.at(i) * trace_evals[384].values.at(i))
                        + (trace_evals[385].values.at(i) * trace_evals[385].values.at(i))));
            *numer += accum.random_coeff_powers[611]
                * (trace_evals[387].values.at(i)
                    - ((trace_evals[385].values.at(i) * trace_evals[385].values.at(i))
                        + (trace_evals[386].values.at(i) * trace_evals[386].values.at(i))));
            *numer += accum.random_coeff_powers[610]
                * (trace_evals[388].values.at(i)
                    - ((trace_evals[386].values.at(i) * trace_evals[386].values.at(i))
                        + (trace_evals[387].values.at(i) * trace_evals[387].values.at(i))));
            *numer += accum.random_coeff_powers[609]
                * (trace_evals[389].values.at(i)
                    - ((trace_evals[387].values.at(i) * trace_evals[387].values.at(i))
                        + (trace_evals[388].values.at(i) * trace_evals[388].values.at(i))));
            *numer += accum.random_coeff_powers[608]
                * (trace_evals[390].values.at(i)
                    - ((trace_evals[388].values.at(i) * trace_evals[388].values.at(i))
                        + (trace_evals[389].values.at(i) * trace_evals[389].values.at(i))));
            *numer += accum.random_coeff_powers[607]
                * (trace_evals[391].values.at(i)
                    - ((trace_evals[389].values.at(i) * trace_evals[389].values.at(i))
                        + (trace_evals[390].values.at(i) * trace_evals[390].values.at(i))));
            *numer += accum.random_coeff_powers[606]
                * (trace_evals[392].values.at(i)
                    - ((trace_evals[390].values.at(i) * trace_evals[390].values.at(i))
                        + (trace_evals[391].values.at(i) * trace_evals[391].values.at(i))));
            *numer += accum.random_coeff_powers[605]
                * (trace_evals[393].values.at(i)
                    - ((trace_evals[391].values.at(i) * trace_evals[391].values.at(i))
                        + (trace_evals[392].values.at(i) * trace_evals[392].values.at(i))));
            *numer += accum.random_coeff_powers[604]
                * (trace_evals[394].values.at(i)
                    - ((trace_evals[392].values.at(i) * trace_evals[392].values.at(i))
                        + (trace_evals[393].values.at(i) * trace_evals[393].values.at(i))));
            *numer += accum.random_coeff_powers[603]
                * (trace_evals[395].values.at(i)
                    - ((trace_evals[393].values.at(i) * trace_evals[393].values.at(i))
                        + (trace_evals[394].values.at(i) * trace_evals[394].values.at(i))));
            *numer += accum.random_coeff_powers[602]
                * (trace_evals[396].values.at(i)
                    - ((trace_evals[394].values.at(i) * trace_evals[394].values.at(i))
                        + (trace_evals[395].values.at(i) * trace_evals[395].values.at(i))));
            *numer += accum.random_coeff_powers[601]
                * (trace_evals[397].values.at(i)
                    - ((trace_evals[395].values.at(i) * trace_evals[395].values.at(i))
                        + (trace_evals[396].values.at(i) * trace_evals[396].values.at(i))));
            *numer += accum.random_coeff_powers[600]
                * (trace_evals[398].values.at(i)
                    - ((trace_evals[396].values.at(i) * trace_evals[396].values.at(i))
                        + (trace_evals[397].values.at(i) * trace_evals[397].values.at(i))));
            *numer += accum.random_coeff_powers[599]
                * (trace_evals[399].values.at(i)
                    - ((trace_evals[397].values.at(i) * trace_evals[397].values.at(i))
                        + (trace_evals[398].values.at(i) * trace_evals[398].values.at(i))));
            *numer += accum.random_coeff_powers[598]
                * (trace_evals[400].values.at(i)
                    - ((trace_evals[398].values.at(i) * trace_evals[398].values.at(i))
                        + (trace_evals[399].values.at(i) * trace_evals[399].values.at(i))));
            *numer += accum.random_coeff_powers[597]
                * (trace_evals[401].values.at(i)
                    - ((trace_evals[399].values.at(i) * trace_evals[399].values.at(i))
                        + (trace_evals[400].values.at(i) * trace_evals[400].values.at(i))));
            *numer += accum.random_coeff_powers[596]
                * (trace_evals[402].values.at(i)
                    - ((trace_evals[400].values.at(i) * trace_evals[400].values.at(i))
                        + (trace_evals[401].values.at(i) * trace_evals[401].values.at(i))));
            *numer += accum.random_coeff_powers[595]
                * (trace_evals[403].values.at(i)
                    - ((trace_evals[401].values.at(i) * trace_evals[401].values.at(i))
                        + (trace_evals[402].values.at(i) * trace_evals[402].values.at(i))));
            *numer += accum.random_coeff_powers[594]
                * (trace_evals[404].values.at(i)
                    - ((trace_evals[402].values.at(i) * trace_evals[402].values.at(i))
                        + (trace_evals[403].values.at(i) * trace_evals[403].values.at(i))));
            *numer += accum.random_coeff_powers[593]
                * (trace_evals[405].values.at(i)
                    - ((trace_evals[403].values.at(i) * trace_evals[403].values.at(i))
                        + (trace_evals[404].values.at(i) * trace_evals[404].values.at(i))));
            *numer += accum.random_coeff_powers[592]
                * (trace_evals[406].values.at(i)
                    - ((trace_evals[404].values.at(i) * trace_evals[404].values.at(i))
                        + (trace_evals[405].values.at(i) * trace_evals[405].values.at(i))));
            *numer += accum.random_coeff_powers[591]
                * (trace_evals[407].values.at(i)
                    - ((trace_evals[405].values.at(i) * trace_evals[405].values.at(i))
                        + (trace_evals[406].values.at(i) * trace_evals[406].values.at(i))));
            *numer += accum.random_coeff_powers[590]
                * (trace_evals[408].values.at(i)
                    - ((trace_evals[406].values.at(i) * trace_evals[406].values.at(i))
                        + (trace_evals[407].values.at(i) * trace_evals[407].values.at(i))));
            *numer += accum.random_coeff_powers[589]
                * (trace_evals[409].values.at(i)
                    - ((trace_evals[407].values.at(i) * trace_evals[407].values.at(i))
                        + (trace_evals[408].values.at(i) * trace_evals[408].values.at(i))));
            *numer += accum.random_coeff_powers[588]
                * (trace_evals[410].values.at(i)
                    - ((trace_evals[408].values.at(i) * trace_evals[408].values.at(i))
                        + (trace_evals[409].values.at(i) * trace_evals[409].values.at(i))));
            *numer += accum.random_coeff_powers[587]
                * (trace_evals[411].values.at(i)
                    - ((trace_evals[409].values.at(i) * trace_evals[409].values.at(i))
                        + (trace_evals[410].values.at(i) * trace_evals[410].values.at(i))));
            *numer += accum.random_coeff_powers[586]
                * (trace_evals[412].values.at(i)
                    - ((trace_evals[410].values.at(i) * trace_evals[410].values.at(i))
                        + (trace_evals[411].values.at(i) * trace_evals[411].values.at(i))));
            *numer += accum.random_coeff_powers[585]
                * (trace_evals[413].values.at(i)
                    - ((trace_evals[411].values.at(i) * trace_evals[411].values.at(i))
                        + (trace_evals[412].values.at(i) * trace_evals[412].values.at(i))));
            *numer += accum.random_coeff_powers[584]
                * (trace_evals[414].values.at(i)
                    - ((trace_evals[412].values.at(i) * trace_evals[412].values.at(i))
                        + (trace_evals[413].values.at(i) * trace_evals[413].values.at(i))));
            *numer += accum.random_coeff_powers[583]
                * (trace_evals[415].values.at(i)
                    - ((trace_evals[413].values.at(i) * trace_evals[413].values.at(i))
                        + (trace_evals[414].values.at(i) * trace_evals[414].values.at(i))));
            *numer += accum.random_coeff_powers[582]
                * (trace_evals[416].values.at(i)
                    - ((trace_evals[414].values.at(i) * trace_evals[414].values.at(i))
                        + (trace_evals[415].values.at(i) * trace_evals[415].values.at(i))));
            *numer += accum.random_coeff_powers[581]
                * (trace_evals[417].values.at(i)
                    - ((trace_evals[415].values.at(i) * trace_evals[415].values.at(i))
                        + (trace_evals[416].values.at(i) * trace_evals[416].values.at(i))));
            *numer += accum.random_coeff_powers[580]
                * (trace_evals[418].values.at(i)
                    - ((trace_evals[416].values.at(i) * trace_evals[416].values.at(i))
                        + (trace_evals[417].values.at(i) * trace_evals[417].values.at(i))));
            *numer += accum.random_coeff_powers[579]
                * (trace_evals[419].values.at(i)
                    - ((trace_evals[417].values.at(i) * trace_evals[417].values.at(i))
                        + (trace_evals[418].values.at(i) * trace_evals[418].values.at(i))));
            *numer += accum.random_coeff_powers[578]
                * (trace_evals[420].values.at(i)
                    - ((trace_evals[418].values.at(i) * trace_evals[418].values.at(i))
                        + (trace_evals[419].values.at(i) * trace_evals[419].values.at(i))));
            *numer += accum.random_coeff_powers[577]
                * (trace_evals[421].values.at(i)
                    - ((trace_evals[419].values.at(i) * trace_evals[419].values.at(i))
                        + (trace_evals[420].values.at(i) * trace_evals[420].values.at(i))));
            *numer += accum.random_coeff_powers[576]
                * (trace_evals[422].values.at(i)
                    - ((trace_evals[420].values.at(i) * trace_evals[420].values.at(i))
                        + (trace_evals[421].values.at(i) * trace_evals[421].values.at(i))));
            *numer += accum.random_coeff_powers[575]
                * (trace_evals[423].values.at(i)
                    - ((trace_evals[421].values.at(i) * trace_evals[421].values.at(i))
                        + (trace_evals[422].values.at(i) * trace_evals[422].values.at(i))));
            *numer += accum.random_coeff_powers[574]
                * (trace_evals[424].values.at(i)
                    - ((trace_evals[422].values.at(i) * trace_evals[422].values.at(i))
                        + (trace_evals[423].values.at(i) * trace_evals[423].values.at(i))));
            *numer += accum.random_coeff_powers[573]
                * (trace_evals[425].values.at(i)
                    - ((trace_evals[423].values.at(i) * trace_evals[423].values.at(i))
                        + (trace_evals[424].values.at(i) * trace_evals[424].values.at(i))));
            *numer += accum.random_coeff_powers[572]
                * (trace_evals[426].values.at(i)
                    - ((trace_evals[424].values.at(i) * trace_evals[424].values.at(i))
                        + (trace_evals[425].values.at(i) * trace_evals[425].values.at(i))));
            *numer += accum.random_coeff_powers[571]
                * (trace_evals[427].values.at(i)
                    - ((trace_evals[425].values.at(i) * trace_evals[425].values.at(i))
                        + (trace_evals[426].values.at(i) * trace_evals[426].values.at(i))));
            *numer += accum.random_coeff_powers[570]
                * (trace_evals[428].values.at(i)
                    - ((trace_evals[426].values.at(i) * trace_evals[426].values.at(i))
                        + (trace_evals[427].values.at(i) * trace_evals[427].values.at(i))));
            *numer += accum.random_coeff_powers[569]
                * (trace_evals[429].values.at(i)
                    - ((trace_evals[427].values.at(i) * trace_evals[427].values.at(i))
                        + (trace_evals[428].values.at(i) * trace_evals[428].values.at(i))));
            *numer += accum.random_coeff_powers[568]
                * (trace_evals[430].values.at(i)
                    - ((trace_evals[428].values.at(i) * trace_evals[428].values.at(i))
                        + (trace_evals[429].values.at(i) * trace_evals[429].values.at(i))));
            *numer += accum.random_coeff_powers[567]
                * (trace_evals[431].values.at(i)
                    - ((trace_evals[429].values.at(i) * trace_evals[429].values.at(i))
                        + (trace_evals[430].values.at(i) * trace_evals[430].values.at(i))));
            *numer += accum.random_coeff_powers[566]
                * (trace_evals[432].values.at(i)
                    - ((trace_evals[430].values.at(i) * trace_evals[430].values.at(i))
                        + (trace_evals[431].values.at(i) * trace_evals[431].values.at(i))));
            *numer += accum.random_coeff_powers[565]
                * (trace_evals[433].values.at(i)
                    - ((trace_evals[431].values.at(i) * trace_evals[431].values.at(i))
                        + (trace_evals[432].values.at(i) * trace_evals[432].values.at(i))));
            *numer += accum.random_coeff_powers[564]
                * (trace_evals[434].values.at(i)
                    - ((trace_evals[432].values.at(i) * trace_evals[432].values.at(i))
                        + (trace_evals[433].values.at(i) * trace_evals[433].values.at(i))));
            *numer += accum.random_coeff_powers[563]
                * (trace_evals[435].values.at(i)
                    - ((trace_evals[433].values.at(i) * trace_evals[433].values.at(i))
                        + (trace_evals[434].values.at(i) * trace_evals[434].values.at(i))));
            *numer += accum.random_coeff_powers[562]
                * (trace_evals[436].values.at(i)
                    - ((trace_evals[434].values.at(i) * trace_evals[434].values.at(i))
                        + (trace_evals[435].values.at(i) * trace_evals[435].values.at(i))));
            *numer += accum.random_coeff_powers[561]
                * (trace_evals[437].values.at(i)
                    - ((trace_evals[435].values.at(i) * trace_evals[435].values.at(i))
                        + (trace_evals[436].values.at(i) * trace_evals[436].values.at(i))));
            *numer += accum.random_coeff_powers[560]
                * (trace_evals[438].values.at(i)
                    - ((trace_evals[436].values.at(i) * trace_evals[436].values.at(i))
                        + (trace_evals[437].values.at(i) * trace_evals[437].values.at(i))));
            *numer += accum.random_coeff_powers[559]
                * (trace_evals[439].values.at(i)
                    - ((trace_evals[437].values.at(i) * trace_evals[437].values.at(i))
                        + (trace_evals[438].values.at(i) * trace_evals[438].values.at(i))));
            *numer += accum.random_coeff_powers[558]
                * (trace_evals[440].values.at(i)
                    - ((trace_evals[438].values.at(i) * trace_evals[438].values.at(i))
                        + (trace_evals[439].values.at(i) * trace_evals[439].values.at(i))));
            *numer += accum.random_coeff_powers[557]
                * (trace_evals[441].values.at(i)
                    - ((trace_evals[439].values.at(i) * trace_evals[439].values.at(i))
                        + (trace_evals[440].values.at(i) * trace_evals[440].values.at(i))));
            *numer += accum.random_coeff_powers[556]
                * (trace_evals[442].values.at(i)
                    - ((trace_evals[440].values.at(i) * trace_evals[440].values.at(i))
                        + (trace_evals[441].values.at(i) * trace_evals[441].values.at(i))));
            *numer += accum.random_coeff_powers[555]
                * (trace_evals[443].values.at(i)
                    - ((trace_evals[441].values.at(i) * trace_evals[441].values.at(i))
                        + (trace_evals[442].values.at(i) * trace_evals[442].values.at(i))));
            *numer += accum.random_coeff_powers[554]
                * (trace_evals[444].values.at(i)
                    - ((trace_evals[442].values.at(i) * trace_evals[442].values.at(i))
                        + (trace_evals[443].values.at(i) * trace_evals[443].values.at(i))));
            *numer += accum.random_coeff_powers[553]
                * (trace_evals[445].values.at(i)
                    - ((trace_evals[443].values.at(i) * trace_evals[443].values.at(i))
                        + (trace_evals[444].values.at(i) * trace_evals[444].values.at(i))));
            *numer += accum.random_coeff_powers[552]
                * (trace_evals[446].values.at(i)
                    - ((trace_evals[444].values.at(i) * trace_evals[444].values.at(i))
                        + (trace_evals[445].values.at(i) * trace_evals[445].values.at(i))));
            *numer += accum.random_coeff_powers[551]
                * (trace_evals[447].values.at(i)
                    - ((trace_evals[445].values.at(i) * trace_evals[445].values.at(i))
                        + (trace_evals[446].values.at(i) * trace_evals[446].values.at(i))));
            *numer += accum.random_coeff_powers[550]
                * (trace_evals[448].values.at(i)
                    - ((trace_evals[446].values.at(i) * trace_evals[446].values.at(i))
                        + (trace_evals[447].values.at(i) * trace_evals[447].values.at(i))));
            *numer += accum.random_coeff_powers[549]
                * (trace_evals[449].values.at(i)
                    - ((trace_evals[447].values.at(i) * trace_evals[447].values.at(i))
                        + (trace_evals[448].values.at(i) * trace_evals[448].values.at(i))));
            *numer += accum.random_coeff_powers[548]
                * (trace_evals[450].values.at(i)
                    - ((trace_evals[448].values.at(i) * trace_evals[448].values.at(i))
                        + (trace_evals[449].values.at(i) * trace_evals[449].values.at(i))));
            *numer += accum.random_coeff_powers[547]
                * (trace_evals[451].values.at(i)
                    - ((trace_evals[449].values.at(i) * trace_evals[449].values.at(i))
                        + (trace_evals[450].values.at(i) * trace_evals[450].values.at(i))));
            *numer += accum.random_coeff_powers[546]
                * (trace_evals[452].values.at(i)
                    - ((trace_evals[450].values.at(i) * trace_evals[450].values.at(i))
                        + (trace_evals[451].values.at(i) * trace_evals[451].values.at(i))));
            *numer += accum.random_coeff_powers[545]
                * (trace_evals[453].values.at(i)
                    - ((trace_evals[451].values.at(i) * trace_evals[451].values.at(i))
                        + (trace_evals[452].values.at(i) * trace_evals[452].values.at(i))));
            *numer += accum.random_coeff_powers[544]
                * (trace_evals[454].values.at(i)
                    - ((trace_evals[452].values.at(i) * trace_evals[452].values.at(i))
                        + (trace_evals[453].values.at(i) * trace_evals[453].values.at(i))));
            *numer += accum.random_coeff_powers[543]
                * (trace_evals[455].values.at(i)
                    - ((trace_evals[453].values.at(i) * trace_evals[453].values.at(i))
                        + (trace_evals[454].values.at(i) * trace_evals[454].values.at(i))));
            *numer += accum.random_coeff_powers[542]
                * (trace_evals[456].values.at(i)
                    - ((trace_evals[454].values.at(i) * trace_evals[454].values.at(i))
                        + (trace_evals[455].values.at(i) * trace_evals[455].values.at(i))));
            *numer += accum.random_coeff_powers[541]
                * (trace_evals[457].values.at(i)
                    - ((trace_evals[455].values.at(i) * trace_evals[455].values.at(i))
                        + (trace_evals[456].values.at(i) * trace_evals[456].values.at(i))));
            *numer += accum.random_coeff_powers[540]
                * (trace_evals[458].values.at(i)
                    - ((trace_evals[456].values.at(i) * trace_evals[456].values.at(i))
                        + (trace_evals[457].values.at(i) * trace_evals[457].values.at(i))));
            *numer += accum.random_coeff_powers[539]
                * (trace_evals[459].values.at(i)
                    - ((trace_evals[457].values.at(i) * trace_evals[457].values.at(i))
                        + (trace_evals[458].values.at(i) * trace_evals[458].values.at(i))));
            *numer += accum.random_coeff_powers[538]
                * (trace_evals[460].values.at(i)
                    - ((trace_evals[458].values.at(i) * trace_evals[458].values.at(i))
                        + (trace_evals[459].values.at(i) * trace_evals[459].values.at(i))));
            *numer += accum.random_coeff_powers[537]
                * (trace_evals[461].values.at(i)
                    - ((trace_evals[459].values.at(i) * trace_evals[459].values.at(i))
                        + (trace_evals[460].values.at(i) * trace_evals[460].values.at(i))));
            *numer += accum.random_coeff_powers[536]
                * (trace_evals[462].values.at(i)
                    - ((trace_evals[460].values.at(i) * trace_evals[460].values.at(i))
                        + (trace_evals[461].values.at(i) * trace_evals[461].values.at(i))));
            *numer += accum.random_coeff_powers[535]
                * (trace_evals[463].values.at(i)
                    - ((trace_evals[461].values.at(i) * trace_evals[461].values.at(i))
                        + (trace_evals[462].values.at(i) * trace_evals[462].values.at(i))));
            *numer += accum.random_coeff_powers[534]
                * (trace_evals[464].values.at(i)
                    - ((trace_evals[462].values.at(i) * trace_evals[462].values.at(i))
                        + (trace_evals[463].values.at(i) * trace_evals[463].values.at(i))));
            *numer += accum.random_coeff_powers[533]
                * (trace_evals[465].values.at(i)
                    - ((trace_evals[463].values.at(i) * trace_evals[463].values.at(i))
                        + (trace_evals[464].values.at(i) * trace_evals[464].values.at(i))));
            *numer += accum.random_coeff_powers[532]
                * (trace_evals[466].values.at(i)
                    - ((trace_evals[464].values.at(i) * trace_evals[464].values.at(i))
                        + (trace_evals[465].values.at(i) * trace_evals[465].values.at(i))));
            *numer += accum.random_coeff_powers[531]
                * (trace_evals[467].values.at(i)
                    - ((trace_evals[465].values.at(i) * trace_evals[465].values.at(i))
                        + (trace_evals[466].values.at(i) * trace_evals[466].values.at(i))));
            *numer += accum.random_coeff_powers[530]
                * (trace_evals[468].values.at(i)
                    - ((trace_evals[466].values.at(i) * trace_evals[466].values.at(i))
                        + (trace_evals[467].values.at(i) * trace_evals[467].values.at(i))));
            *numer += accum.random_coeff_powers[529]
                * (trace_evals[469].values.at(i)
                    - ((trace_evals[467].values.at(i) * trace_evals[467].values.at(i))
                        + (trace_evals[468].values.at(i) * trace_evals[468].values.at(i))));
            *numer += accum.random_coeff_powers[528]
                * (trace_evals[470].values.at(i)
                    - ((trace_evals[468].values.at(i) * trace_evals[468].values.at(i))
                        + (trace_evals[469].values.at(i) * trace_evals[469].values.at(i))));
            *numer += accum.random_coeff_powers[527]
                * (trace_evals[471].values.at(i)
                    - ((trace_evals[469].values.at(i) * trace_evals[469].values.at(i))
                        + (trace_evals[470].values.at(i) * trace_evals[470].values.at(i))));
            *numer += accum.random_coeff_powers[526]
                * (trace_evals[472].values.at(i)
                    - ((trace_evals[470].values.at(i) * trace_evals[470].values.at(i))
                        + (trace_evals[471].values.at(i) * trace_evals[471].values.at(i))));
            *numer += accum.random_coeff_powers[525]
                * (trace_evals[473].values.at(i)
                    - ((trace_evals[471].values.at(i) * trace_evals[471].values.at(i))
                        + (trace_evals[472].values.at(i) * trace_evals[472].values.at(i))));
            *numer += accum.random_coeff_powers[524]
                * (trace_evals[474].values.at(i)
                    - ((trace_evals[472].values.at(i) * trace_evals[472].values.at(i))
                        + (trace_evals[473].values.at(i) * trace_evals[473].values.at(i))));
            *numer += accum.random_coeff_powers[523]
                * (trace_evals[475].values.at(i)
                    - ((trace_evals[473].values.at(i) * trace_evals[473].values.at(i))
                        + (trace_evals[474].values.at(i) * trace_evals[474].values.at(i))));
            *numer += accum.random_coeff_powers[522]
                * (trace_evals[476].values.at(i)
                    - ((trace_evals[474].values.at(i) * trace_evals[474].values.at(i))
                        + (trace_evals[475].values.at(i) * trace_evals[475].values.at(i))));
            *numer += accum.random_coeff_powers[521]
                * (trace_evals[477].values.at(i)
                    - ((trace_evals[475].values.at(i) * trace_evals[475].values.at(i))
                        + (trace_evals[476].values.at(i) * trace_evals[476].values.at(i))));
            *numer += accum.random_coeff_powers[520]
                * (trace_evals[478].values.at(i)
                    - ((trace_evals[476].values.at(i) * trace_evals[476].values.at(i))
                        + (trace_evals[477].values.at(i) * trace_evals[477].values.at(i))));
            *numer += accum.random_coeff_powers[519]
                * (trace_evals[479].values.at(i)
                    - ((trace_evals[477].values.at(i) * trace_evals[477].values.at(i))
                        + (trace_evals[478].values.at(i) * trace_evals[478].values.at(i))));
            *numer += accum.random_coeff_powers[518]
                * (trace_evals[480].values.at(i)
                    - ((trace_evals[478].values.at(i) * trace_evals[478].values.at(i))
                        + (trace_evals[479].values.at(i) * trace_evals[479].values.at(i))));
            *numer += accum.random_coeff_powers[517]
                * (trace_evals[481].values.at(i)
                    - ((trace_evals[479].values.at(i) * trace_evals[479].values.at(i))
                        + (trace_evals[480].values.at(i) * trace_evals[480].values.at(i))));
            *numer += accum.random_coeff_powers[516]
                * (trace_evals[482].values.at(i)
                    - ((trace_evals[480].values.at(i) * trace_evals[480].values.at(i))
                        + (trace_evals[481].values.at(i) * trace_evals[481].values.at(i))));
            *numer += accum.random_coeff_powers[515]
                * (trace_evals[483].values.at(i)
                    - ((trace_evals[481].values.at(i) * trace_evals[481].values.at(i))
                        + (trace_evals[482].values.at(i) * trace_evals[482].values.at(i))));
            *numer += accum.random_coeff_powers[514]
                * (trace_evals[484].values.at(i)
                    - ((trace_evals[482].values.at(i) * trace_evals[482].values.at(i))
                        + (trace_evals[483].values.at(i) * trace_evals[483].values.at(i))));
            *numer += accum.random_coeff_powers[513]
                * (trace_evals[485].values.at(i)
                    - ((trace_evals[483].values.at(i) * trace_evals[483].values.at(i))
                        + (trace_evals[484].values.at(i) * trace_evals[484].values.at(i))));
            *numer += accum.random_coeff_powers[512]
                * (trace_evals[486].values.at(i)
                    - ((trace_evals[484].values.at(i) * trace_evals[484].values.at(i))
                        + (trace_evals[485].values.at(i) * trace_evals[485].values.at(i))));
            *numer += accum.random_coeff_powers[511]
                * (trace_evals[487].values.at(i)
                    - ((trace_evals[485].values.at(i) * trace_evals[485].values.at(i))
                        + (trace_evals[486].values.at(i) * trace_evals[486].values.at(i))));
            *numer += accum.random_coeff_powers[510]
                * (trace_evals[488].values.at(i)
                    - ((trace_evals[486].values.at(i) * trace_evals[486].values.at(i))
                        + (trace_evals[487].values.at(i) * trace_evals[487].values.at(i))));
            *numer += accum.random_coeff_powers[509]
                * (trace_evals[489].values.at(i)
                    - ((trace_evals[487].values.at(i) * trace_evals[487].values.at(i))
                        + (trace_evals[488].values.at(i) * trace_evals[488].values.at(i))));
            *numer += accum.random_coeff_powers[508]
                * (trace_evals[490].values.at(i)
                    - ((trace_evals[488].values.at(i) * trace_evals[488].values.at(i))
                        + (trace_evals[489].values.at(i) * trace_evals[489].values.at(i))));
            *numer += accum.random_coeff_powers[507]
                * (trace_evals[491].values.at(i)
                    - ((trace_evals[489].values.at(i) * trace_evals[489].values.at(i))
                        + (trace_evals[490].values.at(i) * trace_evals[490].values.at(i))));
            *numer += accum.random_coeff_powers[506]
                * (trace_evals[492].values.at(i)
                    - ((trace_evals[490].values.at(i) * trace_evals[490].values.at(i))
                        + (trace_evals[491].values.at(i) * trace_evals[491].values.at(i))));
            *numer += accum.random_coeff_powers[505]
                * (trace_evals[493].values.at(i)
                    - ((trace_evals[491].values.at(i) * trace_evals[491].values.at(i))
                        + (trace_evals[492].values.at(i) * trace_evals[492].values.at(i))));
            *numer += accum.random_coeff_powers[504]
                * (trace_evals[494].values.at(i)
                    - ((trace_evals[492].values.at(i) * trace_evals[492].values.at(i))
                        + (trace_evals[493].values.at(i) * trace_evals[493].values.at(i))));
            *numer += accum.random_coeff_powers[503]
                * (trace_evals[495].values.at(i)
                    - ((trace_evals[493].values.at(i) * trace_evals[493].values.at(i))
                        + (trace_evals[494].values.at(i) * trace_evals[494].values.at(i))));
            *numer += accum.random_coeff_powers[502]
                * (trace_evals[496].values.at(i)
                    - ((trace_evals[494].values.at(i) * trace_evals[494].values.at(i))
                        + (trace_evals[495].values.at(i) * trace_evals[495].values.at(i))));
            *numer += accum.random_coeff_powers[501]
                * (trace_evals[497].values.at(i)
                    - ((trace_evals[495].values.at(i) * trace_evals[495].values.at(i))
                        + (trace_evals[496].values.at(i) * trace_evals[496].values.at(i))));
            *numer += accum.random_coeff_powers[500]
                * (trace_evals[498].values.at(i)
                    - ((trace_evals[496].values.at(i) * trace_evals[496].values.at(i))
                        + (trace_evals[497].values.at(i) * trace_evals[497].values.at(i))));
            *numer += accum.random_coeff_powers[499]
                * (trace_evals[499].values.at(i)
                    - ((trace_evals[497].values.at(i) * trace_evals[497].values.at(i))
                        + (trace_evals[498].values.at(i) * trace_evals[498].values.at(i))));
            *numer += accum.random_coeff_powers[498]
                * (trace_evals[500].values.at(i)
                    - ((trace_evals[498].values.at(i) * trace_evals[498].values.at(i))
                        + (trace_evals[499].values.at(i) * trace_evals[499].values.at(i))));
            *numer += accum.random_coeff_powers[497]
                * (trace_evals[501].values.at(i)
                    - ((trace_evals[499].values.at(i) * trace_evals[499].values.at(i))
                        + (trace_evals[500].values.at(i) * trace_evals[500].values.at(i))));
            *numer += accum.random_coeff_powers[496]
                * (trace_evals[502].values.at(i)
                    - ((trace_evals[500].values.at(i) * trace_evals[500].values.at(i))
                        + (trace_evals[501].values.at(i) * trace_evals[501].values.at(i))));
            *numer += accum.random_coeff_powers[495]
                * (trace_evals[503].values.at(i)
                    - ((trace_evals[501].values.at(i) * trace_evals[501].values.at(i))
                        + (trace_evals[502].values.at(i) * trace_evals[502].values.at(i))));
            *numer += accum.random_coeff_powers[494]
                * (trace_evals[504].values.at(i)
                    - ((trace_evals[502].values.at(i) * trace_evals[502].values.at(i))
                        + (trace_evals[503].values.at(i) * trace_evals[503].values.at(i))));
            *numer += accum.random_coeff_powers[493]
                * (trace_evals[505].values.at(i)
                    - ((trace_evals[503].values.at(i) * trace_evals[503].values.at(i))
                        + (trace_evals[504].values.at(i) * trace_evals[504].values.at(i))));
            *numer += accum.random_coeff_powers[492]
                * (trace_evals[506].values.at(i)
                    - ((trace_evals[504].values.at(i) * trace_evals[504].values.at(i))
                        + (trace_evals[505].values.at(i) * trace_evals[505].values.at(i))));
            *numer += accum.random_coeff_powers[491]
                * (trace_evals[507].values.at(i)
                    - ((trace_evals[505].values.at(i) * trace_evals[505].values.at(i))
                        + (trace_evals[506].values.at(i) * trace_evals[506].values.at(i))));
            *numer += accum.random_coeff_powers[490]
                * (trace_evals[508].values.at(i)
                    - ((trace_evals[506].values.at(i) * trace_evals[506].values.at(i))
                        + (trace_evals[507].values.at(i) * trace_evals[507].values.at(i))));
            *numer += accum.random_coeff_powers[489]
                * (trace_evals[509].values.at(i)
                    - ((trace_evals[507].values.at(i) * trace_evals[507].values.at(i))
                        + (trace_evals[508].values.at(i) * trace_evals[508].values.at(i))));
            *numer += accum.random_coeff_powers[488]
                * (trace_evals[510].values.at(i)
                    - ((trace_evals[508].values.at(i) * trace_evals[508].values.at(i))
                        + (trace_evals[509].values.at(i) * trace_evals[509].values.at(i))));
            *numer += accum.random_coeff_powers[487]
                * (trace_evals[511].values.at(i)
                    - ((trace_evals[509].values.at(i) * trace_evals[509].values.at(i))
                        + (trace_evals[510].values.at(i) * trace_evals[510].values.at(i))));
            *numer += accum.random_coeff_powers[486]
                * (trace_evals[512].values.at(i)
                    - ((trace_evals[510].values.at(i) * trace_evals[510].values.at(i))
                        + (trace_evals[511].values.at(i) * trace_evals[511].values.at(i))));
            *numer += accum.random_coeff_powers[485]
                * (trace_evals[513].values.at(i)
                    - ((trace_evals[511].values.at(i) * trace_evals[511].values.at(i))
                        + (trace_evals[512].values.at(i) * trace_evals[512].values.at(i))));
            *numer += accum.random_coeff_powers[484]
                * (trace_evals[514].values.at(i)
                    - ((trace_evals[512].values.at(i) * trace_evals[512].values.at(i))
                        + (trace_evals[513].values.at(i) * trace_evals[513].values.at(i))));
            *numer += accum.random_coeff_powers[483]
                * (trace_evals[515].values.at(i)
                    - ((trace_evals[513].values.at(i) * trace_evals[513].values.at(i))
                        + (trace_evals[514].values.at(i) * trace_evals[514].values.at(i))));
            *numer += accum.random_coeff_powers[482]
                * (trace_evals[516].values.at(i)
                    - ((trace_evals[514].values.at(i) * trace_evals[514].values.at(i))
                        + (trace_evals[515].values.at(i) * trace_evals[515].values.at(i))));
            *numer += accum.random_coeff_powers[481]
                * (trace_evals[517].values.at(i)
                    - ((trace_evals[515].values.at(i) * trace_evals[515].values.at(i))
                        + (trace_evals[516].values.at(i) * trace_evals[516].values.at(i))));
            *numer += accum.random_coeff_powers[480]
                * (trace_evals[518].values.at(i)
                    - ((trace_evals[516].values.at(i) * trace_evals[516].values.at(i))
                        + (trace_evals[517].values.at(i) * trace_evals[517].values.at(i))));
            *numer += accum.random_coeff_powers[479]
                * (trace_evals[519].values.at(i)
                    - ((trace_evals[517].values.at(i) * trace_evals[517].values.at(i))
                        + (trace_evals[518].values.at(i) * trace_evals[518].values.at(i))));
            *numer += accum.random_coeff_powers[478]
                * (trace_evals[520].values.at(i)
                    - ((trace_evals[518].values.at(i) * trace_evals[518].values.at(i))
                        + (trace_evals[519].values.at(i) * trace_evals[519].values.at(i))));
            *numer += accum.random_coeff_powers[477]
                * (trace_evals[521].values.at(i)
                    - ((trace_evals[519].values.at(i) * trace_evals[519].values.at(i))
                        + (trace_evals[520].values.at(i) * trace_evals[520].values.at(i))));
            *numer += accum.random_coeff_powers[476]
                * (trace_evals[522].values.at(i)
                    - ((trace_evals[520].values.at(i) * trace_evals[520].values.at(i))
                        + (trace_evals[521].values.at(i) * trace_evals[521].values.at(i))));
            *numer += accum.random_coeff_powers[475]
                * (trace_evals[523].values.at(i)
                    - ((trace_evals[521].values.at(i) * trace_evals[521].values.at(i))
                        + (trace_evals[522].values.at(i) * trace_evals[522].values.at(i))));
            *numer += accum.random_coeff_powers[474]
                * (trace_evals[524].values.at(i)
                    - ((trace_evals[522].values.at(i) * trace_evals[522].values.at(i))
                        + (trace_evals[523].values.at(i) * trace_evals[523].values.at(i))));
            *numer += accum.random_coeff_powers[473]
                * (trace_evals[525].values.at(i)
                    - ((trace_evals[523].values.at(i) * trace_evals[523].values.at(i))
                        + (trace_evals[524].values.at(i) * trace_evals[524].values.at(i))));
            *numer += accum.random_coeff_powers[472]
                * (trace_evals[526].values.at(i)
                    - ((trace_evals[524].values.at(i) * trace_evals[524].values.at(i))
                        + (trace_evals[525].values.at(i) * trace_evals[525].values.at(i))));
            *numer += accum.random_coeff_powers[471]
                * (trace_evals[527].values.at(i)
                    - ((trace_evals[525].values.at(i) * trace_evals[525].values.at(i))
                        + (trace_evals[526].values.at(i) * trace_evals[526].values.at(i))));
            *numer += accum.random_coeff_powers[470]
                * (trace_evals[528].values.at(i)
                    - ((trace_evals[526].values.at(i) * trace_evals[526].values.at(i))
                        + (trace_evals[527].values.at(i) * trace_evals[527].values.at(i))));
            *numer += accum.random_coeff_powers[469]
                * (trace_evals[529].values.at(i)
                    - ((trace_evals[527].values.at(i) * trace_evals[527].values.at(i))
                        + (trace_evals[528].values.at(i) * trace_evals[528].values.at(i))));
            *numer += accum.random_coeff_powers[468]
                * (trace_evals[530].values.at(i)
                    - ((trace_evals[528].values.at(i) * trace_evals[528].values.at(i))
                        + (trace_evals[529].values.at(i) * trace_evals[529].values.at(i))));
            *numer += accum.random_coeff_powers[467]
                * (trace_evals[531].values.at(i)
                    - ((trace_evals[529].values.at(i) * trace_evals[529].values.at(i))
                        + (trace_evals[530].values.at(i) * trace_evals[530].values.at(i))));
            *numer += accum.random_coeff_powers[466]
                * (trace_evals[532].values.at(i)
                    - ((trace_evals[530].values.at(i) * trace_evals[530].values.at(i))
                        + (trace_evals[531].values.at(i) * trace_evals[531].values.at(i))));
            *numer += accum.random_coeff_powers[465]
                * (trace_evals[533].values.at(i)
                    - ((trace_evals[531].values.at(i) * trace_evals[531].values.at(i))
                        + (trace_evals[532].values.at(i) * trace_evals[532].values.at(i))));
            *numer += accum.random_coeff_powers[464]
                * (trace_evals[534].values.at(i)
                    - ((trace_evals[532].values.at(i) * trace_evals[532].values.at(i))
                        + (trace_evals[533].values.at(i) * trace_evals[533].values.at(i))));
            *numer += accum.random_coeff_powers[463]
                * (trace_evals[535].values.at(i)
                    - ((trace_evals[533].values.at(i) * trace_evals[533].values.at(i))
                        + (trace_evals[534].values.at(i) * trace_evals[534].values.at(i))));
            *numer += accum.random_coeff_powers[462]
                * (trace_evals[536].values.at(i)
                    - ((trace_evals[534].values.at(i) * trace_evals[534].values.at(i))
                        + (trace_evals[535].values.at(i) * trace_evals[535].values.at(i))));
            *numer += accum.random_coeff_powers[461]
                * (trace_evals[537].values.at(i)
                    - ((trace_evals[535].values.at(i) * trace_evals[535].values.at(i))
                        + (trace_evals[536].values.at(i) * trace_evals[536].values.at(i))));
            *numer += accum.random_coeff_powers[460]
                * (trace_evals[538].values.at(i)
                    - ((trace_evals[536].values.at(i) * trace_evals[536].values.at(i))
                        + (trace_evals[537].values.at(i) * trace_evals[537].values.at(i))));
            *numer += accum.random_coeff_powers[459]
                * (trace_evals[539].values.at(i)
                    - ((trace_evals[537].values.at(i) * trace_evals[537].values.at(i))
                        + (trace_evals[538].values.at(i) * trace_evals[538].values.at(i))));
            *numer += accum.random_coeff_powers[458]
                * (trace_evals[540].values.at(i)
                    - ((trace_evals[538].values.at(i) * trace_evals[538].values.at(i))
                        + (trace_evals[539].values.at(i) * trace_evals[539].values.at(i))));
            *numer += accum.random_coeff_powers[457]
                * (trace_evals[541].values.at(i)
                    - ((trace_evals[539].values.at(i) * trace_evals[539].values.at(i))
                        + (trace_evals[540].values.at(i) * trace_evals[540].values.at(i))));
            *numer += accum.random_coeff_powers[456]
                * (trace_evals[542].values.at(i)
                    - ((trace_evals[540].values.at(i) * trace_evals[540].values.at(i))
                        + (trace_evals[541].values.at(i) * trace_evals[541].values.at(i))));
            *numer += accum.random_coeff_powers[455]
                * (trace_evals[543].values.at(i)
                    - ((trace_evals[541].values.at(i) * trace_evals[541].values.at(i))
                        + (trace_evals[542].values.at(i) * trace_evals[542].values.at(i))));
            *numer += accum.random_coeff_powers[454]
                * (trace_evals[544].values.at(i)
                    - ((trace_evals[542].values.at(i) * trace_evals[542].values.at(i))
                        + (trace_evals[543].values.at(i) * trace_evals[543].values.at(i))));
            *numer += accum.random_coeff_powers[453]
                * (trace_evals[545].values.at(i)
                    - ((trace_evals[543].values.at(i) * trace_evals[543].values.at(i))
                        + (trace_evals[544].values.at(i) * trace_evals[544].values.at(i))));
            *numer += accum.random_coeff_powers[452]
                * (trace_evals[546].values.at(i)
                    - ((trace_evals[544].values.at(i) * trace_evals[544].values.at(i))
                        + (trace_evals[545].values.at(i) * trace_evals[545].values.at(i))));
            *numer += accum.random_coeff_powers[451]
                * (trace_evals[547].values.at(i)
                    - ((trace_evals[545].values.at(i) * trace_evals[545].values.at(i))
                        + (trace_evals[546].values.at(i) * trace_evals[546].values.at(i))));
            *numer += accum.random_coeff_powers[450]
                * (trace_evals[548].values.at(i)
                    - ((trace_evals[546].values.at(i) * trace_evals[546].values.at(i))
                        + (trace_evals[547].values.at(i) * trace_evals[547].values.at(i))));
            *numer += accum.random_coeff_powers[449]
                * (trace_evals[549].values.at(i)
                    - ((trace_evals[547].values.at(i) * trace_evals[547].values.at(i))
                        + (trace_evals[548].values.at(i) * trace_evals[548].values.at(i))));
            *numer += accum.random_coeff_powers[448]
                * (trace_evals[550].values.at(i)
                    - ((trace_evals[548].values.at(i) * trace_evals[548].values.at(i))
                        + (trace_evals[549].values.at(i) * trace_evals[549].values.at(i))));
            *numer += accum.random_coeff_powers[447]
                * (trace_evals[551].values.at(i)
                    - ((trace_evals[549].values.at(i) * trace_evals[549].values.at(i))
                        + (trace_evals[550].values.at(i) * trace_evals[550].values.at(i))));
            *numer += accum.random_coeff_powers[446]
                * (trace_evals[552].values.at(i)
                    - ((trace_evals[550].values.at(i) * trace_evals[550].values.at(i))
                        + (trace_evals[551].values.at(i) * trace_evals[551].values.at(i))));
            *numer += accum.random_coeff_powers[445]
                * (trace_evals[553].values.at(i)
                    - ((trace_evals[551].values.at(i) * trace_evals[551].values.at(i))
                        + (trace_evals[552].values.at(i) * trace_evals[552].values.at(i))));
            *numer += accum.random_coeff_powers[444]
                * (trace_evals[554].values.at(i)
                    - ((trace_evals[552].values.at(i) * trace_evals[552].values.at(i))
                        + (trace_evals[553].values.at(i) * trace_evals[553].values.at(i))));
            *numer += accum.random_coeff_powers[443]
                * (trace_evals[555].values.at(i)
                    - ((trace_evals[553].values.at(i) * trace_evals[553].values.at(i))
                        + (trace_evals[554].values.at(i) * trace_evals[554].values.at(i))));
            *numer += accum.random_coeff_powers[442]
                * (trace_evals[556].values.at(i)
                    - ((trace_evals[554].values.at(i) * trace_evals[554].values.at(i))
                        + (trace_evals[555].values.at(i) * trace_evals[555].values.at(i))));
            *numer += accum.random_coeff_powers[441]
                * (trace_evals[557].values.at(i)
                    - ((trace_evals[555].values.at(i) * trace_evals[555].values.at(i))
                        + (trace_evals[556].values.at(i) * trace_evals[556].values.at(i))));
            *numer += accum.random_coeff_powers[440]
                * (trace_evals[558].values.at(i)
                    - ((trace_evals[556].values.at(i) * trace_evals[556].values.at(i))
                        + (trace_evals[557].values.at(i) * trace_evals[557].values.at(i))));
            *numer += accum.random_coeff_powers[439]
                * (trace_evals[559].values.at(i)
                    - ((trace_evals[557].values.at(i) * trace_evals[557].values.at(i))
                        + (trace_evals[558].values.at(i) * trace_evals[558].values.at(i))));
            *numer += accum.random_coeff_powers[438]
                * (trace_evals[560].values.at(i)
                    - ((trace_evals[558].values.at(i) * trace_evals[558].values.at(i))
                        + (trace_evals[559].values.at(i) * trace_evals[559].values.at(i))));
            *numer += accum.random_coeff_powers[437]
                * (trace_evals[561].values.at(i)
                    - ((trace_evals[559].values.at(i) * trace_evals[559].values.at(i))
                        + (trace_evals[560].values.at(i) * trace_evals[560].values.at(i))));
            *numer += accum.random_coeff_powers[436]
                * (trace_evals[562].values.at(i)
                    - ((trace_evals[560].values.at(i) * trace_evals[560].values.at(i))
                        + (trace_evals[561].values.at(i) * trace_evals[561].values.at(i))));
            *numer += accum.random_coeff_powers[435]
                * (trace_evals[563].values.at(i)
                    - ((trace_evals[561].values.at(i) * trace_evals[561].values.at(i))
                        + (trace_evals[562].values.at(i) * trace_evals[562].values.at(i))));
            *numer += accum.random_coeff_powers[434]
                * (trace_evals[564].values.at(i)
                    - ((trace_evals[562].values.at(i) * trace_evals[562].values.at(i))
                        + (trace_evals[563].values.at(i) * trace_evals[563].values.at(i))));
            *numer += accum.random_coeff_powers[433]
                * (trace_evals[565].values.at(i)
                    - ((trace_evals[563].values.at(i) * trace_evals[563].values.at(i))
                        + (trace_evals[564].values.at(i) * trace_evals[564].values.at(i))));
            *numer += accum.random_coeff_powers[432]
                * (trace_evals[566].values.at(i)
                    - ((trace_evals[564].values.at(i) * trace_evals[564].values.at(i))
                        + (trace_evals[565].values.at(i) * trace_evals[565].values.at(i))));
            *numer += accum.random_coeff_powers[431]
                * (trace_evals[567].values.at(i)
                    - ((trace_evals[565].values.at(i) * trace_evals[565].values.at(i))
                        + (trace_evals[566].values.at(i) * trace_evals[566].values.at(i))));
            *numer += accum.random_coeff_powers[430]
                * (trace_evals[568].values.at(i)
                    - ((trace_evals[566].values.at(i) * trace_evals[566].values.at(i))
                        + (trace_evals[567].values.at(i) * trace_evals[567].values.at(i))));
            *numer += accum.random_coeff_powers[429]
                * (trace_evals[569].values.at(i)
                    - ((trace_evals[567].values.at(i) * trace_evals[567].values.at(i))
                        + (trace_evals[568].values.at(i) * trace_evals[568].values.at(i))));
            *numer += accum.random_coeff_powers[428]
                * (trace_evals[570].values.at(i)
                    - ((trace_evals[568].values.at(i) * trace_evals[568].values.at(i))
                        + (trace_evals[569].values.at(i) * trace_evals[569].values.at(i))));
            *numer += accum.random_coeff_powers[427]
                * (trace_evals[571].values.at(i)
                    - ((trace_evals[569].values.at(i) * trace_evals[569].values.at(i))
                        + (trace_evals[570].values.at(i) * trace_evals[570].values.at(i))));
            *numer += accum.random_coeff_powers[426]
                * (trace_evals[572].values.at(i)
                    - ((trace_evals[570].values.at(i) * trace_evals[570].values.at(i))
                        + (trace_evals[571].values.at(i) * trace_evals[571].values.at(i))));
            *numer += accum.random_coeff_powers[425]
                * (trace_evals[573].values.at(i)
                    - ((trace_evals[571].values.at(i) * trace_evals[571].values.at(i))
                        + (trace_evals[572].values.at(i) * trace_evals[572].values.at(i))));
            *numer += accum.random_coeff_powers[424]
                * (trace_evals[574].values.at(i)
                    - ((trace_evals[572].values.at(i) * trace_evals[572].values.at(i))
                        + (trace_evals[573].values.at(i) * trace_evals[573].values.at(i))));
            *numer += accum.random_coeff_powers[423]
                * (trace_evals[575].values.at(i)
                    - ((trace_evals[573].values.at(i) * trace_evals[573].values.at(i))
                        + (trace_evals[574].values.at(i) * trace_evals[574].values.at(i))));
            *numer += accum.random_coeff_powers[422]
                * (trace_evals[576].values.at(i)
                    - ((trace_evals[574].values.at(i) * trace_evals[574].values.at(i))
                        + (trace_evals[575].values.at(i) * trace_evals[575].values.at(i))));
            *numer += accum.random_coeff_powers[421]
                * (trace_evals[577].values.at(i)
                    - ((trace_evals[575].values.at(i) * trace_evals[575].values.at(i))
                        + (trace_evals[576].values.at(i) * trace_evals[576].values.at(i))));
            *numer += accum.random_coeff_powers[420]
                * (trace_evals[578].values.at(i)
                    - ((trace_evals[576].values.at(i) * trace_evals[576].values.at(i))
                        + (trace_evals[577].values.at(i) * trace_evals[577].values.at(i))));
            *numer += accum.random_coeff_powers[419]
                * (trace_evals[579].values.at(i)
                    - ((trace_evals[577].values.at(i) * trace_evals[577].values.at(i))
                        + (trace_evals[578].values.at(i) * trace_evals[578].values.at(i))));
            *numer += accum.random_coeff_powers[418]
                * (trace_evals[580].values.at(i)
                    - ((trace_evals[578].values.at(i) * trace_evals[578].values.at(i))
                        + (trace_evals[579].values.at(i) * trace_evals[579].values.at(i))));
            *numer += accum.random_coeff_powers[417]
                * (trace_evals[581].values.at(i)
                    - ((trace_evals[579].values.at(i) * trace_evals[579].values.at(i))
                        + (trace_evals[580].values.at(i) * trace_evals[580].values.at(i))));
            *numer += accum.random_coeff_powers[416]
                * (trace_evals[582].values.at(i)
                    - ((trace_evals[580].values.at(i) * trace_evals[580].values.at(i))
                        + (trace_evals[581].values.at(i) * trace_evals[581].values.at(i))));
            *numer += accum.random_coeff_powers[415]
                * (trace_evals[583].values.at(i)
                    - ((trace_evals[581].values.at(i) * trace_evals[581].values.at(i))
                        + (trace_evals[582].values.at(i) * trace_evals[582].values.at(i))));
            *numer += accum.random_coeff_powers[414]
                * (trace_evals[584].values.at(i)
                    - ((trace_evals[582].values.at(i) * trace_evals[582].values.at(i))
                        + (trace_evals[583].values.at(i) * trace_evals[583].values.at(i))));
            *numer += accum.random_coeff_powers[413]
                * (trace_evals[585].values.at(i)
                    - ((trace_evals[583].values.at(i) * trace_evals[583].values.at(i))
                        + (trace_evals[584].values.at(i) * trace_evals[584].values.at(i))));
            *numer += accum.random_coeff_powers[412]
                * (trace_evals[586].values.at(i)
                    - ((trace_evals[584].values.at(i) * trace_evals[584].values.at(i))
                        + (trace_evals[585].values.at(i) * trace_evals[585].values.at(i))));
            *numer += accum.random_coeff_powers[411]
                * (trace_evals[587].values.at(i)
                    - ((trace_evals[585].values.at(i) * trace_evals[585].values.at(i))
                        + (trace_evals[586].values.at(i) * trace_evals[586].values.at(i))));
            *numer += accum.random_coeff_powers[410]
                * (trace_evals[588].values.at(i)
                    - ((trace_evals[586].values.at(i) * trace_evals[586].values.at(i))
                        + (trace_evals[587].values.at(i) * trace_evals[587].values.at(i))));
            *numer += accum.random_coeff_powers[409]
                * (trace_evals[589].values.at(i)
                    - ((trace_evals[587].values.at(i) * trace_evals[587].values.at(i))
                        + (trace_evals[588].values.at(i) * trace_evals[588].values.at(i))));
            *numer += accum.random_coeff_powers[408]
                * (trace_evals[590].values.at(i)
                    - ((trace_evals[588].values.at(i) * trace_evals[588].values.at(i))
                        + (trace_evals[589].values.at(i) * trace_evals[589].values.at(i))));
            *numer += accum.random_coeff_powers[407]
                * (trace_evals[591].values.at(i)
                    - ((trace_evals[589].values.at(i) * trace_evals[589].values.at(i))
                        + (trace_evals[590].values.at(i) * trace_evals[590].values.at(i))));
            *numer += accum.random_coeff_powers[406]
                * (trace_evals[592].values.at(i)
                    - ((trace_evals[590].values.at(i) * trace_evals[590].values.at(i))
                        + (trace_evals[591].values.at(i) * trace_evals[591].values.at(i))));
            *numer += accum.random_coeff_powers[405]
                * (trace_evals[593].values.at(i)
                    - ((trace_evals[591].values.at(i) * trace_evals[591].values.at(i))
                        + (trace_evals[592].values.at(i) * trace_evals[592].values.at(i))));
            *numer += accum.random_coeff_powers[404]
                * (trace_evals[594].values.at(i)
                    - ((trace_evals[592].values.at(i) * trace_evals[592].values.at(i))
                        + (trace_evals[593].values.at(i) * trace_evals[593].values.at(i))));
            *numer += accum.random_coeff_powers[403]
                * (trace_evals[595].values.at(i)
                    - ((trace_evals[593].values.at(i) * trace_evals[593].values.at(i))
                        + (trace_evals[594].values.at(i) * trace_evals[594].values.at(i))));
            *numer += accum.random_coeff_powers[402]
                * (trace_evals[596].values.at(i)
                    - ((trace_evals[594].values.at(i) * trace_evals[594].values.at(i))
                        + (trace_evals[595].values.at(i) * trace_evals[595].values.at(i))));
            *numer += accum.random_coeff_powers[401]
                * (trace_evals[597].values.at(i)
                    - ((trace_evals[595].values.at(i) * trace_evals[595].values.at(i))
                        + (trace_evals[596].values.at(i) * trace_evals[596].values.at(i))));
            *numer += accum.random_coeff_powers[400]
                * (trace_evals[598].values.at(i)
                    - ((trace_evals[596].values.at(i) * trace_evals[596].values.at(i))
                        + (trace_evals[597].values.at(i) * trace_evals[597].values.at(i))));
            *numer += accum.random_coeff_powers[399]
                * (trace_evals[599].values.at(i)
                    - ((trace_evals[597].values.at(i) * trace_evals[597].values.at(i))
                        + (trace_evals[598].values.at(i) * trace_evals[598].values.at(i))));
            *numer += accum.random_coeff_powers[398]
                * (trace_evals[600].values.at(i)
                    - ((trace_evals[598].values.at(i) * trace_evals[598].values.at(i))
                        + (trace_evals[599].values.at(i) * trace_evals[599].values.at(i))));
            *numer += accum.random_coeff_powers[397]
                * (trace_evals[601].values.at(i)
                    - ((trace_evals[599].values.at(i) * trace_evals[599].values.at(i))
                        + (trace_evals[600].values.at(i) * trace_evals[600].values.at(i))));
            *numer += accum.random_coeff_powers[396]
                * (trace_evals[602].values.at(i)
                    - ((trace_evals[600].values.at(i) * trace_evals[600].values.at(i))
                        + (trace_evals[601].values.at(i) * trace_evals[601].values.at(i))));
            *numer += accum.random_coeff_powers[395]
                * (trace_evals[603].values.at(i)
                    - ((trace_evals[601].values.at(i) * trace_evals[601].values.at(i))
                        + (trace_evals[602].values.at(i) * trace_evals[602].values.at(i))));
            *numer += accum.random_coeff_powers[394]
                * (trace_evals[604].values.at(i)
                    - ((trace_evals[602].values.at(i) * trace_evals[602].values.at(i))
                        + (trace_evals[603].values.at(i) * trace_evals[603].values.at(i))));
            *numer += accum.random_coeff_powers[393]
                * (trace_evals[605].values.at(i)
                    - ((trace_evals[603].values.at(i) * trace_evals[603].values.at(i))
                        + (trace_evals[604].values.at(i) * trace_evals[604].values.at(i))));
            *numer += accum.random_coeff_powers[392]
                * (trace_evals[606].values.at(i)
                    - ((trace_evals[604].values.at(i) * trace_evals[604].values.at(i))
                        + (trace_evals[605].values.at(i) * trace_evals[605].values.at(i))));
            *numer += accum.random_coeff_powers[391]
                * (trace_evals[607].values.at(i)
                    - ((trace_evals[605].values.at(i) * trace_evals[605].values.at(i))
                        + (trace_evals[606].values.at(i) * trace_evals[606].values.at(i))));
            *numer += accum.random_coeff_powers[390]
                * (trace_evals[608].values.at(i)
                    - ((trace_evals[606].values.at(i) * trace_evals[606].values.at(i))
                        + (trace_evals[607].values.at(i) * trace_evals[607].values.at(i))));
            *numer += accum.random_coeff_powers[389]
                * (trace_evals[609].values.at(i)
                    - ((trace_evals[607].values.at(i) * trace_evals[607].values.at(i))
                        + (trace_evals[608].values.at(i) * trace_evals[608].values.at(i))));
            *numer += accum.random_coeff_powers[388]
                * (trace_evals[610].values.at(i)
                    - ((trace_evals[608].values.at(i) * trace_evals[608].values.at(i))
                        + (trace_evals[609].values.at(i) * trace_evals[609].values.at(i))));
            *numer += accum.random_coeff_powers[387]
                * (trace_evals[611].values.at(i)
                    - ((trace_evals[609].values.at(i) * trace_evals[609].values.at(i))
                        + (trace_evals[610].values.at(i) * trace_evals[610].values.at(i))));
            *numer += accum.random_coeff_powers[386]
                * (trace_evals[612].values.at(i)
                    - ((trace_evals[610].values.at(i) * trace_evals[610].values.at(i))
                        + (trace_evals[611].values.at(i) * trace_evals[611].values.at(i))));
            *numer += accum.random_coeff_powers[385]
                * (trace_evals[613].values.at(i)
                    - ((trace_evals[611].values.at(i) * trace_evals[611].values.at(i))
                        + (trace_evals[612].values.at(i) * trace_evals[612].values.at(i))));
            *numer += accum.random_coeff_powers[384]
                * (trace_evals[614].values.at(i)
                    - ((trace_evals[612].values.at(i) * trace_evals[612].values.at(i))
                        + (trace_evals[613].values.at(i) * trace_evals[613].values.at(i))));
            *numer += accum.random_coeff_powers[383]
                * (trace_evals[615].values.at(i)
                    - ((trace_evals[613].values.at(i) * trace_evals[613].values.at(i))
                        + (trace_evals[614].values.at(i) * trace_evals[614].values.at(i))));
            *numer += accum.random_coeff_powers[382]
                * (trace_evals[616].values.at(i)
                    - ((trace_evals[614].values.at(i) * trace_evals[614].values.at(i))
                        + (trace_evals[615].values.at(i) * trace_evals[615].values.at(i))));
            *numer += accum.random_coeff_powers[381]
                * (trace_evals[617].values.at(i)
                    - ((trace_evals[615].values.at(i) * trace_evals[615].values.at(i))
                        + (trace_evals[616].values.at(i) * trace_evals[616].values.at(i))));
            *numer += accum.random_coeff_powers[380]
                * (trace_evals[618].values.at(i)
                    - ((trace_evals[616].values.at(i) * trace_evals[616].values.at(i))
                        + (trace_evals[617].values.at(i) * trace_evals[617].values.at(i))));
            *numer += accum.random_coeff_powers[379]
                * (trace_evals[619].values.at(i)
                    - ((trace_evals[617].values.at(i) * trace_evals[617].values.at(i))
                        + (trace_evals[618].values.at(i) * trace_evals[618].values.at(i))));
            *numer += accum.random_coeff_powers[378]
                * (trace_evals[620].values.at(i)
                    - ((trace_evals[618].values.at(i) * trace_evals[618].values.at(i))
                        + (trace_evals[619].values.at(i) * trace_evals[619].values.at(i))));
            *numer += accum.random_coeff_powers[377]
                * (trace_evals[621].values.at(i)
                    - ((trace_evals[619].values.at(i) * trace_evals[619].values.at(i))
                        + (trace_evals[620].values.at(i) * trace_evals[620].values.at(i))));
            *numer += accum.random_coeff_powers[376]
                * (trace_evals[622].values.at(i)
                    - ((trace_evals[620].values.at(i) * trace_evals[620].values.at(i))
                        + (trace_evals[621].values.at(i) * trace_evals[621].values.at(i))));
            *numer += accum.random_coeff_powers[375]
                * (trace_evals[623].values.at(i)
                    - ((trace_evals[621].values.at(i) * trace_evals[621].values.at(i))
                        + (trace_evals[622].values.at(i) * trace_evals[622].values.at(i))));
            *numer += accum.random_coeff_powers[374]
                * (trace_evals[624].values.at(i)
                    - ((trace_evals[622].values.at(i) * trace_evals[622].values.at(i))
                        + (trace_evals[623].values.at(i) * trace_evals[623].values.at(i))));
            *numer += accum.random_coeff_powers[373]
                * (trace_evals[625].values.at(i)
                    - ((trace_evals[623].values.at(i) * trace_evals[623].values.at(i))
                        + (trace_evals[624].values.at(i) * trace_evals[624].values.at(i))));
            *numer += accum.random_coeff_powers[372]
                * (trace_evals[626].values.at(i)
                    - ((trace_evals[624].values.at(i) * trace_evals[624].values.at(i))
                        + (trace_evals[625].values.at(i) * trace_evals[625].values.at(i))));
            *numer += accum.random_coeff_powers[371]
                * (trace_evals[627].values.at(i)
                    - ((trace_evals[625].values.at(i) * trace_evals[625].values.at(i))
                        + (trace_evals[626].values.at(i) * trace_evals[626].values.at(i))));
            *numer += accum.random_coeff_powers[370]
                * (trace_evals[628].values.at(i)
                    - ((trace_evals[626].values.at(i) * trace_evals[626].values.at(i))
                        + (trace_evals[627].values.at(i) * trace_evals[627].values.at(i))));
            *numer += accum.random_coeff_powers[369]
                * (trace_evals[629].values.at(i)
                    - ((trace_evals[627].values.at(i) * trace_evals[627].values.at(i))
                        + (trace_evals[628].values.at(i) * trace_evals[628].values.at(i))));
            *numer += accum.random_coeff_powers[368]
                * (trace_evals[630].values.at(i)
                    - ((trace_evals[628].values.at(i) * trace_evals[628].values.at(i))
                        + (trace_evals[629].values.at(i) * trace_evals[629].values.at(i))));
            *numer += accum.random_coeff_powers[367]
                * (trace_evals[631].values.at(i)
                    - ((trace_evals[629].values.at(i) * trace_evals[629].values.at(i))
                        + (trace_evals[630].values.at(i) * trace_evals[630].values.at(i))));
            *numer += accum.random_coeff_powers[366]
                * (trace_evals[632].values.at(i)
                    - ((trace_evals[630].values.at(i) * trace_evals[630].values.at(i))
                        + (trace_evals[631].values.at(i) * trace_evals[631].values.at(i))));
            *numer += accum.random_coeff_powers[365]
                * (trace_evals[633].values.at(i)
                    - ((trace_evals[631].values.at(i) * trace_evals[631].values.at(i))
                        + (trace_evals[632].values.at(i) * trace_evals[632].values.at(i))));
            *numer += accum.random_coeff_powers[364]
                * (trace_evals[634].values.at(i)
                    - ((trace_evals[632].values.at(i) * trace_evals[632].values.at(i))
                        + (trace_evals[633].values.at(i) * trace_evals[633].values.at(i))));
            *numer += accum.random_coeff_powers[363]
                * (trace_evals[635].values.at(i)
                    - ((trace_evals[633].values.at(i) * trace_evals[633].values.at(i))
                        + (trace_evals[634].values.at(i) * trace_evals[634].values.at(i))));
            *numer += accum.random_coeff_powers[362]
                * (trace_evals[636].values.at(i)
                    - ((trace_evals[634].values.at(i) * trace_evals[634].values.at(i))
                        + (trace_evals[635].values.at(i) * trace_evals[635].values.at(i))));
            *numer += accum.random_coeff_powers[361]
                * (trace_evals[637].values.at(i)
                    - ((trace_evals[635].values.at(i) * trace_evals[635].values.at(i))
                        + (trace_evals[636].values.at(i) * trace_evals[636].values.at(i))));
            *numer += accum.random_coeff_powers[360]
                * (trace_evals[638].values.at(i)
                    - ((trace_evals[636].values.at(i) * trace_evals[636].values.at(i))
                        + (trace_evals[637].values.at(i) * trace_evals[637].values.at(i))));
            *numer += accum.random_coeff_powers[359]
                * (trace_evals[639].values.at(i)
                    - ((trace_evals[637].values.at(i) * trace_evals[637].values.at(i))
                        + (trace_evals[638].values.at(i) * trace_evals[638].values.at(i))));
            *numer += accum.random_coeff_powers[358]
                * (trace_evals[640].values.at(i)
                    - ((trace_evals[638].values.at(i) * trace_evals[638].values.at(i))
                        + (trace_evals[639].values.at(i) * trace_evals[639].values.at(i))));
            *numer += accum.random_coeff_powers[357]
                * (trace_evals[641].values.at(i)
                    - ((trace_evals[639].values.at(i) * trace_evals[639].values.at(i))
                        + (trace_evals[640].values.at(i) * trace_evals[640].values.at(i))));
            *numer += accum.random_coeff_powers[356]
                * (trace_evals[642].values.at(i)
                    - ((trace_evals[640].values.at(i) * trace_evals[640].values.at(i))
                        + (trace_evals[641].values.at(i) * trace_evals[641].values.at(i))));
            *numer += accum.random_coeff_powers[355]
                * (trace_evals[643].values.at(i)
                    - ((trace_evals[641].values.at(i) * trace_evals[641].values.at(i))
                        + (trace_evals[642].values.at(i) * trace_evals[642].values.at(i))));
            *numer += accum.random_coeff_powers[354]
                * (trace_evals[644].values.at(i)
                    - ((trace_evals[642].values.at(i) * trace_evals[642].values.at(i))
                        + (trace_evals[643].values.at(i) * trace_evals[643].values.at(i))));
            *numer += accum.random_coeff_powers[353]
                * (trace_evals[645].values.at(i)
                    - ((trace_evals[643].values.at(i) * trace_evals[643].values.at(i))
                        + (trace_evals[644].values.at(i) * trace_evals[644].values.at(i))));
            *numer += accum.random_coeff_powers[352]
                * (trace_evals[646].values.at(i)
                    - ((trace_evals[644].values.at(i) * trace_evals[644].values.at(i))
                        + (trace_evals[645].values.at(i) * trace_evals[645].values.at(i))));
            *numer += accum.random_coeff_powers[351]
                * (trace_evals[647].values.at(i)
                    - ((trace_evals[645].values.at(i) * trace_evals[645].values.at(i))
                        + (trace_evals[646].values.at(i) * trace_evals[646].values.at(i))));
            *numer += accum.random_coeff_powers[350]
                * (trace_evals[648].values.at(i)
                    - ((trace_evals[646].values.at(i) * trace_evals[646].values.at(i))
                        + (trace_evals[647].values.at(i) * trace_evals[647].values.at(i))));
            *numer += accum.random_coeff_powers[349]
                * (trace_evals[649].values.at(i)
                    - ((trace_evals[647].values.at(i) * trace_evals[647].values.at(i))
                        + (trace_evals[648].values.at(i) * trace_evals[648].values.at(i))));
            *numer += accum.random_coeff_powers[348]
                * (trace_evals[650].values.at(i)
                    - ((trace_evals[648].values.at(i) * trace_evals[648].values.at(i))
                        + (trace_evals[649].values.at(i) * trace_evals[649].values.at(i))));
            *numer += accum.random_coeff_powers[347]
                * (trace_evals[651].values.at(i)
                    - ((trace_evals[649].values.at(i) * trace_evals[649].values.at(i))
                        + (trace_evals[650].values.at(i) * trace_evals[650].values.at(i))));
            *numer += accum.random_coeff_powers[346]
                * (trace_evals[652].values.at(i)
                    - ((trace_evals[650].values.at(i) * trace_evals[650].values.at(i))
                        + (trace_evals[651].values.at(i) * trace_evals[651].values.at(i))));
            *numer += accum.random_coeff_powers[345]
                * (trace_evals[653].values.at(i)
                    - ((trace_evals[651].values.at(i) * trace_evals[651].values.at(i))
                        + (trace_evals[652].values.at(i) * trace_evals[652].values.at(i))));
            *numer += accum.random_coeff_powers[344]
                * (trace_evals[654].values.at(i)
                    - ((trace_evals[652].values.at(i) * trace_evals[652].values.at(i))
                        + (trace_evals[653].values.at(i) * trace_evals[653].values.at(i))));
            *numer += accum.random_coeff_powers[343]
                * (trace_evals[655].values.at(i)
                    - ((trace_evals[653].values.at(i) * trace_evals[653].values.at(i))
                        + (trace_evals[654].values.at(i) * trace_evals[654].values.at(i))));
            *numer += accum.random_coeff_powers[342]
                * (trace_evals[656].values.at(i)
                    - ((trace_evals[654].values.at(i) * trace_evals[654].values.at(i))
                        + (trace_evals[655].values.at(i) * trace_evals[655].values.at(i))));
            *numer += accum.random_coeff_powers[341]
                * (trace_evals[657].values.at(i)
                    - ((trace_evals[655].values.at(i) * trace_evals[655].values.at(i))
                        + (trace_evals[656].values.at(i) * trace_evals[656].values.at(i))));
            *numer += accum.random_coeff_powers[340]
                * (trace_evals[658].values.at(i)
                    - ((trace_evals[656].values.at(i) * trace_evals[656].values.at(i))
                        + (trace_evals[657].values.at(i) * trace_evals[657].values.at(i))));
            *numer += accum.random_coeff_powers[339]
                * (trace_evals[659].values.at(i)
                    - ((trace_evals[657].values.at(i) * trace_evals[657].values.at(i))
                        + (trace_evals[658].values.at(i) * trace_evals[658].values.at(i))));
            *numer += accum.random_coeff_powers[338]
                * (trace_evals[660].values.at(i)
                    - ((trace_evals[658].values.at(i) * trace_evals[658].values.at(i))
                        + (trace_evals[659].values.at(i) * trace_evals[659].values.at(i))));
            *numer += accum.random_coeff_powers[337]
                * (trace_evals[661].values.at(i)
                    - ((trace_evals[659].values.at(i) * trace_evals[659].values.at(i))
                        + (trace_evals[660].values.at(i) * trace_evals[660].values.at(i))));
            *numer += accum.random_coeff_powers[336]
                * (trace_evals[662].values.at(i)
                    - ((trace_evals[660].values.at(i) * trace_evals[660].values.at(i))
                        + (trace_evals[661].values.at(i) * trace_evals[661].values.at(i))));
            *numer += accum.random_coeff_powers[335]
                * (trace_evals[663].values.at(i)
                    - ((trace_evals[661].values.at(i) * trace_evals[661].values.at(i))
                        + (trace_evals[662].values.at(i) * trace_evals[662].values.at(i))));
            *numer += accum.random_coeff_powers[334]
                * (trace_evals[664].values.at(i)
                    - ((trace_evals[662].values.at(i) * trace_evals[662].values.at(i))
                        + (trace_evals[663].values.at(i) * trace_evals[663].values.at(i))));
            *numer += accum.random_coeff_powers[333]
                * (trace_evals[665].values.at(i)
                    - ((trace_evals[663].values.at(i) * trace_evals[663].values.at(i))
                        + (trace_evals[664].values.at(i) * trace_evals[664].values.at(i))));
            *numer += accum.random_coeff_powers[332]
                * (trace_evals[666].values.at(i)
                    - ((trace_evals[664].values.at(i) * trace_evals[664].values.at(i))
                        + (trace_evals[665].values.at(i) * trace_evals[665].values.at(i))));
            *numer += accum.random_coeff_powers[331]
                * (trace_evals[667].values.at(i)
                    - ((trace_evals[665].values.at(i) * trace_evals[665].values.at(i))
                        + (trace_evals[666].values.at(i) * trace_evals[666].values.at(i))));
            *numer += accum.random_coeff_powers[330]
                * (trace_evals[668].values.at(i)
                    - ((trace_evals[666].values.at(i) * trace_evals[666].values.at(i))
                        + (trace_evals[667].values.at(i) * trace_evals[667].values.at(i))));
            *numer += accum.random_coeff_powers[329]
                * (trace_evals[669].values.at(i)
                    - ((trace_evals[667].values.at(i) * trace_evals[667].values.at(i))
                        + (trace_evals[668].values.at(i) * trace_evals[668].values.at(i))));
            *numer += accum.random_coeff_powers[328]
                * (trace_evals[670].values.at(i)
                    - ((trace_evals[668].values.at(i) * trace_evals[668].values.at(i))
                        + (trace_evals[669].values.at(i) * trace_evals[669].values.at(i))));
            *numer += accum.random_coeff_powers[327]
                * (trace_evals[671].values.at(i)
                    - ((trace_evals[669].values.at(i) * trace_evals[669].values.at(i))
                        + (trace_evals[670].values.at(i) * trace_evals[670].values.at(i))));
            *numer += accum.random_coeff_powers[326]
                * (trace_evals[672].values.at(i)
                    - ((trace_evals[670].values.at(i) * trace_evals[670].values.at(i))
                        + (trace_evals[671].values.at(i) * trace_evals[671].values.at(i))));
            *numer += accum.random_coeff_powers[325]
                * (trace_evals[673].values.at(i)
                    - ((trace_evals[671].values.at(i) * trace_evals[671].values.at(i))
                        + (trace_evals[672].values.at(i) * trace_evals[672].values.at(i))));
            *numer += accum.random_coeff_powers[324]
                * (trace_evals[674].values.at(i)
                    - ((trace_evals[672].values.at(i) * trace_evals[672].values.at(i))
                        + (trace_evals[673].values.at(i) * trace_evals[673].values.at(i))));
            *numer += accum.random_coeff_powers[323]
                * (trace_evals[675].values.at(i)
                    - ((trace_evals[673].values.at(i) * trace_evals[673].values.at(i))
                        + (trace_evals[674].values.at(i) * trace_evals[674].values.at(i))));
            *numer += accum.random_coeff_powers[322]
                * (trace_evals[676].values.at(i)
                    - ((trace_evals[674].values.at(i) * trace_evals[674].values.at(i))
                        + (trace_evals[675].values.at(i) * trace_evals[675].values.at(i))));
            *numer += accum.random_coeff_powers[321]
                * (trace_evals[677].values.at(i)
                    - ((trace_evals[675].values.at(i) * trace_evals[675].values.at(i))
                        + (trace_evals[676].values.at(i) * trace_evals[676].values.at(i))));
            *numer += accum.random_coeff_powers[320]
                * (trace_evals[678].values.at(i)
                    - ((trace_evals[676].values.at(i) * trace_evals[676].values.at(i))
                        + (trace_evals[677].values.at(i) * trace_evals[677].values.at(i))));
            *numer += accum.random_coeff_powers[319]
                * (trace_evals[679].values.at(i)
                    - ((trace_evals[677].values.at(i) * trace_evals[677].values.at(i))
                        + (trace_evals[678].values.at(i) * trace_evals[678].values.at(i))));
            *numer += accum.random_coeff_powers[318]
                * (trace_evals[680].values.at(i)
                    - ((trace_evals[678].values.at(i) * trace_evals[678].values.at(i))
                        + (trace_evals[679].values.at(i) * trace_evals[679].values.at(i))));
            *numer += accum.random_coeff_powers[317]
                * (trace_evals[681].values.at(i)
                    - ((trace_evals[679].values.at(i) * trace_evals[679].values.at(i))
                        + (trace_evals[680].values.at(i) * trace_evals[680].values.at(i))));
            *numer += accum.random_coeff_powers[316]
                * (trace_evals[682].values.at(i)
                    - ((trace_evals[680].values.at(i) * trace_evals[680].values.at(i))
                        + (trace_evals[681].values.at(i) * trace_evals[681].values.at(i))));
            *numer += accum.random_coeff_powers[315]
                * (trace_evals[683].values.at(i)
                    - ((trace_evals[681].values.at(i) * trace_evals[681].values.at(i))
                        + (trace_evals[682].values.at(i) * trace_evals[682].values.at(i))));
            *numer += accum.random_coeff_powers[314]
                * (trace_evals[684].values.at(i)
                    - ((trace_evals[682].values.at(i) * trace_evals[682].values.at(i))
                        + (trace_evals[683].values.at(i) * trace_evals[683].values.at(i))));
            *numer += accum.random_coeff_powers[313]
                * (trace_evals[685].values.at(i)
                    - ((trace_evals[683].values.at(i) * trace_evals[683].values.at(i))
                        + (trace_evals[684].values.at(i) * trace_evals[684].values.at(i))));
            *numer += accum.random_coeff_powers[312]
                * (trace_evals[686].values.at(i)
                    - ((trace_evals[684].values.at(i) * trace_evals[684].values.at(i))
                        + (trace_evals[685].values.at(i) * trace_evals[685].values.at(i))));
            *numer += accum.random_coeff_powers[311]
                * (trace_evals[687].values.at(i)
                    - ((trace_evals[685].values.at(i) * trace_evals[685].values.at(i))
                        + (trace_evals[686].values.at(i) * trace_evals[686].values.at(i))));
            *numer += accum.random_coeff_powers[310]
                * (trace_evals[688].values.at(i)
                    - ((trace_evals[686].values.at(i) * trace_evals[686].values.at(i))
                        + (trace_evals[687].values.at(i) * trace_evals[687].values.at(i))));
            *numer += accum.random_coeff_powers[309]
                * (trace_evals[689].values.at(i)
                    - ((trace_evals[687].values.at(i) * trace_evals[687].values.at(i))
                        + (trace_evals[688].values.at(i) * trace_evals[688].values.at(i))));
            *numer += accum.random_coeff_powers[308]
                * (trace_evals[690].values.at(i)
                    - ((trace_evals[688].values.at(i) * trace_evals[688].values.at(i))
                        + (trace_evals[689].values.at(i) * trace_evals[689].values.at(i))));
            *numer += accum.random_coeff_powers[307]
                * (trace_evals[691].values.at(i)
                    - ((trace_evals[689].values.at(i) * trace_evals[689].values.at(i))
                        + (trace_evals[690].values.at(i) * trace_evals[690].values.at(i))));
            *numer += accum.random_coeff_powers[306]
                * (trace_evals[692].values.at(i)
                    - ((trace_evals[690].values.at(i) * trace_evals[690].values.at(i))
                        + (trace_evals[691].values.at(i) * trace_evals[691].values.at(i))));
            *numer += accum.random_coeff_powers[305]
                * (trace_evals[693].values.at(i)
                    - ((trace_evals[691].values.at(i) * trace_evals[691].values.at(i))
                        + (trace_evals[692].values.at(i) * trace_evals[692].values.at(i))));
            *numer += accum.random_coeff_powers[304]
                * (trace_evals[694].values.at(i)
                    - ((trace_evals[692].values.at(i) * trace_evals[692].values.at(i))
                        + (trace_evals[693].values.at(i) * trace_evals[693].values.at(i))));
            *numer += accum.random_coeff_powers[303]
                * (trace_evals[695].values.at(i)
                    - ((trace_evals[693].values.at(i) * trace_evals[693].values.at(i))
                        + (trace_evals[694].values.at(i) * trace_evals[694].values.at(i))));
            *numer += accum.random_coeff_powers[302]
                * (trace_evals[696].values.at(i)
                    - ((trace_evals[694].values.at(i) * trace_evals[694].values.at(i))
                        + (trace_evals[695].values.at(i) * trace_evals[695].values.at(i))));
            *numer += accum.random_coeff_powers[301]
                * (trace_evals[697].values.at(i)
                    - ((trace_evals[695].values.at(i) * trace_evals[695].values.at(i))
                        + (trace_evals[696].values.at(i) * trace_evals[696].values.at(i))));
            *numer += accum.random_coeff_powers[300]
                * (trace_evals[698].values.at(i)
                    - ((trace_evals[696].values.at(i) * trace_evals[696].values.at(i))
                        + (trace_evals[697].values.at(i) * trace_evals[697].values.at(i))));
            *numer += accum.random_coeff_powers[299]
                * (trace_evals[699].values.at(i)
                    - ((trace_evals[697].values.at(i) * trace_evals[697].values.at(i))
                        + (trace_evals[698].values.at(i) * trace_evals[698].values.at(i))));
            *numer += accum.random_coeff_powers[298]
                * (trace_evals[700].values.at(i)
                    - ((trace_evals[698].values.at(i) * trace_evals[698].values.at(i))
                        + (trace_evals[699].values.at(i) * trace_evals[699].values.at(i))));
            *numer += accum.random_coeff_powers[297]
                * (trace_evals[701].values.at(i)
                    - ((trace_evals[699].values.at(i) * trace_evals[699].values.at(i))
                        + (trace_evals[700].values.at(i) * trace_evals[700].values.at(i))));
            *numer += accum.random_coeff_powers[296]
                * (trace_evals[702].values.at(i)
                    - ((trace_evals[700].values.at(i) * trace_evals[700].values.at(i))
                        + (trace_evals[701].values.at(i) * trace_evals[701].values.at(i))));
            *numer += accum.random_coeff_powers[295]
                * (trace_evals[703].values.at(i)
                    - ((trace_evals[701].values.at(i) * trace_evals[701].values.at(i))
                        + (trace_evals[702].values.at(i) * trace_evals[702].values.at(i))));
            *numer += accum.random_coeff_powers[294]
                * (trace_evals[704].values.at(i)
                    - ((trace_evals[702].values.at(i) * trace_evals[702].values.at(i))
                        + (trace_evals[703].values.at(i) * trace_evals[703].values.at(i))));
            *numer += accum.random_coeff_powers[293]
                * (trace_evals[705].values.at(i)
                    - ((trace_evals[703].values.at(i) * trace_evals[703].values.at(i))
                        + (trace_evals[704].values.at(i) * trace_evals[704].values.at(i))));
            *numer += accum.random_coeff_powers[292]
                * (trace_evals[706].values.at(i)
                    - ((trace_evals[704].values.at(i) * trace_evals[704].values.at(i))
                        + (trace_evals[705].values.at(i) * trace_evals[705].values.at(i))));
            *numer += accum.random_coeff_powers[291]
                * (trace_evals[707].values.at(i)
                    - ((trace_evals[705].values.at(i) * trace_evals[705].values.at(i))
                        + (trace_evals[706].values.at(i) * trace_evals[706].values.at(i))));
            *numer += accum.random_coeff_powers[290]
                * (trace_evals[708].values.at(i)
                    - ((trace_evals[706].values.at(i) * trace_evals[706].values.at(i))
                        + (trace_evals[707].values.at(i) * trace_evals[707].values.at(i))));
            *numer += accum.random_coeff_powers[289]
                * (trace_evals[709].values.at(i)
                    - ((trace_evals[707].values.at(i) * trace_evals[707].values.at(i))
                        + (trace_evals[708].values.at(i) * trace_evals[708].values.at(i))));
            *numer += accum.random_coeff_powers[288]
                * (trace_evals[710].values.at(i)
                    - ((trace_evals[708].values.at(i) * trace_evals[708].values.at(i))
                        + (trace_evals[709].values.at(i) * trace_evals[709].values.at(i))));
            *numer += accum.random_coeff_powers[287]
                * (trace_evals[711].values.at(i)
                    - ((trace_evals[709].values.at(i) * trace_evals[709].values.at(i))
                        + (trace_evals[710].values.at(i) * trace_evals[710].values.at(i))));
            *numer += accum.random_coeff_powers[286]
                * (trace_evals[712].values.at(i)
                    - ((trace_evals[710].values.at(i) * trace_evals[710].values.at(i))
                        + (trace_evals[711].values.at(i) * trace_evals[711].values.at(i))));
            *numer += accum.random_coeff_powers[285]
                * (trace_evals[713].values.at(i)
                    - ((trace_evals[711].values.at(i) * trace_evals[711].values.at(i))
                        + (trace_evals[712].values.at(i) * trace_evals[712].values.at(i))));
            *numer += accum.random_coeff_powers[284]
                * (trace_evals[714].values.at(i)
                    - ((trace_evals[712].values.at(i) * trace_evals[712].values.at(i))
                        + (trace_evals[713].values.at(i) * trace_evals[713].values.at(i))));
            *numer += accum.random_coeff_powers[283]
                * (trace_evals[715].values.at(i)
                    - ((trace_evals[713].values.at(i) * trace_evals[713].values.at(i))
                        + (trace_evals[714].values.at(i) * trace_evals[714].values.at(i))));
            *numer += accum.random_coeff_powers[282]
                * (trace_evals[716].values.at(i)
                    - ((trace_evals[714].values.at(i) * trace_evals[714].values.at(i))
                        + (trace_evals[715].values.at(i) * trace_evals[715].values.at(i))));
            *numer += accum.random_coeff_powers[281]
                * (trace_evals[717].values.at(i)
                    - ((trace_evals[715].values.at(i) * trace_evals[715].values.at(i))
                        + (trace_evals[716].values.at(i) * trace_evals[716].values.at(i))));
            *numer += accum.random_coeff_powers[280]
                * (trace_evals[718].values.at(i)
                    - ((trace_evals[716].values.at(i) * trace_evals[716].values.at(i))
                        + (trace_evals[717].values.at(i) * trace_evals[717].values.at(i))));
            *numer += accum.random_coeff_powers[279]
                * (trace_evals[719].values.at(i)
                    - ((trace_evals[717].values.at(i) * trace_evals[717].values.at(i))
                        + (trace_evals[718].values.at(i) * trace_evals[718].values.at(i))));
            *numer += accum.random_coeff_powers[278]
                * (trace_evals[720].values.at(i)
                    - ((trace_evals[718].values.at(i) * trace_evals[718].values.at(i))
                        + (trace_evals[719].values.at(i) * trace_evals[719].values.at(i))));
            *numer += accum.random_coeff_powers[277]
                * (trace_evals[721].values.at(i)
                    - ((trace_evals[719].values.at(i) * trace_evals[719].values.at(i))
                        + (trace_evals[720].values.at(i) * trace_evals[720].values.at(i))));
            *numer += accum.random_coeff_powers[276]
                * (trace_evals[722].values.at(i)
                    - ((trace_evals[720].values.at(i) * trace_evals[720].values.at(i))
                        + (trace_evals[721].values.at(i) * trace_evals[721].values.at(i))));
            *numer += accum.random_coeff_powers[275]
                * (trace_evals[723].values.at(i)
                    - ((trace_evals[721].values.at(i) * trace_evals[721].values.at(i))
                        + (trace_evals[722].values.at(i) * trace_evals[722].values.at(i))));
            *numer += accum.random_coeff_powers[274]
                * (trace_evals[724].values.at(i)
                    - ((trace_evals[722].values.at(i) * trace_evals[722].values.at(i))
                        + (trace_evals[723].values.at(i) * trace_evals[723].values.at(i))));
            *numer += accum.random_coeff_powers[273]
                * (trace_evals[725].values.at(i)
                    - ((trace_evals[723].values.at(i) * trace_evals[723].values.at(i))
                        + (trace_evals[724].values.at(i) * trace_evals[724].values.at(i))));
            *numer += accum.random_coeff_powers[272]
                * (trace_evals[726].values.at(i)
                    - ((trace_evals[724].values.at(i) * trace_evals[724].values.at(i))
                        + (trace_evals[725].values.at(i) * trace_evals[725].values.at(i))));
            *numer += accum.random_coeff_powers[271]
                * (trace_evals[727].values.at(i)
                    - ((trace_evals[725].values.at(i) * trace_evals[725].values.at(i))
                        + (trace_evals[726].values.at(i) * trace_evals[726].values.at(i))));
            *numer += accum.random_coeff_powers[270]
                * (trace_evals[728].values.at(i)
                    - ((trace_evals[726].values.at(i) * trace_evals[726].values.at(i))
                        + (trace_evals[727].values.at(i) * trace_evals[727].values.at(i))));
            *numer += accum.random_coeff_powers[269]
                * (trace_evals[729].values.at(i)
                    - ((trace_evals[727].values.at(i) * trace_evals[727].values.at(i))
                        + (trace_evals[728].values.at(i) * trace_evals[728].values.at(i))));
            *numer += accum.random_coeff_powers[268]
                * (trace_evals[730].values.at(i)
                    - ((trace_evals[728].values.at(i) * trace_evals[728].values.at(i))
                        + (trace_evals[729].values.at(i) * trace_evals[729].values.at(i))));
            *numer += accum.random_coeff_powers[267]
                * (trace_evals[731].values.at(i)
                    - ((trace_evals[729].values.at(i) * trace_evals[729].values.at(i))
                        + (trace_evals[730].values.at(i) * trace_evals[730].values.at(i))));
            *numer += accum.random_coeff_powers[266]
                * (trace_evals[732].values.at(i)
                    - ((trace_evals[730].values.at(i) * trace_evals[730].values.at(i))
                        + (trace_evals[731].values.at(i) * trace_evals[731].values.at(i))));
            *numer += accum.random_coeff_powers[265]
                * (trace_evals[733].values.at(i)
                    - ((trace_evals[731].values.at(i) * trace_evals[731].values.at(i))
                        + (trace_evals[732].values.at(i) * trace_evals[732].values.at(i))));
            *numer += accum.random_coeff_powers[264]
                * (trace_evals[734].values.at(i)
                    - ((trace_evals[732].values.at(i) * trace_evals[732].values.at(i))
                        + (trace_evals[733].values.at(i) * trace_evals[733].values.at(i))));
            *numer += accum.random_coeff_powers[263]
                * (trace_evals[735].values.at(i)
                    - ((trace_evals[733].values.at(i) * trace_evals[733].values.at(i))
                        + (trace_evals[734].values.at(i) * trace_evals[734].values.at(i))));
            *numer += accum.random_coeff_powers[262]
                * (trace_evals[736].values.at(i)
                    - ((trace_evals[734].values.at(i) * trace_evals[734].values.at(i))
                        + (trace_evals[735].values.at(i) * trace_evals[735].values.at(i))));
            *numer += accum.random_coeff_powers[261]
                * (trace_evals[737].values.at(i)
                    - ((trace_evals[735].values.at(i) * trace_evals[735].values.at(i))
                        + (trace_evals[736].values.at(i) * trace_evals[736].values.at(i))));
            *numer += accum.random_coeff_powers[260]
                * (trace_evals[738].values.at(i)
                    - ((trace_evals[736].values.at(i) * trace_evals[736].values.at(i))
                        + (trace_evals[737].values.at(i) * trace_evals[737].values.at(i))));
            *numer += accum.random_coeff_powers[259]
                * (trace_evals[739].values.at(i)
                    - ((trace_evals[737].values.at(i) * trace_evals[737].values.at(i))
                        + (trace_evals[738].values.at(i) * trace_evals[738].values.at(i))));
            *numer += accum.random_coeff_powers[258]
                * (trace_evals[740].values.at(i)
                    - ((trace_evals[738].values.at(i) * trace_evals[738].values.at(i))
                        + (trace_evals[739].values.at(i) * trace_evals[739].values.at(i))));
            *numer += accum.random_coeff_powers[257]
                * (trace_evals[741].values.at(i)
                    - ((trace_evals[739].values.at(i) * trace_evals[739].values.at(i))
                        + (trace_evals[740].values.at(i) * trace_evals[740].values.at(i))));
            *numer += accum.random_coeff_powers[256]
                * (trace_evals[742].values.at(i)
                    - ((trace_evals[740].values.at(i) * trace_evals[740].values.at(i))
                        + (trace_evals[741].values.at(i) * trace_evals[741].values.at(i))));
            *numer += accum.random_coeff_powers[255]
                * (trace_evals[743].values.at(i)
                    - ((trace_evals[741].values.at(i) * trace_evals[741].values.at(i))
                        + (trace_evals[742].values.at(i) * trace_evals[742].values.at(i))));
            *numer += accum.random_coeff_powers[254]
                * (trace_evals[744].values.at(i)
                    - ((trace_evals[742].values.at(i) * trace_evals[742].values.at(i))
                        + (trace_evals[743].values.at(i) * trace_evals[743].values.at(i))));
            *numer += accum.random_coeff_powers[253]
                * (trace_evals[745].values.at(i)
                    - ((trace_evals[743].values.at(i) * trace_evals[743].values.at(i))
                        + (trace_evals[744].values.at(i) * trace_evals[744].values.at(i))));
            *numer += accum.random_coeff_powers[252]
                * (trace_evals[746].values.at(i)
                    - ((trace_evals[744].values.at(i) * trace_evals[744].values.at(i))
                        + (trace_evals[745].values.at(i) * trace_evals[745].values.at(i))));
            *numer += accum.random_coeff_powers[251]
                * (trace_evals[747].values.at(i)
                    - ((trace_evals[745].values.at(i) * trace_evals[745].values.at(i))
                        + (trace_evals[746].values.at(i) * trace_evals[746].values.at(i))));
            *numer += accum.random_coeff_powers[250]
                * (trace_evals[748].values.at(i)
                    - ((trace_evals[746].values.at(i) * trace_evals[746].values.at(i))
                        + (trace_evals[747].values.at(i) * trace_evals[747].values.at(i))));
            *numer += accum.random_coeff_powers[249]
                * (trace_evals[749].values.at(i)
                    - ((trace_evals[747].values.at(i) * trace_evals[747].values.at(i))
                        + (trace_evals[748].values.at(i) * trace_evals[748].values.at(i))));
            *numer += accum.random_coeff_powers[248]
                * (trace_evals[750].values.at(i)
                    - ((trace_evals[748].values.at(i) * trace_evals[748].values.at(i))
                        + (trace_evals[749].values.at(i) * trace_evals[749].values.at(i))));
            *numer += accum.random_coeff_powers[247]
                * (trace_evals[751].values.at(i)
                    - ((trace_evals[749].values.at(i) * trace_evals[749].values.at(i))
                        + (trace_evals[750].values.at(i) * trace_evals[750].values.at(i))));
            *numer += accum.random_coeff_powers[246]
                * (trace_evals[752].values.at(i)
                    - ((trace_evals[750].values.at(i) * trace_evals[750].values.at(i))
                        + (trace_evals[751].values.at(i) * trace_evals[751].values.at(i))));
            *numer += accum.random_coeff_powers[245]
                * (trace_evals[753].values.at(i)
                    - ((trace_evals[751].values.at(i) * trace_evals[751].values.at(i))
                        + (trace_evals[752].values.at(i) * trace_evals[752].values.at(i))));
            *numer += accum.random_coeff_powers[244]
                * (trace_evals[754].values.at(i)
                    - ((trace_evals[752].values.at(i) * trace_evals[752].values.at(i))
                        + (trace_evals[753].values.at(i) * trace_evals[753].values.at(i))));
            *numer += accum.random_coeff_powers[243]
                * (trace_evals[755].values.at(i)
                    - ((trace_evals[753].values.at(i) * trace_evals[753].values.at(i))
                        + (trace_evals[754].values.at(i) * trace_evals[754].values.at(i))));
            *numer += accum.random_coeff_powers[242]
                * (trace_evals[756].values.at(i)
                    - ((trace_evals[754].values.at(i) * trace_evals[754].values.at(i))
                        + (trace_evals[755].values.at(i) * trace_evals[755].values.at(i))));
            *numer += accum.random_coeff_powers[241]
                * (trace_evals[757].values.at(i)
                    - ((trace_evals[755].values.at(i) * trace_evals[755].values.at(i))
                        + (trace_evals[756].values.at(i) * trace_evals[756].values.at(i))));
            *numer += accum.random_coeff_powers[240]
                * (trace_evals[758].values.at(i)
                    - ((trace_evals[756].values.at(i) * trace_evals[756].values.at(i))
                        + (trace_evals[757].values.at(i) * trace_evals[757].values.at(i))));
            *numer += accum.random_coeff_powers[239]
                * (trace_evals[759].values.at(i)
                    - ((trace_evals[757].values.at(i) * trace_evals[757].values.at(i))
                        + (trace_evals[758].values.at(i) * trace_evals[758].values.at(i))));
            *numer += accum.random_coeff_powers[238]
                * (trace_evals[760].values.at(i)
                    - ((trace_evals[758].values.at(i) * trace_evals[758].values.at(i))
                        + (trace_evals[759].values.at(i) * trace_evals[759].values.at(i))));
            *numer += accum.random_coeff_powers[237]
                * (trace_evals[761].values.at(i)
                    - ((trace_evals[759].values.at(i) * trace_evals[759].values.at(i))
                        + (trace_evals[760].values.at(i) * trace_evals[760].values.at(i))));
            *numer += accum.random_coeff_powers[236]
                * (trace_evals[762].values.at(i)
                    - ((trace_evals[760].values.at(i) * trace_evals[760].values.at(i))
                        + (trace_evals[761].values.at(i) * trace_evals[761].values.at(i))));
            *numer += accum.random_coeff_powers[235]
                * (trace_evals[763].values.at(i)
                    - ((trace_evals[761].values.at(i) * trace_evals[761].values.at(i))
                        + (trace_evals[762].values.at(i) * trace_evals[762].values.at(i))));
            *numer += accum.random_coeff_powers[234]
                * (trace_evals[764].values.at(i)
                    - ((trace_evals[762].values.at(i) * trace_evals[762].values.at(i))
                        + (trace_evals[763].values.at(i) * trace_evals[763].values.at(i))));
            *numer += accum.random_coeff_powers[233]
                * (trace_evals[765].values.at(i)
                    - ((trace_evals[763].values.at(i) * trace_evals[763].values.at(i))
                        + (trace_evals[764].values.at(i) * trace_evals[764].values.at(i))));
            *numer += accum.random_coeff_powers[232]
                * (trace_evals[766].values.at(i)
                    - ((trace_evals[764].values.at(i) * trace_evals[764].values.at(i))
                        + (trace_evals[765].values.at(i) * trace_evals[765].values.at(i))));
            *numer += accum.random_coeff_powers[231]
                * (trace_evals[767].values.at(i)
                    - ((trace_evals[765].values.at(i) * trace_evals[765].values.at(i))
                        + (trace_evals[766].values.at(i) * trace_evals[766].values.at(i))));
            *numer += accum.random_coeff_powers[230]
                * (trace_evals[768].values.at(i)
                    - ((trace_evals[766].values.at(i) * trace_evals[766].values.at(i))
                        + (trace_evals[767].values.at(i) * trace_evals[767].values.at(i))));
            *numer += accum.random_coeff_powers[229]
                * (trace_evals[769].values.at(i)
                    - ((trace_evals[767].values.at(i) * trace_evals[767].values.at(i))
                        + (trace_evals[768].values.at(i) * trace_evals[768].values.at(i))));
            *numer += accum.random_coeff_powers[228]
                * (trace_evals[770].values.at(i)
                    - ((trace_evals[768].values.at(i) * trace_evals[768].values.at(i))
                        + (trace_evals[769].values.at(i) * trace_evals[769].values.at(i))));
            *numer += accum.random_coeff_powers[227]
                * (trace_evals[771].values.at(i)
                    - ((trace_evals[769].values.at(i) * trace_evals[769].values.at(i))
                        + (trace_evals[770].values.at(i) * trace_evals[770].values.at(i))));
            *numer += accum.random_coeff_powers[226]
                * (trace_evals[772].values.at(i)
                    - ((trace_evals[770].values.at(i) * trace_evals[770].values.at(i))
                        + (trace_evals[771].values.at(i) * trace_evals[771].values.at(i))));
            *numer += accum.random_coeff_powers[225]
                * (trace_evals[773].values.at(i)
                    - ((trace_evals[771].values.at(i) * trace_evals[771].values.at(i))
                        + (trace_evals[772].values.at(i) * trace_evals[772].values.at(i))));
            *numer += accum.random_coeff_powers[224]
                * (trace_evals[774].values.at(i)
                    - ((trace_evals[772].values.at(i) * trace_evals[772].values.at(i))
                        + (trace_evals[773].values.at(i) * trace_evals[773].values.at(i))));
            *numer += accum.random_coeff_powers[223]
                * (trace_evals[775].values.at(i)
                    - ((trace_evals[773].values.at(i) * trace_evals[773].values.at(i))
                        + (trace_evals[774].values.at(i) * trace_evals[774].values.at(i))));
            *numer += accum.random_coeff_powers[222]
                * (trace_evals[776].values.at(i)
                    - ((trace_evals[774].values.at(i) * trace_evals[774].values.at(i))
                        + (trace_evals[775].values.at(i) * trace_evals[775].values.at(i))));
            *numer += accum.random_coeff_powers[221]
                * (trace_evals[777].values.at(i)
                    - ((trace_evals[775].values.at(i) * trace_evals[775].values.at(i))
                        + (trace_evals[776].values.at(i) * trace_evals[776].values.at(i))));
            *numer += accum.random_coeff_powers[220]
                * (trace_evals[778].values.at(i)
                    - ((trace_evals[776].values.at(i) * trace_evals[776].values.at(i))
                        + (trace_evals[777].values.at(i) * trace_evals[777].values.at(i))));
            *numer += accum.random_coeff_powers[219]
                * (trace_evals[779].values.at(i)
                    - ((trace_evals[777].values.at(i) * trace_evals[777].values.at(i))
                        + (trace_evals[778].values.at(i) * trace_evals[778].values.at(i))));
            *numer += accum.random_coeff_powers[218]
                * (trace_evals[780].values.at(i)
                    - ((trace_evals[778].values.at(i) * trace_evals[778].values.at(i))
                        + (trace_evals[779].values.at(i) * trace_evals[779].values.at(i))));
            *numer += accum.random_coeff_powers[217]
                * (trace_evals[781].values.at(i)
                    - ((trace_evals[779].values.at(i) * trace_evals[779].values.at(i))
                        + (trace_evals[780].values.at(i) * trace_evals[780].values.at(i))));
            *numer += accum.random_coeff_powers[216]
                * (trace_evals[782].values.at(i)
                    - ((trace_evals[780].values.at(i) * trace_evals[780].values.at(i))
                        + (trace_evals[781].values.at(i) * trace_evals[781].values.at(i))));
            *numer += accum.random_coeff_powers[215]
                * (trace_evals[783].values.at(i)
                    - ((trace_evals[781].values.at(i) * trace_evals[781].values.at(i))
                        + (trace_evals[782].values.at(i) * trace_evals[782].values.at(i))));
            *numer += accum.random_coeff_powers[214]
                * (trace_evals[784].values.at(i)
                    - ((trace_evals[782].values.at(i) * trace_evals[782].values.at(i))
                        + (trace_evals[783].values.at(i) * trace_evals[783].values.at(i))));
            *numer += accum.random_coeff_powers[213]
                * (trace_evals[785].values.at(i)
                    - ((trace_evals[783].values.at(i) * trace_evals[783].values.at(i))
                        + (trace_evals[784].values.at(i) * trace_evals[784].values.at(i))));
            *numer += accum.random_coeff_powers[212]
                * (trace_evals[786].values.at(i)
                    - ((trace_evals[784].values.at(i) * trace_evals[784].values.at(i))
                        + (trace_evals[785].values.at(i) * trace_evals[785].values.at(i))));
            *numer += accum.random_coeff_powers[211]
                * (trace_evals[787].values.at(i)
                    - ((trace_evals[785].values.at(i) * trace_evals[785].values.at(i))
                        + (trace_evals[786].values.at(i) * trace_evals[786].values.at(i))));
            *numer += accum.random_coeff_powers[210]
                * (trace_evals[788].values.at(i)
                    - ((trace_evals[786].values.at(i) * trace_evals[786].values.at(i))
                        + (trace_evals[787].values.at(i) * trace_evals[787].values.at(i))));
            *numer += accum.random_coeff_powers[209]
                * (trace_evals[789].values.at(i)
                    - ((trace_evals[787].values.at(i) * trace_evals[787].values.at(i))
                        + (trace_evals[788].values.at(i) * trace_evals[788].values.at(i))));
            *numer += accum.random_coeff_powers[208]
                * (trace_evals[790].values.at(i)
                    - ((trace_evals[788].values.at(i) * trace_evals[788].values.at(i))
                        + (trace_evals[789].values.at(i) * trace_evals[789].values.at(i))));
            *numer += accum.random_coeff_powers[207]
                * (trace_evals[791].values.at(i)
                    - ((trace_evals[789].values.at(i) * trace_evals[789].values.at(i))
                        + (trace_evals[790].values.at(i) * trace_evals[790].values.at(i))));
            *numer += accum.random_coeff_powers[206]
                * (trace_evals[792].values.at(i)
                    - ((trace_evals[790].values.at(i) * trace_evals[790].values.at(i))
                        + (trace_evals[791].values.at(i) * trace_evals[791].values.at(i))));
            *numer += accum.random_coeff_powers[205]
                * (trace_evals[793].values.at(i)
                    - ((trace_evals[791].values.at(i) * trace_evals[791].values.at(i))
                        + (trace_evals[792].values.at(i) * trace_evals[792].values.at(i))));
            *numer += accum.random_coeff_powers[204]
                * (trace_evals[794].values.at(i)
                    - ((trace_evals[792].values.at(i) * trace_evals[792].values.at(i))
                        + (trace_evals[793].values.at(i) * trace_evals[793].values.at(i))));
            *numer += accum.random_coeff_powers[203]
                * (trace_evals[795].values.at(i)
                    - ((trace_evals[793].values.at(i) * trace_evals[793].values.at(i))
                        + (trace_evals[794].values.at(i) * trace_evals[794].values.at(i))));
            *numer += accum.random_coeff_powers[202]
                * (trace_evals[796].values.at(i)
                    - ((trace_evals[794].values.at(i) * trace_evals[794].values.at(i))
                        + (trace_evals[795].values.at(i) * trace_evals[795].values.at(i))));
            *numer += accum.random_coeff_powers[201]
                * (trace_evals[797].values.at(i)
                    - ((trace_evals[795].values.at(i) * trace_evals[795].values.at(i))
                        + (trace_evals[796].values.at(i) * trace_evals[796].values.at(i))));
            *numer += accum.random_coeff_powers[200]
                * (trace_evals[798].values.at(i)
                    - ((trace_evals[796].values.at(i) * trace_evals[796].values.at(i))
                        + (trace_evals[797].values.at(i) * trace_evals[797].values.at(i))));
            *numer += accum.random_coeff_powers[199]
                * (trace_evals[799].values.at(i)
                    - ((trace_evals[797].values.at(i) * trace_evals[797].values.at(i))
                        + (trace_evals[798].values.at(i) * trace_evals[798].values.at(i))));
            *numer += accum.random_coeff_powers[198]
                * (trace_evals[800].values.at(i)
                    - ((trace_evals[798].values.at(i) * trace_evals[798].values.at(i))
                        + (trace_evals[799].values.at(i) * trace_evals[799].values.at(i))));
            *numer += accum.random_coeff_powers[197]
                * (trace_evals[801].values.at(i)
                    - ((trace_evals[799].values.at(i) * trace_evals[799].values.at(i))
                        + (trace_evals[800].values.at(i) * trace_evals[800].values.at(i))));
            *numer += accum.random_coeff_powers[196]
                * (trace_evals[802].values.at(i)
                    - ((trace_evals[800].values.at(i) * trace_evals[800].values.at(i))
                        + (trace_evals[801].values.at(i) * trace_evals[801].values.at(i))));
            *numer += accum.random_coeff_powers[195]
                * (trace_evals[803].values.at(i)
                    - ((trace_evals[801].values.at(i) * trace_evals[801].values.at(i))
                        + (trace_evals[802].values.at(i) * trace_evals[802].values.at(i))));
            *numer += accum.random_coeff_powers[194]
                * (trace_evals[804].values.at(i)
                    - ((trace_evals[802].values.at(i) * trace_evals[802].values.at(i))
                        + (trace_evals[803].values.at(i) * trace_evals[803].values.at(i))));
            *numer += accum.random_coeff_powers[193]
                * (trace_evals[805].values.at(i)
                    - ((trace_evals[803].values.at(i) * trace_evals[803].values.at(i))
                        + (trace_evals[804].values.at(i) * trace_evals[804].values.at(i))));
            *numer += accum.random_coeff_powers[192]
                * (trace_evals[806].values.at(i)
                    - ((trace_evals[804].values.at(i) * trace_evals[804].values.at(i))
                        + (trace_evals[805].values.at(i) * trace_evals[805].values.at(i))));
            *numer += accum.random_coeff_powers[191]
                * (trace_evals[807].values.at(i)
                    - ((trace_evals[805].values.at(i) * trace_evals[805].values.at(i))
                        + (trace_evals[806].values.at(i) * trace_evals[806].values.at(i))));
            *numer += accum.random_coeff_powers[190]
                * (trace_evals[808].values.at(i)
                    - ((trace_evals[806].values.at(i) * trace_evals[806].values.at(i))
                        + (trace_evals[807].values.at(i) * trace_evals[807].values.at(i))));
            *numer += accum.random_coeff_powers[189]
                * (trace_evals[809].values.at(i)
                    - ((trace_evals[807].values.at(i) * trace_evals[807].values.at(i))
                        + (trace_evals[808].values.at(i) * trace_evals[808].values.at(i))));
            *numer += accum.random_coeff_powers[188]
                * (trace_evals[810].values.at(i)
                    - ((trace_evals[808].values.at(i) * trace_evals[808].values.at(i))
                        + (trace_evals[809].values.at(i) * trace_evals[809].values.at(i))));
            *numer += accum.random_coeff_powers[187]
                * (trace_evals[811].values.at(i)
                    - ((trace_evals[809].values.at(i) * trace_evals[809].values.at(i))
                        + (trace_evals[810].values.at(i) * trace_evals[810].values.at(i))));
            *numer += accum.random_coeff_powers[186]
                * (trace_evals[812].values.at(i)
                    - ((trace_evals[810].values.at(i) * trace_evals[810].values.at(i))
                        + (trace_evals[811].values.at(i) * trace_evals[811].values.at(i))));
            *numer += accum.random_coeff_powers[185]
                * (trace_evals[813].values.at(i)
                    - ((trace_evals[811].values.at(i) * trace_evals[811].values.at(i))
                        + (trace_evals[812].values.at(i) * trace_evals[812].values.at(i))));
            *numer += accum.random_coeff_powers[184]
                * (trace_evals[814].values.at(i)
                    - ((trace_evals[812].values.at(i) * trace_evals[812].values.at(i))
                        + (trace_evals[813].values.at(i) * trace_evals[813].values.at(i))));
            *numer += accum.random_coeff_powers[183]
                * (trace_evals[815].values.at(i)
                    - ((trace_evals[813].values.at(i) * trace_evals[813].values.at(i))
                        + (trace_evals[814].values.at(i) * trace_evals[814].values.at(i))));
            *numer += accum.random_coeff_powers[182]
                * (trace_evals[816].values.at(i)
                    - ((trace_evals[814].values.at(i) * trace_evals[814].values.at(i))
                        + (trace_evals[815].values.at(i) * trace_evals[815].values.at(i))));
            *numer += accum.random_coeff_powers[181]
                * (trace_evals[817].values.at(i)
                    - ((trace_evals[815].values.at(i) * trace_evals[815].values.at(i))
                        + (trace_evals[816].values.at(i) * trace_evals[816].values.at(i))));
            *numer += accum.random_coeff_powers[180]
                * (trace_evals[818].values.at(i)
                    - ((trace_evals[816].values.at(i) * trace_evals[816].values.at(i))
                        + (trace_evals[817].values.at(i) * trace_evals[817].values.at(i))));
            *numer += accum.random_coeff_powers[179]
                * (trace_evals[819].values.at(i)
                    - ((trace_evals[817].values.at(i) * trace_evals[817].values.at(i))
                        + (trace_evals[818].values.at(i) * trace_evals[818].values.at(i))));
            *numer += accum.random_coeff_powers[178]
                * (trace_evals[820].values.at(i)
                    - ((trace_evals[818].values.at(i) * trace_evals[818].values.at(i))
                        + (trace_evals[819].values.at(i) * trace_evals[819].values.at(i))));
            *numer += accum.random_coeff_powers[177]
                * (trace_evals[821].values.at(i)
                    - ((trace_evals[819].values.at(i) * trace_evals[819].values.at(i))
                        + (trace_evals[820].values.at(i) * trace_evals[820].values.at(i))));
            *numer += accum.random_coeff_powers[176]
                * (trace_evals[822].values.at(i)
                    - ((trace_evals[820].values.at(i) * trace_evals[820].values.at(i))
                        + (trace_evals[821].values.at(i) * trace_evals[821].values.at(i))));
            *numer += accum.random_coeff_powers[175]
                * (trace_evals[823].values.at(i)
                    - ((trace_evals[821].values.at(i) * trace_evals[821].values.at(i))
                        + (trace_evals[822].values.at(i) * trace_evals[822].values.at(i))));
            *numer += accum.random_coeff_powers[174]
                * (trace_evals[824].values.at(i)
                    - ((trace_evals[822].values.at(i) * trace_evals[822].values.at(i))
                        + (trace_evals[823].values.at(i) * trace_evals[823].values.at(i))));
            *numer += accum.random_coeff_powers[173]
                * (trace_evals[825].values.at(i)
                    - ((trace_evals[823].values.at(i) * trace_evals[823].values.at(i))
                        + (trace_evals[824].values.at(i) * trace_evals[824].values.at(i))));
            *numer += accum.random_coeff_powers[172]
                * (trace_evals[826].values.at(i)
                    - ((trace_evals[824].values.at(i) * trace_evals[824].values.at(i))
                        + (trace_evals[825].values.at(i) * trace_evals[825].values.at(i))));
            *numer += accum.random_coeff_powers[171]
                * (trace_evals[827].values.at(i)
                    - ((trace_evals[825].values.at(i) * trace_evals[825].values.at(i))
                        + (trace_evals[826].values.at(i) * trace_evals[826].values.at(i))));
            *numer += accum.random_coeff_powers[170]
                * (trace_evals[828].values.at(i)
                    - ((trace_evals[826].values.at(i) * trace_evals[826].values.at(i))
                        + (trace_evals[827].values.at(i) * trace_evals[827].values.at(i))));
            *numer += accum.random_coeff_powers[169]
                * (trace_evals[829].values.at(i)
                    - ((trace_evals[827].values.at(i) * trace_evals[827].values.at(i))
                        + (trace_evals[828].values.at(i) * trace_evals[828].values.at(i))));
            *numer += accum.random_coeff_powers[168]
                * (trace_evals[830].values.at(i)
                    - ((trace_evals[828].values.at(i) * trace_evals[828].values.at(i))
                        + (trace_evals[829].values.at(i) * trace_evals[829].values.at(i))));
            *numer += accum.random_coeff_powers[167]
                * (trace_evals[831].values.at(i)
                    - ((trace_evals[829].values.at(i) * trace_evals[829].values.at(i))
                        + (trace_evals[830].values.at(i) * trace_evals[830].values.at(i))));
            *numer += accum.random_coeff_powers[166]
                * (trace_evals[832].values.at(i)
                    - ((trace_evals[830].values.at(i) * trace_evals[830].values.at(i))
                        + (trace_evals[831].values.at(i) * trace_evals[831].values.at(i))));
            *numer += accum.random_coeff_powers[165]
                * (trace_evals[833].values.at(i)
                    - ((trace_evals[831].values.at(i) * trace_evals[831].values.at(i))
                        + (trace_evals[832].values.at(i) * trace_evals[832].values.at(i))));
            *numer += accum.random_coeff_powers[164]
                * (trace_evals[834].values.at(i)
                    - ((trace_evals[832].values.at(i) * trace_evals[832].values.at(i))
                        + (trace_evals[833].values.at(i) * trace_evals[833].values.at(i))));
            *numer += accum.random_coeff_powers[163]
                * (trace_evals[835].values.at(i)
                    - ((trace_evals[833].values.at(i) * trace_evals[833].values.at(i))
                        + (trace_evals[834].values.at(i) * trace_evals[834].values.at(i))));
            *numer += accum.random_coeff_powers[162]
                * (trace_evals[836].values.at(i)
                    - ((trace_evals[834].values.at(i) * trace_evals[834].values.at(i))
                        + (trace_evals[835].values.at(i) * trace_evals[835].values.at(i))));
            *numer += accum.random_coeff_powers[161]
                * (trace_evals[837].values.at(i)
                    - ((trace_evals[835].values.at(i) * trace_evals[835].values.at(i))
                        + (trace_evals[836].values.at(i) * trace_evals[836].values.at(i))));
            *numer += accum.random_coeff_powers[160]
                * (trace_evals[838].values.at(i)
                    - ((trace_evals[836].values.at(i) * trace_evals[836].values.at(i))
                        + (trace_evals[837].values.at(i) * trace_evals[837].values.at(i))));
            *numer += accum.random_coeff_powers[159]
                * (trace_evals[839].values.at(i)
                    - ((trace_evals[837].values.at(i) * trace_evals[837].values.at(i))
                        + (trace_evals[838].values.at(i) * trace_evals[838].values.at(i))));
            *numer += accum.random_coeff_powers[158]
                * (trace_evals[840].values.at(i)
                    - ((trace_evals[838].values.at(i) * trace_evals[838].values.at(i))
                        + (trace_evals[839].values.at(i) * trace_evals[839].values.at(i))));
            *numer += accum.random_coeff_powers[157]
                * (trace_evals[841].values.at(i)
                    - ((trace_evals[839].values.at(i) * trace_evals[839].values.at(i))
                        + (trace_evals[840].values.at(i) * trace_evals[840].values.at(i))));
            *numer += accum.random_coeff_powers[156]
                * (trace_evals[842].values.at(i)
                    - ((trace_evals[840].values.at(i) * trace_evals[840].values.at(i))
                        + (trace_evals[841].values.at(i) * trace_evals[841].values.at(i))));
            *numer += accum.random_coeff_powers[155]
                * (trace_evals[843].values.at(i)
                    - ((trace_evals[841].values.at(i) * trace_evals[841].values.at(i))
                        + (trace_evals[842].values.at(i) * trace_evals[842].values.at(i))));
            *numer += accum.random_coeff_powers[154]
                * (trace_evals[844].values.at(i)
                    - ((trace_evals[842].values.at(i) * trace_evals[842].values.at(i))
                        + (trace_evals[843].values.at(i) * trace_evals[843].values.at(i))));
            *numer += accum.random_coeff_powers[153]
                * (trace_evals[845].values.at(i)
                    - ((trace_evals[843].values.at(i) * trace_evals[843].values.at(i))
                        + (trace_evals[844].values.at(i) * trace_evals[844].values.at(i))));
            *numer += accum.random_coeff_powers[152]
                * (trace_evals[846].values.at(i)
                    - ((trace_evals[844].values.at(i) * trace_evals[844].values.at(i))
                        + (trace_evals[845].values.at(i) * trace_evals[845].values.at(i))));
            *numer += accum.random_coeff_powers[151]
                * (trace_evals[847].values.at(i)
                    - ((trace_evals[845].values.at(i) * trace_evals[845].values.at(i))
                        + (trace_evals[846].values.at(i) * trace_evals[846].values.at(i))));
            *numer += accum.random_coeff_powers[150]
                * (trace_evals[848].values.at(i)
                    - ((trace_evals[846].values.at(i) * trace_evals[846].values.at(i))
                        + (trace_evals[847].values.at(i) * trace_evals[847].values.at(i))));
            *numer += accum.random_coeff_powers[149]
                * (trace_evals[849].values.at(i)
                    - ((trace_evals[847].values.at(i) * trace_evals[847].values.at(i))
                        + (trace_evals[848].values.at(i) * trace_evals[848].values.at(i))));
            *numer += accum.random_coeff_powers[148]
                * (trace_evals[850].values.at(i)
                    - ((trace_evals[848].values.at(i) * trace_evals[848].values.at(i))
                        + (trace_evals[849].values.at(i) * trace_evals[849].values.at(i))));
            *numer += accum.random_coeff_powers[147]
                * (trace_evals[851].values.at(i)
                    - ((trace_evals[849].values.at(i) * trace_evals[849].values.at(i))
                        + (trace_evals[850].values.at(i) * trace_evals[850].values.at(i))));
            *numer += accum.random_coeff_powers[146]
                * (trace_evals[852].values.at(i)
                    - ((trace_evals[850].values.at(i) * trace_evals[850].values.at(i))
                        + (trace_evals[851].values.at(i) * trace_evals[851].values.at(i))));
            *numer += accum.random_coeff_powers[145]
                * (trace_evals[853].values.at(i)
                    - ((trace_evals[851].values.at(i) * trace_evals[851].values.at(i))
                        + (trace_evals[852].values.at(i) * trace_evals[852].values.at(i))));
            *numer += accum.random_coeff_powers[144]
                * (trace_evals[854].values.at(i)
                    - ((trace_evals[852].values.at(i) * trace_evals[852].values.at(i))
                        + (trace_evals[853].values.at(i) * trace_evals[853].values.at(i))));
            *numer += accum.random_coeff_powers[143]
                * (trace_evals[855].values.at(i)
                    - ((trace_evals[853].values.at(i) * trace_evals[853].values.at(i))
                        + (trace_evals[854].values.at(i) * trace_evals[854].values.at(i))));
            *numer += accum.random_coeff_powers[142]
                * (trace_evals[856].values.at(i)
                    - ((trace_evals[854].values.at(i) * trace_evals[854].values.at(i))
                        + (trace_evals[855].values.at(i) * trace_evals[855].values.at(i))));
            *numer += accum.random_coeff_powers[141]
                * (trace_evals[857].values.at(i)
                    - ((trace_evals[855].values.at(i) * trace_evals[855].values.at(i))
                        + (trace_evals[856].values.at(i) * trace_evals[856].values.at(i))));
            *numer += accum.random_coeff_powers[140]
                * (trace_evals[858].values.at(i)
                    - ((trace_evals[856].values.at(i) * trace_evals[856].values.at(i))
                        + (trace_evals[857].values.at(i) * trace_evals[857].values.at(i))));
            *numer += accum.random_coeff_powers[139]
                * (trace_evals[859].values.at(i)
                    - ((trace_evals[857].values.at(i) * trace_evals[857].values.at(i))
                        + (trace_evals[858].values.at(i) * trace_evals[858].values.at(i))));
            *numer += accum.random_coeff_powers[138]
                * (trace_evals[860].values.at(i)
                    - ((trace_evals[858].values.at(i) * trace_evals[858].values.at(i))
                        + (trace_evals[859].values.at(i) * trace_evals[859].values.at(i))));
            *numer += accum.random_coeff_powers[137]
                * (trace_evals[861].values.at(i)
                    - ((trace_evals[859].values.at(i) * trace_evals[859].values.at(i))
                        + (trace_evals[860].values.at(i) * trace_evals[860].values.at(i))));
            *numer += accum.random_coeff_powers[136]
                * (trace_evals[862].values.at(i)
                    - ((trace_evals[860].values.at(i) * trace_evals[860].values.at(i))
                        + (trace_evals[861].values.at(i) * trace_evals[861].values.at(i))));
            *numer += accum.random_coeff_powers[135]
                * (trace_evals[863].values.at(i)
                    - ((trace_evals[861].values.at(i) * trace_evals[861].values.at(i))
                        + (trace_evals[862].values.at(i) * trace_evals[862].values.at(i))));
            *numer += accum.random_coeff_powers[134]
                * (trace_evals[864].values.at(i)
                    - ((trace_evals[862].values.at(i) * trace_evals[862].values.at(i))
                        + (trace_evals[863].values.at(i) * trace_evals[863].values.at(i))));
            *numer += accum.random_coeff_powers[133]
                * (trace_evals[865].values.at(i)
                    - ((trace_evals[863].values.at(i) * trace_evals[863].values.at(i))
                        + (trace_evals[864].values.at(i) * trace_evals[864].values.at(i))));
            *numer += accum.random_coeff_powers[132]
                * (trace_evals[866].values.at(i)
                    - ((trace_evals[864].values.at(i) * trace_evals[864].values.at(i))
                        + (trace_evals[865].values.at(i) * trace_evals[865].values.at(i))));
            *numer += accum.random_coeff_powers[131]
                * (trace_evals[867].values.at(i)
                    - ((trace_evals[865].values.at(i) * trace_evals[865].values.at(i))
                        + (trace_evals[866].values.at(i) * trace_evals[866].values.at(i))));
            *numer += accum.random_coeff_powers[130]
                * (trace_evals[868].values.at(i)
                    - ((trace_evals[866].values.at(i) * trace_evals[866].values.at(i))
                        + (trace_evals[867].values.at(i) * trace_evals[867].values.at(i))));
            *numer += accum.random_coeff_powers[129]
                * (trace_evals[869].values.at(i)
                    - ((trace_evals[867].values.at(i) * trace_evals[867].values.at(i))
                        + (trace_evals[868].values.at(i) * trace_evals[868].values.at(i))));
            *numer += accum.random_coeff_powers[128]
                * (trace_evals[870].values.at(i)
                    - ((trace_evals[868].values.at(i) * trace_evals[868].values.at(i))
                        + (trace_evals[869].values.at(i) * trace_evals[869].values.at(i))));
            *numer += accum.random_coeff_powers[127]
                * (trace_evals[871].values.at(i)
                    - ((trace_evals[869].values.at(i) * trace_evals[869].values.at(i))
                        + (trace_evals[870].values.at(i) * trace_evals[870].values.at(i))));
            *numer += accum.random_coeff_powers[126]
                * (trace_evals[872].values.at(i)
                    - ((trace_evals[870].values.at(i) * trace_evals[870].values.at(i))
                        + (trace_evals[871].values.at(i) * trace_evals[871].values.at(i))));
            *numer += accum.random_coeff_powers[125]
                * (trace_evals[873].values.at(i)
                    - ((trace_evals[871].values.at(i) * trace_evals[871].values.at(i))
                        + (trace_evals[872].values.at(i) * trace_evals[872].values.at(i))));
            *numer += accum.random_coeff_powers[124]
                * (trace_evals[874].values.at(i)
                    - ((trace_evals[872].values.at(i) * trace_evals[872].values.at(i))
                        + (trace_evals[873].values.at(i) * trace_evals[873].values.at(i))));
            *numer += accum.random_coeff_powers[123]
                * (trace_evals[875].values.at(i)
                    - ((trace_evals[873].values.at(i) * trace_evals[873].values.at(i))
                        + (trace_evals[874].values.at(i) * trace_evals[874].values.at(i))));
            *numer += accum.random_coeff_powers[122]
                * (trace_evals[876].values.at(i)
                    - ((trace_evals[874].values.at(i) * trace_evals[874].values.at(i))
                        + (trace_evals[875].values.at(i) * trace_evals[875].values.at(i))));
            *numer += accum.random_coeff_powers[121]
                * (trace_evals[877].values.at(i)
                    - ((trace_evals[875].values.at(i) * trace_evals[875].values.at(i))
                        + (trace_evals[876].values.at(i) * trace_evals[876].values.at(i))));
            *numer += accum.random_coeff_powers[120]
                * (trace_evals[878].values.at(i)
                    - ((trace_evals[876].values.at(i) * trace_evals[876].values.at(i))
                        + (trace_evals[877].values.at(i) * trace_evals[877].values.at(i))));
            *numer += accum.random_coeff_powers[119]
                * (trace_evals[879].values.at(i)
                    - ((trace_evals[877].values.at(i) * trace_evals[877].values.at(i))
                        + (trace_evals[878].values.at(i) * trace_evals[878].values.at(i))));
            *numer += accum.random_coeff_powers[118]
                * (trace_evals[880].values.at(i)
                    - ((trace_evals[878].values.at(i) * trace_evals[878].values.at(i))
                        + (trace_evals[879].values.at(i) * trace_evals[879].values.at(i))));
            *numer += accum.random_coeff_powers[117]
                * (trace_evals[881].values.at(i)
                    - ((trace_evals[879].values.at(i) * trace_evals[879].values.at(i))
                        + (trace_evals[880].values.at(i) * trace_evals[880].values.at(i))));
            *numer += accum.random_coeff_powers[116]
                * (trace_evals[882].values.at(i)
                    - ((trace_evals[880].values.at(i) * trace_evals[880].values.at(i))
                        + (trace_evals[881].values.at(i) * trace_evals[881].values.at(i))));
            *numer += accum.random_coeff_powers[115]
                * (trace_evals[883].values.at(i)
                    - ((trace_evals[881].values.at(i) * trace_evals[881].values.at(i))
                        + (trace_evals[882].values.at(i) * trace_evals[882].values.at(i))));
            *numer += accum.random_coeff_powers[114]
                * (trace_evals[884].values.at(i)
                    - ((trace_evals[882].values.at(i) * trace_evals[882].values.at(i))
                        + (trace_evals[883].values.at(i) * trace_evals[883].values.at(i))));
            *numer += accum.random_coeff_powers[113]
                * (trace_evals[885].values.at(i)
                    - ((trace_evals[883].values.at(i) * trace_evals[883].values.at(i))
                        + (trace_evals[884].values.at(i) * trace_evals[884].values.at(i))));
            *numer += accum.random_coeff_powers[112]
                * (trace_evals[886].values.at(i)
                    - ((trace_evals[884].values.at(i) * trace_evals[884].values.at(i))
                        + (trace_evals[885].values.at(i) * trace_evals[885].values.at(i))));
            *numer += accum.random_coeff_powers[111]
                * (trace_evals[887].values.at(i)
                    - ((trace_evals[885].values.at(i) * trace_evals[885].values.at(i))
                        + (trace_evals[886].values.at(i) * trace_evals[886].values.at(i))));
            *numer += accum.random_coeff_powers[110]
                * (trace_evals[888].values.at(i)
                    - ((trace_evals[886].values.at(i) * trace_evals[886].values.at(i))
                        + (trace_evals[887].values.at(i) * trace_evals[887].values.at(i))));
            *numer += accum.random_coeff_powers[109]
                * (trace_evals[889].values.at(i)
                    - ((trace_evals[887].values.at(i) * trace_evals[887].values.at(i))
                        + (trace_evals[888].values.at(i) * trace_evals[888].values.at(i))));
            *numer += accum.random_coeff_powers[108]
                * (trace_evals[890].values.at(i)
                    - ((trace_evals[888].values.at(i) * trace_evals[888].values.at(i))
                        + (trace_evals[889].values.at(i) * trace_evals[889].values.at(i))));
            *numer += accum.random_coeff_powers[107]
                * (trace_evals[891].values.at(i)
                    - ((trace_evals[889].values.at(i) * trace_evals[889].values.at(i))
                        + (trace_evals[890].values.at(i) * trace_evals[890].values.at(i))));
            *numer += accum.random_coeff_powers[106]
                * (trace_evals[892].values.at(i)
                    - ((trace_evals[890].values.at(i) * trace_evals[890].values.at(i))
                        + (trace_evals[891].values.at(i) * trace_evals[891].values.at(i))));
            *numer += accum.random_coeff_powers[105]
                * (trace_evals[893].values.at(i)
                    - ((trace_evals[891].values.at(i) * trace_evals[891].values.at(i))
                        + (trace_evals[892].values.at(i) * trace_evals[892].values.at(i))));
            *numer += accum.random_coeff_powers[104]
                * (trace_evals[894].values.at(i)
                    - ((trace_evals[892].values.at(i) * trace_evals[892].values.at(i))
                        + (trace_evals[893].values.at(i) * trace_evals[893].values.at(i))));
            *numer += accum.random_coeff_powers[103]
                * (trace_evals[895].values.at(i)
                    - ((trace_evals[893].values.at(i) * trace_evals[893].values.at(i))
                        + (trace_evals[894].values.at(i) * trace_evals[894].values.at(i))));
            *numer += accum.random_coeff_powers[102]
                * (trace_evals[896].values.at(i)
                    - ((trace_evals[894].values.at(i) * trace_evals[894].values.at(i))
                        + (trace_evals[895].values.at(i) * trace_evals[895].values.at(i))));
            *numer += accum.random_coeff_powers[101]
                * (trace_evals[897].values.at(i)
                    - ((trace_evals[895].values.at(i) * trace_evals[895].values.at(i))
                        + (trace_evals[896].values.at(i) * trace_evals[896].values.at(i))));
            *numer += accum.random_coeff_powers[100]
                * (trace_evals[898].values.at(i)
                    - ((trace_evals[896].values.at(i) * trace_evals[896].values.at(i))
                        + (trace_evals[897].values.at(i) * trace_evals[897].values.at(i))));
            *numer += accum.random_coeff_powers[99]
                * (trace_evals[899].values.at(i)
                    - ((trace_evals[897].values.at(i) * trace_evals[897].values.at(i))
                        + (trace_evals[898].values.at(i) * trace_evals[898].values.at(i))));
            *numer += accum.random_coeff_powers[98]
                * (trace_evals[900].values.at(i)
                    - ((trace_evals[898].values.at(i) * trace_evals[898].values.at(i))
                        + (trace_evals[899].values.at(i) * trace_evals[899].values.at(i))));
            *numer += accum.random_coeff_powers[97]
                * (trace_evals[901].values.at(i)
                    - ((trace_evals[899].values.at(i) * trace_evals[899].values.at(i))
                        + (trace_evals[900].values.at(i) * trace_evals[900].values.at(i))));
            *numer += accum.random_coeff_powers[96]
                * (trace_evals[902].values.at(i)
                    - ((trace_evals[900].values.at(i) * trace_evals[900].values.at(i))
                        + (trace_evals[901].values.at(i) * trace_evals[901].values.at(i))));
            *numer += accum.random_coeff_powers[95]
                * (trace_evals[903].values.at(i)
                    - ((trace_evals[901].values.at(i) * trace_evals[901].values.at(i))
                        + (trace_evals[902].values.at(i) * trace_evals[902].values.at(i))));
            *numer += accum.random_coeff_powers[94]
                * (trace_evals[904].values.at(i)
                    - ((trace_evals[902].values.at(i) * trace_evals[902].values.at(i))
                        + (trace_evals[903].values.at(i) * trace_evals[903].values.at(i))));
            *numer += accum.random_coeff_powers[93]
                * (trace_evals[905].values.at(i)
                    - ((trace_evals[903].values.at(i) * trace_evals[903].values.at(i))
                        + (trace_evals[904].values.at(i) * trace_evals[904].values.at(i))));
            *numer += accum.random_coeff_powers[92]
                * (trace_evals[906].values.at(i)
                    - ((trace_evals[904].values.at(i) * trace_evals[904].values.at(i))
                        + (trace_evals[905].values.at(i) * trace_evals[905].values.at(i))));
            *numer += accum.random_coeff_powers[91]
                * (trace_evals[907].values.at(i)
                    - ((trace_evals[905].values.at(i) * trace_evals[905].values.at(i))
                        + (trace_evals[906].values.at(i) * trace_evals[906].values.at(i))));
            *numer += accum.random_coeff_powers[90]
                * (trace_evals[908].values.at(i)
                    - ((trace_evals[906].values.at(i) * trace_evals[906].values.at(i))
                        + (trace_evals[907].values.at(i) * trace_evals[907].values.at(i))));
            *numer += accum.random_coeff_powers[89]
                * (trace_evals[909].values.at(i)
                    - ((trace_evals[907].values.at(i) * trace_evals[907].values.at(i))
                        + (trace_evals[908].values.at(i) * trace_evals[908].values.at(i))));
            *numer += accum.random_coeff_powers[88]
                * (trace_evals[910].values.at(i)
                    - ((trace_evals[908].values.at(i) * trace_evals[908].values.at(i))
                        + (trace_evals[909].values.at(i) * trace_evals[909].values.at(i))));
            *numer += accum.random_coeff_powers[87]
                * (trace_evals[911].values.at(i)
                    - ((trace_evals[909].values.at(i) * trace_evals[909].values.at(i))
                        + (trace_evals[910].values.at(i) * trace_evals[910].values.at(i))));
            *numer += accum.random_coeff_powers[86]
                * (trace_evals[912].values.at(i)
                    - ((trace_evals[910].values.at(i) * trace_evals[910].values.at(i))
                        + (trace_evals[911].values.at(i) * trace_evals[911].values.at(i))));
            *numer += accum.random_coeff_powers[85]
                * (trace_evals[913].values.at(i)
                    - ((trace_evals[911].values.at(i) * trace_evals[911].values.at(i))
                        + (trace_evals[912].values.at(i) * trace_evals[912].values.at(i))));
            *numer += accum.random_coeff_powers[84]
                * (trace_evals[914].values.at(i)
                    - ((trace_evals[912].values.at(i) * trace_evals[912].values.at(i))
                        + (trace_evals[913].values.at(i) * trace_evals[913].values.at(i))));
            *numer += accum.random_coeff_powers[83]
                * (trace_evals[915].values.at(i)
                    - ((trace_evals[913].values.at(i) * trace_evals[913].values.at(i))
                        + (trace_evals[914].values.at(i) * trace_evals[914].values.at(i))));
            *numer += accum.random_coeff_powers[82]
                * (trace_evals[916].values.at(i)
                    - ((trace_evals[914].values.at(i) * trace_evals[914].values.at(i))
                        + (trace_evals[915].values.at(i) * trace_evals[915].values.at(i))));
            *numer += accum.random_coeff_powers[81]
                * (trace_evals[917].values.at(i)
                    - ((trace_evals[915].values.at(i) * trace_evals[915].values.at(i))
                        + (trace_evals[916].values.at(i) * trace_evals[916].values.at(i))));
            *numer += accum.random_coeff_powers[80]
                * (trace_evals[918].values.at(i)
                    - ((trace_evals[916].values.at(i) * trace_evals[916].values.at(i))
                        + (trace_evals[917].values.at(i) * trace_evals[917].values.at(i))));
            *numer += accum.random_coeff_powers[79]
                * (trace_evals[919].values.at(i)
                    - ((trace_evals[917].values.at(i) * trace_evals[917].values.at(i))
                        + (trace_evals[918].values.at(i) * trace_evals[918].values.at(i))));
            *numer += accum.random_coeff_powers[78]
                * (trace_evals[920].values.at(i)
                    - ((trace_evals[918].values.at(i) * trace_evals[918].values.at(i))
                        + (trace_evals[919].values.at(i) * trace_evals[919].values.at(i))));
            *numer += accum.random_coeff_powers[77]
                * (trace_evals[921].values.at(i)
                    - ((trace_evals[919].values.at(i) * trace_evals[919].values.at(i))
                        + (trace_evals[920].values.at(i) * trace_evals[920].values.at(i))));
            *numer += accum.random_coeff_powers[76]
                * (trace_evals[922].values.at(i)
                    - ((trace_evals[920].values.at(i) * trace_evals[920].values.at(i))
                        + (trace_evals[921].values.at(i) * trace_evals[921].values.at(i))));
            *numer += accum.random_coeff_powers[75]
                * (trace_evals[923].values.at(i)
                    - ((trace_evals[921].values.at(i) * trace_evals[921].values.at(i))
                        + (trace_evals[922].values.at(i) * trace_evals[922].values.at(i))));
            *numer += accum.random_coeff_powers[74]
                * (trace_evals[924].values.at(i)
                    - ((trace_evals[922].values.at(i) * trace_evals[922].values.at(i))
                        + (trace_evals[923].values.at(i) * trace_evals[923].values.at(i))));
            *numer += accum.random_coeff_powers[73]
                * (trace_evals[925].values.at(i)
                    - ((trace_evals[923].values.at(i) * trace_evals[923].values.at(i))
                        + (trace_evals[924].values.at(i) * trace_evals[924].values.at(i))));
            *numer += accum.random_coeff_powers[72]
                * (trace_evals[926].values.at(i)
                    - ((trace_evals[924].values.at(i) * trace_evals[924].values.at(i))
                        + (trace_evals[925].values.at(i) * trace_evals[925].values.at(i))));
            *numer += accum.random_coeff_powers[71]
                * (trace_evals[927].values.at(i)
                    - ((trace_evals[925].values.at(i) * trace_evals[925].values.at(i))
                        + (trace_evals[926].values.at(i) * trace_evals[926].values.at(i))));
            *numer += accum.random_coeff_powers[70]
                * (trace_evals[928].values.at(i)
                    - ((trace_evals[926].values.at(i) * trace_evals[926].values.at(i))
                        + (trace_evals[927].values.at(i) * trace_evals[927].values.at(i))));
            *numer += accum.random_coeff_powers[69]
                * (trace_evals[929].values.at(i)
                    - ((trace_evals[927].values.at(i) * trace_evals[927].values.at(i))
                        + (trace_evals[928].values.at(i) * trace_evals[928].values.at(i))));
            *numer += accum.random_coeff_powers[68]
                * (trace_evals[930].values.at(i)
                    - ((trace_evals[928].values.at(i) * trace_evals[928].values.at(i))
                        + (trace_evals[929].values.at(i) * trace_evals[929].values.at(i))));
            *numer += accum.random_coeff_powers[67]
                * (trace_evals[931].values.at(i)
                    - ((trace_evals[929].values.at(i) * trace_evals[929].values.at(i))
                        + (trace_evals[930].values.at(i) * trace_evals[930].values.at(i))));
            *numer += accum.random_coeff_powers[66]
                * (trace_evals[932].values.at(i)
                    - ((trace_evals[930].values.at(i) * trace_evals[930].values.at(i))
                        + (trace_evals[931].values.at(i) * trace_evals[931].values.at(i))));
            *numer += accum.random_coeff_powers[65]
                * (trace_evals[933].values.at(i)
                    - ((trace_evals[931].values.at(i) * trace_evals[931].values.at(i))
                        + (trace_evals[932].values.at(i) * trace_evals[932].values.at(i))));
            *numer += accum.random_coeff_powers[64]
                * (trace_evals[934].values.at(i)
                    - ((trace_evals[932].values.at(i) * trace_evals[932].values.at(i))
                        + (trace_evals[933].values.at(i) * trace_evals[933].values.at(i))));
            *numer += accum.random_coeff_powers[63]
                * (trace_evals[935].values.at(i)
                    - ((trace_evals[933].values.at(i) * trace_evals[933].values.at(i))
                        + (trace_evals[934].values.at(i) * trace_evals[934].values.at(i))));
            *numer += accum.random_coeff_powers[62]
                * (trace_evals[936].values.at(i)
                    - ((trace_evals[934].values.at(i) * trace_evals[934].values.at(i))
                        + (trace_evals[935].values.at(i) * trace_evals[935].values.at(i))));
            *numer += accum.random_coeff_powers[61]
                * (trace_evals[937].values.at(i)
                    - ((trace_evals[935].values.at(i) * trace_evals[935].values.at(i))
                        + (trace_evals[936].values.at(i) * trace_evals[936].values.at(i))));
            *numer += accum.random_coeff_powers[60]
                * (trace_evals[938].values.at(i)
                    - ((trace_evals[936].values.at(i) * trace_evals[936].values.at(i))
                        + (trace_evals[937].values.at(i) * trace_evals[937].values.at(i))));
            *numer += accum.random_coeff_powers[59]
                * (trace_evals[939].values.at(i)
                    - ((trace_evals[937].values.at(i) * trace_evals[937].values.at(i))
                        + (trace_evals[938].values.at(i) * trace_evals[938].values.at(i))));
            *numer += accum.random_coeff_powers[58]
                * (trace_evals[940].values.at(i)
                    - ((trace_evals[938].values.at(i) * trace_evals[938].values.at(i))
                        + (trace_evals[939].values.at(i) * trace_evals[939].values.at(i))));
            *numer += accum.random_coeff_powers[57]
                * (trace_evals[941].values.at(i)
                    - ((trace_evals[939].values.at(i) * trace_evals[939].values.at(i))
                        + (trace_evals[940].values.at(i) * trace_evals[940].values.at(i))));
            *numer += accum.random_coeff_powers[56]
                * (trace_evals[942].values.at(i)
                    - ((trace_evals[940].values.at(i) * trace_evals[940].values.at(i))
                        + (trace_evals[941].values.at(i) * trace_evals[941].values.at(i))));
            *numer += accum.random_coeff_powers[55]
                * (trace_evals[943].values.at(i)
                    - ((trace_evals[941].values.at(i) * trace_evals[941].values.at(i))
                        + (trace_evals[942].values.at(i) * trace_evals[942].values.at(i))));
            *numer += accum.random_coeff_powers[54]
                * (trace_evals[944].values.at(i)
                    - ((trace_evals[942].values.at(i) * trace_evals[942].values.at(i))
                        + (trace_evals[943].values.at(i) * trace_evals[943].values.at(i))));
            *numer += accum.random_coeff_powers[53]
                * (trace_evals[945].values.at(i)
                    - ((trace_evals[943].values.at(i) * trace_evals[943].values.at(i))
                        + (trace_evals[944].values.at(i) * trace_evals[944].values.at(i))));
            *numer += accum.random_coeff_powers[52]
                * (trace_evals[946].values.at(i)
                    - ((trace_evals[944].values.at(i) * trace_evals[944].values.at(i))
                        + (trace_evals[945].values.at(i) * trace_evals[945].values.at(i))));
            *numer += accum.random_coeff_powers[51]
                * (trace_evals[947].values.at(i)
                    - ((trace_evals[945].values.at(i) * trace_evals[945].values.at(i))
                        + (trace_evals[946].values.at(i) * trace_evals[946].values.at(i))));
            *numer += accum.random_coeff_powers[50]
                * (trace_evals[948].values.at(i)
                    - ((trace_evals[946].values.at(i) * trace_evals[946].values.at(i))
                        + (trace_evals[947].values.at(i) * trace_evals[947].values.at(i))));
            *numer += accum.random_coeff_powers[49]
                * (trace_evals[949].values.at(i)
                    - ((trace_evals[947].values.at(i) * trace_evals[947].values.at(i))
                        + (trace_evals[948].values.at(i) * trace_evals[948].values.at(i))));
            *numer += accum.random_coeff_powers[48]
                * (trace_evals[950].values.at(i)
                    - ((trace_evals[948].values.at(i) * trace_evals[948].values.at(i))
                        + (trace_evals[949].values.at(i) * trace_evals[949].values.at(i))));
            *numer += accum.random_coeff_powers[47]
                * (trace_evals[951].values.at(i)
                    - ((trace_evals[949].values.at(i) * trace_evals[949].values.at(i))
                        + (trace_evals[950].values.at(i) * trace_evals[950].values.at(i))));
            *numer += accum.random_coeff_powers[46]
                * (trace_evals[952].values.at(i)
                    - ((trace_evals[950].values.at(i) * trace_evals[950].values.at(i))
                        + (trace_evals[951].values.at(i) * trace_evals[951].values.at(i))));
            *numer += accum.random_coeff_powers[45]
                * (trace_evals[953].values.at(i)
                    - ((trace_evals[951].values.at(i) * trace_evals[951].values.at(i))
                        + (trace_evals[952].values.at(i) * trace_evals[952].values.at(i))));
            *numer += accum.random_coeff_powers[44]
                * (trace_evals[954].values.at(i)
                    - ((trace_evals[952].values.at(i) * trace_evals[952].values.at(i))
                        + (trace_evals[953].values.at(i) * trace_evals[953].values.at(i))));
            *numer += accum.random_coeff_powers[43]
                * (trace_evals[955].values.at(i)
                    - ((trace_evals[953].values.at(i) * trace_evals[953].values.at(i))
                        + (trace_evals[954].values.at(i) * trace_evals[954].values.at(i))));
            *numer += accum.random_coeff_powers[42]
                * (trace_evals[956].values.at(i)
                    - ((trace_evals[954].values.at(i) * trace_evals[954].values.at(i))
                        + (trace_evals[955].values.at(i) * trace_evals[955].values.at(i))));
            *numer += accum.random_coeff_powers[41]
                * (trace_evals[957].values.at(i)
                    - ((trace_evals[955].values.at(i) * trace_evals[955].values.at(i))
                        + (trace_evals[956].values.at(i) * trace_evals[956].values.at(i))));
            *numer += accum.random_coeff_powers[40]
                * (trace_evals[958].values.at(i)
                    - ((trace_evals[956].values.at(i) * trace_evals[956].values.at(i))
                        + (trace_evals[957].values.at(i) * trace_evals[957].values.at(i))));
            *numer += accum.random_coeff_powers[39]
                * (trace_evals[959].values.at(i)
                    - ((trace_evals[957].values.at(i) * trace_evals[957].values.at(i))
                        + (trace_evals[958].values.at(i) * trace_evals[958].values.at(i))));
            *numer += accum.random_coeff_powers[38]
                * (trace_evals[960].values.at(i)
                    - ((trace_evals[958].values.at(i) * trace_evals[958].values.at(i))
                        + (trace_evals[959].values.at(i) * trace_evals[959].values.at(i))));
            *numer += accum.random_coeff_powers[37]
                * (trace_evals[961].values.at(i)
                    - ((trace_evals[959].values.at(i) * trace_evals[959].values.at(i))
                        + (trace_evals[960].values.at(i) * trace_evals[960].values.at(i))));
            *numer += accum.random_coeff_powers[36]
                * (trace_evals[962].values.at(i)
                    - ((trace_evals[960].values.at(i) * trace_evals[960].values.at(i))
                        + (trace_evals[961].values.at(i) * trace_evals[961].values.at(i))));
            *numer += accum.random_coeff_powers[35]
                * (trace_evals[963].values.at(i)
                    - ((trace_evals[961].values.at(i) * trace_evals[961].values.at(i))
                        + (trace_evals[962].values.at(i) * trace_evals[962].values.at(i))));
            *numer += accum.random_coeff_powers[34]
                * (trace_evals[964].values.at(i)
                    - ((trace_evals[962].values.at(i) * trace_evals[962].values.at(i))
                        + (trace_evals[963].values.at(i) * trace_evals[963].values.at(i))));
            *numer += accum.random_coeff_powers[33]
                * (trace_evals[965].values.at(i)
                    - ((trace_evals[963].values.at(i) * trace_evals[963].values.at(i))
                        + (trace_evals[964].values.at(i) * trace_evals[964].values.at(i))));
            *numer += accum.random_coeff_powers[32]
                * (trace_evals[966].values.at(i)
                    - ((trace_evals[964].values.at(i) * trace_evals[964].values.at(i))
                        + (trace_evals[965].values.at(i) * trace_evals[965].values.at(i))));
            *numer += accum.random_coeff_powers[31]
                * (trace_evals[967].values.at(i)
                    - ((trace_evals[965].values.at(i) * trace_evals[965].values.at(i))
                        + (trace_evals[966].values.at(i) * trace_evals[966].values.at(i))));
            *numer += accum.random_coeff_powers[30]
                * (trace_evals[968].values.at(i)
                    - ((trace_evals[966].values.at(i) * trace_evals[966].values.at(i))
                        + (trace_evals[967].values.at(i) * trace_evals[967].values.at(i))));
            *numer += accum.random_coeff_powers[29]
                * (trace_evals[969].values.at(i)
                    - ((trace_evals[967].values.at(i) * trace_evals[967].values.at(i))
                        + (trace_evals[968].values.at(i) * trace_evals[968].values.at(i))));
            *numer += accum.random_coeff_powers[28]
                * (trace_evals[970].values.at(i)
                    - ((trace_evals[968].values.at(i) * trace_evals[968].values.at(i))
                        + (trace_evals[969].values.at(i) * trace_evals[969].values.at(i))));
            *numer += accum.random_coeff_powers[27]
                * (trace_evals[971].values.at(i)
                    - ((trace_evals[969].values.at(i) * trace_evals[969].values.at(i))
                        + (trace_evals[970].values.at(i) * trace_evals[970].values.at(i))));
            *numer += accum.random_coeff_powers[26]
                * (trace_evals[972].values.at(i)
                    - ((trace_evals[970].values.at(i) * trace_evals[970].values.at(i))
                        + (trace_evals[971].values.at(i) * trace_evals[971].values.at(i))));
            *numer += accum.random_coeff_powers[25]
                * (trace_evals[973].values.at(i)
                    - ((trace_evals[971].values.at(i) * trace_evals[971].values.at(i))
                        + (trace_evals[972].values.at(i) * trace_evals[972].values.at(i))));
            *numer += accum.random_coeff_powers[24]
                * (trace_evals[974].values.at(i)
                    - ((trace_evals[972].values.at(i) * trace_evals[972].values.at(i))
                        + (trace_evals[973].values.at(i) * trace_evals[973].values.at(i))));
            *numer += accum.random_coeff_powers[23]
                * (trace_evals[975].values.at(i)
                    - ((trace_evals[973].values.at(i) * trace_evals[973].values.at(i))
                        + (trace_evals[974].values.at(i) * trace_evals[974].values.at(i))));
            *numer += accum.random_coeff_powers[22]
                * (trace_evals[976].values.at(i)
                    - ((trace_evals[974].values.at(i) * trace_evals[974].values.at(i))
                        + (trace_evals[975].values.at(i) * trace_evals[975].values.at(i))));
            *numer += accum.random_coeff_powers[21]
                * (trace_evals[977].values.at(i)
                    - ((trace_evals[975].values.at(i) * trace_evals[975].values.at(i))
                        + (trace_evals[976].values.at(i) * trace_evals[976].values.at(i))));
            *numer += accum.random_coeff_powers[20]
                * (trace_evals[978].values.at(i)
                    - ((trace_evals[976].values.at(i) * trace_evals[976].values.at(i))
                        + (trace_evals[977].values.at(i) * trace_evals[977].values.at(i))));
            *numer += accum.random_coeff_powers[19]
                * (trace_evals[979].values.at(i)
                    - ((trace_evals[977].values.at(i) * trace_evals[977].values.at(i))
                        + (trace_evals[978].values.at(i) * trace_evals[978].values.at(i))));
            *numer += accum.random_coeff_powers[18]
                * (trace_evals[980].values.at(i)
                    - ((trace_evals[978].values.at(i) * trace_evals[978].values.at(i))
                        + (trace_evals[979].values.at(i) * trace_evals[979].values.at(i))));
            *numer += accum.random_coeff_powers[17]
                * (trace_evals[981].values.at(i)
                    - ((trace_evals[979].values.at(i) * trace_evals[979].values.at(i))
                        + (trace_evals[980].values.at(i) * trace_evals[980].values.at(i))));
            *numer += accum.random_coeff_powers[16]
                * (trace_evals[982].values.at(i)
                    - ((trace_evals[980].values.at(i) * trace_evals[980].values.at(i))
                        + (trace_evals[981].values.at(i) * trace_evals[981].values.at(i))));
            *numer += accum.random_coeff_powers[15]
                * (trace_evals[983].values.at(i)
                    - ((trace_evals[981].values.at(i) * trace_evals[981].values.at(i))
                        + (trace_evals[982].values.at(i) * trace_evals[982].values.at(i))));
            *numer += accum.random_coeff_powers[14]
                * (trace_evals[984].values.at(i)
                    - ((trace_evals[982].values.at(i) * trace_evals[982].values.at(i))
                        + (trace_evals[983].values.at(i) * trace_evals[983].values.at(i))));
            *numer += accum.random_coeff_powers[13]
                * (trace_evals[985].values.at(i)
                    - ((trace_evals[983].values.at(i) * trace_evals[983].values.at(i))
                        + (trace_evals[984].values.at(i) * trace_evals[984].values.at(i))));
            *numer += accum.random_coeff_powers[12]
                * (trace_evals[986].values.at(i)
                    - ((trace_evals[984].values.at(i) * trace_evals[984].values.at(i))
                        + (trace_evals[985].values.at(i) * trace_evals[985].values.at(i))));
            *numer += accum.random_coeff_powers[11]
                * (trace_evals[987].values.at(i)
                    - ((trace_evals[985].values.at(i) * trace_evals[985].values.at(i))
                        + (trace_evals[986].values.at(i) * trace_evals[986].values.at(i))));
            *numer += accum.random_coeff_powers[10]
                * (trace_evals[988].values.at(i)
                    - ((trace_evals[986].values.at(i) * trace_evals[986].values.at(i))
                        + (trace_evals[987].values.at(i) * trace_evals[987].values.at(i))));
            *numer += accum.random_coeff_powers[9]
                * (trace_evals[989].values.at(i)
                    - ((trace_evals[987].values.at(i) * trace_evals[987].values.at(i))
                        + (trace_evals[988].values.at(i) * trace_evals[988].values.at(i))));
            *numer += accum.random_coeff_powers[8]
                * (trace_evals[990].values.at(i)
                    - ((trace_evals[988].values.at(i) * trace_evals[988].values.at(i))
                        + (trace_evals[989].values.at(i) * trace_evals[989].values.at(i))));
            *numer += accum.random_coeff_powers[7]
                * (trace_evals[991].values.at(i)
                    - ((trace_evals[989].values.at(i) * trace_evals[989].values.at(i))
                        + (trace_evals[990].values.at(i) * trace_evals[990].values.at(i))));
            *numer += accum.random_coeff_powers[6]
                * (trace_evals[992].values.at(i)
                    - ((trace_evals[990].values.at(i) * trace_evals[990].values.at(i))
                        + (trace_evals[991].values.at(i) * trace_evals[991].values.at(i))));
            *numer += accum.random_coeff_powers[5]
                * (trace_evals[993].values.at(i)
                    - ((trace_evals[991].values.at(i) * trace_evals[991].values.at(i))
                        + (trace_evals[992].values.at(i) * trace_evals[992].values.at(i))));
            *numer += accum.random_coeff_powers[4]
                * (trace_evals[994].values.at(i)
                    - ((trace_evals[992].values.at(i) * trace_evals[992].values.at(i))
                        + (trace_evals[993].values.at(i) * trace_evals[993].values.at(i))));
            *numer += accum.random_coeff_powers[3]
                * (trace_evals[995].values.at(i)
                    - ((trace_evals[993].values.at(i) * trace_evals[993].values.at(i))
                        + (trace_evals[994].values.at(i) * trace_evals[994].values.at(i))));
            *numer += accum.random_coeff_powers[2]
                * (trace_evals[996].values.at(i)
                    - ((trace_evals[994].values.at(i) * trace_evals[994].values.at(i))
                        + (trace_evals[995].values.at(i) * trace_evals[995].values.at(i))));
            *numer += accum.random_coeff_powers[1]
                * (trace_evals[997].values.at(i)
                    - ((trace_evals[995].values.at(i) * trace_evals[995].values.at(i))
                        + (trace_evals[996].values.at(i) * trace_evals[996].values.at(i))));
            *numer += accum.random_coeff_powers[0]
                * (trace_evals[998].values.at(i)
                    - ((trace_evals[996].values.at(i) * trace_evals[996].values.at(i))
                        + (trace_evals[997].values.at(i) * trace_evals[997].values.at(i))));
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
