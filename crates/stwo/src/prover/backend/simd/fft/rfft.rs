//! Regular (forward) fft.

use std::array;
use std::cell::RefCell;
use std::simd::{simd_swizzle, u32x2, u32x4, u32x8, u32x16};

use itertools::Itertools;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

use super::{
    CACHED_FFT_LOG_SIZE, MIN_FFT_LOG_SIZE, TwiddleTreeView, compute_first_twiddles, mul_twiddle,
    transpose_vecs,
};
use crate::core::circle::Coset;
use crate::core::utils::bit_reverse;
use crate::parallel_iter;
use crate::prover::backend::simd::m31::{LOG_N_LANES, PackedBaseField};
use crate::prover::backend::simd::utils::{UnsafeConst, UnsafeMut};

/// Performs a Circle Fast Fourier Transform (CFFT) on the given values.
///
/// # Arguments
///
/// * `src`: A pointer to the values to transform.
/// * `dst`: A pointer to the destination array.
/// * `twiddle_dbl`: The tree of the doubles of the twiddle factors of the domain.
/// * `log_n_elements`: The log of the number of elements in the `values` array.
///
/// # Panics
///
/// This function will panic if `log_n_elements` is less than `MIN_FFT_LOG_SIZE`, or if
/// `twiddle_dbl` does not have `log_n_elements - 1` layers.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`].
pub unsafe fn fft(
    src: *const u32,
    dst: *mut u32,
    twiddle_dbl: TwiddleTreeView<'_, u32>,
    log_n_elements: usize,
) {
    assert!(log_n_elements >= MIN_FFT_LOG_SIZE as usize);
    assert_eq!(twiddle_dbl.n_layers(), log_n_elements - 1);
    let log_n_vecs = log_n_elements - LOG_N_LANES as usize;
    if log_n_elements <= CACHED_FFT_LOG_SIZE as usize {
        fft_lower_with_vecwise(src, dst, twiddle_dbl, log_n_elements, log_n_elements);
        return;
    }

    let fft_layers_pre_transpose = log_n_vecs.div_ceil(2);
    let fft_layers_post_transpose = log_n_vecs / 2;
    fft_lower_without_vecwise(
        src,
        dst,
        twiddle_dbl.skip_layers(3 + fft_layers_pre_transpose),
        log_n_elements,
        fft_layers_post_transpose,
    );
    transpose_vecs(dst, log_n_vecs);
    fft_lower_with_vecwise(
        dst,
        dst,
        twiddle_dbl,
        log_n_elements,
        fft_layers_pre_transpose + LOG_N_LANES as usize,
    );
}

/// The log of the target size in M31 elements of the buffer in which a task of [`fft_subdomains`]
/// computes the first phase of its blocks.
const SUBDOMAINS_SCRATCH_LOG_ELEMENTS: usize = 15;
/// The minimum log of the number of blocks of a task of [`fft_subdomains`]: at least 2 blocks, so
/// that each transposed store is a run of at least 2 vectors (128 B). For blocks larger than half
/// the scratch target (ffts above 2^25), the scratch grows past the target to hold 2 blocks.
const MIN_LOG_BLOCKS_PER_TASK: usize = 1;
/// The maximum log of the number of blocks of a task of [`fft_subdomains`]: at most 16 blocks
/// (runs of up to 1 KiB). Larger tasks leave fewer tasks for small ffts (8 at 2^17–2^18); 16 was
/// the fastest for many-column extension in a sweep of 1 to 16 blocks.
const MAX_LOG_BLOCKS_PER_TASK: usize = 4;
/// The log of the number of elements in a 4 KiB page.
const LOG_PAGE_ELEMENTS: usize = 10;
/// The log of the number of pages that one task of [`fft_subdomains`] touches before the fft.
const LOG_PREFAULT_PAGES: usize = 5;

thread_local! {
    /// The scratch buffer of [`fft_subdomains`], kept by each thread across calls.
    static SUBDOMAINS_SCRATCH: RefCell<Vec<u32x16>> = const { RefCell::new(Vec::new()) };
}

/// Performs [`fft`] of the same values for the `2^log_n_subdomains` subdomains of a domain, where
/// `log_n_subdomains = twiddles.n_layers() + 1 - log_n_elements`: the evaluation on subdomain `i`,
/// with the twiddles `twiddles.subdomain(log_n_subdomains, i)`, is written to
/// `dst + (i << log_n_elements)`.
///
/// The result equals calling [`fft`] once per subdomain. For sizes above [`CACHED_FFT_LOG_SIZE`],
/// the first phase of the fft is fused with the transposition after it. Each task takes a few
/// neighbouring blocks of `src`. For each subdomain, it reads the blocks, transforms them using the
/// scratch buffer, and stores the result to its transposed place in `dst` in contiguous runs. The
/// blocks should be small enough to stay in the cache, so that `src` is read from memory once for
/// all the subdomains.
///
/// # Arguments
///
/// * `src`: A pointer to the values to transform, aligned to 64 bytes.
/// * `dst`: A pointer to the destination arrays, aligned to 64 bytes.
/// * `twiddles`: The tree of the doubles of the twiddle factors of the whole domain.
/// * `log_n_elements`: The log of the number of elements in `src`.
///
/// # Panics
///
/// This function will panic if `log_n_elements` is less than `MIN_FFT_LOG_SIZE`, or if `twiddles`
/// has fewer than `log_n_elements - 1` layers.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`],
/// if `src` is not valid for reads of `1 << log_n_elements` elements, if `dst` does not have room
/// for `1 << (twiddles.n_layers() + 1)` elements, or if `dst` overlaps `src`.
pub unsafe fn fft_subdomains(
    src: *const u32,
    dst: *mut u32,
    twiddles: TwiddleTreeView<'_, u32>,
    log_n_elements: usize,
) {
    fft_subdomains_with_scratch_log(
        src,
        dst,
        twiddles,
        log_n_elements,
        SUBDOMAINS_SCRATCH_LOG_ELEMENTS,
    );
}

