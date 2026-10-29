use core::array::Span;
use stwo_verifier_core::ColumnSpan;
use stwo_verifier_core::fields::qm31::QM31;
use crate::{CommonLookupElements, PreprocessedMaskValues};

/// A component is a set of trace columns of the same sizes along with a set of constraints on them.
/// Each component implements this trait with its own constraints.
pub trait AirComponent {
    fn evaluate_constraints_at_point(
        self: @Component,
        ref sum: QM31,
        ref preprocessed_mask_values: PreprocessedMaskValues,
        ref trace_mask_values: ColumnSpan<Span<QM31>>,
        ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
        random_coeff: QM31,
        common_lookup_elements: @CommonLookupElements,
    );
}

/// The state every AIR component holds: its log size and its logup claimed sum. The lookup
/// elements are the same for every component, so they are passed to
/// `evaluate_constraints_at_point` rather than copied here.
#[derive(Drop)]
pub struct Component {
    pub log_size: u32,
    pub claimed_sum: QM31,
}
