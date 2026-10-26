use std::slice;

/// A borrowed view of a twiddle tree buffer (the layout of `TwiddleTree::{twiddles, itwiddles}`),
/// possibly restricted to the tree of a subdomain.
///
/// A buffer of size `2^K` stores `K` layers, from the outermost (`2^(K-1)` elements) to the
/// innermost (one element), followed by one padding element. Each layer is in bit-reversed order,
/// so the tree of the `i`-th of `2^k` subdomains (in bit-reversed order) is the `i`-th of `2^k`
/// equal chunks of each of the outer layers.
///
/// Invariant, kept by the constructors: if `n_layers > 0`, then
/// `2^(n_layers + log_n_chunks) <= buffer.len()` and `chunk < 2^log_n_chunks`, so every layer of
/// the view lies in `buffer`.
pub struct TwiddleTreeView<'a, T> {
    buffer: &'a [T],
    n_layers: u32,
    /// Each parent layer is split into `2^log_n_chunks` chunks.
    log_n_chunks: u32,
    /// The index of the chunk viewed in each parent layer.
    chunk: usize,
}

impl<T> Clone for TwiddleTreeView<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TwiddleTreeView<'_, T> {}

impl<'a, T> TwiddleTreeView<'a, T> {
    /// The whole tree stored in `buffer`.
    pub fn new(buffer: &'a [T]) -> Self {
        let n_layers = buffer.len().checked_ilog2().unwrap_or(0);
        Self { buffer, n_layers, log_n_chunks: 0, chunk: 0 }
    }

    /// The subtree of the circle domain of size `2^log_size` in the tower: the innermost
    /// `log_size - 1` layers.
    pub fn sub_tree(self, log_size: u32) -> Self {
        let n_layers = log_size - 1;
        assert!(n_layers <= self.n_layers, "Not enough twiddles!");
        Self { n_layers, ..self }
    }

    /// The tree of the `index`-th of the `2^log_n_subdomains` subdomains, in bit-reversed order.
    pub fn subdomain(self, log_n_subdomains: u32, index: usize) -> Self {
        assert!(log_n_subdomains <= self.n_layers);
        assert!(index < 1 << log_n_subdomains);
        Self {
            buffer: self.buffer,
            n_layers: self.n_layers - log_n_subdomains,
            log_n_chunks: self.log_n_chunks + log_n_subdomains,
            chunk: (self.chunk << log_n_subdomains) | index,
        }
    }

    pub const fn n_layers(&self) -> usize {
        self.n_layers as usize
    }

    /// Drops the `n` outermost layers.
    #[inline]
    pub fn skip_layers(self, n: usize) -> Self {
        debug_assert!(n <= self.n_layers());
        Self { n_layers: self.n_layers - n as u32, ..self }
    }

