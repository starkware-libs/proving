use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::ops::eq;
use circuits_unpacker::unpacker::Node;
use itertools::zip_eq;

#[cfg(test)]
#[path = "tree_root_test.rs"]
mod test;

/// Pins the unpacker's resolved root to the verified root proof's contribution, word by word.
pub fn bind_root<Value: IValue>(
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