/// [`fft_subdomains`] with the log of the size in M31 elements of the scratch buffer as a
/// parameter.
///
/// # Safety
///
/// See [`fft_subdomains`].
unsafe fn fft_subdomains_with_scratch_log(
    src: *const u32,
    dst: *mut u32,
    twiddles: TwiddleTreeView<'_, u32>,
    log_n_elements: usize,
    scratch_log_elements: usize,
) {
    assert!(log_n_elements >= MIN_FFT_LOG_SIZE as usize);
    // Each subdomain's tree has `log_n_elements - 1` layers.
    let log_n_subdomains =
        (twiddles.n_layers() + 1).checked_sub(log_n_elements).expect("Not enough twiddles!");
    let subdomains = twiddles.subdomains_iter(log_n_subdomains as u32);
    if log_n_elements <= CACHED_FFT_LOG_SIZE as usize {
        for (i, subdomain_twiddles) in subdomains.enumerate() {
            fft(src, dst.add(i << log_n_elements), subdomain_twiddles, log_n_elements);
        }
        return;
    }

    // As in `transpose_vecs`, a vector index is (a, b, c) with |a| = |c| = n_post_transpose_layers
    // and |b| = has_middle_bit: log_n_vecs = 2 * n_post_transpose_layers + has_middle_bit, so
    // has_middle_bit is 0 or 1, and b is the middle bit left over when log_n_vecs is odd. The first
    // phase applies the layers that act on c, so a block (a, b) has 2^n_post_transpose_layers
    // vectors, and the transposition moves the vector (a, b, c) to (c, b, a). As in `fft`, the
    // layer counts are named after their place in the ifft: here the post-transpose layers run
    // first.
    let log_n_vecs = log_n_elements - LOG_N_LANES as usize;
    let n_post_transpose_layers = log_n_vecs / 2;
    let n_pre_transpose_layers = log_n_vecs - n_post_transpose_layers;
    let has_middle_bit = log_n_vecs & 1;
    let block_vecs = 1usize << n_post_transpose_layers;
    let block_elems = block_vecs << LOG_N_LANES;
    // Each task transforms 2^log_blocks_per_task blocks with consecutive `a` into the scratch, so
    // that their transposed vectors land in contiguous runs: as many blocks as fit in the scratch
    // target, but 2 to 16 of them.
    let log_blocks_per_task = scratch_log_elements
        .saturating_sub(n_post_transpose_layers + LOG_N_LANES as usize)
        .clamp(MIN_LOG_BLOCKS_PER_TASK, MAX_LOG_BLOCKS_PER_TASK);
    // A task's blocks share `b` and have consecutive `a`, which has `n_post_transpose_layers` bits.
    assert!(log_blocks_per_task <= n_post_transpose_layers);
    let blocks_per_task = 1usize << log_blocks_per_task;
    let n_tasks = 1usize << (n_post_transpose_layers - log_blocks_per_task + has_middle_bit);

    let src = UnsafeConst(src);
    let dsts = (0..1usize << log_n_subdomains)
        .map(|i| UnsafeMut(dst.add(i << log_n_elements)))
        .collect_vec();

    // The first phase stores to each page of `dst` from several tasks at about the same time, and
    // if the page is not mapped yet, each of them takes a page fault for it. Touching every page
    // first, from a single task, avoids the repeated faults.
    let log_chunk = LOG_PAGE_ELEMENTS + LOG_PREFAULT_PAGES;
    let n_chunks = 1usize << (log_n_elements - log_chunk);
    parallel_iter!(0..dsts.len() * n_chunks).for_each(|task| {
        let chunk = dsts[task / n_chunks].get().add((task % n_chunks) << log_chunk);
        for page in 0..1usize << LOG_PREFAULT_PAGES {
            chunk.add(page << LOG_PAGE_ELEMENTS).write(0);
        }
    });

    let first_phase = |scratch: &mut [u32x16], task: usize| {
        let src = src.get();
        let scr = scratch.as_mut_ptr() as *mut u32;
        // `has_middle_bit` is 0 or 1, so it is also the mask of b.
        let b = task & has_middle_bit;
        let a0 = (task >> has_middle_bit) << log_blocks_per_task;
        for (dst, twiddle_dbl) in dsts.iter().zip(subdomains.clone()) {
            let dst = dst.get();
            let twiddle_dbl = twiddle_dbl.skip_layers(3 + n_pre_transpose_layers);
            // The layers of `fft_lower_without_vecwise` on the blocks (a, b) of the task.
            for i in 0..blocks_per_task {
                let index_h = ((a0 + i) << has_middle_bit) | b;
                fft_block_without_vecwise(
                    src.add(index_h * block_elems),
                    scr.add(i * block_elems),
                    twiddle_dbl,
                    n_post_transpose_layers,
                    index_h,
                );
            }
            // The vector c of the block (a, b) goes to (c, b, a).
            for c in 0..block_vecs {
                let out = dst.add(
                    ((c << n_pre_transpose_layers) | (b << n_post_transpose_layers) | a0)
                        << LOG_N_LANES,
                );
                for i in 0..blocks_per_task {
                    PackedBaseField::load(scr.add(i * block_elems + (c << LOG_N_LANES)))
                        .store(out.add(i << LOG_N_LANES));
                }
            }
        }
    };

    let scratch_len = blocks_per_task * block_vecs;
    parallel_iter!(0..n_tasks).for_each(|task| {
        SUBDOMAINS_SCRATCH.with(|cell| match cell.try_borrow_mut() {
            Ok(mut scratch) => {
                if scratch.len() < scratch_len {
                    scratch.resize(scratch_len, u32x16::splat(0));
                }
                first_phase(&mut scratch, task);
            }
            // Only reachable if a task of this thread is re-entered; use a buffer of its own.
            Err(_) => first_phase(&mut vec![u32x16::splat(0); scratch_len], task),
        })
    });

    for (dst, twiddle_dbl) in dsts.iter().zip(subdomains) {
        let dst = dst.get();
        fft_lower_with_vecwise(
            dst,
            dst,
            twiddle_dbl,
            log_n_elements,
            n_pre_transpose_layers + LOG_N_LANES as usize,
        );
    }
}

