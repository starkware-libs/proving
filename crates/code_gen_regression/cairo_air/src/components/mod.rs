use std::sync::atomic::AtomicU32;

use stwo_prover::core::backend::simd::conversion::Pack;
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::backend::Backend;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::pcs::TreeSubspan;
use stwo_prover::core::poly::circle::CircleEvaluation;
use stwo_prover::core::poly::BitReversedOrder;

pub mod add_ap_opcode;
pub mod constraints_regression_test_values;
pub mod jnz_opcode_taken;
pub mod memory_address_to_id;
pub mod memory_id_to_big;
pub mod mul_mod_builtin;
pub mod partial_ec_mul;
pub mod pedersen_points_table;
pub mod prelude;
pub mod range_check_12;
pub mod range_check_18;
pub mod range_check_19;
pub mod range_check_19_b;
pub mod range_check_19_c;
pub mod range_check_19_d;
pub mod range_check_3_6_6_3;
pub mod range_check_4_3;
pub mod range_check_6;
pub mod range_check_7_2_5;
pub mod range_check_8;
pub mod range_check_9_9;
pub mod range_check_9_9_b;
pub mod range_check_9_9_c;
pub mod range_check_9_9_d;
pub mod range_check_builtin_bits_128;
pub mod subroutines;
pub mod triple_xor_32;
pub mod verify_bitwise_xor_8;
pub mod verify_instruction;

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

pub struct AtomicMultiplicityColumn {
    _data: Vec<AtomicU32>,
}
impl AtomicMultiplicityColumn {
    pub fn new(size: usize) -> Self {
        unimplemented!()
    }
    pub fn into_simd_vec(self) -> Vec<PackedM31> {
        unimplemented!()
    }
}
pub trait TreeBuilder<B: Backend> {
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan;
}

#[derive(Clone)]
pub struct RelationUse {
    pub relation_id: &'static str,
    pub uses: u32,
}
