use circuits::blake::{HashValue, blake2s_u32s};
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::ops::{Constant, Guess, eq};
use circuits_unpacker::unpacker::Node;
use itertools::{Itertools, zip_eq};
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::Blake2Felt252;
use stwo::core::fields::qm31::QM31;

#[cfg(test)]
#[path = "aggregator_test.rs"]
mod test;

/// The recursion tree's leaf-slot capacity; the aggregator wrapper takes the first slot, so at
/// most `CAPACITY - 1` tasks are aggregated. Must equal `aggregator_wrapper.cairo`'s CAPACITY.
pub const CAPACITY: usize = 512;

/// Guesses `aggregated_output_digest` and asserts that `[wrapper_program_hash, every task slot's
/// digest..., aggregated_output_digest]` is the preimage of `wrapper_digest` under the leaf
/// bootloader's `blake2s(cairo0-encode(felts))`; returns the bound digest.
// TODO(yair): slot order is bound to the wrapper's task order; consider a permutation argument
// instead to allow first-come-first-served parallel tree folding.
pub fn open_aggregator_output<Value: IValue>(
    context: &mut Context<Value>,
    wrapper_program_hash: &Felt,
    aggregated_output_digest: [u32; 8],
    wrapper_digest: &HashValue<Var>,
    task_leaves: &[Node<Var>],
) -> HashValue<Var> {
    let aggregated_output_digest =
        HashValue(aggregated_output_digest.map(Value::pack_u32)).guess(context);

    // The encoded program-hash prefix, then the wrapper output's digests.
    let zero = QM31::pack_u32(0).constant(context);
    let mut preimage_words = Blake2Felt252::encode_felts_to_u32s(&[*wrapper_program_hash])
        .into_iter()
        .map(|word| QM31::pack_u32(word).constant(context))
        .collect_vec();
    for word in task_leaves
        .iter()
        .flat_map(|leaf| leaf.subtree_hash.iter())
        .chain(aggregated_output_digest.iter())
    {
        // A digest is eight u32s; `encode_felts_to_u32s` encodes each u32 — a felt below `2^63` —
        // as a zero word followed by the value itself.
        preimage_words.push(zero);
        preimage_words.push(*word);
    }

    let n_bytes = 4 * preimage_words.len();
    let digest = blake2s_u32s(context, preimage_words, n_bytes);
    for (digest_word, wrapper_word) in zip_eq(digest.iter(), wrapper_digest.iter()) {
        eq(context, *digest_word.raw(), *wrapper_word.raw());
    }
    aggregated_output_digest
}
