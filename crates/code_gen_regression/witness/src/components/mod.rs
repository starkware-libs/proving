use stwo::core::fields::m31::M31;
use stwo::core::pcs::TreeSubspan;
use stwo::prover::backend::simd::conversion::Pack;
use stwo::prover::backend::simd::m31::PackedM31;
use stwo::prover::backend::Backend;
use stwo::prover::poly::circle::CircleEvaluation;
use stwo::prover::poly::BitReversedOrder;
use stwo_cairo_common::prover_types::simd::PackedFelt252;

pub mod add_ap_opcode;
pub mod jnz_opcode_taken;
pub mod memory_address_to_id;
pub mod memory_id_to_big;
pub mod mul_mod_builtin;
pub mod partial_ec_mul;
pub mod pedersen_points_table;
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
    pub const fn new(_padding_offset: usize) -> Self {
        unimplemented!()
    }

    pub fn packed_at(&self, _vec_row: usize) -> PackedM31 {
        unimplemented!()
    }
}

pub trait TreeBuilder<B: Backend> {
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan;
}

pub struct AtomicMultiplicityColumn {}
impl AtomicMultiplicityColumn {
    pub const fn new(size: usize) -> Self {
        Self {}
    }
    pub fn into_simd_vec(&self) -> Vec<PackedM31> {
        vec![]
    }
}

pub struct PackedPedersenPointsTable {}
impl PackedPedersenPointsTable {
    pub fn deduce_output([input]: [PackedM31; 1]) -> [PackedFelt252; 2] {
        unimplemented!()
    }
}
pub mod range_check_11;
pub mod range_check_18_b;
pub mod range_check_19_e;
pub mod range_check_19_f;
pub mod range_check_19_g;
pub mod range_check_19_h;
pub mod range_check_9_9_e;
pub mod range_check_9_9_f;
pub mod range_check_9_9_g;
pub mod range_check_9_9_h;
