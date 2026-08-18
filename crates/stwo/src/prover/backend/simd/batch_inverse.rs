//! Chunked batch inversion shared by the packed field types.

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use super::m31::PackedM31;
use crate::core::utils::uninit_vec;

// Optimal chunk sizes were determined empirically on an intel 155u machine, except the QM31 one,
// re-measured on a sapphire rapids xeon: the norm descent adds scratch buffers to the working
// set, so it wants a smaller chunk than the ~1 << 11 the old algorithm did.
pub(super) const PACKED_M31_BATCH_INVERSE_CHUNK_SIZE: usize = 1 << 9;
pub(super) const PACKED_CM31_BATCH_INVERSE_CHUNK_SIZE: usize = 1 << 10;
pub(super) const PACKED_QM31_BATCH_INVERSE_CHUNK_SIZE: usize = 1 << 10;

/// Two [`PackedM31`] buffers, one chunk long, for the extension field batch inversions to hold
/// base field norms and their inverses in.
pub(super) struct BatchInverseScratch {
    base_norms: Vec<PackedM31>,
    base_norm_invs: Vec<PackedM31>,
}

impl BatchInverseScratch {
    fn new(len: usize) -> Self {
        Self { base_norms: unsafe { uninit_vec(len) }, base_norm_invs: unsafe { uninit_vec(len) } }
    }

    /// The norm and norm inverse buffers, both truncated to `len`.
    ///
    /// The buffers are allocated once per task at the full chunk length, so the final chunk of a
    /// column whose length is not a multiple of it needs the truncation: without it the tail
    /// would batch invert uninitialized elements.
    pub(super) fn buffers(&mut self, len: usize) -> (&mut [PackedM31], &mut [PackedM31]) {
        (&mut self.base_norms[..len], &mut self.base_norm_invs[..len])
    }
}

/// Splits `column` into chunks small enough for their intermediates to stay in cache, and
/// inverts each with `invert_chunk`, in parallel when the `parallel` feature is on.
///
/// `invert_chunk` receives a chunk, its destination, and scratch space to hold the chunk's base
/// field norms and their inverses — extension field inversion reduces to inverting those norms
/// (see [`super::cm31`] and [`super::qm31`]).
pub(super) fn batch_inverse_via_base_norms<T: Send + Sync>(
    column: &[T],
    dst: &mut [T],
    chunk_size: usize,
    invert_chunk: impl Fn(&[T], &mut [T], &mut BatchInverseScratch) + Send + Sync,
) {
    assert!(column.len() <= dst.len());
    let scratch_len = column.len().min(chunk_size);

    #[cfg(not(feature = "parallel"))]
    {
        let mut scratch = BatchInverseScratch::new(scratch_len);
        dst.chunks_mut(chunk_size)
            .zip(column.chunks(chunk_size))
            .for_each(|(dst, column)| invert_chunk(column, dst, &mut scratch));
    }

    #[cfg(feature = "parallel")]
    dst.par_chunks_mut(chunk_size).zip(column.par_chunks(chunk_size)).for_each_init(
        || BatchInverseScratch::new(scratch_len),
        |scratch, (dst, column)| invert_chunk(column, dst, scratch),
    );
}
