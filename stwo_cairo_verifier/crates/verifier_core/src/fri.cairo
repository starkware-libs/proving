use core::dict::Felt252Dict;
use core::iter::{IntoIterator, Iterator};
use crate::Hash;
use crate::channel::{Channel, ChannelTrait};
use crate::circle::{CirclePointM31Impl, CosetImpl};
use crate::fields::Invertible;
use crate::fields::m31::M31;
use crate::fields::qm31::{QM31, QM31Serde, QM31Trait, QM31_EXTENSION_DEGREE};
use crate::poly::circle::{CanonicCosetImpl, CircleDomain, CircleDomainImpl};
use crate::poly::line::{LineDomain, LineDomainImpl, LineDomainTrait, LineEvaluationImpl, LinePoly};
use crate::poly::utils::fri_fold;
use crate::queries::{Queries, QueriesImpl};
use crate::utils::{OptionImpl, SpanExTrait, bit_reverse_index, pow2};
use crate::vcs::MerkleHasher;
use crate::vcs::verifier::{MerkleDecommitment, MerkleVerifier, MerkleVerifierTrait};

/// Log of the number of QM31 evaluations packed into a single Merkle leaf.
pub const LOG_PACKED_LEAF_SIZE: u32 = 2;

/// The range of supported FRI fold steps.
const MIN_FOLD_STEP: u32 = 1;
const MAX_FOLD_STEP: u32 = 4;
/// The minimum number of proof-of-work bits before the FRI query phase.
const MIN_POW_BITS: u32 = 20;

#[derive(Drop, Serde, Copy, PartialEq)]
pub struct FriParams {
    pub pow_bits: u32,
    pub log_blowup_factor: u32,
    pub log_last_layer_degree_bound: u32,
    pub n_queries: usize,
    pub fold_step: u32,
}

#[generate_trait]
pub impl FriParamsImpl of FriParamsTrait {
    fn security_bits(self: @FriParams) -> u32 {
        *self.pow_bits + *self.log_blowup_factor * *self.n_queries
    }

    /// Panics if the config isn't supported by the verifier, or doesn't reach
    /// `min_security_bits`.
    fn validate(self: @FriParams, min_security_bits: u32) {
        assert!(
            self.security_bits() >= min_security_bits,
            "{}",
            FriVerificationError::SecurityBitsTooLow,
        );
        let fold_step = *self.fold_step;
        assert!(
            MIN_FOLD_STEP <= fold_step && fold_step <= MAX_FOLD_STEP,
            "{}",
            FriVerificationError::InvalidFoldStep,
        );
        // Only a constant last layer is supported.
        assert!(
            *self.log_last_layer_degree_bound == 0,
            "{}",
            FriVerificationError::LastLayerLogDegreeMustBeZero,
        );
        assert!(*self.pow_bits >= MIN_POW_BITS, "{}", FriVerificationError::ProofOfWorkBitsTooLow);
    }

    fn mix_into(self: @FriParams, ref channel: Channel) {
        let FriParams {
            pow_bits, log_blowup_factor, log_last_layer_degree_bound, n_queries, fold_step,
        } = self;

        channel
            .mix_u32s(
                [
                    *pow_bits, *log_blowup_factor, *n_queries, *log_last_layer_degree_bound,
                    *fold_step,
                ]
                    .span(),
            );
    }
}

#[derive(Drop)]
pub struct FriVerifier {
    params: FriParams,
    /// The verifiers of the single FRI layers. `layers[0]` is the first (circle) layer, followed by
    /// the inner (line) layers, in decreasing order by size of layer domain.
    layers: Array<FriLayerVerifier>,
    last_layer_domain: LineDomain,
    last_layer_poly: LinePoly,
    proof_of_work_nonce: u64,
}