/// Computes partial fft on `2^log_size` M31 elements.
///
/// # Arguments
///
/// - `src`: A pointer to the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the destination array, aligned to 64 bytes.
/// - `twiddle_dbl`: The doubles of the twiddle factors for each layer of the the fft. Layer `i`
///   holds `2^(log_size - 1 - i)` twiddles.
/// - `log_size`: The log of the number of number of M31 elements in the array.
/// - `fft_layers`: The number of fft layers to apply, out of log_size.
///
/// # Panics
///
/// Panics if `log_size` is not at least 5, or if `twiddle_dbl` does not have `log_size - 1` layers.
///
/// # Safety
///
/// `src` and `dst` must have same alignment as [`PackedBaseField`].
/// `fft_layers` must be at least 5.
pub unsafe fn fft_lower_with_vecwise(
    src: *const u32,
    dst: *mut u32,
    twiddle_dbl: TwiddleTreeView<'_, u32>,
    log_size: usize,
    fft_layers: usize,
) {
    const VECWISE_FFT_BITS: usize = LOG_N_LANES as usize + 1;
    assert!(log_size >= VECWISE_FFT_BITS);

    assert_eq!(twiddle_dbl.n_layers(), log_size - 1);

    let src = UnsafeConst(src);
    let dst = UnsafeMut(dst);
    parallel_iter!(0..1 << (log_size - fft_layers)).for_each(|index_h| {
        let block = index_h << fft_layers;
        let mut src = src.get().add(block);
        let dst = dst.get().add(block);
        for layer in (VECWISE_FFT_BITS..fft_layers).step_by(3).rev() {
            match fft_layers - layer {
                1 => {
                    fft1_loop(src, dst, twiddle_dbl.skip_layers(layer - 1), layer, index_h);
                }
                2 => {
                    fft2_loop(src, dst, twiddle_dbl.skip_layers(layer - 1), layer, index_h);
                }
                _ => {
                    fft3_loop(
                        src,
                        dst,
                        twiddle_dbl.skip_layers(layer - 1),
                        fft_layers - layer - 3,
                        layer,
                        index_h,
                    );
                }
            }
            src = dst;
        }
        fft_vecwise_loop(src, dst, twiddle_dbl, fft_layers - VECWISE_FFT_BITS, index_h);
    });
}

/// Computes partial fft on `2^log_size` M31 elements, skipping the vecwise layers (lower 4 bits of
/// the index).
///
/// # Arguments
///
/// - `src`: A pointer to the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the destination array, aligned to 64 bytes.
/// - `twiddle_dbl`: The doubles of the twiddle factors for each layer of the the fft.
/// - `log_size`: The log of the number of number of M31 elements in the array.
/// - `fft_layers`: The number of fft layers to apply, out of log_size - VEC_LOG_SIZE.
///
/// # Panics
///
/// Panics if `log_size` is not at least 4.
///
/// # Safety
///
/// `src` and `dst` must have same alignment as [`PackedBaseField`].
/// `fft_layers` must be at least 4.
/// `twiddle_dbl` must have at least `fft_layers` layers.
pub unsafe fn fft_lower_without_vecwise(
    src: *const u32,
    dst: *mut u32,
    twiddle_dbl: TwiddleTreeView<'_, u32>,
    log_size: usize,
    fft_layers: usize,
) {
    assert!(log_size >= LOG_N_LANES as usize);

    let src = UnsafeConst(src);
    let dst = UnsafeMut(dst);
    parallel_iter!(0..1 << (log_size - fft_layers - LOG_N_LANES as usize)).for_each(|index_h| {
        let block = index_h << (fft_layers + LOG_N_LANES as usize);
        fft_block_without_vecwise(
            src.get().add(block),
            dst.get().add(block),
            twiddle_dbl,
            fft_layers,
            index_h,
        );
    });
}

