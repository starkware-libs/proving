use stwo_verifier_utils::zip_eq::zip_eq;
use crate::channel::{Channel, ChannelTrait};
use crate::circle::CirclePoint;
use crate::fields::m31::M31;
use crate::fields::qm31::{QM31, QM31Serde};
use crate::fri::{FriParams, FriProof, FriVerifierTrait};
use crate::pcs::quotients::fri_answers;
use crate::utils::{
    ArrayImpl, ColumnIndicesPerLogDegreeBound, ColumnIndicesPerTreePerLogDegreeBound, DictImpl,
    group_columns_by_log_degree_bound, pad_and_transpose_columns_per_log_deg_bound_per_tree,
};
use crate::vcs::MerkleHasher;
use crate::vcs::verifier::{MerkleDecommitment, MerkleVerifier, MerkleVerifierTrait};
use crate::{ColumnSpan, Hash, TreeArray, TreeSpan, queries};

/// Sampled mask values.
///
/// Each column is sampled at one of the following point sets:
/// - `[oods_point]`
/// - `[oods_point - g, oods_point]`
///   where `g` is the trace generator for that column.
///
/// `AIR::eval_composition_polynomial_at_point` ensures that each column has the correct
/// number of samples, while `CommitmentSchemeVerifierTrait::verify_values` ensures that the sampled
/// values themselves are valid.
pub type SampledValues = TreeSpan<ColumnSpan<Span<QM31>>>;

/// For each tree, stores all queried trace values, ordered first by descending column size,
/// then by column index, and finally by query position.
///
/// For illustration:
/// Suppose the first tree has two columns (c_1, c_3) with log degree bound 5,
/// one column (c_2) with log degree bound 4, and the queries are at positions 5 and 122.
///
/// In this case, `queried_values[0]` would be:
///
/// [
///     // size 5 columns
///     c_1@5, c_3@5, c_1@122, c_3@122,
///     // size 4 columns
///     c_2@fold(5), c_2@fold(122),
/// ],
///
/// where `c_1@5` denotes the value of column 1 at query position 5,
/// and `fold(5)` denotes the folded position of query 5.
pub type QueriedValues = TreeArray<Span<M31>>;

#[derive(Drop, Serde)]
pub struct CommitmentSchemeProof {
    pub commitments: TreeSpan<Hash>,
    /// Proof of work nonce ground before the OODS point is drawn.
    pub oods_pow_nonce: u64,
    pub sampled_values: SampledValues,
    pub decommitments: TreeArray<MerkleDecommitment<MerkleHasher>>,
    pub queried_values: QueriedValues,
    pub fri_proof: FriProof,
}

/// The verifier side of a FRI polynomial commitment scheme. See [super].
#[derive(Drop)]
pub struct CommitmentSchemeVerifier {
    /// The largest log degree bound of a committed polynomial.
    pub(crate) log_degree_bound: u32,
    /// The FRI parameters to verify against.
    pub(crate) fri_params: FriParams,
    /// The Merkle verifiers of each Merkle tree.
    trees: TreeArray<MerkleVerifier<MerkleHasher>>,
    /// For each tree, the indices of its columns grouped by log degree bound: entry `i` holds the
    /// indices of columns of log degree bound `i`.
    ///
    /// Both `trees` and `columns_per_log_deg_bound_per_tree` are guaranteed
    /// to always have the same length.
    columns_per_log_deg_bound_per_tree: TreeArray<ColumnIndicesPerLogDegreeBound>,
}

#[generate_trait]
pub impl CommitmentSchemeVerifierImpl of CommitmentSchemeVerifierTrait {
    /// Creates a verifier with no committed trees.
    ///
    /// `fri_params` are not read from the proof; the caller supplies the canonical, hardcoded
    /// parameters, so the prover cannot choose them. They are not validated here.
    fn new(log_degree_bound: u32, fri_params: FriParams) -> CommitmentSchemeVerifier {
        CommitmentSchemeVerifier {
            log_degree_bound,
            fri_params,
            trees: array![],
            columns_per_log_deg_bound_per_tree: array![],
        }
    }