#[generate_trait]
pub impl FriVerifierImpl of FriVerifierTrait {
    /// Verifies the commitment stage of FRI.
    ///
    /// `log_degree_bound` should be the committed circle polynomial log degree bound.
    fn commit(
        ref channel: Channel, params: FriParams, proof: FriProof, log_degree_bound: u32,
    ) -> FriVerifier {
        let FriProof {
            first_layer: first_layer_proof,
            inner_layers: mut inner_layer_proofs,
            last_layer_poly,
            proof_of_work_nonce,
        } = proof;

        channel.mix_commitment(first_layer_proof.commitment);

        let commitment_domain_log_size = log_degree_bound + params.log_blowup_factor;
        let commitment_domain = CanonicCosetImpl::new(commitment_domain_log_size).circle_domain();

        let first_layer = FriLayerVerifier {
            log_degree_bound,
            domain: FriLayerDomain::Circle(commitment_domain),
            folding_alpha: channel.draw_secure_felt(),
            proof: @first_layer_proof,
            // The circle-to-line fold is always equal to `params.fold_step`.
            fold_step: params.fold_step,
        };
        let mut layers = array![first_layer];

        let mut layer_log_bound = log_degree_bound - params.fold_step;
        let mut layer_domain = LineDomainImpl::new_unchecked(
            CosetImpl::half_odds(layer_log_bound + params.log_blowup_factor),
        );

        let n_inner_layers = inner_layer_proofs.len();
        for (layer_index, layer_proof) in inner_layer_proofs.into_iter().enumerate() {
            channel.mix_commitment(*layer_proof.commitment);

            // Compute the folding step for this layer.
            let fold_step = if layer_index == n_inner_layers - 1 {
                // At the last inner layer, fold by the amount needed to reach exactly the
                // last layer degree bound.
                let remaining = layer_log_bound - params.log_last_layer_degree_bound;
                assert!(
                    1 <= remaining && remaining <= params.fold_step,
                    "{}",
                    FriVerificationError::InvalidNumFriLayers,
                );
                remaining
            } else {
                params.fold_step
            };

            layers
                .append(
                    FriLayerVerifier {
                        log_degree_bound: layer_log_bound,
                        domain: FriLayerDomain::Line(layer_domain),
                        folding_alpha: channel.draw_secure_felt(),
                        proof: layer_proof,
                        fold_step,
                    },
                );

            layer_log_bound -= fold_step;

            layer_domain = layer_domain.repeated_double(fold_step);
        }
        assert!(
            layer_log_bound == params.log_last_layer_degree_bound,
            "{}",
            FriVerificationError::InvalidNumFriLayers,
        );

        assert!(
            last_layer_poly.log_size == params.log_last_layer_degree_bound,
            "{}",
            FriVerificationError::LastLayerDegreeInvalid,
        );

        channel.mix_felts(last_layer_poly.coeffs.span());

        FriVerifier {
            params, layers, last_layer_domain: layer_domain, last_layer_poly, proof_of_work_nonce,
        }
    }

    /// Verifies the decommitment stage of FRI.
    ///
    /// The query positions are in bit-reversed order: position q refers to the domain point at
    /// bit_reverse_index(q).
    fn decommit(self: FriVerifier, queries: Queries, first_layer_query_evals: Span<QM31>) {
        let mut layer_queries = queries;
        let mut layer_query_evals = first_layer_query_evals;
        for layer in self.layers.span() {
            let fold_preimage_evals = layer
                .verify(queries: layer_queries, evals_at_queries: layer_query_evals);
            layer_queries = layer_queries.fold(*layer.fold_step);
            layer_query_evals = layer.fold(fold_preimage_evals).span();
        }

        decommit_last_layer(self, layer_queries, layer_query_evals)
    }

    /// Verifies the query proof of work and mixes the nonce into the channel.
    ///
    /// Must be called before `sample_query_positions`: the nonce gates the query positions
    /// drawn from the channel.
    fn verify_proof_of_work(self: @FriVerifier, ref channel: Channel) {
        let proof_of_work_nonce = *self.proof_of_work_nonce;
        assert!(
            channel.verify_pow_nonce(*self.params.pow_bits, proof_of_work_nonce),
            "{}",
            FriVerificationError::ProofOfWork,
        );
        channel.mix_u64(proof_of_work_nonce);
    }

    /// Samples and returns the query positions on the first layer's domain.
    fn sample_query_positions(self: @FriVerifier, ref channel: Channel) -> Queries {
        let query_domain_log_size = self.layers[0].domain.log_size();
        let n_queries = *self.params.n_queries;
        QueriesImpl::generate(ref channel, query_domain_log_size, n_queries)
    }
}

