use std::collections::HashMap;

use circuits::blake::HashValue;
use circuits::circuit::Permutation;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, eq};
use circuits::wrappers::{M31Wrapper, U32Wrapper};
use itertools::zip_eq;

#[cfg(test)]
#[path = "permutation_test.rs"]
pub mod test;

/// Constrains `outputs` to be a multiset permutation of `inputs`, treating each tuple (a
/// fixed-arity slice of `u32` words) as an atomic element. All tuples — inputs and outputs — must
/// have the same arity `w`.
///
/// Every word is *tagged* with the index of its source tuple in the `u` coordinate — words are
/// packed as `(low_u16, high_u16, 0, 0)`, so the `u` coordinate is free and a tagged word is
/// `(low_u16, high_u16, index, 0)`:
///
/// * each input tuple `j` is tagged with the constant `j`;
/// * each output tuple is tagged with a single base-field-guessed source index `s`, reused for all
///   `w` of its words.
///
/// A per-column [`Permutation`] gate then ties the tagged inputs to fresh permutation-output
/// variables, and each such variable is constrained equal to the corresponding tagged output. The
/// per-column multiset checks plus the shared-per-tuple tag together guarantee each output tuple
/// equals exactly one input tuple, i.e. `outputs` is a genuine permutation of `inputs`.
///
/// Assumes the words of `inputs` and `outputs` are valid `u32` packings (`(low, high, 0, 0)`), i.e.
/// the [`U32Wrapper`] invariant; the soundness of the tag depends on the `u`/`iu` coordinates being
/// zero.
///
/// Panics if `inputs` and `outputs` have different lengths, if the tuples have inconsistent
/// arities, or if `outputs` is not a multiset permutation of `inputs` (some output tuple has no
/// matching input tuple).
pub fn permute_tuples<Value: IValue>(
    ctx: &mut Context<Value>,
    inputs: &[Vec<U32Wrapper<Var>>],
    outputs: &[Vec<U32Wrapper<Var>>],
) {
    let n = inputs.len();
    assert_eq!(n, outputs.len(), "inputs and outputs must have the same length");
    if n == 0 {
        return;
    }

    // Every tuple (input and output) must have the same arity, the number of permutation columns.
    let arity = inputs[0].len();
    assert!(
        inputs.iter().chain(outputs).all(|t| t.len() == arity),
        "all tuples must have the same arity"
    );

    // The `arity` u32 words of a tuple, used as the lookup key for matching outputs to inputs.
    let key_from_tuple = |ctx: &Context<Value>, tuple: &[U32Wrapper<Var>]| -> Vec<u32> {
        tuple.iter().map(|w| ctx.get(*w.get()).unpack_u32()).collect()
    };

    // Tag each input word with its tuple index `j`: `(low, high, 0, 0) + (0, 0, j, 0)`, and map
    // each distinct input-tuple value (its `arity` u32 words) to the input indices that hold it.
    // Duplicate input tuples map to several indices and are consumed one per matching output.
    let mut tagged_in: Vec<Vec<Var>> = (0..arity).map(|_| Vec::with_capacity(n)).collect();
    let mut indices_by_value: HashMap<Vec<u32>, Vec<usize>> = HashMap::with_capacity(n);
    for (j, tuple) in inputs.iter().enumerate() {
        let tag = ctx.constant(qm31_from_u32s(0, 0, j as u32, 0));
        for (col, word) in zip_eq(tagged_in.iter_mut(), tuple.iter()) {
            col.push(eval!(ctx, (*word.get()) + (tag)));
        }

        indices_by_value.entry(key_from_tuple(ctx, tuple)).or_default().push(j);
    }

    // `u = (0, 0, 1, 0)`; multiplying a base-field index `s` by it lifts it to `(0, 0, s, 0)`.
    let u = ctx.u();

    // `tagged_out[c]` collects the fresh permutation-output variables of word-column `c`.
    let mut tagged_out: Vec<Vec<Var>> = (0..arity).map(|_| Vec::with_capacity(n)).collect();

    for out_tuple in outputs {
        // Find and guess the source index: an as-yet-unused input whose tuple equals this output,
        // constrained to the base field and lifted into the `u` coordinate. One guess per output
        // tuple, shared across all of its words.
        let index_in_inputs = indices_by_value
            .get_mut(&key_from_tuple(ctx, out_tuple))
            .and_then(|idxs| idxs.pop())
            .expect("output tuple is not a permutation of the inputs");
        let index_var = M31Wrapper::new_unsafe(Value::from_qm31(qm31_from_u32s(
            index_in_inputs as u32,
            0,
            0,
            0,
        )))
        .guess(ctx);
        let tag_u = eval!(ctx, (*index_var.get()) * (u));

        // Tag each word of this tuple and add it to its word-column as one permutation output.
        for (col, out_word) in zip_eq(tagged_out.iter_mut(), out_tuple.iter()) {
            // Tagged output word: `(low, high, 0, 0) + (0, 0, index, 0)`.
            let tagged_word = eval!(ctx, (*out_word.get()) + (tag_u));
            // The per-column Permutation gate yields its own fresh variable, so allocate one and
            // pin it to `tagged_word` with the `eq`. The gate (added per column below) then proves
            // these tagged outputs are a multiset-permutation of the column's tagged inputs.
            let perm_out = ctx.new_var(ctx.get(tagged_word));
            eq(ctx, perm_out, tagged_word);
            col.push(perm_out);
        }
    }

    // One Permutation gate per word-column: its tagged inputs and permutation outputs must agree as
    // multisets.
    for (in_col, out_col) in zip_eq(tagged_in, tagged_out) {
        ctx.stats.permutation_inputs += in_col.len();
        ctx.circuit.permutation.push(Permutation {
            inputs: in_col.iter().map(|v| v.idx).collect(),
            outputs: out_col.iter().map(|v| v.idx).collect(),
        });
    }
}

/// Constrains `outputs` to be a multiset permutation of `inputs`, treating each [`HashValue`] as an
/// atomic tuple. Thin wrapper over [`permute_tuples`] at arity
/// [`BLAKE2S_DIGEST_N_WORDS`](circuits::blake::BLAKE2S_DIGEST_N_WORDS).
pub fn permute_hash_values<Value: IValue>(
    ctx: &mut Context<Value>,
    inputs: &[HashValue<Var>],
    outputs: &[HashValue<Var>],
) {
    let tuples = |hashes: &[HashValue<Var>]| -> Vec<Vec<U32Wrapper<Var>>> {
        hashes.iter().map(|h| h.to_vec()).collect()
    };
    permute_tuples(ctx, &tuples(inputs), &tuples(outputs));
}
