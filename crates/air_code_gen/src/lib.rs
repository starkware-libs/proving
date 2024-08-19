#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
pub mod airs;
pub mod code_gen;
use std::ops::Index;

use num_traits::Zero;
use stwo_prover::core::backend::simd::m31::PackedM31;

#[derive(Clone, Copy)]
pub struct AirFnIO<const N: usize>(pub [PackedM31; N]);
impl<const N: usize> AirFnIO<N> {
    pub fn concat<const M: usize>(&self, other: &AirFnIO<M>) -> AirFnIO<{ N + M }> {
        let mut res = [PackedM31::zero(); N + M];
        res[..N].copy_from_slice(&self.0);
        res[N..].copy_from_slice(&other.0);
        res.into()
    }
}

impl From<AirFnIO<1>> for PackedM31 {
    fn from(p: AirFnIO<1>) -> Self {
        p[0]
    }
}

impl From<PackedM31> for AirFnIO<1> {
    fn from(p: PackedM31) -> Self {
        Self([p])
    }
}

impl<const N: usize> From<[PackedM31; N]> for AirFnIO<N> {
    fn from(arr: [PackedM31; N]) -> Self {
        Self(arr)
    }
}

impl<const N: usize> From<AirFnIO<N>> for [PackedM31; N] {
    fn from(afio: AirFnIO<N>) -> Self {
        afio.0
    }
}

impl<const N: usize> AsRef<[PackedM31]> for AirFnIO<N> {
    fn as_ref(&self) -> &[PackedM31] {
        &self.0
    }
}

impl<const N: usize> Index<usize> for AirFnIO<N> {
    type Output = PackedM31;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