/// Applies `fft_layers` fft layers, skipping the vecwise layers, to the block of `index_h`: the
/// block-wise part of [`fft_lower_without_vecwise`].
///
/// # Arguments
///
/// - `src`: A pointer to the block of `index_h` in the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the block of `index_h` in the destination array, aligned to 64 bytes.
/// - `twiddle_dbl`: The doubles of the twiddle factors for each layer of the fft.
/// - `fft_layers`: The number of fft layers to apply.
/// - `index_h`: The index of the block.
///
/// # Safety
///
/// `src` and `dst` must have same alignment as [`PackedBaseField`], `src` must be valid for reads
/// and `dst` for writes of `2^(fft_layers + LOG_N_LANES)` elements, and `src` and `dst` must be
/// equal or not overlap.
/// `twiddle_dbl` must have at least `fft_layers` layers.
#[inline(always)]
unsafe fn fft_block_without_vecwise(
    mut src: *const u32,
    dst: *mut u32,
    twiddle_dbl: TwiddleTreeView<'_, u32>,
    fft_layers: usize,
    index_h: usize,
) {
    for layer in (0..fft_layers).step_by(3).rev() {
        let fixed_layer = layer + LOG_N_LANES as usize;
        match fft_layers - layer {
            1 => {
                fft1_loop(src, dst, twiddle_dbl.skip_layers(layer), fixed_layer, index_h);
            }
            2 => {
                fft2_loop(src, dst, twiddle_dbl.skip_layers(layer), fixed_layer, index_h);
            }
            _ => {
                fft3_loop(
                    src,
                    dst,
                    twiddle_dbl.skip_layers(layer),
                    fft_layers - layer - 3,
                    fixed_layer,
                    index_h,
                );
            }
        }
        src = dst;
    }
}

/// Runs the last 5 fft layers across the entire array.
///
/// # Arguments
///
/// - `src`: A pointer to the block of `index_h` in the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the block of `index_h` in the destination array, aligned to 64 bytes.
/// - `twiddle_dbl`: The doubles of the twiddle factors for each of the 5 fft layers.
/// - `high_bits`: The number of bits this loops needs to run on.
/// - `index_h`: The higher part of the index, iterated by the caller.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`].
unsafe fn fft_vecwise_loop(
    src: *const u32,
    dst: *mut u32,
    twiddle_dbl: TwiddleTreeView<'_, u32>,
    loop_bits: usize,
    index_h: usize,
) {
    let twiddle_dbl0 = twiddle_dbl.layer(0);
    let twiddle_dbl1 = twiddle_dbl.layer(1);
    let twiddle_dbl2 = twiddle_dbl.layer(2);
    let twiddle_dbl3 = twiddle_dbl.layer(3);
    for index_l in 0..1 << loop_bits {
        let index = (index_h << loop_bits) + index_l;
        let mut val0 = PackedBaseField::load(src.add(index_l * 32));
        let mut val1 = PackedBaseField::load(src.add(index_l * 32 + 16));
        (val0, val1) =
            simd_butterfly(val0, val1, u32x16::splat(*twiddle_dbl3.get_unchecked(index)));
        (val0, val1) = vecwise_butterflies(
            val0,
            val1,
            array::from_fn(|i| *twiddle_dbl0.get_unchecked(index * 8 + i)),
            array::from_fn(|i| *twiddle_dbl1.get_unchecked(index * 4 + i)),
            array::from_fn(|i| *twiddle_dbl2.get_unchecked(index * 2 + i)),
        );
        val0.store(dst.add(index_l * 32));
        val1.store(dst.add(index_l * 32 + 16));
    }
}

/// Runs 3 fft layers across the entire array.
///
/// # Arguments
///
/// - `src`: A pointer to the block of `index_h` in the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the block of `index_h` in the destination array, aligned to 64 bytes.
/// - `twiddle_dbl`: The doubles of the twiddle factors for each of the 3 fft layers.
/// - `loop_bits`: The number of bits this loops needs to run on.
/// - `layer`: The layer number of the first fft layer to apply. The layers `layer`, `layer + 1`,
///   `layer + 2` are applied.
/// - `index_h`: The higher part of the index, iterated by the caller.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`].
unsafe fn fft3_loop(
    src: *const u32,
    dst: *mut u32,
    twiddle_dbl: TwiddleTreeView<'_, u32>,
    loop_bits: usize,
    layer: usize,
    index_h: usize,
) {
    let twiddle_dbl0 = twiddle_dbl.layer(0);
    let twiddle_dbl1 = twiddle_dbl.layer(1);
    let twiddle_dbl2 = twiddle_dbl.layer(2);
    for index_l in 0..1 << loop_bits {
        let index = (index_h << loop_bits) + index_l;
        let offset = index_l << (layer + 3);
        let twiddles0: [u32x16; 4] = array::from_fn(|i| {
            u32x16::splat(*twiddle_dbl0.get_unchecked((index * 4 + i) & (twiddle_dbl0.len() - 1)))
        });
        let twiddles1: [u32x16; 2] = array::from_fn(|i| {
            u32x16::splat(*twiddle_dbl1.get_unchecked((index * 2 + i) & (twiddle_dbl1.len() - 1)))
        });
        let twiddles2: [u32x16; 1] = array::from_fn(|i| {
            u32x16::splat(*twiddle_dbl2.get_unchecked((index + i) & (twiddle_dbl2.len() - 1)))
        });
        for l in (0..1 << layer).step_by(1 << LOG_N_LANES as usize) {
            fft3(src, dst, offset + l, layer, twiddles0, twiddles1, twiddles2);
        }
    }
}

