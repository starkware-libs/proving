use stwo_prover::core::air::accumulation::PointEvaluationAccumulator;
use stwo_prover::core::air::mask::fixed_mask_points;
use stwo_prover::core::air::Component;
use stwo_prover::core::circle::CirclePoint;
use stwo_prover::core::constraints::coset_vanishing;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::poly::circle::CanonicCoset;
use stwo_prover::core::ColumnVec;

#[allow(non_camel_case_types)]
pub struct Fib__100 {
    pub log_n_instances: u32,
}

impl Component for Fib__100 {
    fn n_constraints(&self) -> usize {
        98
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_n_instances + 1
    }

    fn trace_log_degree_bounds(&self) -> Vec<u32> {
        vec![self.log_n_instances; 99]
    }

    fn mask_points(
        &self,
        point: CirclePoint<SecureField>,
    ) -> ColumnVec<Vec<CirclePoint<SecureField>>> {
        fixed_mask_points(&vec![vec![0_usize]; 99], point)
    }

    #[allow(unused_parens)]
    fn evaluate_constraint_quotients_at_point(
        &self,
        point: CirclePoint<SecureField>,
        mask: &ColumnVec<Vec<SecureField>>,
        evaluation_accumulator: &mut PointEvaluationAccumulator,
    ) {
        let constraint_zero_domain = CanonicCoset::new(self.log_n_instances).coset;
        let denominator_inv = coset_vanishing(constraint_zero_domain, point).inverse();
        let numerator = (mask[1][0]
            - ((BaseField::from_u32_unchecked(1) * BaseField::from_u32_unchecked(1))
                + (mask[0][0] * mask[0][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[2][0] - ((mask[0][0] * mask[0][0]) + (mask[1][0] * mask[1][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[3][0] - ((mask[1][0] * mask[1][0]) + (mask[2][0] * mask[2][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[4][0] - ((mask[2][0] * mask[2][0]) + (mask[3][0] * mask[3][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[5][0] - ((mask[3][0] * mask[3][0]) + (mask[4][0] * mask[4][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[6][0] - ((mask[4][0] * mask[4][0]) + (mask[5][0] * mask[5][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[7][0] - ((mask[5][0] * mask[5][0]) + (mask[6][0] * mask[6][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[8][0] - ((mask[6][0] * mask[6][0]) + (mask[7][0] * mask[7][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[9][0] - ((mask[7][0] * mask[7][0]) + (mask[8][0] * mask[8][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[10][0] - ((mask[8][0] * mask[8][0]) + (mask[9][0] * mask[9][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[11][0] - ((mask[9][0] * mask[9][0]) + (mask[10][0] * mask[10][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[12][0] - ((mask[10][0] * mask[10][0]) + (mask[11][0] * mask[11][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[13][0] - ((mask[11][0] * mask[11][0]) + (mask[12][0] * mask[12][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[14][0] - ((mask[12][0] * mask[12][0]) + (mask[13][0] * mask[13][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[15][0] - ((mask[13][0] * mask[13][0]) + (mask[14][0] * mask[14][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[16][0] - ((mask[14][0] * mask[14][0]) + (mask[15][0] * mask[15][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[17][0] - ((mask[15][0] * mask[15][0]) + (mask[16][0] * mask[16][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[18][0] - ((mask[16][0] * mask[16][0]) + (mask[17][0] * mask[17][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[19][0] - ((mask[17][0] * mask[17][0]) + (mask[18][0] * mask[18][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[20][0] - ((mask[18][0] * mask[18][0]) + (mask[19][0] * mask[19][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[21][0] - ((mask[19][0] * mask[19][0]) + (mask[20][0] * mask[20][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[22][0] - ((mask[20][0] * mask[20][0]) + (mask[21][0] * mask[21][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[23][0] - ((mask[21][0] * mask[21][0]) + (mask[22][0] * mask[22][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[24][0] - ((mask[22][0] * mask[22][0]) + (mask[23][0] * mask[23][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[25][0] - ((mask[23][0] * mask[23][0]) + (mask[24][0] * mask[24][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[26][0] - ((mask[24][0] * mask[24][0]) + (mask[25][0] * mask[25][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[27][0] - ((mask[25][0] * mask[25][0]) + (mask[26][0] * mask[26][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[28][0] - ((mask[26][0] * mask[26][0]) + (mask[27][0] * mask[27][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[29][0] - ((mask[27][0] * mask[27][0]) + (mask[28][0] * mask[28][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[30][0] - ((mask[28][0] * mask[28][0]) + (mask[29][0] * mask[29][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[31][0] - ((mask[29][0] * mask[29][0]) + (mask[30][0] * mask[30][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[32][0] - ((mask[30][0] * mask[30][0]) + (mask[31][0] * mask[31][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[33][0] - ((mask[31][0] * mask[31][0]) + (mask[32][0] * mask[32][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[34][0] - ((mask[32][0] * mask[32][0]) + (mask[33][0] * mask[33][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[35][0] - ((mask[33][0] * mask[33][0]) + (mask[34][0] * mask[34][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[36][0] - ((mask[34][0] * mask[34][0]) + (mask[35][0] * mask[35][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[37][0] - ((mask[35][0] * mask[35][0]) + (mask[36][0] * mask[36][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[38][0] - ((mask[36][0] * mask[36][0]) + (mask[37][0] * mask[37][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[39][0] - ((mask[37][0] * mask[37][0]) + (mask[38][0] * mask[38][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[40][0] - ((mask[38][0] * mask[38][0]) + (mask[39][0] * mask[39][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[41][0] - ((mask[39][0] * mask[39][0]) + (mask[40][0] * mask[40][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[42][0] - ((mask[40][0] * mask[40][0]) + (mask[41][0] * mask[41][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[43][0] - ((mask[41][0] * mask[41][0]) + (mask[42][0] * mask[42][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[44][0] - ((mask[42][0] * mask[42][0]) + (mask[43][0] * mask[43][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[45][0] - ((mask[43][0] * mask[43][0]) + (mask[44][0] * mask[44][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[46][0] - ((mask[44][0] * mask[44][0]) + (mask[45][0] * mask[45][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[47][0] - ((mask[45][0] * mask[45][0]) + (mask[46][0] * mask[46][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[48][0] - ((mask[46][0] * mask[46][0]) + (mask[47][0] * mask[47][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[49][0] - ((mask[47][0] * mask[47][0]) + (mask[48][0] * mask[48][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[50][0] - ((mask[48][0] * mask[48][0]) + (mask[49][0] * mask[49][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[51][0] - ((mask[49][0] * mask[49][0]) + (mask[50][0] * mask[50][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[52][0] - ((mask[50][0] * mask[50][0]) + (mask[51][0] * mask[51][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[53][0] - ((mask[51][0] * mask[51][0]) + (mask[52][0] * mask[52][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[54][0] - ((mask[52][0] * mask[52][0]) + (mask[53][0] * mask[53][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[55][0] - ((mask[53][0] * mask[53][0]) + (mask[54][0] * mask[54][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[56][0] - ((mask[54][0] * mask[54][0]) + (mask[55][0] * mask[55][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[57][0] - ((mask[55][0] * mask[55][0]) + (mask[56][0] * mask[56][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[58][0] - ((mask[56][0] * mask[56][0]) + (mask[57][0] * mask[57][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[59][0] - ((mask[57][0] * mask[57][0]) + (mask[58][0] * mask[58][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[60][0] - ((mask[58][0] * mask[58][0]) + (mask[59][0] * mask[59][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[61][0] - ((mask[59][0] * mask[59][0]) + (mask[60][0] * mask[60][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[62][0] - ((mask[60][0] * mask[60][0]) + (mask[61][0] * mask[61][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[63][0] - ((mask[61][0] * mask[61][0]) + (mask[62][0] * mask[62][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[64][0] - ((mask[62][0] * mask[62][0]) + (mask[63][0] * mask[63][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[65][0] - ((mask[63][0] * mask[63][0]) + (mask[64][0] * mask[64][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[66][0] - ((mask[64][0] * mask[64][0]) + (mask[65][0] * mask[65][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[67][0] - ((mask[65][0] * mask[65][0]) + (mask[66][0] * mask[66][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[68][0] - ((mask[66][0] * mask[66][0]) + (mask[67][0] * mask[67][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[69][0] - ((mask[67][0] * mask[67][0]) + (mask[68][0] * mask[68][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[70][0] - ((mask[68][0] * mask[68][0]) + (mask[69][0] * mask[69][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[71][0] - ((mask[69][0] * mask[69][0]) + (mask[70][0] * mask[70][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[72][0] - ((mask[70][0] * mask[70][0]) + (mask[71][0] * mask[71][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[73][0] - ((mask[71][0] * mask[71][0]) + (mask[72][0] * mask[72][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[74][0] - ((mask[72][0] * mask[72][0]) + (mask[73][0] * mask[73][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[75][0] - ((mask[73][0] * mask[73][0]) + (mask[74][0] * mask[74][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[76][0] - ((mask[74][0] * mask[74][0]) + (mask[75][0] * mask[75][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[77][0] - ((mask[75][0] * mask[75][0]) + (mask[76][0] * mask[76][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[78][0] - ((mask[76][0] * mask[76][0]) + (mask[77][0] * mask[77][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[79][0] - ((mask[77][0] * mask[77][0]) + (mask[78][0] * mask[78][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[80][0] - ((mask[78][0] * mask[78][0]) + (mask[79][0] * mask[79][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[81][0] - ((mask[79][0] * mask[79][0]) + (mask[80][0] * mask[80][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[82][0] - ((mask[80][0] * mask[80][0]) + (mask[81][0] * mask[81][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[83][0] - ((mask[81][0] * mask[81][0]) + (mask[82][0] * mask[82][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[84][0] - ((mask[82][0] * mask[82][0]) + (mask[83][0] * mask[83][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[85][0] - ((mask[83][0] * mask[83][0]) + (mask[84][0] * mask[84][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[86][0] - ((mask[84][0] * mask[84][0]) + (mask[85][0] * mask[85][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[87][0] - ((mask[85][0] * mask[85][0]) + (mask[86][0] * mask[86][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[88][0] - ((mask[86][0] * mask[86][0]) + (mask[87][0] * mask[87][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[89][0] - ((mask[87][0] * mask[87][0]) + (mask[88][0] * mask[88][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[90][0] - ((mask[88][0] * mask[88][0]) + (mask[89][0] * mask[89][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[91][0] - ((mask[89][0] * mask[89][0]) + (mask[90][0] * mask[90][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[92][0] - ((mask[90][0] * mask[90][0]) + (mask[91][0] * mask[91][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[93][0] - ((mask[91][0] * mask[91][0]) + (mask[92][0] * mask[92][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[94][0] - ((mask[92][0] * mask[92][0]) + (mask[93][0] * mask[93][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[95][0] - ((mask[93][0] * mask[93][0]) + (mask[94][0] * mask[94][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[96][0] - ((mask[94][0] * mask[94][0]) + (mask[95][0] * mask[95][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[97][0] - ((mask[95][0] * mask[95][0]) + (mask[96][0] * mask[96][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
        let numerator = (mask[98][0] - ((mask[96][0] * mask[96][0]) + (mask[97][0] * mask[97][0])));
        evaluation_accumulator.accumulate(numerator * denominator_inv);
    }
}
