use circuits::context::{Context, Var};
use circuits::eval;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{eq, permute, pointwise_mul};
use circuits::simd::Simd;
use circuits::wrappers::M31Wrapper;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::FieldExpOps;

use crate::verify::LOG_SIZE_BITS;

/// Generates the keys for sorting the query values by the column indices and log sizes.
/// The key for the i'th column is (log_size * column_idx_bound + i) * u, where
/// `column_idx_bound = 1 << column_idx_bits`.
/// Those keys guarantee a stable sort of the query values by the column indices and log sizes.
fn generate_sort_keys<Value: IValue>(
    context: &mut Context<Value>,
    column_log_sizes: &[Var],
    column_idx_bits: u32,
) -> Vec<Var> {
    let column_idx_bound: u32 = 1 << column_idx_bits;
    let u = context.u();
    let shift = context.constant(column_idx_bound.into());
    let mut sort_keys = Vec::with_capacity(column_log_sizes.len());
    for (column_idx, log_size) in column_log_sizes.iter().enumerate() {
        let shifted_log_size = eval!(context, (*log_size) * (shift));
        let tag = eval!(context, (shifted_log_size) + (context.constant(column_idx.into())));
        let sort_key = eval!(context, (u) * (tag));
        sort_keys.push(sort_key);
    }

    sort_keys
}

/// Helper struct for sorting query values.
pub struct QuerySorter {
    // Bit width of a sort key, equal to `LOG_SIZE_BITS + column_idx_bits`.
    // Unused when sorting is skipped.
    key_bits: u32,

    // The key for the i'th column is (log_size * (1 << column_idx_bits) + column_idx) * u.
    // An empty vector if sorting is skipped (or if there are no columns to sort).
    sort_keys: Vec<Var>,

    // The sort keys after sorting or an empty vector if this is the first sort.
    // This is used to verify that the sorting is the same as in the previous sort as it is cheaper
    // than using `verify_sorted_keys`.
    sorted_keys: Vec<Var>,
}
impl QuerySorter {
    pub fn new(context: &mut Context<impl IValue>, column_log_sizes: &[Var]) -> Self {
        let column_idx_bits = column_log_sizes.len().next_power_of_two().ilog2();
        let key_bits = LOG_SIZE_BITS + column_idx_bits;
        // Sort keys must fit in M31 and the diff range check must be sound.
        // See `verify_sorted_keys` for the derivation.
        assert!(key_bits <= 30);
        let sort_keys = generate_sort_keys(context, column_log_sizes, column_idx_bits);
        Self { key_bits, sort_keys, sorted_keys: vec![] }
    }

    /// Returns a sorter that skips sorting.
    pub fn skip_sorting() -> Self {
        Self { key_bits: 0, sort_keys: vec![], sorted_keys: vec![] }
    }

    /// Verifies that `sorted_keys` is sorted. The `key_bits <= 30` bound asserted in
    /// `new` ensures the range-check soundness argument below.
    fn verify_sorted_keys<Value: IValue>(&self, context: &mut Context<Value>, sorted_keys: &[Var]) {
        let Some(mut prev) = sorted_keys.first() else {
            // if there are no sorted keys, return.
            return;
        };

        let u_inverse = context.constant(qm31_from_u32s(0, 0, 1, 0).inverse());
        let diffs = sorted_keys
            .iter()
            .skip(1)
            .map(|curr| {
                let u_diff = eval!(context, (*curr) - (*prev));
                prev = curr;

                // extract diff from u * diff.
                let diff = eval!(context, (u_diff) * (u_inverse));
                M31Wrapper::new_unsafe(diff)
            })
            .collect_vec();

        let packed = Simd::pack(context, &diffs);
        // call extract_bits to range check the values.

        // key_bits is used for both the key width and the range check.
        // Keys lie in [0, 2**key_bits - 1], so diffs lie in [-(2**key_bits - 1), 2**key_bits - 1].
        //
        // Positive diffs are accepted by construction. For a negative diff to be
        // rejected, its M31 representation must be >= 2**key_bits. In M31 (modulus
        // p = 2**31 - 1), a negative value -x is represented as p - x, so the
        // most-negative diff -(2**key_bits - 1) is represented as
        // (2**31 - 1) - (2**key_bits - 1) = 2**31 - 2**key_bits. The rejection
        // condition becomes
        //
        //   2**31 - 2**key_bits >= 2**key_bits,
        //
        // i.e. key_bits <= 30.
        let _bits = extract_bits(context, &packed, self.key_bits);
    }

    /// Sorts the query values by the sort keys and returns the sorted values and keys.
    /// Assumes that the sorting of the keys is validated by the caller.
    ///
    /// The sort keys are constructed u*key, where key is an m31 element.
    pub fn sort<Value: IValue>(
        &mut self,
        context: &mut Context<Value>,
        query_values: Vec<M31Wrapper<Var>>,
    ) -> Vec<M31Wrapper<Var>> {
        if self.sort_keys.is_empty() {
            return query_values;
        }

        let tagged_values = zip_eq(query_values, &self.sort_keys)
            .map(|(value, sort_key)| eval!(context, (*value.get()) + (*sort_key)))
            .collect_vec();

        let sorted = permute(context, &tagged_values, IValue::sort_by_u_coordinate);

        let (sorted_values, sorted_keys): (Vec<_>, Vec<_>) = sorted
            .iter()
            .map(|var| {
                let value = pointwise_mul(context, *var, context.one());
                let u_key = eval!(context, (*var) - (value));
                (M31Wrapper::new_unsafe(value), u_key)
            })
            .unzip();

        if self.sorted_keys.is_empty() {
            // If this is the first sort, verify the sorted keys and store them for future
            // verification. Note that if self.sort_keys is empty, sorted_keys is also
            // empty, verify_sorted_keys will do nothing and self.sorted_keys will remain empty.
            self.verify_sorted_keys(context, &sorted_keys);
            self.sorted_keys = sorted_keys;
        } else {
            // Check that the sorting is the same as in the previous sort.
            for (key, prev_key) in zip_eq(sorted_keys, &self.sorted_keys) {
                eq(context, key, *prev_key);
            }
        };

        sorted_values
    }
}