/// Runs 2 fft layers across the entire array.
///
/// # Arguments
///
/// - `src`: A pointer to the block of `index_h` in the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the block of `index_h` in the destination array, aligned to 64 bytes.
/// - `twiddle_dbl`: The doubles of the twiddle factors for each of the 2 fft layers.
/// - `loop_bits`: The number of bits this loops needs to run on.
/// - `layer`: The layer number of the first fft layer to apply. The layers `layer`, `layer + 1` are
///   applied.
/// - `index`: The index, iterated by the caller.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`].
unsafe fn fft2_loop(
    src: *const u32,
    dst: *mut u32,
    twiddle_dbl: TwiddleTreeView<'_, u32>,
    layer: usize,
    index: usize,
) {
    let twiddle_dbl0 = twiddle_dbl.layer(0);
    let twiddle_dbl1 = twiddle_dbl.layer(1);
    let twiddles0: [u32x16; 2] = array::from_fn(|i| {
        u32x16::splat(*twiddle_dbl0.get_unchecked((index * 2 + i) & (twiddle_dbl0.len() - 1)))
    });
    let twiddles1: [u32x16; 1] = array::from_fn(|i| {
        u32x16::splat(*twiddle_dbl1.get_unchecked((index + i) & (twiddle_dbl1.len() - 1)))
    });
    for l in (0..1 << layer).step_by(1 << LOG_N_LANES as usize) {
        fft2(src, dst, l, layer, twiddles0, twiddles1);
    }
}

/// Runs 1 fft layer across the entire array.
///
/// # Arguments
///
/// - `src`: A pointer to the block of `index_h` in the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the block of `index_h` in the destination array, aligned to 64 bytes.
/// - `twiddle_dbl`: The doubles of the twiddle factors for the fft layer.
/// - `layer`: The layer number of the fft layer to apply.
/// - `index_h`: The higher part of the index, iterated by the caller.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`].
unsafe fn fft1_loop(
    src: *const u32,
    dst: *mut u32,
    twiddle_dbl: TwiddleTreeView<'_, u32>,
    layer: usize,
    index: usize,
) {
    let twiddle_dbl0 = twiddle_dbl.layer(0);
    let twiddles0: [u32x16; 1] = array::from_fn(|i| {
        u32x16::splat(*twiddle_dbl0.get_unchecked((index + i) & (twiddle_dbl0.len() - 1)))
    });
    for l in (0..1 << layer).step_by(1 << LOG_N_LANES as usize) {
        fft1(src, dst, l, layer, twiddles0);
    }
}

/// Computes the butterfly operation for packed M31 elements.
///
/// Returns `val0 + t val1, val0 - t val1`. `val0, val1` are packed M31 elements. 16 M31 words at
/// each. Each value is assumed to be in unreduced form, [0, P] including P. Returned values are in
/// unreduced form, [0, P] including P. twiddle_dbl holds 16 values, each is a *double* of a twiddle
/// factor, in unreduced form, [0, 2*P].
pub fn simd_butterfly(
    val0: PackedBaseField,
    val1: PackedBaseField,
    twiddle_dbl: u32x16,
) -> (PackedBaseField, PackedBaseField) {
    let prod = mul_twiddle(val1, twiddle_dbl);
    (val0 + prod, val0 - prod)
}

/// Runs fft on 2 vectors of 16 M31 elements.
///
/// This amounts to 4 butterfly layers, each with 16 butterflies.
/// Each of the vectors represents natural ordered polynomial coefficient.
/// Each value in a vectors is in unreduced form: [0, P] including P.
/// Takes 4 twiddle arrays, one for each layer, holding the double of the corresponding twiddle.
/// The first layer (higher bit of the index) takes 2 twiddles.
/// The second layer takes 4 twiddles.
/// etc.
pub fn vecwise_butterflies(
    mut val0: PackedBaseField,
    mut val1: PackedBaseField,
    twiddle1_dbl: [u32; 8],
    twiddle2_dbl: [u32; 4],
    twiddle3_dbl: [u32; 2],
) -> (PackedBaseField, PackedBaseField) {
    // TODO(andrew): Can the permute be fused with the _mm512_srli_epi64 inside the butterfly?
    // The implementation is the exact reverse of vecwise_ibutterflies().
    // See the comments in its body for more info.
    let t =
        simd_swizzle!(u32x2::from(twiddle3_dbl), [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1]);
    (val0, val1) = val0.interleave(val1);
    (val0, val1) = simd_butterfly(val0, val1, t);

    let t =
        simd_swizzle!(u32x4::from(twiddle2_dbl), [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]);
    (val0, val1) = val0.interleave(val1);
    (val0, val1) = simd_butterfly(val0, val1, t);

    let (t0, t1) = compute_first_twiddles(u32x8::from(twiddle1_dbl));
    (val0, val1) = val0.interleave(val1);
    (val0, val1) = simd_butterfly(val0, val1, t1);

    (val0, val1) = val0.interleave(val1);
    (val0, val1) = simd_butterfly(val0, val1, t0);

    val0.interleave(val1)
}

/// Returns the line twiddles (x points) for an fft on a coset.
pub fn get_twiddle_dbls(mut coset: Coset) -> Vec<Vec<u32>> {
    let mut res = vec![];
    for _ in 0..coset.log_size() {
        res.push(coset.iter().take(coset.size() / 2).map(|p| p.x.0 * 2).collect_vec());
        bit_reverse(res.last_mut().unwrap());
        coset = coset.double();
    }

    res
}