/// Verifies the last layer.
#[inline]
fn decommit_last_layer(verifier: FriVerifier, mut queries: Queries, mut query_evals: Span<QM31>) {
    let FriVerifier { last_layer_poly, .. } = verifier;

    let single_value_box: Box<[QM31; 1]> = *last_layer_poly
        .coeffs
        .span()
        .try_into()
        .unwrap_or_else(|| panic!("{}", FriVerificationError::LastLayerLogDegreeMustBeZero));
    let [expected_last_layer_value] = single_value_box.unbox();

    for query_eval in query_evals {
        assert!(
            *query_eval == expected_last_layer_value,
            "{}",
            FriVerificationError::LastLayerEvaluationsInvalid,
        );
    }
}

/// Returns the column query positions needed for verification.
///
/// The column log sizes must be unique and in descending order.
/// Returned column query positions are mapped by their log size.
fn get_query_positions_by_log_size(
    mut queries: Queries, mut unique_column_log_sizes: Span<u32>,
) -> Felt252Dict<Nullable<Span<usize>>> {
    let mut query_positions_by_log_size: Felt252Dict<Nullable<Span<usize>>> = Default::default();

    for column_log_size in unique_column_log_sizes {
        let n_folds = queries.log_domain_size - *column_log_size;

        if n_folds != 0 {
            queries = queries.fold(n_folds);
        }

        query_positions_by_log_size
            .insert((*column_log_size).into(), NullableTrait::new(queries.positions));
    }

    query_positions_by_log_size
}

/// A FRI proof.
#[derive(Drop, Serde)]
pub struct FriProof {
    pub first_layer: FriLayerProof,
    pub inner_layers: Span<FriLayerProof>,
    pub last_layer_poly: LinePoly,
    /// Nonce for the proof of work that gates the query phase.
    pub proof_of_work_nonce: u64,
}

/// The commitment domain of a FRI layer.
#[derive(Drop, Copy, Debug)]
enum FriLayerDomain {
    /// The domain of the first layer, folded circle-to-line.
    Circle: CircleDomain,
    /// The domain of an inner layer, folded line-to-line.
    Line: LineDomain,
}

#[generate_trait]
impl FriLayerDomainImpl of FriLayerDomainTrait {
    /// Returns the log size of the domain.
    fn log_size(self: @FriLayerDomain) -> u32 {
        match self {
            FriLayerDomain::Circle(circle_domain) => circle_domain.log_size(),
            FriLayerDomain::Line(line_domain) => line_domain.log_size(),
        }
    }
}

/// Verifier of a single FRI layer.
///
/// The first layer is committed on a `CircleDomain` and is folded circle-to-line. The inner
/// layers are committed on a `LineDomain` and are folded line-to-line.
#[derive(Drop, Debug)]
struct FriLayerVerifier {
    /// The log degree bound of the polynomial committed in this layer.
    log_degree_bound: u32,
    /// The commitment domain of this layer.
    domain: FriLayerDomain,
    /// The random folding coefficient of this layer.
    folding_alpha: QM31,
    /// The proof of this layer.
    proof: @FriLayerProof,
    /// The number of folds applied to this layer.
    fold_step: u32,
}

#[generate_trait]
impl FriLayerVerifierImpl of FriLayerVerifierTrait {
    /// Verifies the layer's Merkle decommitment and returns the evaluations needed for folding
    /// the layer.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// * The queries are sampled on the wrong domain.
    /// * The proof doesn't store the right number of evaluations.
    /// * The Merkle decommitment is invalid.
    fn verify(
        self: @FriLayerVerifier, queries: Queries, evals_at_queries: Span<QM31>,
    ) -> FoldPreimageEvals {
        let column_log_size = self.domain.log_size();
        assert!(queries.log_domain_size == column_log_size);

        let fold_preimage_evals = rebuild_fold_preimage_evals(
            queries, evals_at_queries, **self.proof.fri_witness, *self.fold_step,
        );

        // For decommitment, each QM31 col must be split into its constituent M31 coordinate cols.
        let mut decommitted_values = array![];
        for (_, evals) in fold_preimage_evals.indexed_evals {
            for eval in *evals {
                // Split the QM31 into its M31 coordinate values.
                let [v0, v1, v2, v3] = (*eval).to_fixed_array();
                decommitted_values.append(v0);
                decommitted_values.append(v1);
                decommitted_values.append(v2);
                decommitted_values.append(v3);
            };
        }

        let leaf_log_size: u32 = if *self.fold_step >= LOG_PACKED_LEAF_SIZE {
            LOG_PACKED_LEAF_SIZE
        } else {
            0
        };
        let merkle_positions = build_merkle_verification_inputs(
            fold_preimage_evals, *self.fold_step, leaf_log_size,
        );
        let merkle_verifier = MerkleVerifier {
            root: **self.proof.commitment,
            tree_height: column_log_size - leaf_log_size,
            n_committed_columns: QM31_EXTENSION_DEGREE * pow2(leaf_log_size),
        };

        merkle_verifier
            .verify(merkle_positions, decommitted_values.span(), **self.proof.decommitment);

        fold_preimage_evals
    }

