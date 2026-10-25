use core::array::Span;
use stwo_verifier_core::ColumnSpan;
use stwo_verifier_core::fields::qm31::QM31;
use crate::PreprocessedMaskValues;

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
<<<<<<< HEAD

/// The state every AIR component holds: its claim and its logup claimed sum. Generic over the
/// claim, which is the only per-component part. The lookup elements are the same for every
/// component, so they are passed to `evaluate_constraints_at_point` rather than copied here.
#[derive(Drop)]
pub struct Component<Claim> {
    pub claim: Claim,
    pub claimed_sum: QM31,
}
||||||| 0dbaec2ac

/// A trait for creating a new component.
pub trait NewComponent<T> {
    type Claim;

    fn new(
        claim: @Self::Claim, claimed_sum: QM31, common_lookup_elements: @CommonLookupElements,
    ) -> T;

    fn try_new(
        claim: @Option<Self::Claim>,
        ref claimed_sums: Span<QM31>,
        interaction_elements: @CommonLookupElements,
    ) -> Option<
        T,
    > {
        let Some(claim) = claim else {
            return None;
        };
        let claimed_sum = *claimed_sums.pop_front().unwrap();
        Some(Self::new(claim, claimed_sum, interaction_elements))
    }
}
=======
>>>>>>> a4b96ad32727e219b4a5a2c022f5ab419819045f
