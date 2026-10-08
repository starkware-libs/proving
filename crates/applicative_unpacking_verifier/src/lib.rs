//! Building blocks of the applicative unpacking verifier: a circuit that verifies the recursive
//! proof tree's root proof, unpacks the tree's leaf entries, and matches them against what the
//! aggregator committed to.

pub mod aggregator;
pub mod tree_root;
