use crate::components::decode_instruction_8ad7e540e219b042::component::DecodeInstruction8Ad7E540E219B042;
use crate::components::prelude::constraint_eval::*;
use crate::components::read_small::component::ReadSmall;

pub(super) const N_TRACE_COLUMNS: usize = 10;

pub struct Eval {
    pub claim: Claim,
    pub memory_address_to_id_lookup_elements: relations::MemoryAddressToId,
    pub memory_id_to_big_lookup_elements: relations::MemoryIdToBig,
    pub opcodes_lookup_elements: relations::Opcodes,
    pub verify_instruction_lookup_elements: relations::VerifyInstruction,
}

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
pub struct Claim {
    pub log_size: u32,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let trace_log_sizes = vec![self.log_size; N_TRACE_COLUMNS];
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 3];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_log_sizes])
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.log_size as u64);
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
        let M31_1 = E::F::from(M31::from(1));
        let M31_134217728 = E::F::from(M31::from(134217728));
        let M31_2 = E::F::from(M31::from(2));
        let M31_262144 = E::F::from(M31::from(262144));
        let M31_512 = E::F::from(M31::from(512));
        let input_pc_col0 = eval.next_trace_mask();
        let input_ap_col1 = eval.next_trace_mask();
        let input_fp_col2 = eval.next_trace_mask();
        let op1_id_col3 = eval.next_trace_mask();
        let msb_col4 = eval.next_trace_mask();
        let mid_limbs_set_col5 = eval.next_trace_mask();
        let op1_limb_0_col6 = eval.next_trace_mask();
        let op1_limb_1_col7 = eval.next_trace_mask();
        let op1_limb_2_col8 = eval.next_trace_mask();
        let padding = eval.next_trace_mask();

        eval.add_constraint(padding.clone() * padding.clone() - padding.clone());

        let [decode_instruction_8ad7e540e219b042_output_limb_0, decode_instruction_8ad7e540e219b042_output_limb_1, decode_instruction_8ad7e540e219b042_output_limb_2, decode_instruction_8ad7e540e219b042_output_limb_3, decode_instruction_8ad7e540e219b042_output_limb_4, decode_instruction_8ad7e540e219b042_output_limb_5, decode_instruction_8ad7e540e219b042_output_limb_6, decode_instruction_8ad7e540e219b042_output_limb_7, decode_instruction_8ad7e540e219b042_output_limb_8, decode_instruction_8ad7e540e219b042_output_limb_9, decode_instruction_8ad7e540e219b042_output_limb_10, decode_instruction_8ad7e540e219b042_output_limb_11, decode_instruction_8ad7e540e219b042_output_limb_12, decode_instruction_8ad7e540e219b042_output_limb_13, decode_instruction_8ad7e540e219b042_output_limb_14, decode_instruction_8ad7e540e219b042_output_limb_15, decode_instruction_8ad7e540e219b042_output_limb_16, decode_instruction_8ad7e540e219b042_output_limb_17, decode_instruction_8ad7e540e219b042_output_limb_18] =
            DecodeInstruction8Ad7E540E219B042::evaluate(
                input_pc_col0.clone(),
                &mut eval,
                &self.verify_instruction_lookup_elements,
            );
        let [read_small_output_limb_0, read_small_output_limb_1] = ReadSmall::evaluate(
            (input_pc_col0.clone() + M31_1.clone()),
            &mut eval,
            &self.memory_address_to_id_lookup_elements,
            &self.memory_id_to_big_lookup_elements,
        );
        eval.add_to_relation(RelationEntry::new(
            &self.opcodes_lookup_elements,
            E::EF::from(padding.clone()),
            &[
                input_pc_col0.clone(),
                input_ap_col1.clone(),
                input_fp_col2.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.opcodes_lookup_elements,
            -E::EF::from(padding.clone()),
            &[
                (input_pc_col0.clone() + M31_2.clone()),
                (input_ap_col1.clone()
                    + ((((op1_limb_0_col6.clone()
                        + (op1_limb_1_col7.clone() * M31_512.clone()))
                        + (op1_limb_2_col8.clone() * M31_262144.clone()))
                        - msb_col4.clone())
                        - (M31_134217728.clone() * mid_limbs_set_col5.clone()))),
                input_fp_col2.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}
