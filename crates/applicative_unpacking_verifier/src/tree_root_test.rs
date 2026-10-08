use circuits::blake::HashValue;
use circuits::context::TraceContext;
use circuits::ops::Guess;
use circuits_unpacker::unpacker::Node;
use stwo::core::fields::qm31::QM31;

use super::bind_root;

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
