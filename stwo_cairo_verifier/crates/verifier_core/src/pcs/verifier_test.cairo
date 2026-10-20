use crate::Hash;
use crate::channel::Channel;
use crate::fri::FriParams;
use super::verifier::{CommitmentSchemeVerifier, CommitmentSchemeVerifierImpl};

/// Creates a commitment scheme verifier with the given log degree bound and arbitrary FRI params.
fn new_commitment_scheme(log_degree_bound: u32) -> CommitmentSchemeVerifier {
    let fri_params = FriParams {
        pow_bits: 20,
        log_blowup_factor: 1,
        log_last_layer_degree_bound: 0,
        n_queries: 1,
        fold_step: 1,
    };
    CommitmentSchemeVerifierImpl::new(log_degree_bound, fri_params)
}

/// Commits a tree with the given column log degree bounds and an arbitrary root.
fn commit_tree(ref commitment_scheme: CommitmentSchemeVerifier, column_log_deg_bounds: Span<u32>) {
    let mut root_data = array![0, 0, 0, 0, 0, 0, 0, 0].span();
    let root: Hash = Serde::deserialize(ref root_data).unwrap();
    let mut channel: Channel = Default::default();
    commitment_scheme.commit(root, column_log_deg_bounds, ref channel);
}

#[test]
fn test_column_indices_per_tree_per_log_deg_bound() {
    let mut commitment_scheme = new_commitment_scheme(3);
    commit_tree(ref commitment_scheme, array![3, 1, 3].span());
    commit_tree(ref commitment_scheme, array![3, 2].span());
    let res = commitment_scheme.column_indices_per_tree_per_log_deg_bound();
    assert_eq!(
        res,
        array![
            array![array![].span(), array![].span()].span(),
            array![array![1].span(), array![].span()].span(),
            array![array![].span(), array![1].span()].span(),
            array![array![0, 2].span(), array![0].span()].span(),
        ]
            .span(),
    );
}

/// A column whose log degree bound exceeds the commitment scheme's must be rejected.
#[test]
#[should_panic(
    expected: "Max column log degree bound must equal the commitment scheme's log degree bound",
)]
fn test_column_log_degree_bound_too_large_fail() {
    let mut commitment_scheme = new_commitment_scheme(3);
    commit_tree(ref commitment_scheme, array![3, 4].span());
}

/// A tree whose largest column log degree bound is below the commitment scheme's must be rejected.
#[test]
#[should_panic(
    expected: "Max column log degree bound must equal the commitment scheme's log degree bound",
)]
fn test_column_log_degree_bound_too_small_fail() {
    let mut commitment_scheme = new_commitment_scheme(3);
    commit_tree(ref commitment_scheme, array![2, 1].span());
}
