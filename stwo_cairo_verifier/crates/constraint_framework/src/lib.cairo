use core::box::BoxImpl;
use core::dict::{Felt252Dict, Felt252DictEntryTrait, Felt252DictTrait};
use stwo_verifier_core::ColumnSpan;
use stwo_verifier_core::channel::{Channel, ChannelTrait};
use stwo_verifier_core::fields::qm31::QM31;
use stwo_verifier_core::utils::{ArrayImpl, pow2};

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

#[derive(Drop)]
pub struct PreprocessedMaskValues {
    /// The value of each preprocessed column at the out of domain point, indexed by
    /// `PreprocessedColumnIdx`.
    pub values: Span<QM31>,
}

#[generate_trait]
pub impl PreprocessedMaskValuesImpl of PreprocessedMaskValuesTrait {
    /// Reads one mask value per preprocessed column.
    ///
    /// The number of preprocessed columns is hardcoded by the verifier and each preprocessed column
    /// contributes exactly one sample.
    fn new(
        preprocessed_mask_values: ColumnSpan<Span<QM31>>, n_preprocessed_columns: usize,
    ) -> PreprocessedMaskValues {
        assert!(preprocessed_mask_values.len() == n_preprocessed_columns);
        let mut values = array![];
        for column_mask_values in preprocessed_mask_values {
            let [mask_value] = (*(*column_mask_values)
                .try_into()
                .expect('preprocessed column mask != 1'))
                .unbox();
            values.append(mask_value);
        }

        PreprocessedMaskValues { values: values.span() }
    }

    fn get(self: @PreprocessedMaskValues, idx: PreprocessedColumnIdx) -> QM31 {
        *self.values[idx]
    }
}

#[derive(Debug, Default, Drop)]
enum PreprocessedColumnsAllocationMode {
    #[default]
    Dynamic,
    Static,
}

pub type PreprocessedColumnIdx = u32;

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

