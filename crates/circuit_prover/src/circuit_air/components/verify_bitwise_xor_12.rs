use crate::circuit_air::components::prelude::*;
pub const ELEM_BITS: u32 = 12;
pub const EXPAND_BITS: u32 = 2;
pub const LIMB_BITS: u32 = ELEM_BITS - EXPAND_BITS;
pub const LOG_SIZE: u32 = (ELEM_BITS - EXPAND_BITS) * 2;
pub const N_MULT_COLUMNS: usize = 1 << (EXPAND_BITS * 2);
pub const N_TRACE_COLUMNS: usize = N_MULT_COLUMNS;

pub struct Eval {
    pub claim: Claim,
    pub common_lookup_elements: relations::CommonLookupElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Claim {}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let trace_log_sizes = vec![LOG_SIZE; N_TRACE_COLUMNS];
        let interaction_log_sizes =
            vec![LOG_SIZE; SECURE_EXTENSION_DEGREE * N_MULT_COLUMNS.div_ceil(2)];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_log_sizes])
    }

    pub fn mix_into(&self, _channel: &mut impl Channel) {}
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct InteractionClaim {
    pub claimed_sum: SecureField,
}
impl InteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.claimed_sum]);
    }
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        LOG_SIZE
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let relation_id = E::F::from(M31::from(648362599));
        // al, bl are the constant columns for the inputs: All pairs of elements in [0,
        // 2^LIMB_BITS).
        // cl is the constant column for the xor: al ^ bl.
        let a_low = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "bitwise_xor_10_0".to_owned() });
        let b_low = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "bitwise_xor_10_1".to_owned() });
        let c_low = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "bitwise_xor_10_2".to_owned() });

        for i in 0..1 << EXPAND_BITS {
            for j in 0..1 << EXPAND_BITS {
                let multiplicity = eval.next_trace_mask();

                let a = a_low.clone() + E::F::from(M31(i << LIMB_BITS));
                let b = b_low.clone() + E::F::from(M31(j << LIMB_BITS));
                let c = c_low.clone() + E::F::from(M31((i ^ j) << LIMB_BITS));

                eval.add_to_relation(RelationEntry::new(
                    &self.common_lookup_elements,
                    -E::EF::from(multiplicity),
                    &[relation_id.clone(), a, b, c],
                ));
            }
        }

        eval.finalize_logup_in_pairs();
        eval
    }
}

#[cfg(test)]
mod tests {
    use stwo::core::air::accumulation::PointEvaluationAccumulator;
    use stwo::core::fields::cm31::CM31;
    use stwo::core::fields::qm31::QM31;
    use stwo_constraint_framework::PointEvaluator;

    use super::*;

    /// The evaluation result of `verify_bitwise_xor_12_point_evaluation`. The Cairo test uses the
    /// same constant.
    const VERIFY_BITWISE_XOR_12_EVAL_RESULT: QM31 =
        QM31(CM31(M31(1192162821), M31(798775654)), CM31(M31(402281409), M31(1482006777)));

