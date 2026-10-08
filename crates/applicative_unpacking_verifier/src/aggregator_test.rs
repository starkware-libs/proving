use circuit_multiverifier::test_utils::native_blake_u32s;
use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue};
use circuits::context::{TraceContext, Var};
use circuits::ops::Guess;
use circuits_unpacker::unpacker::Node;
use itertools::Itertools;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::Blake2Felt252;
use stwo::core::fields::qm31::QM31;

use super::{CAPACITY, open_aggregator_output};

/// Arbitrary non-zero leaf digests (a real `subtree_hash` is a Blake2s digest).
fn leaf_hash(i: u32) -> [u32; 8] {
    std::array::from_fn(|j| 0x1000_0000 + i * 8 + j as u32)
}

const AGGREGATED_OUTPUT_DIGEST: [u32; 8] = [21, 22, 23, 24, 25, 26, 27, 28];

/// The output's final digest, guessed as the enclosing circuit guesses it.
fn guess_aggregated_output_digest(context: &mut TraceContext) -> HashValue<Var> {
    HashValue::<QM31>::from(AGGREGATED_OUTPUT_DIGEST).guess(context)
}

/// An arbitrary program-hash felt above `2^63` (a blake program hash virtually always is).
fn wrapper_program_hash() -> Felt {
    Felt::from_hex_unchecked("0x71234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd")
}

/// The wrapper's output for the given tasks: the leaf digests (in the given order), zero digests
/// padding to `CAPACITY - 1`, and the aggregated-output digest.
fn wrapper_output_words(leaf_digests: &[[u32; 8]]) -> Vec<u32> {
    let mut words: Vec<u32> = leaf_digests.iter().flatten().copied().collect();
    words.resize((CAPACITY - 1) * BLAKE2S_DIGEST_N_WORDS, 0);
    words.extend(AGGREGATED_OUTPUT_DIGEST);
    words
}

/// The wrapper's leaf digest: the leaf bootloader's digest over the encoded
/// `[wrapper_program_hash, output words...]` felts.
fn native_wrapper_digest(output_words: &[u32]) -> [u32; 8] {
    let felts = std::iter::once(wrapper_program_hash())
        .chain(output_words.iter().map(|word| Felt::from(*word)))
        .collect_vec();
    native_blake_u32s(&Blake2Felt252::encode_felts_to_u32s(&felts))
}

/// Guesses the tree's `CAPACITY` leaf slots: the wrapper in slot 0, `n` task slots, then all-zero
/// padding slots.
fn guess_leaves(context: &mut TraceContext, n: u32, wrapper_digest: [u32; 8]) -> Vec<Node<Var>> {
    (0..CAPACITY as u32)
        .map(|i| {
            let node = if i == 0 {
                Node::<QM31> {
                    circuit_hash: HashValue::from([0xaau32; 8]),
                    subtree_hash: HashValue::from(wrapper_digest),
                }
            } else if i <= n {
                Node::<QM31> {
                    circuit_hash: HashValue::from([i; 8]),
                    subtree_hash: HashValue::from(leaf_hash(i - 1)),
                }
            } else {
                Node::<QM31> {
                    circuit_hash: HashValue::from([0u32; 8]),
                    subtree_hash: HashValue::from([0u32; 8]),
                }
            };
            node.guess(context)
        })
        .collect_vec()
}

/// Builds the gadget over `n` task leaves, the wrapper's slot holding `wrapper_digest`.
fn build(n: u32, wrapper_digest: [u32; 8]) -> TraceContext {
    let mut context = TraceContext::default();
    let leaves = guess_leaves(&mut context, n, wrapper_digest);
    let aggregated_output_digest = guess_aggregated_output_digest(&mut context);
    let (wrapper_leaf, task_leaves) = leaves.split_first().unwrap();
    open_aggregator_output(
        &mut context,
        &wrapper_program_hash(),
        &aggregated_output_digest,
        &wrapper_leaf.subtree_hash,
        task_leaves,
    );
    context
}

#[test]
fn accepts_a_matching_output() {
    let digests = (0..3).map(leaf_hash).collect_vec();
    let finalized =
        build(3, native_wrapper_digest(&wrapper_output_words(&digests))).finalize(false);
    finalized.circuit().check_yields();
    assert!(finalized.is_circuit_valid());
}

#[test]
fn accepts_a_full_capacity_output() {
    let n = CAPACITY as u32 - 1;
    let digests = (0..n).map(leaf_hash).collect_vec();
    let finalized =
        build(n, native_wrapper_digest(&wrapper_output_words(&digests))).finalize(false);
    finalized.circuit().check_yields();
    assert!(finalized.is_circuit_valid());
}

/// A tampered wrapper slot does not match the digest recomputed from the other slots.
#[test]
fn rejects_a_tampered_wrapper_digest() {
    let digests = (0..3).map(leaf_hash).collect_vec();
    let mut wrapper_digest = native_wrapper_digest(&wrapper_output_words(&digests));
    wrapper_digest[0] ^= 1;
    assert!(!build(3, wrapper_digest).finalize(false).is_circuit_valid());
}

/// A wrapper that hashed the leaves in a different order does not match the tree's slot order.
#[test]
fn rejects_reordered_digests() {
    let digests = (0..3).rev().map(leaf_hash).collect_vec();
    assert!(
        !build(3, native_wrapper_digest(&wrapper_output_words(&digests)))
            .finalize(false)
            .is_circuit_valid()
    );
}

/// A wrapper that omitted a proven leaf (a zero digest in its place) does not match the tree.
#[test]
fn rejects_a_missing_leaf() {
    let digests = (0..2).map(leaf_hash).collect_vec();
    assert!(
        !build(3, native_wrapper_digest(&wrapper_output_words(&digests)))
            .finalize(false)
            .is_circuit_valid()
    );
}

#[test]
#[should_panic(expected = "CAPACITY - 1 task slots")]
fn panics_on_a_wrong_slot_count() {
    let mut context = TraceContext::default();
    let leaves = guess_leaves(&mut context, 1, [0u32; 8]);
    let aggregated_output_digest = guess_aggregated_output_digest(&mut context);
    let (wrapper_leaf, task_leaves) = leaves.split_first().unwrap();
    open_aggregator_output(
        &mut context,
        &wrapper_program_hash(),
        &aggregated_output_digest,
        &wrapper_leaf.subtree_hash,
        &task_leaves[..CAPACITY - 2],
    );
}

/// The emitted circuit depends only on the capacity, never on the real task count.
#[test]
fn circuit_is_fixed_across_n() {
    let circuit_for = |n: u32| {
        let digests = (0..n).map(leaf_hash).collect_vec();
        build(n, native_wrapper_digest(&wrapper_output_words(&digests))).finalize(false)
    };
    let reference = circuit_for(1);
    for n in 2..=4 {
        assert!(
            circuit_for(n).circuit() == reference.circuit(),
            "circuit differs between n=1 and n={n}"
        );
    }
}