    /// Folds the layer's evaluations into evaluations of the next layer.
    ///
    /// The first (circle) layer is folded circle-to-line, and inner (line) layers are folded
    /// line-to-line.
    fn fold(self: @FriLayerVerifier, fold_preimage_evals: FoldPreimageEvals) -> Array<QM31> {
        match *self.domain {
            FriLayerDomain::Circle(circle_domain) => fold_preimage_evals
                .fold_circle(*self.folding_alpha, circle_domain, *self.fold_step),
            FriLayerDomain::Line(line_domain) => fold_preimage_evals
                .fold_line(*self.folding_alpha, line_domain, *self.fold_step),
        }
    }
}

/// Re-builds the evaluations needed by the verifier for folding and decommitment, on the preimage
/// of each folded query position.
///
/// Each preimage spans the `2^fold_step` consecutive decommitment positions from its start.
///
/// # Panics
///
/// Panics if:
/// * The number of queries doesn't match the number of query evals.
/// * `witness_evals` isn't fully consumed.
fn rebuild_fold_preimage_evals(
    mut queries: Queries,
    mut query_evals: Span<QM31>,
    mut witness_evals: Span<QM31>,
    fold_step: u32,
) -> FoldPreimageEvals {
    let fold_factor = pow2(fold_step);

    let mut indexed_evals = array![];

    let mut query_positions = queries.positions;
    let mut folded_query_positions = queries.fold(fold_step).positions;

    for folded_query_position in folded_query_positions {
        let preimage_start = *folded_query_position * fold_factor;
        let preimage_end = preimage_start + fold_factor;
        let mut preimage_decommitment_positions = (preimage_start..preimage_end).into_iter();
        let mut preimage_evals = array![];

        // Extract the preimage evals.
        for decommitment_position in preimage_decommitment_positions {
            // If the decommitment position is a query position: take the value from `query_evals`,
            // else: take the value from `witness_evals`.
            preimage_evals
                .append(
                    *match query_positions.next_if_eq(@decommitment_position) {
                        Some(_) => query_evals.pop_front().unwrap(),
                        None => witness_evals.pop_front().unwrap(),
                    },
                );
        }

        indexed_evals.append((preimage_start, preimage_evals.span()));
    }

    // Sanity check all the values have been consumed.
    assert!(query_positions.is_empty());
    assert!(query_evals.is_empty());
    // Check all proof evals have been consumed.
    assert!(witness_evals.is_empty(), "{}", FriVerificationError::LayerEvaluationsInvalid);

    FoldPreimageEvals { indexed_evals: indexed_evals.span() }
}

/// Foldable evaluations of a circle polynomial or univariate polynomial on the preimages, under
/// the fold, of the folded query positions.
#[derive(Drop, Copy)]
struct FoldPreimageEvals {
    /// A span of `(preimage_start, evals)`, sorted by `preimage_start`, where:
    /// - `preimage_start` is the committed (bit-reversed) position of the initial point of the
    ///   preimage coset of size `2^fold_step`.
    /// - `evals` are the evaluations of the polynomial over this coset, in committed order.
    indexed_evals: Span<(usize, Span<QM31>)>,
}

