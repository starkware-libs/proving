use circuit_common::N_RESERVED;
use circuit_multiverifier::verify::{CircuitVerifierInput, SharedConfig, verify_child};
use circuits::blake::HashValue;
use circuits::context::{Context, FinalizedContext};
use circuits::ivalue::{IValue, NoValue};
use circuits_stark_verifier::proof::empty_proof;
use circuits_unpacker::tree::BinaryTree;
use circuits_unpacker::unpacker::{Node, RecursionTreeParams};
use itertools::Itertools;
use starknet_types_core::felt::Felt;
use stwo::core::fields::qm31::QM31;

use crate::aggregator::{CAPACITY, open_aggregator_output};
use crate::tree_root::unpack_verified_tree;

#[cfg(test)]
#[path = "unpacking_verifier_test.rs"]
mod test;

/// The parameters that fix the emitted circuit, as opposed to the per-proof witnesses: the
/// circuit-hash whitelists and the aggregator wrapper's program hash.
pub struct UnpackingCircuitParams<'a> {
    /// The circuit hashes allowed for the tree's leaves.
    pub leaf_circuit_whitelist: &'a [HashValue<QM31>],
    /// The circuit hashes allowed for the tree's internal nodes (the multiverifier circuits).
    pub internal_node_circuit_whitelist: &'a [HashValue<QM31>],
    /// The aggregator wrapper task's program hash: its encoding heads the recomputed digest, so
    /// only this program's outputs verify.
    pub aggregator_wrapper_program_hash: Felt,
}

/// Builds the applicative unpacking verifier: a circuit proving that the aggregator wrapper
/// aggregated exactly the tasks the recursive proof tree proves.
///
/// It verifies the tree's root (multiverifier) proof against `shared_config` and unpacks the
/// witness `tree` under it ([`unpack_verified_tree`]). The wrapper's own leaf proof is the tree's
/// first leaf, whose digest [`open_aggregator_output`] recomputes from the other slots.
///
/// The circuit's `N_RESERVED` outputs are the U256-encoded
/// `poseidon(modified_aggregator_program_hash, aggregated_output...)`, which a consumer opens by
/// recomputing the poseidon hash.
pub fn build_applicative_unpacking_verifier_circuit<Value: IValue>(
    tree_root_input: CircuitVerifierInput<Value>,
    shared_config: &SharedConfig,
    tree: &BinaryTree<Node<QM31>, HashValue<QM31>>,
    aggregated_output_digest: [u32; 8],
    params: &UnpackingCircuitParams<'_>,
) -> FinalizedContext<Value> {
    let mut context = Context::new(N_RESERVED);

    let recursion_tree = RecursionTreeParams::new(
        CAPACITY,
        params.leaf_circuit_whitelist,
        params.internal_node_circuit_whitelist,
    );
    let (root_circuit_hash, root_output_digest) =
        verify_child(&mut context, tree_root_input, shared_config);
    let leaves = unpack_verified_tree(
        &mut context,
        tree,
        &recursion_tree,
        &root_circuit_hash,
        &root_output_digest,
    );

    let (wrapper_leaf, task_leaves) =
        leaves.split_first().expect("the unpacker returns all `CAPACITY` leaf slots");
    let aggregated_output_digest = open_aggregator_output(
        &mut context,
        &params.aggregator_wrapper_program_hash,
        aggregated_output_digest,
        &wrapper_leaf.subtree_hash,
        task_leaves,
    );
    context.set_outputs(&aggregated_output_digest.iter().map(|word| *word.raw()).collect_vec());

    let context = context.finalize(false);
    #[cfg(test)]
    context.circuit().check_yields();
    context
}

/// Builds the applicative unpacking verifier topology for `shared_config` and `params` — the
/// tree's node whitelists and the wrapper's program hash, which are baked into the circuit. The
/// witnesses (proofs, tree, final digest) only seed values, so any well-formed placeholders yield
/// the one circuit the parameters fix.
pub fn build_applicative_unpacking_verifier_context(
    shared_config: &SharedConfig,
    params: &UnpackingCircuitParams<'_>,
) -> FinalizedContext<NoValue> {
    let empty_input = CircuitVerifierInput {
        proof: empty_proof(&shared_config.proof_config),
        preprocessed_root: HashValue::no_value(),
        output_digest: HashValue::no_value(),
    };
    let zero_node = || Node::<QM31> {
        circuit_hash: HashValue::from([0u32; 8]),
        subtree_hash: HashValue::from([0u32; 8]),
    };
    let tree = BinaryTree::Internal(
        HashValue::from([0u32; 8]),
        Box::new([BinaryTree::Leaf(zero_node()), BinaryTree::Leaf(zero_node())]),
    );
    build_applicative_unpacking_verifier_circuit::<NoValue>(
        empty_input,
        shared_config,
        &tree,
        [0u32; 8],
        params,
    )
}
