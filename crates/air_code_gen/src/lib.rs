#![feature(portable_simd)]
#![feature(iter_array_chunks)]
pub mod airs;
pub mod code_gen;
use std::fmt::Debug;

use stwo_prover::core::backend::simd::m31::PackedM31;

// Avoid [PackedM31; 1] all around the code, used for chaining felts in the lookup combine.
pub(crate) trait AirFuncIO {
    type Output: Debug;

    fn io_array(self) -> Self::Output;
}

// Implement the trait for arrays.
impl<const N: usize> AirFuncIO for [PackedM31; N] {
    type Output = Self;

    fn io_array(self) -> Self::Output {
        self
    }
}

// Implement the trait for non-array.
impl AirFuncIO for PackedM31 {
    type Output = [PackedM31; 1];

    fn io_array(self) -> Self::Output {
        [self]
    }
}
