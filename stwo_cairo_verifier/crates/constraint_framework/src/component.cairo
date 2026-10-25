use core::array::Span;
use stwo_verifier_core::ColumnSpan;
use stwo_verifier_core::fields::qm31::QM31;
use crate::{CommonLookupElements, PreprocessedMaskValues};

/// A component is a set of trace columns of the same sizes along with a set of constraints on them.
pub trait AirComponent<T> {
    fn evaluate_constraints_at_point(
        self: @T,
        ref sum: QM31,
        ref preprocessed_mask_values: PreprocessedMaskValues,
        ref trace_mask_values: ColumnSpan<Span<QM31>>,
        ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
        random_coeff: QM31,
        common_lookup_elements: @CommonLookupElements,
    );
}

/// The state every AIR component holds: its claim and its logup claimed sum. Generic over the
/// claim, which is the only per-component part. The lookup elements are the same for every
/// component, so they are passed to `evaluate_constraints_at_point` rather than copied here.
#[derive(Drop)]
pub struct Component<Claim> {
    pub claim: Claim,
    pub claimed_sum: QM31,
}
