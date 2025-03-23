use stwo_prover::core::backend::simd::conversion::Pack;
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::backend::Backend;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::pcs::TreeSubspan;
use stwo_prover::core::poly::circle::CircleEvaluation;
use stwo_prover::core::poly::BitReversedOrder;

pub mod add_ap_opcode_imm;
pub mod cond_decode_small_sign;
pub mod decode_instruction_8ad7e540e219b042;
pub mod decode_instruction_e03055818c3f043;
pub mod encode_offsets;
pub mod jnz_opcode_taken_dst_base_fp;
pub mod mem_verify;
pub mod memory_address_to_id;
pub mod memory_id_to_big;
pub mod narrow_fib_num_steps_20;
mod prelude;
pub mod range_check_4_3;
pub mod range_check_6;
pub mod range_check_7_2_5;
pub mod range_check_builtin_bits_128;
pub mod range_check_last_limb_bits_in_ms_limb_2;
pub mod read_positive_num_bits_128;
pub mod read_positive_num_bits_252;
pub mod read_small;
pub mod verify_instruction;
pub mod wide_fib_num_narrow_8_narrow_size_20;

// TODO(Ohad): remove.
pub fn pack_values<T: Pack>(_values: &[T]) -> Vec<T::SimdType> {
    unimplemented!()
}

#[derive(Debug, Clone)]
pub struct Enabler {
    pub padding_offset: usize,
}
impl Enabler {
    pub const fn new(padding_offset: usize) -> Self {
        unimplemented!()
    }

    pub fn packed_at(&self, vec_row: usize) -> PackedM31 {
        unimplemented!()
    }
}

pub trait TreeBuilder<B: Backend> {
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan;
}
