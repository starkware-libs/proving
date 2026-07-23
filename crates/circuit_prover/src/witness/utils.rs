use rayon::iter::ParallelIterator;
use rayon::slice::ParallelSlice;
use stwo::prover::backend::simd::conversion::Pack;
use stwo::prover::backend::simd::m31::N_LANES;

pub fn pack_values<T: Pack + Copy + Sync>(values: &[T]) -> Vec<T::SimdType>
where
    T::SimdType: Send,
{
    assert!(
        values.len().is_multiple_of(N_LANES),
        "pack_values: length {} is not a multiple of N_LANES ({N_LANES})",
        values.len(),
    );
    values.par_chunks_exact(N_LANES).map(|c| T::pack(c.try_into().unwrap())).collect()
}
