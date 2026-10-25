use circuit_multiverifier::test_utils::{PCS_CONFIG, multiverifier_preprocessed_column_log_sizes};
use circuit_multiverifier::verify::{SharedConfig, shared_config};
use circuits::blake::HashValue;
use circuits_unpacker::tree::BinaryTree;
use circuits_unpacker::unpacker::Node;
use starknet_types_core::felt::Felt;
use stwo::core::fields::qm31::QM31;

use super::{UnpackingCircuitParams, build_applicative_unpacking_verifier_context};

fn test_shared_config() -> SharedConfig {
    shared_config(multiverifier_preprocessed_column_log_sizes(), PCS_CONFIG)
}

fn hash(value: u32) -> HashValue<QM31> {
    HashValue::from([value; 8])
}

/// An arbitrary program-hash felt.
fn wrapper_program_hash() -> Felt {
    Felt::from_hex_unchecked("0x71234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd")
}

#[test]
fn topology_builds() {
    let leaf_set = [hash(1)];
    let node_set = [hash(2)];
    let params = UnpackingCircuitParams {
        leaf_circuit_whitelist: &leaf_set,
        internal_node_circuit_whitelist: &node_set,
        aggregator_wrapper_program_hash: wrapper_program_hash(),
    };
    // `build_applicative_unpacking_verifier_circuit` runs `check_yields` itself under cfg(test).
    build_applicative_unpacking_verifier_context(&test_shared_config(), &params);
}

/// The emitted circuit depends only on the parameters, never on the witness tree's shape.
#[test]
fn circuit_is_fixed_across_tree_shapes() {
    use circuit_multiverifier::verify::CircuitVerifierInput;
    use circuits::ivalue::NoValue;
    use circuits_stark_verifier::proof::empty_proof;

    let leaf_set = [hash(1)];
    let node_set = [hash(2)];
    let params = UnpackingCircuitParams {
        leaf_circuit_whitelist: &leaf_set,
        internal_node_circuit_whitelist: &node_set,
        aggregator_wrapper_program_hash: wrapper_program_hash(),
    };
    let shared_config = test_shared_config();

    let zero_node = || Node::<QM31> {
        circuit_hash: HashValue::from([0u32; 8]),
        subtree_hash: HashValue::from([0u32; 8]),
    };
    let leaf = || BinaryTree::Leaf(zero_node());
    let node = |children: [BinaryTree<Node<QM31>, HashValue<QM31>>; 2]| {
        BinaryTree::Internal(HashValue::from([0u32; 8]), Box::new(children))
    };
    // Two, three (caterpillar), and four (balanced) leaves at the same capacity.
    let trees = [
        node([leaf(), leaf()]),
        node([node([leaf(), leaf()]), leaf()]),
        node([node([leaf(), leaf()]), node([leaf(), leaf()])]),
    ];

    let circuit_for = |tree: &BinaryTree<Node<QM31>, HashValue<QM31>>| {
        let empty_input = CircuitVerifierInput {
            proof: empty_proof(&shared_config.proof_config),
            preprocessed_root: HashValue::no_value(),
            output_digest: HashValue::no_value(),
        };
        super::build_applicative_unpacking_verifier_circuit::<NoValue>(
            empty_input,
            &shared_config,
            tree,
            [0u32; 8],
            &params,
        )
    };

    let reference = circuit_for(&trees[0]);
    for (i, tree) in trees.iter().enumerate().skip(1) {
        assert!(
            circuit_for(tree).circuit() == reference.circuit(),
            "the circuit differs between trees[0] and trees[{i}]"
        );
    }
}
