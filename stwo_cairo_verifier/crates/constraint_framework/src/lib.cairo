use core::box::BoxImpl;
use core::dict::{Felt252Dict, Felt252DictEntryTrait, Felt252DictTrait, SquashedFelt252DictTrait};
use core::nullable::{Nullable, NullableTrait};
use stwo_verifier_core::channel::{Channel, ChannelTrait};
use stwo_verifier_core::fields::qm31::QM31;
use stwo_verifier_core::utils::{ArrayImpl, pow2};
use stwo_verifier_core::{ColumnSpan, N_TREES, TreeArray};

pub mod claim;
pub mod component;
pub mod test_utils;
pub mod utils;

pub use component::{AirComponent, NewComponent};

/// Represents the value of the prefix sum column at some index.
/// Should be used to eliminate padded rows for the logup sum.
// Copied from:
pub type ClaimedPrefixSum = (QM31, usize);

// The maximal number of felts we support combining in CommonLookupElements::combine.
const MAX_RELATION_SIZE: usize = 128;

#[derive(Drop, Clone)]
pub struct CommonLookupElements {
    pub z: QM31,
    pub alpha: QM31,
}

#[generate_trait]
pub impl LookupElementsImpl of LookupElementsTrait {
    fn draw(ref channel: Channel) -> CommonLookupElements {
        let [z, alpha]: [QM31; 2] = (*channel.draw_secure_felts(2).span().try_into().unwrap())
            .unbox();

        CommonLookupElements { z, alpha }
    }

    /// Computes the sum of terms `-z + values[i] * self.alpha^i`, over i in 0..values.len(), by
    /// Horner evaluation.
    ///
    /// Generic over the value type T, so both the constraint evaluation at the OOD point (T = QM31)
    /// and the public logup terms (T = M31) go through this implementation.
    fn combine<T, +Copy<T>, +Into<T, QM31>>(
        self: @CommonLookupElements, mut values: Span<T>,
    ) -> QM31 {
        assert!(values.len() <= MAX_RELATION_SIZE);
        let alpha = *self.alpha;
        let mut sum: QM31 = (*values.pop_back().unwrap()).into();

        while let Some(value) = values.pop_back() {
            sum = sum * alpha + (*value).into();
        }

        sum - *self.z
    }
}

#[derive(PanicDestruct)]
pub struct PreprocessedMaskValues {
    /// Maps a preprocessed column index to a nullable value with the value of the column at the out
    /// of domain point and a boolean indicating if the value was used in a constraint.
    pub values: Felt252Dict<Nullable<(QM31, bool)>>,
}

#[generate_trait]
pub impl PreprocessedMaskValuesImpl of PreprocessedMaskValuesTrait {
    fn new(mut preprocessed_mask_values: ColumnSpan<Span<QM31>>) -> PreprocessedMaskValues {
        let mut values: Felt252Dict<Nullable<(QM31, bool)>> = Default::default();

        let mut idx = 0;
        for column_mask_values in preprocessed_mask_values.into_iter() {
            let mut desnapped = *column_mask_values;

            if let Some(boxed_mask_value) = desnapped.try_into() {
                let [mask_value]: [QM31; 1] = (*boxed_mask_value).unbox();
                values.insert(idx, NullableTrait::new((mask_value, false)));
            } else {
                // Preprocessed columns should have at most one mask item.
                assert!(desnapped.is_empty());
            }
            idx += 1;
        }

        PreprocessedMaskValues { values }
    }

    fn get_and_mark_used(ref self: PreprocessedMaskValues, idx: PreprocessedColumnIdx) -> QM31 {
        let (entry, nullable_value) = self.values.entry(idx.into());
        let (value, used) = nullable_value.deref();

        let used_value = if used {
            nullable_value
        } else {
            NullableTrait::new((value, true))
        };
        self.values = entry.finalize(used_value);

        value
    }


    /// Validates that all the preprocessed_mask_values that were sent in the proof were used by at
    /// least one component.
    fn validate_usage(self: PreprocessedMaskValues) {
        for (_, _, nullable_value) in self.values.squash().into_entries() {
            let (_value, used) = nullable_value.deref();
            assert!(used);
        }
    }
}

/// Validates that every `mask_value` provided in the proof (in `sampled_values`) is used by at
/// least one component.
///
/// Since `eval_composition_polynomial_at_point` is responsible for validating the *structure*
/// of `sampled_values` in the proof, it needs to ensure that all sampled preprocessed
/// mask values are actually used. Otherwise, the prover would have the freedom to
/// send a sample of a column even if it is unused, adding another term to the FRI quotients.
///
/// Additionally, there is a sanity check that the columns in the trace and interaction-trace were
/// consumed by the components.
/// This is not strictly necessary as the verifier generates the column indices on its own and only
/// access samples of columns for which it knows about.
pub fn validate_mask_usage(
    preprocessed_mask_values: PreprocessedMaskValues,
    trace_mask_values: ColumnSpan<Span<QM31>>,
    interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
) {
    preprocessed_mask_values.validate_usage();
    assert!(trace_mask_values.is_empty());
    assert!(interaction_trace_mask_values.is_empty());
}

/// Override the preprocessed trace log sizes, since they come from a global setting
/// rather than computed by concatenating preprocessed log sizes of the individual
/// components.
/// TODO(ilya): consider removing the generation of `_invalid_preprocessed_trace_log_sizes`.
pub fn override_preprocessed_trace_log_sizes(
    aggregated_log_sizes: TreeArray<Span<u32>>, preprocessed_column_log_sizes: Span<u32>,
) -> TreeArray<Span<u32>> {
    let boxed_triplet: Box<[Span<u32>; N_TREES]> = *aggregated_log_sizes.span().try_into().unwrap();
    let [_invalid_preprocessed_trace_log_sizes, trace_log_sizes, interaction_log_sizes] =
        boxed_triplet
        .unbox();

    array![preprocessed_column_log_sizes, trace_log_sizes, interaction_log_sizes]
}

#[derive(Debug, Default, Drop)]
enum PreprocessedColumnsAllocationMode {
    #[default]
    Dynamic,
    Static,
}

pub type PreprocessedColumnIdx = u32;

// Used for columns not present in the preprocessed trace
pub const INVALID_COLUMN_IDX: PreprocessedColumnIdx = 1000000000;

// A dict from relation_id, which is a string encoded as a felt252, to the number of uses of the
// corresponding relation.
pub type RelationUsesDict = Felt252Dict<u64>;

// A tuple of (relation_id, uses).
pub type RelationUse = (felt252, u32);

pub fn accumulate_relation_uses(
    ref relation_uses: RelationUsesDict, relation_uses_per_row: Span<RelationUse>, log_size: u32,
) {
    let component_size = pow2(log_size);
    for relation_use in relation_uses_per_row {
        let (relation_id, uses) = *relation_use;
        let (entry, prev_uses) = relation_uses.entry(relation_id);
        relation_uses = entry.finalize(prev_uses + uses.into() * component_size.into());
    }
}

