//! In-circuit Merkle "unpacker": proving a Merkle `root` commits to a whole leaf multiset, where
//! each entry is a [`Node`](unpacker::Node) pairing a `circuit_hash` with a `subtree_hash`.
//!
//! - [`permutation`] — multiset permutation over fixed-arity word tuples.
//! - [`tree`] — the [`BinaryTree`](tree::BinaryTree) shape.
//! - [`unpacker`] — the commitment circuit
//!   ([`unpack_recursion_tree`](unpacker::unpack_recursion_tree)).

pub mod permutation;
pub mod tree;
pub mod unpacker;
