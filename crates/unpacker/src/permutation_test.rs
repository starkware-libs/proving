use circuits::blake::HashValue;
use circuits::circuit::Circuit;
use circuits::context::{Context, TraceContext, Var};
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::Guess;
use circuits::wrappers::U32Wrapper;
use stwo::core::fields::qm31::QM31;

use crate::permutation::permute_hash_values;

fn guess_hashes(context: &mut TraceContext, words: &[[u32; 8]]) -> Vec<HashValue<Var>> {
    words.iter().map(|w| HashValue::from(*w).guess(context)).collect()
}

#[test]
fn test_permute_hash_values() {
    let mut context = TraceContext::default();

    let inputs_u32: [[u32; 8]; 3] = [
        [1, 2, 3, 4, 5, 6, 7, 8],
        [9, 10, 11, 12, 13, 14, 15, 16],
        [17, 18, 19, 20, 21, 22, 23, 24],
    ];
    // A genuine reordering of the input hashes (reverse).
    let outputs_u32: Vec<[u32; 8]> = inputs_u32.iter().rev().copied().collect();

    let inputs = guess_hashes(&mut context, &inputs_u32);
    let outputs = guess_hashes(&mut context, &outputs_u32);

    permute_hash_values(&mut context, &inputs, &outputs);

    let context = context.finalize(false);
    context.circuit().check_yields();
    assert!(context.is_circuit_valid());
}

/// `outputs` that is not a permutation of `inputs` (a hash absent from the inputs) has no source
/// index, so witness generation panics.
#[test]
#[should_panic(expected = "output hash is not a permutation of the inputs")]
fn test_permute_hash_values_panics_on_non_permutation() {
    let mut context = TraceContext::default();

    let inputs_u32: [[u32; 8]; 2] = [[1, 2, 3, 4, 5, 6, 7, 8], [9, 10, 11, 12, 13, 14, 15, 16]];
    // Second hash does not appear among the inputs.
    let outputs_u32: [[u32; 8]; 2] = [[9, 10, 11, 12, 13, 14, 15, 16], [99, 2, 3, 4, 5, 6, 7, 8]];

    let inputs = guess_hashes(&mut context, &inputs_u32);
    let outputs = guess_hashes(&mut context, &outputs_u32);

    permute_hash_values(&mut context, &inputs, &outputs);
}

/// An output hash assembled from words of *different* input hashes (each word-column is
/// individually a valid permutation, but the mixed hash is not one of the inputs) is not a
/// permutation, so witness generation panics.
#[test]
#[should_panic(expected = "output hash is not a permutation of the inputs")]
fn test_permute_hash_values_panics_on_column_mixing() {
    let mut context = TraceContext::default();

    let inputs_u32: [[u32; 8]; 2] = [[1, 2, 3, 4, 5, 6, 7, 8], [11, 12, 13, 14, 15, 16, 17, 18]];
    // Each column below is a valid permutation of the inputs' column, but word 1 is swapped between
    // the two hashes, so neither output hash equals a whole input hash.
    let outputs_u32: [[u32; 8]; 2] = [[1, 12, 3, 4, 5, 6, 7, 8], [11, 2, 13, 14, 15, 16, 17, 18]];

    let inputs = guess_hashes(&mut context, &inputs_u32);
    let outputs = guess_hashes(&mut context, &outputs_u32);

    permute_hash_values(&mut context, &inputs, &outputs);
}

/// Inputs containing duplicate hashes: matching must consume the per-value index list one at a time
/// and still produce a valid bijection.
#[test]
fn test_permute_hash_values_with_duplicates() {
    let mut context = TraceContext::default();

    let inputs_u32: [[u32; 8]; 3] = [
        [1, 2, 3, 4, 5, 6, 7, 8],
        [1, 2, 3, 4, 5, 6, 7, 8], // duplicate of input 0
        [9, 10, 11, 12, 13, 14, 15, 16],
    ];
    // Same multiset, reordered; the two equal hashes are mapped to the two equal-valued positions.
    let outputs_u32: [[u32; 8]; 3] =
        [[9, 10, 11, 12, 13, 14, 15, 16], [1, 2, 3, 4, 5, 6, 7, 8], [1, 2, 3, 4, 5, 6, 7, 8]];

    let inputs = guess_hashes(&mut context, &inputs_u32);
    let outputs = guess_hashes(&mut context, &outputs_u32);

    permute_hash_values(&mut context, &inputs, &outputs);

    let context = context.finalize(false);
    context.circuit().check_yields();
    assert!(context.is_circuit_valid());
}

/// Builds a `HashValue<Value>` from raw words for any `Value` (witness or witness-less), so the
/// same circuit-building code can run under `QM31` and `NoValue`.
fn hash_from<Value: IValue>(words: [u32; 8]) -> HashValue<Value> {
    HashValue(words.map(|w| U32Wrapper::new_unsafe(Value::pack_u32(w))))
}

/// Runs `permute_hash_values` in a fresh `Context<Value>` and returns the finalized circuit
/// structure (gates only — no witness values).
fn build_permute_circuit<Value: IValue>(
    inputs_u32: &[[u32; 8]],
    outputs_u32: &[[u32; 8]],
) -> Circuit {
    let mut ctx = Context::<Value>::default();
    let inputs: Vec<HashValue<Var>> =
        inputs_u32.iter().map(|w| hash_from::<Value>(*w).guess(&mut ctx)).collect();
    let outputs: Vec<HashValue<Var>> =
        outputs_u32.iter().map(|w| hash_from::<Value>(*w).guess(&mut ctx)).collect();
    permute_hash_values(&mut ctx, &inputs, &outputs);
    ctx.finalize(false).context.circuit
}

/// The circuit topology must not depend on the witness: building with concrete `QM31` values (where
/// the source indices are recovered by matching wire values) must yield exactly the same gates as
/// building with `NoValue` (where no values exist and matching degenerates to sequential indices).
#[test]
fn test_permute_hash_values_structure_is_witness_independent() {
    let inputs_u32: [[u32; 8]; 3] = [
        [1, 2, 3, 4, 5, 6, 7, 8],
        [9, 10, 11, 12, 13, 14, 15, 16],
        [17, 18, 19, 20, 21, 22, 23, 24],
    ];
    // A non-trivial reordering, so the QM31 build's value-matching yields a non-identity
    // permutation while the NoValue build necessarily falls back to the identity — yet the
    // gates must match.
    let outputs_u32: Vec<[u32; 8]> = inputs_u32.iter().rev().copied().collect();

    let with_values = build_permute_circuit::<QM31>(&inputs_u32, &outputs_u32);
    let without_values = build_permute_circuit::<NoValue>(&inputs_u32, &outputs_u32);

    assert_eq!(with_values, without_values);
}