#[generate_trait]
impl FoldPreimageEvalsImpl of FoldPreimageEvalsTrait {
    /// Folds evaluations of a degree `d` univariate polynomial into evaluations of a degree
    /// `d / 2^fold_step` univariate polynomial.
    fn fold_line(
        self: FoldPreimageEvals, fold_alpha: QM31, source_domain: LineDomain, fold_step: u32,
    ) -> Array<QM31> {
        let mut folded_eval = array![];
        let log_domain_size = source_domain.log_size();

        for (preimage_start, evals) in self.indexed_evals {
            let initial_index = bit_reverse_index(*preimage_start, log_domain_size);
            let fold_domain_initial = source_domain.coset.index_at(initial_index);
            let fold_domain = LineDomainImpl::new_unchecked(
                CosetImpl::new(fold_domain_initial, fold_step),
            );
            let mut x_coords = array![];
            let mut j = 0;
            while j < fold_domain.size() {
                let x_coord = fold_domain.at(bit_reverse_index(j, fold_step));
                x_coords.append(x_coord);
                j += 2;
            }

            folded_eval.append(fold_coset(*evals, x_coords.span(), fold_alpha));
        }

        folded_eval
    }

    /// Folds evaluations of a degree `d` circle polynomial into evaluations of a
    /// degree `d / 2^fold_step` univariate polynomial.
    fn fold_circle(
        self: FoldPreimageEvals, fold_alpha: QM31, source_domain: CircleDomain, fold_step: u32,
    ) -> Array<QM31> {
        let mut folded_eval = array![];
        let log_domain_size = source_domain.log_size();

        if fold_step == 1 {
            for (preimage_start, evals) in self.indexed_evals {
                let [v0, v1]: [QM31; 2] = (*(*evals).try_into().unwrap()).unbox();
                let initial_index = bit_reverse_index(*preimage_start, log_domain_size);
                let circle_point = source_domain.at(initial_index);
                folded_eval.append(fri_fold(v0, v1, circle_point.y.inverse(), fold_alpha));
            }
            return folded_eval;
        }

        for (preimage_start, evals) in self.indexed_evals {
            let initial_index = bit_reverse_index(*preimage_start, log_domain_size);
            let fold_domain_initial = source_domain.index_at(initial_index);
            let circle_fold_domain = CircleDomainImpl::new(
                CosetImpl::new(fold_domain_initial, fold_step - 1),
            );

            let mut preimage_evals = *evals;
            let mut x_coords = array![];
            let mut line_eval_domain = array![];
            let mut j = 0;
            // Iterate with stride 4. This is done in order to only populate the array `x_coords`
            // with coordinates that are actually needed.
            // Since fold_step > 1, we are guaranteed that `circle_fold_domain.size() = 2^k` with k
            // >= 2, hence the indices are never out of bounds.
            while let Some(evals) = preimage_evals.multi_pop_front::<4>() {
                let [v0, v1, v2, v3] = evals.unbox();
                let circle_pt_0 = circle_fold_domain.at(bit_reverse_index(j, fold_step));
                // The evaluation `v2` is at the antipode of `circle_pt_0` (and `v3` is at the
                // reflection of `circle_pt_0` about the y-axis). So the twiddle of their fold is
                // `-y_inv`.
                let y_inv = circle_pt_0.y.inverse();
                line_eval_domain.append(fri_fold(v0, v1, y_inv, fold_alpha));
                line_eval_domain.append(fri_fold(v2, v3, -y_inv, fold_alpha));
                x_coords.append(circle_pt_0.x);
                j += 4;
            }
            assert!(preimage_evals.is_empty());
            let alpha_sq = fold_alpha * fold_alpha;
            folded_eval.append(fold_coset(line_eval_domain.span(), x_coords.span(), alpha_sq));
        }

        folded_eval
    }
}

/// Returns the Merkle tree positions to verify for the decommitted preimages.
///
/// Each preimage spans `2^fold_step` consecutive decommitment positions, and each Merkle leaf
/// packs `2^leaf_log_size` of them. Preimage starts are aligned to the leaf size, so each
/// preimage spans `2^(fold_step - leaf_log_size)` consecutive leaves.
fn build_merkle_verification_inputs(
    fold_preimage_evals: FoldPreimageEvals, fold_step: u32, leaf_log_size: u32,
) -> Span<u32> {
    let leaf_size = pow2(leaf_log_size);
    let n_leaves_per_preimage = pow2(fold_step - leaf_log_size);
    let mut merkle_positions = array![];
    for (preimage_start, _) in fold_preimage_evals.indexed_evals {
        let first_leaf = *preimage_start / leaf_size;
        for leaf in first_leaf..first_leaf + n_leaves_per_preimage {
            merkle_positions.append(leaf);
        }
    }
    merkle_positions.span()
}