    /// Returns the column indices per tree, per log degree bound, i.e. for each log degree bound,
    /// for each tree, a span of the column indices of that tree with that log degree bound.
    fn column_indices_per_tree_per_log_deg_bound(
        self: @CommitmentSchemeVerifier,
    ) -> ColumnIndicesPerTreePerLogDegreeBound {
        pad_and_transpose_columns_per_log_deg_bound_per_tree(
            self.columns_per_log_deg_bound_per_tree.span(),
        )
    }

    /// Reads a Merkle root commitment for a set of columns from the prover and registers it with
    /// the verifier.
    ///
    /// This function mixes the commitment root into the Fiat-Shamir channel, appends a new
    /// MerkleVerifier for this commitment to the verifier's state and stores the degree bounds of
    /// the committed columns.
    ///
    /// # Arguments
    /// * `commitment` - The Merkle root of the committed columns.
    /// * `log_degree_bound_per_column` - The log degree bounds for each column in the commitment.
    /// The largest must equal `self.log_degree_bound` (i.e. all trees are lifted to
    /// `self.log_degree_bound`).
    /// * `channel` - The Fiat-Shamir channel used for mixing and randomness.
    fn commit(
        ref self: CommitmentSchemeVerifier,
        commitment: Hash,
        log_degree_bound_per_column: ColumnSpan<u32>,
        ref channel: Channel,
    ) {
        // Mix the commitment root into the Fiat-Shamir channel.
        channel.mix_commitment(commitment);
        self
            .trees
            .append(
                MerkleVerifier {
                    root: commitment,
                    tree_height: self.log_degree_bound + self.fri_params.log_blowup_factor,
                    n_committed_columns: log_degree_bound_per_column.len(),
                },
            );
        let columns_per_log_deg_bound = group_columns_by_log_degree_bound(
            log_degree_bound_per_column,
        );
        assert!(
            columns_per_log_deg_bound.len() == self.log_degree_bound + 1,
            "Max column log degree bound must equal the commitment scheme's log degree bound",
        );
        self.columns_per_log_deg_bound_per_tree.append(columns_per_log_deg_bound);
    }

    /// Verifies that `proof.sampled_values` corresponds to the evaluations
    /// of the committed polynomials at `oods_point`.
    fn verify_values(
        self: CommitmentSchemeVerifier,
        oods_point: CirclePoint<QM31>,
        proof: CommitmentSchemeProof,
        ref channel: Channel,
    ) {
        let CommitmentSchemeProof {
            commitments: _,
            // Verified before the OODS point was drawn, see `verify`.
            oods_pow_nonce: _,
            sampled_values,
            decommitments,
            queried_values: queried_values_per_tree,
            fri_proof,
        } = proof;

        mix_sampled_values(sampled_values, ref channel);

        let random_coeff = channel.draw_secure_felt();

        // FRI commitment phase on OODS quotients.
        let mut fri_verifier = FriVerifierTrait::commit(
            ref channel, self.fri_params, fri_proof, self.log_degree_bound,
        );

        // Get FRI query positions. Gated by the query proof of work.
        fri_verifier.verify_proof_of_work(ref channel);
        let queries = fri_verifier.sample_query_positions(ref channel);
        let query_positions = queries.positions;
        // Verify Merkle decommitments.
        for (tree, (queried_values, decommitment)) in zip_eq(
            self.trees.span(), zip_eq(queried_values_per_tree.span(), decommitments),
        ) {
            tree.verify(query_positions, *queried_values, decommitment);
        }
        // Answer FRI queries.
        let fri_answers = fri_answers(
            self.column_indices_per_tree_per_log_deg_bound(),
            self.fri_params.log_blowup_factor,
            oods_point,
            sampled_values,
            random_coeff,
            query_positions,
            queried_values_per_tree,
            self.log_degree_bound,
        );

        fri_verifier.decommit(queries, fri_answers);
    }
}

#[inline]
fn mix_sampled_values(sampled_values: TreeSpan<ColumnSpan<Span<QM31>>>, ref channel: Channel) {
    let mut flattened_sampled_values = array![];

    for sampled_values in sampled_values {
        for column_sampled_values in *sampled_values {
            for sampled_value in *column_sampled_values {
                flattened_sampled_values.append(*sampled_value);
            };
        };
    }

    channel.mix_felts(flattened_sampled_values.span());
}
