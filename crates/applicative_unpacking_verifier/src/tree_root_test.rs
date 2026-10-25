use circuit_multiverifier::test_utils::{
    self, MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH, MULTIVERIFIER_PREPROCESSED_ROOT, PCS_CONFIG,
    PRIVACY_CAIRO_VERIFIER_OUTPUT_DIGEST, PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT,
    multiverifier_preprocessed_column_log_sizes, native_blake_u32s,
};
use circuit_multiverifier::verify::{
    CircuitVerifierInput, SharedConfig, shared_config, verify_child,
};
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuits::blake::HashValue;
use circuits::context::TraceContext;
use circuits::ops::Guess;
use circuits_unpacker::tree::BinaryTree;
use circuits_unpacker::unpacker::{Node, RecursionTreeParams};
use itertools::{Itertools, chain};
use stwo::core::fields::qm31::QM31;

use super::{bind_root, unpack_verified_tree};
use crate::aggregator::CAPACITY;

const CIRCUIT_HASH_WORDS: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
const OUTPUT_WORDS: [u32; 8] = [9, 10, 11, 12, 13, 14, 15, 16];

fn bind_root_test_helper(circuit_hash_words: [u32; 8], output_words: [u32; 8]) -> TraceContext {
    let mut context = TraceContext::default();
    let root = Node::<QM31> {
        circuit_hash: HashValue::from(CIRCUIT_HASH_WORDS),
        subtree_hash: HashValue::from(OUTPUT_WORDS),
    }
    .guess(&mut context);
    let circuit_hash = HashValue::<QM31>::from(circuit_hash_words).guess(&mut context);
    let output_digest = HashValue::<QM31>::from(output_words).guess(&mut context);
    bind_root(&mut context, &root, &circuit_hash, &output_digest);
    context
}

#[test]
fn bind_root_accepts_a_matching_contribution() {
    let context = bind_root_test_helper(CIRCUIT_HASH_WORDS, OUTPUT_WORDS);
    assert!(context.finalize(true).is_circuit_valid());
}

#[test]
fn bind_root_rejects_a_circuit_hash_mismatch() {
    let mut mismatched = CIRCUIT_HASH_WORDS;
    mismatched[3] += 1;
    let context = bind_root_test_helper(mismatched, OUTPUT_WORDS);
    assert!(!context.finalize(true).is_circuit_valid());
}

#[test]
fn bind_root_rejects_an_output_mismatch() {
    let mut mismatched = OUTPUT_WORDS;
    mismatched[7] += 1;
    let context = bind_root_test_helper(CIRCUIT_HASH_WORDS, mismatched);
    assert!(!context.finalize(true).is_circuit_valid());
}

/// The stored multiverifier fixture as a tree-root input: a real self-fold root proof over two
/// identical (privacy) Cairo verifier proofs, plus the witness tree it committed to.
struct RootFixture {
    shared_config: SharedConfig,
    input: CircuitVerifierInput<QM31>,
    leaf_node: Node<QM31>,
    leaf_circuit: HashValue<QM31>,
    multiverifier_circuit: HashValue<QM31>,
}

fn root_fixture() -> RootFixture {
    let shared_config = shared_config(multiverifier_preprocessed_column_log_sizes(), PCS_CONFIG);
    let leaf_circuit_hash = test_utils::leaf_circuit_hash(
        PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT.into(),
        &shared_config,
    );
    let multiverifier_circuit_hash =
        test_utils::leaf_circuit_hash(MULTIVERIFIER_PREPROCESSED_ROOT.into(), &shared_config);

    // The root's output digest: the leaf's `(circuit_hash ‖ outputs)` contribution, twice.
    let contribution: Vec<u32> =
        leaf_circuit_hash.into_iter().chain(PRIVACY_CAIRO_VERIFIER_OUTPUT_DIGEST).collect();
    let output_digest =
        native_blake_u32s(&chain!(&contribution, &contribution).copied().collect_vec());

    let bytes = std::fs::read(MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH).unwrap();
    let proof =
        deserialize_proof_with_config(&mut bytes.as_slice(), &shared_config.proof_config).unwrap();
    RootFixture {
        shared_config,
        input: CircuitVerifierInput {
            proof,
            preprocessed_root: MULTIVERIFIER_PREPROCESSED_ROOT.into(),
            output_digest: output_digest.into(),
        },
        leaf_node: Node {
            circuit_hash: HashValue::from(leaf_circuit_hash),
            subtree_hash: HashValue::from(PRIVACY_CAIRO_VERIFIER_OUTPUT_DIGEST),
        },
        leaf_circuit: HashValue::from(leaf_circuit_hash),
        multiverifier_circuit: HashValue::from(multiverifier_circuit_hash),
    }
}