/// Applies 3 butterfly layers on 8 vectors of 16 M31 elements.
///
/// Vectorized over the 16 elements of the vectors.
/// Used for radix-8 ifft.
/// Each butterfly layer, has 3 SIMD butterflies.
/// Total of 12 SIMD butterflies.
///
/// # Arguments
///
/// - `src`: A pointer to the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the destination array, aligned to 64 bytes.
/// - `offset`: The offset of the first value in the array.
/// - `log_step`: The log of the distance in the array, in M31 elements, between each pair of values
///   that need to be transformed. For layer i this is i - 4.
/// - `twiddles_dbl0/1/2`: The double of the twiddles for the 3 layers of butterflies. Each layer
///   has 4/2/1 twiddles.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`].
#[inline(always)]
pub unsafe fn fft3(
    src: *const u32,
    dst: *mut u32,
    offset: usize,
    log_step: usize,
    twiddles_dbl0: [u32x16; 4],
    twiddles_dbl1: [u32x16; 2],
    twiddles_dbl2: [u32x16; 1],
) {
    // Load the 8 SIMD vectors from the array.
    let mut val0 = PackedBaseField::load(src.add(offset + (0 << log_step)));
    let mut val1 = PackedBaseField::load(src.add(offset + (1 << log_step)));
    let mut val2 = PackedBaseField::load(src.add(offset + (2 << log_step)));
    let mut val3 = PackedBaseField::load(src.add(offset + (3 << log_step)));
    let mut val4 = PackedBaseField::load(src.add(offset + (4 << log_step)));
    let mut val5 = PackedBaseField::load(src.add(offset + (5 << log_step)));
    let mut val6 = PackedBaseField::load(src.add(offset + (6 << log_step)));
    let mut val7 = PackedBaseField::load(src.add(offset + (7 << log_step)));

    // Apply the third layer of butterflies.
    (val0, val4) = simd_butterfly(val0, val4, twiddles_dbl2[0]);
    (val1, val5) = simd_butterfly(val1, val5, twiddles_dbl2[0]);
    (val2, val6) = simd_butterfly(val2, val6, twiddles_dbl2[0]);
    (val3, val7) = simd_butterfly(val3, val7, twiddles_dbl2[0]);

    // Apply the second layer of butterflies.
    (val0, val2) = simd_butterfly(val0, val2, twiddles_dbl1[0]);
    (val1, val3) = simd_butterfly(val1, val3, twiddles_dbl1[0]);
    (val4, val6) = simd_butterfly(val4, val6, twiddles_dbl1[1]);
    (val5, val7) = simd_butterfly(val5, val7, twiddles_dbl1[1]);

    // Apply the first layer of butterflies.
    (val0, val1) = simd_butterfly(val0, val1, twiddles_dbl0[0]);
    (val2, val3) = simd_butterfly(val2, val3, twiddles_dbl0[1]);
    (val4, val5) = simd_butterfly(val4, val5, twiddles_dbl0[2]);
    (val6, val7) = simd_butterfly(val6, val7, twiddles_dbl0[3]);

    // Store the 8 SIMD vectors back to the array.
    val0.store(dst.add(offset + (0 << log_step)));
    val1.store(dst.add(offset + (1 << log_step)));
    val2.store(dst.add(offset + (2 << log_step)));
    val3.store(dst.add(offset + (3 << log_step)));
    val4.store(dst.add(offset + (4 << log_step)));
    val5.store(dst.add(offset + (5 << log_step)));
    val6.store(dst.add(offset + (6 << log_step)));
    val7.store(dst.add(offset + (7 << log_step)));
}

/// Applies 2 butterfly layers on 4 vectors of 16 M31 elements.
///
/// Vectorized over the 16 elements of the vectors.
/// Used for radix-4 fft.
/// Each butterfly layer, has 2 SIMD butterflies.
/// Total of 4 SIMD butterflies.
///
/// # Arguments
///
/// - `src`: A pointer to the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the destination array, aligned to 64 bytes.
/// - `offset`: The offset of the first value in the array.
/// - `log_step`: The log of the distance in the array, in M31 elements, between each pair of values
///   that need to be transformed. For layer i this is i - 4.
/// - `twiddles_dbl0/1`: The double of the twiddles for the 2 layers of butterflies. Each layer has
///   2/1 twiddles.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`].
#[inline(always)]
pub unsafe fn fft2(
    src: *const u32,
    dst: *mut u32,
    offset: usize,
    log_step: usize,
    twiddles_dbl0: [u32x16; 2],
    twiddles_dbl1: [u32x16; 1],
) {
    // Load the 4 SIMD vectors from the array.
    let mut val0 = PackedBaseField::load(src.add(offset + (0 << log_step)));
    let mut val1 = PackedBaseField::load(src.add(offset + (1 << log_step)));
    let mut val2 = PackedBaseField::load(src.add(offset + (2 << log_step)));
    let mut val3 = PackedBaseField::load(src.add(offset + (3 << log_step)));

    // Apply the second layer of butterflies.
    (val0, val2) = simd_butterfly(val0, val2, twiddles_dbl1[0]);
    (val1, val3) = simd_butterfly(val1, val3, twiddles_dbl1[0]);

    // Apply the first layer of butterflies.
    (val0, val1) = simd_butterfly(val0, val1, twiddles_dbl0[0]);
    (val2, val3) = simd_butterfly(val2, val3, twiddles_dbl0[1]);

    // Store the 4 SIMD vectors back to the array.
    val0.store(dst.add(offset + (0 << log_step)));
    val1.store(dst.add(offset + (1 << log_step)));
    val2.store(dst.add(offset + (2 << log_step)));
    val3.store(dst.add(offset + (3 << log_step)));
}

