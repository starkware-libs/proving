use crate::components::prelude::constraint_eval::*;
use crate::components::read_positive_num_bits_128::component::ReadPositiveNumBits128;

pub(super) const N_TRACE_COLUMNS: usize = 17;

pub struct Eval {
    pub claim: Claim,
    pub memory_address_to_id_lookup_elements: relations::MemoryAddressToId,
    pub memory_id_to_big_lookup_elements: relations::MemoryIdToBig,
}

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
pub struct Claim {
    pub log_size: u32,
    pub range_check_builtin_segment_start: u32,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let trace_log_sizes = vec![self.log_size; N_TRACE_COLUMNS];
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_log_sizes])
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.log_size as u64);
        channel.mix_u64(self.range_check_builtin_segment_start as u64);
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
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
        self.claim.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let seq = eval.get_preprocessed_column(Seq::new(self.log_size()).id());
        let value_id_col0 = eval.next_trace_mask();
        let value_limb_0_col1 = eval.next_trace_mask();
        let value_limb_1_col2 = eval.next_trace_mask();
        let value_limb_2_col3 = eval.next_trace_mask();
        let value_limb_3_col4 = eval.next_trace_mask();
        let value_limb_4_col5 = eval.next_trace_mask();
        let value_limb_5_col6 = eval.next_trace_mask();
        let value_limb_6_col7 = eval.next_trace_mask();
        let value_limb_7_col8 = eval.next_trace_mask();
        let value_limb_8_col9 = eval.next_trace_mask();
        let value_limb_9_col10 = eval.next_trace_mask();
        let value_limb_10_col11 = eval.next_trace_mask();
        let value_limb_11_col12 = eval.next_trace_mask();
        let value_limb_12_col13 = eval.next_trace_mask();
        let value_limb_13_col14 = eval.next_trace_mask();
        let value_limb_14_col15 = eval.next_trace_mask();
        let msb_col16 = eval.next_trace_mask();

        let [read_positive_num_bits_128_output_limb_0, read_positive_num_bits_128_output_limb_1, read_positive_num_bits_128_output_limb_2, read_positive_num_bits_128_output_limb_3, read_positive_num_bits_128_output_limb_4, read_positive_num_bits_128_output_limb_5, read_positive_num_bits_128_output_limb_6, read_positive_num_bits_128_output_limb_7, read_positive_num_bits_128_output_limb_8, read_positive_num_bits_128_output_limb_9, read_positive_num_bits_128_output_limb_10, read_positive_num_bits_128_output_limb_11, read_positive_num_bits_128_output_limb_12, read_positive_num_bits_128_output_limb_13, read_positive_num_bits_128_output_limb_14, read_positive_num_bits_128_output_limb_15, read_positive_num_bits_128_output_limb_16, read_positive_num_bits_128_output_limb_17, read_positive_num_bits_128_output_limb_18, read_positive_num_bits_128_output_limb_19, read_positive_num_bits_128_output_limb_20, read_positive_num_bits_128_output_limb_21, read_positive_num_bits_128_output_limb_22, read_positive_num_bits_128_output_limb_23, read_positive_num_bits_128_output_limb_24, read_positive_num_bits_128_output_limb_25, read_positive_num_bits_128_output_limb_26, read_positive_num_bits_128_output_limb_27, read_positive_num_bits_128_output_limb_28] =
            ReadPositiveNumBits128::evaluate(
                (E::F::from(M31::from(self.claim.range_check_builtin_segment_start)) + seq.clone()),
                &mut eval,
                &self.memory_address_to_id_lookup_elements,
                &self.memory_id_to_big_lookup_elements,
            );
        eval.finalize_logup_in_pairs();
        eval
    }
}
