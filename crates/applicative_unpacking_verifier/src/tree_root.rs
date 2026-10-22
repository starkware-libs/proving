use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::ops::eq;
use circuits_unpacker::tree::BinaryTree;
use circuits_unpacker::unpacker::{Node, RecursionTreeParams, unpack_recursion_tree};
use itertools::zip_eq;
use stwo::core::fields::qm31::QM31;

#[cfg(test)]
#[path = "tree_root_test.rs"]
mod test;

/// Unpacks the tree a verified recursive proof commits to, returning its `params.capacity()` leaf
/// slots under [`unpack_recursion_tree`]'s contract: bound only as a multiset, and a zero
/// `subtree_hash` marks padding rather than a committed leaf.
///
/// `tree` is the witness for the unpacking, `params` the circuit-hash whitelists, and
/// `root_circuit_hash`/`root_output_digest` the root proof's contribution from `verify_child`.
pub fn unpack_verified_tree<Value: IValue>(
    context: &mut Context<Value>,
    tree: &BinaryTree<Node<QM31>, HashValue<QM31>>,
    params: &RecursionTreeParams<'_>,
    root_circuit_hash: &HashValue<Var>,
    root_output_digest: &HashValue<Var>,
) -> Vec<Node<Var>> {
    let unpacked = unpack_recursion_tree(context, tree, params);
    // Check that `unpacked.root` is consistent with the verified proof's contribution. For a tree
    // of two or more leaves, the unpacker already constrained its `circuit_hash` to the
    // internal-node whitelist. A one-leaf tree's root is its leaf, whose circuit hash, a leaf
    // circuit's, can't equal the verified multiverifier proof's, so this check rejects it. It's
    // fine since the applicative tree always holds the wrapper's leaf and at least
    // one task leaf (and even without this invariant - a single-leaf tree gets self-folded once).
    bind_root(context, &unpacked.root, root_circuit_hash, root_output_digest);
    unpacked.leaves
}

/// Pins the unpacker's resolved root to the verified root proof's contribution, word by word.
fn bind_root<Value: IValue>(
    context: &mut Context<Value>,
    root: &Node<Var>,
    circuit_hash: &HashValue<Var>,
    output_digest: &HashValue<Var>,
) {
    for (root_word, proof_word) in zip_eq(root.circuit_hash.iter(), circuit_hash.iter()) {
        eq(context, *root_word.raw(), *proof_word.raw());
    }
    for (root_word, proof_word) in zip_eq(root.subtree_hash.iter(), output_digest.iter()) {
        eq(context, *root_word.raw(), *proof_word.raw());
    }
}