/// Applies 1 butterfly layers on 2 vectors of 16 M31 elements.
///
/// Vectorized over the 16 elements of the vectors.
///
/// # Arguments
///
/// - `src`: A pointer to the values to transform, aligned to 64 bytes.
/// - `dst`: A pointer to the destination array, aligned to 64 bytes.
/// - `offset`: The offset of the first value in the array.
/// - `log_step`: The log of the distance in the array, in M31 elements, between each pair of values
///   that need to be transformed. For layer i this is i - 4.
/// - `twiddles_dbl0`: The double of the twiddles for the butterfly layer.
///
/// # Safety
///
/// Behavior is undefined if `src` and `dst` do not have the same alignment as [`PackedBaseField`].
#[inline(always)]
pub unsafe fn fft1(
    src: *const u32,
    dst: *mut u32,
    offset: usize,
    log_step: usize,
    twiddles_dbl0: [u32x16; 1],
) {
    // Load the 2 SIMD vectors from the array.
    let mut val0 = PackedBaseField::load(src.add(offset + (0 << log_step)));
    let mut val1 = PackedBaseField::load(src.add(offset + (1 << log_step)));

    (val0, val1) = simd_butterfly(val0, val1, twiddles_dbl0[0]);

    // Store the 2 SIMD vectors back to the array.
    val0.store(dst.add(offset + (0 << log_step)));
    val1.store(dst.add(offset + (1 << log_step)));
}

#[cfg(test)]
mod tests {
    use std::mem::transmute;
    use std::simd::u32x16;

    use itertools::Itertools;
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use super::{
        SUBDOMAINS_SCRATCH_LOG_ELEMENTS, fft, fft_lower_with_vecwise,
        fft_subdomains_with_scratch_log, fft3, get_twiddle_dbls, simd_butterfly,
        vecwise_butterflies,
    };
    use crate::core::fft::butterfly as ground_truth_butterfly;
    use crate::core::fields::m31::BaseField;
    use crate::core::poly::circle::{CanonicCoset, CircleDomain};
    use crate::prover::backend::Column;
    use crate::prover::backend::cpu::CpuCirclePoly;
    use crate::prover::backend::simd::column::BaseColumn;
    use crate::prover::backend::simd::fft::{CACHED_FFT_LOG_SIZE, TwiddleTreeView, transpose_vecs};
    use crate::prover::backend::simd::m31::{LOG_N_LANES, N_LANES, PackedBaseField};

    #[test]
    fn test_butterfly() {
        let mut rng = SmallRng::seed_from_u64(0);
        let mut v0: [BaseField; N_LANES] = rng.random();
        let mut v1: [BaseField; N_LANES] = rng.random();
        let twiddle: [BaseField; N_LANES] = rng.random();
        let twiddle_dbl = twiddle.map(|v| v.0 * 2);

        let (r0, r1) = simd_butterfly(v0.into(), v1.into(), twiddle_dbl.into());

        let r0 = r0.to_array();
        let r1 = r1.to_array();
        for i in 0..N_LANES {
            ground_truth_butterfly(&mut v0[i], &mut v1[i], twiddle[i]);
            assert_eq!((v0[i], v1[i]), (r0[i], r1[i]), "mismatch at i={i}");
        }
    }

    #[test]
    fn test_fft3() {
        let mut rng = SmallRng::seed_from_u64(0);
        let values = rng.random::<[BaseField; 8]>().map(PackedBaseField::broadcast);
        let twiddles0: [BaseField; 4] = rng.random();
        let twiddles1: [BaseField; 2] = rng.random();
        let twiddles2: [BaseField; 1] = rng.random();
        let twiddles0_dbl = twiddles0.map(|v| v.0 * 2);
        let twiddles1_dbl = twiddles1.map(|v| v.0 * 2);
        let twiddles2_dbl = twiddles2.map(|v| v.0 * 2);

        let mut res = values;
        unsafe {
            fft3(
                transmute::<*const PackedBaseField, *const u32>(res.as_ptr()),
                transmute::<*mut PackedBaseField, *mut u32>(res.as_mut_ptr()),
                0,
                LOG_N_LANES as usize,
                twiddles0_dbl.map(u32x16::splat),
                twiddles1_dbl.map(u32x16::splat),
                twiddles2_dbl.map(u32x16::splat),
            )
        };

        let mut expected = values.map(|v| v.to_array()[0]);
        for i in 0..8 {
            let j = i ^ 4;
            if i > j {
                continue;
            }
            let (mut v0, mut v1) = (expected[i], expected[j]);
            ground_truth_butterfly(&mut v0, &mut v1, twiddles2[0]);
            (expected[i], expected[j]) = (v0, v1);
        }
        for i in 0..8 {
            let j = i ^ 2;
            if i > j {
                continue;
            }
            let (mut v0, mut v1) = (expected[i], expected[j]);
            ground_truth_butterfly(&mut v0, &mut v1, twiddles1[i / 4]);
            (expected[i], expected[j]) = (v0, v1);
        }
        for i in 0..8 {
            let j = i ^ 1;
            if i > j {
                continue;
            }
            let (mut v0, mut v1) = (expected[i], expected[j]);
            ground_truth_butterfly(&mut v0, &mut v1, twiddles0[i / 2]);
            (expected[i], expected[j]) = (v0, v1);
        }
        for i in 0..8 {
            assert_eq!(res[i].to_array(), [expected[i]; N_LANES], "mismatch at i={i}");
        }
    }