    /// Evaluates the constraints at fixed mask values. The Cairo verifier's hand-written
    /// `verify_bitwise_xor_12` is checked against the same inputs and result, in
    /// `stwo_cairo_verifier/crates/circuit_air/src/components/verify_bitwise_xor_12.cairo`. Keep
    /// the two tests in sync.
    #[test]
    fn verify_bitwise_xor_12_point_evaluation() {
        let preprocessed_values = [
            QM31::from_u32_unchecked(1019132783, 577662261, 1411911446, 1136299686),
            QM31::from_u32_unchecked(1431105596, 751158381, 306231922, 819567871),
            QM31::from_u32_unchecked(23322617, 2140229949, 804779144, 1036244209),
        ];
        let trace_values = [
            QM31::from_u32_unchecked(588513901, 1381709766, 1740209196, 988311219),
            QM31::from_u32_unchecked(1483036430, 1856165774, 1291619160, 488862552),
            QM31::from_u32_unchecked(1198704003, 3577443, 1420176220, 1340529904),
            QM31::from_u32_unchecked(312035360, 2100325867, 945010468, 789533884),
            QM31::from_u32_unchecked(348466428, 729457621, 1945562163, 451511994),
            QM31::from_u32_unchecked(126324276, 1237411384, 1758402834, 428709849),
            QM31::from_u32_unchecked(160222960, 1102775500, 1475204238, 1759325703),
            QM31::from_u32_unchecked(723701119, 1463373183, 868902551, 1713425711),
            QM31::from_u32_unchecked(1809047471, 187583509, 39952429, 1973337145),
            QM31::from_u32_unchecked(130534135, 1760192857, 1965019790, 1421731346),
            QM31::from_u32_unchecked(1092910081, 479412797, 195373735, 2124831833),
            QM31::from_u32_unchecked(2119854101, 910123658, 2033050727, 952931272),
            QM31::from_u32_unchecked(241651259, 1419195850, 908829278, 2085896844),
            QM31::from_u32_unchecked(290059540, 1158547345, 671176026, 1894142936),
            QM31::from_u32_unchecked(1334557888, 1198650582, 351165390, 2096275069),
            QM31::from_u32_unchecked(1496440658, 1901757808, 110280401, 1192176288),
        ];
        // The cumulative sum of each logup batch, at the current row.
        let interaction_values = [
            QM31::from_u32_unchecked(367679456, 1089306077, 1751697368, 179535632),
            QM31::from_u32_unchecked(860107359, 1304465315, 899452500, 1434838909),
            QM31::from_u32_unchecked(1279837932, 1008437098, 1023852013, 1308454055),
            QM31::from_u32_unchecked(825882243, 1162108979, 65486191, 1375952774),
            QM31::from_u32_unchecked(1559776274, 177519278, 2077602132, 412029966),
            QM31::from_u32_unchecked(2096596382, 1690725615, 1425061651, 561703855),
            QM31::from_u32_unchecked(765245467, 2020970709, 779812626, 1860003928),
            QM31::from_u32_unchecked(1482009533, 828057215, 1446916125, 665510814),
        ];
        // The cumulative sum of the last batch, at the previous row.
        let last_batch_prev_row_value =
            QM31::from_u32_unchecked(244453227, 541976962, 504599444, 719906632);
        let claimed_sum = QM31::from_u32_unchecked(780228864, 797287743, 1098116836, 1231001529);
        let random_coeff = QM31::from_u32_unchecked(1739691684, 1643001086, 1073767856, 385276755);

        // Each interaction value is split into its SECURE_EXTENSION_DEGREE base columns. The last
        // batch's columns are also sampled at the previous row.
        let to_columns = |value: QM31| value.to_m31_array().map(QM31::from);
        let (last_value, values) = interaction_values.split_last().unwrap();
        let interaction_mask = values
            .iter()
            .flat_map(|value| to_columns(*value).map(|v| vec![v]))
            .chain(
                to_columns(last_batch_prev_row_value)
                    .into_iter()
                    .zip(to_columns(*last_value))
                    .map(|(prev, cur)| vec![prev, cur]),
            )
            .collect::<Vec<_>>();
        let preprocessed_mask = preprocessed_values.map(|v| vec![v]);
        let trace_mask = trace_values.map(|v| vec![v]);
        let mask = TreeVec::new(vec![
            preprocessed_mask.iter().collect(),
            trace_mask.iter().collect(),
            interaction_mask.iter().collect(),
        ]);

        let eval = Eval {
            claim: Claim {},
            common_lookup_elements: relations::CommonLookupElements::dummy(),
        };
        let mut accumulator = PointEvaluationAccumulator::new(random_coeff);
        // A denominator inverse of 1 gives the plain random linear combination of the constraints.
        eval.evaluate(PointEvaluator::new(
            mask,
            &mut accumulator,
            QM31::one(),
            LOG_SIZE,
            claimed_sum,
        ));

        assert_eq!(accumulator.finalize(), VERIFY_BITWISE_XOR_12_EVAL_RESULT);
    }
}
