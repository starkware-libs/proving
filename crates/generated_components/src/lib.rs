#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![allow(non_snake_case)]
#![feature(portable_simd)]

use stwo_prover::core::backend::simd::m31::PackedM31;

pub mod narrowfib_num_steps_20;
pub mod widefib_num_narrow_8_narrow_size_20;

// TODO(Ohad): make this 2.
pub const LOGUP_BATCH_SIZE: usize = 1;

pub trait SingleToArray {
    fn into(self) -> [PackedM31; 1];
}

impl SingleToArray for PackedM31 {
    fn into(self) -> [PackedM31; 1] {
        [self]
    }
}