/// Folds `2^n` (n > 0) evaluations into a single evaluation using precomputed twiddles.
///
/// # Arguments
///
/// * `eval` - evaluations of length `2^n` on a coset domain.
/// * `x_coords` - x-coordinates of the even-index coset points, of length `2^{n-1}`.
/// * `alpha` - the random folding factor.
pub fn fold_coset(mut eval: Span<QM31>, mut x_coords: Span<M31>, mut alpha: QM31) -> QM31 {
    assert!(eval.len() == 2 * x_coords.len());
    assert!(eval.len() >= 2);

    // In each iteration of the loop, we scan `eval` with a stride of 4. At the end of the loop
    // we are guaranteed that `eval.len() == 2`.
    while eval.len() != 2 {
        let mut next_eval = array![];
        let mut next_x_coords = array![];
        while let Some(boxed_tuple) = eval.multi_pop_front::<4>() {
            let [v0, v1, v2, v3]: [QM31; 4] = boxed_tuple.unbox();
            let [x0, x1]: [M31; 2] = x_coords.multi_pop_front::<2>().unwrap().unbox();
            next_eval.append(fri_fold(v0, v1, x0.inverse(), alpha));
            next_eval.append(fri_fold(v2, v3, x1.inverse(), alpha));
            next_x_coords.append(CirclePointM31Impl::double_x(x0));
        }
        assert!(eval.is_empty());
        alpha = alpha * alpha;
        eval = next_eval.span();
        x_coords = next_x_coords.span();
    }
    let [v0, v1]: [QM31; 2] = (*eval.try_into().unwrap()).unbox();
    let [x0]: [M31; 1] = (*x_coords.try_into().unwrap()).unbox();
    fri_fold(v0, v1, x0.inverse(), alpha)
}

/// Proof of an individual FRI layer.
#[derive(Drop, Clone, Debug, Serde)]
pub struct FriLayerProof {
    /// Values that the verifier needs but cannot deduce from previous computations, in the
    /// order they are needed. This complements the values that were queried. These must be
    /// supplied directly to the verifier.
    pub fri_witness: Span<QM31>,
    pub decommitment: MerkleDecommitment<MerkleHasher>,
    pub commitment: Hash,
}

#[derive(Debug, Drop)]
pub enum FriVerificationError {
    InvalidNumFriLayers,
    LayerEvaluationsInvalid,
    LastLayerDegreeInvalid,
    LastLayerEvaluationsInvalid,
    LastLayerLogDegreeMustBeZero,
    ProofOfWork,
    SecurityBitsTooLow,
    InvalidFoldStep,
    ProofOfWorkBitsTooLow,
}

impl FriVerificationErrorDisplay of core::fmt::Display<FriVerificationError> {
    fn fmt(
        self: @FriVerificationError, ref f: core::fmt::Formatter,
    ) -> Result<(), core::fmt::Error> {
        match self {
            FriVerificationError::InvalidNumFriLayers => write!(f, "Invalid number of FRI layers"),
            FriVerificationError::LayerEvaluationsInvalid => write!(f, "Invalid layer evaluations"),
            FriVerificationError::LastLayerDegreeInvalid => write!(f, "Invalid last layer degree"),
            FriVerificationError::LastLayerEvaluationsInvalid => write!(
                f, "Invalid last layer evaluations",
            ),
            FriVerificationError::LastLayerLogDegreeMustBeZero => write!(
                f, "Last layer log degree must be zero",
            ),
            FriVerificationError::ProofOfWork => write!(f, "Proof Of Work verification failed"),
            FriVerificationError::SecurityBitsTooLow => write!(f, "Security bits are too low"),
            FriVerificationError::InvalidFoldStep => write!(f, "Unsupported FRI fold step"),
            FriVerificationError::ProofOfWorkBitsTooLow => write!(
                f, "Number of proof of work bits is too low",
            ),
        }
    }
}