    /// Returns layer `j`, of size `2^(n_layers - 1 - j)`, where layer 0 is the outermost.
    ///
    /// # Safety
    ///
    /// `j < self.n_layers()`.
    #[inline]
    pub unsafe fn layer(&self, j: usize) -> &'a [T] {
        debug_assert!(j < self.n_layers());
        let log_size = self.n_layers() - 1 - j;
        let parent_start = self.buffer.len() - (2 << (log_size + self.log_n_chunks as usize));
        let start = parent_start + (self.chunk << log_size);
        // SAFETY: By the invariant, the parent layer, of size `2^(log_size + log_n_chunks)`, lies
        // in the buffer, and so does its chunk `chunk < 2^log_n_chunks` of size `2^log_size`.
        slice::from_raw_parts(self.buffer.as_ptr().add(start), 1 << log_size)
    }

    /// Returns an iterator over the trees of the `2^log_n_subdomains` subdomains, in order: its
    /// `i`-th item is `self.subdomain(log_n_subdomains, i)`.
    pub fn subdomains_iter(
        self,
        log_n_subdomains: u32,
    ) -> impl ExactSizeIterator<Item = Self> + Clone {
        assert!(log_n_subdomains <= self.n_layers);
        (0..1 << log_n_subdomains).map(move |i| Self {
            buffer: self.buffer,
            n_layers: self.n_layers - log_n_subdomains,
            log_n_chunks: self.log_n_chunks + log_n_subdomains,
            chunk: (self.chunk << log_n_subdomains) | i,
        })
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::TwiddleTreeView;
    use crate::core::poly::circle::CanonicCoset;
    use crate::core::poly::utils::domain_line_twiddles_from_tree;
    use crate::prover::poly::BitReversedOrder;
    use crate::prover::poly::twiddles::TwiddleBuffer;

    const ROOT_LOG_SIZE: u32 = 8;

    fn buffer() -> Vec<u32> {
        (0..1 << ROOT_LOG_SIZE).collect()
    }

    fn layers(view: TwiddleTreeView<'_, u32>) -> Vec<&[u32]> {
        (0..view.n_layers()).map(|j| unsafe { view.layer(j) }).collect()
    }

    #[test]
    fn test_sub_tree_matches_domain_line_twiddles_from_tree() {
        let buffer = buffer();
        for log_size in 1..=ROOT_LOG_SIZE + 1 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let view = TwiddleTreeView::new(&buffer).sub_tree(log_size);

            assert_eq!(layers(view), domain_line_twiddles_from_tree(domain, &buffer));
        }
    }

    #[test]
    fn test_subdomain_matches_layer_chunks() {
        let buffer = buffer();
        // Includes trees larger than the domain.
        for log_size in 2..=ROOT_LOG_SIZE + 1 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let domain_layers = domain_line_twiddles_from_tree(domain, &buffer);
            for fft_log_size in 2..=log_size {
                let log_n_subdomains = log_size - fft_log_size;
                for i in 0..1 << log_n_subdomains {
                    let view = TwiddleTreeView::new(&buffer)
                        .sub_tree(log_size)
                        .subdomain(log_n_subdomains, i);

                    let expected = (0..fft_log_size as usize - 1)
                        .map(|j| {
                            let len = 1 << (fft_log_size as usize - 2 - j);
                            &domain_layers[j][i * len..(i + 1) * len]
                        })
                        .collect_vec();
                    assert_eq!(layers(view), expected);
                }
            }
        }
    }

    #[test]
    fn test_subdomains_iter_matches_subdomain() {
        let buffer = buffer();
        // A subdomain view, of domains of all sizes, including trees larger than the domain.
        for log_size in 2..=ROOT_LOG_SIZE + 1 {
            let view = TwiddleTreeView::new(&buffer).sub_tree(log_size).subdomain(1, 1);
            for log_n_subdomains in 0..=view.n_layers() as u32 {
                let subdomains = view.subdomains_iter(log_n_subdomains);
                assert_eq!(subdomains.len(), 1 << log_n_subdomains);
                for (i, subdomain_view) in subdomains.enumerate() {
                    assert_eq!(layers(subdomain_view), layers(view.subdomain(log_n_subdomains, i)));
                }
            }
        }
    }

    #[test]
    fn test_first_subdomain_matches_extract_subdomain_twiddles() {
        let buffer = buffer();
        let committed_log_size = ROOT_LOG_SIZE - 1;
        for subdomain_log_size in 1..=committed_log_size {
            let view = TwiddleTreeView::new(&buffer)
                .sub_tree(committed_log_size)
                .subdomain(committed_log_size - subdomain_log_size, 0);

            let extracted = TwiddleBuffer::<BitReversedOrder>::extract_subdomain_twiddles(
                &buffer,
                committed_log_size,
                subdomain_log_size,
            );
            // Compare all layers (excluding the padding element).
            assert_eq!(layers(view).concat(), extracted[..extracted.len() - 1]);
        }
    }

    #[test]
    fn test_skip_layers() {
        let buffer = buffer();
        let view = TwiddleTreeView::new(&buffer).subdomain(2, 1);
        for n in 0..=view.n_layers() {
            assert_eq!(layers(view.skip_layers(n)), layers(view)[n..]);
        }
    }

    #[test]
    fn test_nested_subdomains() {
        let buffer = buffer();
        let view = TwiddleTreeView::new(&buffer);

        assert_eq!(layers(view.subdomain(1, 1).subdomain(2, 2)), layers(view.subdomain(3, 6)));
    }
}