/// Verifies the fixture's root proof and unpacks the witness tree under it at the unpacker's
/// fixed capacity, exercising the padding copy-ups.
fn verify_fixture_tree(
    fixture: RootFixture,
    leaf_set: &[HashValue<QM31>],
    tree: &BinaryTree<Node<QM31>, HashValue<QM31>>,
) -> bool {
    let node_set = [fixture.multiverifier_circuit.clone()];
    let params = RecursionTreeParams::new(CAPACITY, leaf_set, &node_set);
    let mut context = TraceContext::default();
    let (root_circuit_hash, root_output_digest) =
        verify_child(&mut context, fixture.input, &fixture.shared_config);
    unpack_verified_tree(&mut context, tree, &params, &root_circuit_hash, &root_output_digest);
    let finalized = context.finalize(false);
    finalized.circuit().check_yields();
    finalized.is_circuit_valid()
}

#[test]
fn verifies_a_real_root_proof() {
    let fixture = root_fixture();
    let tree = BinaryTree::Internal(
        fixture.multiverifier_circuit.clone(),
        Box::new([
            BinaryTree::Leaf(fixture.leaf_node.clone()),
            BinaryTree::Leaf(fixture.leaf_node.clone()),
        ]),
    );
    let leaf_set = [fixture.leaf_circuit.clone()];
    assert!(verify_fixture_tree(fixture, &leaf_set, &tree));
}

/// A tree with a different leaf multiset reconstructs a different root, failing the binding.
#[test]
fn rejects_a_tampered_tree() {
    let fixture = root_fixture();
    let mut tampered_leaf = fixture.leaf_node.clone();
    tampered_leaf.subtree_hash = HashValue::from({
        let mut words = PRIVACY_CAIRO_VERIFIER_OUTPUT_DIGEST;
        words[0] ^= 1;
        words
    });
    let tree = BinaryTree::Internal(
        fixture.multiverifier_circuit.clone(),
        Box::new([BinaryTree::Leaf(fixture.leaf_node.clone()), BinaryTree::Leaf(tampered_leaf)]),
    );
    let leaf_set = [fixture.leaf_circuit.clone()];
    assert!(!verify_fixture_tree(fixture, &leaf_set, &tree));
}

/// Same, tampering the leaf's `circuit_hash` — whitelisted, so the root binding is what fails.
#[test]
fn rejects_a_tampered_leaf_circuit_hash() {
    let fixture = root_fixture();
    let mut words = test_utils::leaf_circuit_hash(
        PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT.into(),
        &fixture.shared_config,
    );
    words[0] ^= 1;
    let mut tampered_leaf = fixture.leaf_node.clone();
    tampered_leaf.circuit_hash = HashValue::from(words);
    let tree = BinaryTree::Internal(
        fixture.multiverifier_circuit.clone(),
        Box::new([BinaryTree::Leaf(fixture.leaf_node.clone()), BinaryTree::Leaf(tampered_leaf)]),
    );
    let leaf_set = [fixture.leaf_circuit.clone(), HashValue::from(words)];
    assert!(!verify_fixture_tree(fixture, &leaf_set, &tree));
}
