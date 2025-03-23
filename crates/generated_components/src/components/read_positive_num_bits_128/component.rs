use crate::components::prelude::constraint_eval::*;
use crate::components::range_check_last_limb_bits_in_ms_limb_2::component::RangeCheckLastLimbBitsInMsLimb2;

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
pub struct ReadPositiveNumBits128 {}

impl ReadPositiveNumBits128 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    pub fn evaluate<E: EvalAtRow>(
        read_positive_num_bits_128_input: E::F,
        eval: &mut E,
        memory_address_to_id_lookup_elements: &relations::MemoryAddressToId,
        memory_id_to_big_lookup_elements: &relations::MemoryIdToBig,
    ) -> [E::F; 29] {
        let M31_0 = E::F::from(M31::from(0));
        let id_col0 = eval.next_trace_mask();
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

        eval.add_to_relation(RelationEntry::new(
            memory_address_to_id_lookup_elements,
            E::EF::one(),
            &[read_positive_num_bits_128_input.clone(), id_col0.clone()],
        ));

        let () = RangeCheckLastLimbBitsInMsLimb2::evaluate(value_limb_14_col15.clone(), eval);
        eval.add_to_relation(RelationEntry::new(
            memory_id_to_big_lookup_elements,
            E::EF::one(),
            &[
                id_col0.clone(),
                value_limb_0_col1.clone(),
                value_limb_1_col2.clone(),
                value_limb_2_col3.clone(),
                value_limb_3_col4.clone(),
                value_limb_4_col5.clone(),
                value_limb_5_col6.clone(),
                value_limb_6_col7.clone(),
                value_limb_7_col8.clone(),
                value_limb_8_col9.clone(),
                value_limb_9_col10.clone(),
                value_limb_10_col11.clone(),
                value_limb_11_col12.clone(),
                value_limb_12_col13.clone(),
                value_limb_13_col14.clone(),
                value_limb_14_col15.clone(),
            ],
        ));

        [
            value_limb_0_col1.clone(),
            value_limb_1_col2.clone(),
            value_limb_2_col3.clone(),
            value_limb_3_col4.clone(),
            value_limb_4_col5.clone(),
            value_limb_5_col6.clone(),
            value_limb_6_col7.clone(),
            value_limb_7_col8.clone(),
            value_limb_8_col9.clone(),
            value_limb_9_col10.clone(),
            value_limb_10_col11.clone(),
            value_limb_11_col12.clone(),
            value_limb_12_col13.clone(),
            value_limb_13_col14.clone(),
            value_limb_14_col15.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            M31_0.clone(),
            id_col0.clone(),
        ]
    }
}