    #[test]
    fn test_vecwise_butterflies() {
        let domain = CanonicCoset::new(5).circle_domain();
        let twiddle_dbls = get_twiddle_dbls(domain.half_coset);
        assert_eq!(twiddle_dbls.len(), 4);
        let mut rng = SmallRng::seed_from_u64(0);
        let values: [[BaseField; 16]; 2] = rng.random();

        let res = {
            let (val0, val1) = simd_butterfly(
                values[0].into(),
                values[1].into(),
                u32x16::splat(twiddle_dbls[3][0]),
            );
            let (val0, val1) = vecwise_butterflies(
                val0,
                val1,
                twiddle_dbls[0].clone().try_into().unwrap(),
                twiddle_dbls[1].clone().try_into().unwrap(),
                twiddle_dbls[2].clone().try_into().unwrap(),
            );
            [val0.to_array(), val1.to_array()].concat()
        };

        assert_eq!(res, ground_truth_fft(domain, values.as_flattened()));
    }

    #[test]
    fn test_fft_lower() {
        for log_size in 5..12 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let mut rng = SmallRng::seed_from_u64(0);
            let values = (0..domain.size()).map(|_| rng.random()).collect_vec();
            // Flatten the layers into a twiddle tree buffer, with one padding element.
            let mut twiddle_tree = get_twiddle_dbls(domain.half_coset).concat();
            twiddle_tree.push(0);

            let mut res = values.iter().copied().collect::<BaseColumn>();
            unsafe {
                fft_lower_with_vecwise(
                    transmute::<*const PackedBaseField, *const u32>(res.data.as_ptr()),
                    transmute::<*mut PackedBaseField, *mut u32>(res.data.as_mut_ptr()),
                    TwiddleTreeView::new(&twiddle_tree),
                    log_size as usize,
                    log_size as usize,
                )
            }

            assert_eq!(res.to_cpu(), ground_truth_fft(domain, &values));
        }
    }

    #[test]
    fn test_fft_full() {
        for log_size in CACHED_FFT_LOG_SIZE + 1..CACHED_FFT_LOG_SIZE + 3 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let mut rng = SmallRng::seed_from_u64(0);
            let values = (0..domain.size()).map(|_| rng.random()).collect_vec();
            // Flatten the layers into a twiddle tree buffer, with one padding element.
            let mut twiddle_tree = get_twiddle_dbls(domain.half_coset).concat();
            twiddle_tree.push(0);

            let mut res = values.iter().copied().collect::<BaseColumn>();
            unsafe {
                transpose_vecs(
                    transmute::<*mut PackedBaseField, *mut u32>(res.data.as_mut_ptr()),
                    log_size as usize - 4,
                );
                fft(
                    transmute::<*const PackedBaseField, *const u32>(res.data.as_ptr()),
                    transmute::<*mut PackedBaseField, *mut u32>(res.data.as_mut_ptr()),
                    TwiddleTreeView::new(&twiddle_tree),
                    log_size as usize,
                );
            }

            assert_eq!(res.to_cpu(), ground_truth_fft(domain, &values));
        }
    }

    #[test]
    fn test_fft_subdomains() {
        let mut rng = SmallRng::seed_from_u64(0);
        for log_size in CACHED_FFT_LOG_SIZE..CACHED_FFT_LOG_SIZE + 3 {
            for log_subdomains in 0..3 {
                // A twiddle tree one layer larger than the domain.
                let tree_domain = CanonicCoset::new(log_size + log_subdomains + 1).circle_domain();
                let mut twiddle_tree = get_twiddle_dbls(tree_domain.half_coset).concat();
                twiddle_tree.push(0);
                let twiddles =
                    TwiddleTreeView::new(&twiddle_tree).sub_tree(log_size + log_subdomains);
                let values: BaseColumn = (0..1 << log_size).map(|_| rng.random()).collect();
                let n_elements = 1 << (log_size + log_subdomains);
                let mut expected = BaseColumn::zeros(n_elements);
                for i in 0..1 << log_subdomains {
                    unsafe {
                        fft(
                            transmute::<*const PackedBaseField, *const u32>(values.data.as_ptr()),
                            transmute::<*mut PackedBaseField, *mut u32>(expected.data.as_mut_ptr())
                                .add(i << log_size),
                            twiddles.subdomain(log_subdomains, i),
                            log_size as usize,
                        );
                    }
                }

                // Test with different scratch sizes.
                for scratch_log_elements in [0, 12, 13, 14, SUBDOMAINS_SCRATCH_LOG_ELEMENTS] {
                    let mut res = BaseColumn::zeros(n_elements);
                    unsafe {
                        fft_subdomains_with_scratch_log(
                            transmute::<*const PackedBaseField, *const u32>(values.data.as_ptr()),
                            transmute::<*mut PackedBaseField, *mut u32>(res.data.as_mut_ptr()),
                            twiddles,
                            log_size as usize,
                            scratch_log_elements,
                        );
                    }
                    assert_eq!(
                        res.to_cpu(),
                        expected.to_cpu(),
                        "log_size={log_size}, log_subdomains={log_subdomains}, \
                         scratch_log_elements={scratch_log_elements}"
                    );
                }
            }
        }
    }

    fn ground_truth_fft(domain: CircleDomain, values: &[BaseField]) -> Vec<BaseField> {
        let poly = CpuCirclePoly::new(values.to_vec());
        poly.evaluate(domain).values
    }
}
