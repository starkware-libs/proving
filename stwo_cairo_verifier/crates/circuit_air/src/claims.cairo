use stwo_constraint_framework::{CommonLookupElements, LookupElementsTrait};
use stwo_verifier_core::N_TREES;
use stwo_verifier_core::channel::{Channel, ChannelTrait};
use stwo_verifier_core::fields::qm31::{QM31, QM31Serde, QM31Trait};
use stwo_verifier_core::vcs::blake2s_hasher::Blake2sHash;
use stwo_verifier_utils::zip_eq::zip_eq;
use crate::multiverifier_consts::{COMPONENT_LOG_SIZES, PREPROCESSED_COLUMN_LOG_SIZES};
use crate::per_component::{
    N_INTERACTION_COLUMNS_PER_COMPONENT, N_TRACE_COLUMNS_PER_COMPONENT, PerComponent,
    PerComponentTrait,
};
use crate::prelude::{Invertible, M31, One, Zero, m31};
use crate::relations::GATE_RELATION_ID;

/// Variable index of the public input `u`.
const U_VAR_IDX: u32 = 2;

/// The circuit encodes a u32 as two u16 limbs, so this is the shift between them.
const U16_SHIFT: NonZero<u32> = 0x10000;

#[derive(Drop, Serde)]
pub struct CircuitClaim {
    /// The circuit's public output.
    pub output_digest: Blake2sHash,
}

#[generate_trait]
pub impl CircuitClaimImpl of CircuitClaimTrait {
    /// Mixes the outputs in the circuit's wire encoding, matching what the prover mixes.
    fn mix_into(self: @CircuitClaim, ref channel: Channel) {
        let mut wire_values = array![];
        for value in @self.output_digest.hash.unbox() {
            let [lo, hi] = output_limbs(*value);
            wire_values.append(QM31Trait::from_fixed_array([lo, hi, Zero::zero(), Zero::zero()]));
        }
        channel.mix_felts(wire_values.span());
    }
}

/// Splits a u32 output into the circuit's wire limbs `(low_u16, high_u16)`. The wire value is
/// `(low_u16, high_u16, 0, 0)`; the last two coordinates are always zero.
fn output_limbs(value: u32) -> [M31; 2] {
    let (hi, lo) = DivRem::div_rem(value, U16_SHIFT);
    [m31(lo), m31(hi)]
}

/// Circuit interaction claim, holding every component's `claimed_sum` in `ComponentList` order.
/// The circuit is fixed-size, so every component is always present — one field per component.
#[derive(Drop, Serde)]
pub struct CircuitInteractionClaim {
    pub claimed_sum: PerComponent<QM31>,
}

#[generate_trait]
pub impl CircuitInteractionClaimImpl of CircuitInteractionClaimTrait {
    fn mix_into(self: @CircuitInteractionClaim, ref channel: Channel) {
        // Mix every component's claimed sum in `ComponentList` order, in a single call.
        channel.mix_felts(self.claimed_sum.to_fixed_array().span());
    }
}

/// The sum over all of the circuit's logups: every component's claimed sum, plus the public logup
/// sum determined by the public statement.
///
///   `component_sum + u_sum + output_sum`
/// where:
///   - `component_sum` is the sum of all components' `claimed_sums`;
///   - `u_sum` is the `u` term at `U_VAR_IDX` with value `U_VALUE = (0, 0, 1, 0)`;
///   - `output_sum` is the sum over the digest words of `output_digest`.
///
/// Assumes that the circuit lays out its variables in a fixed order: `var[0] = 0`, `var[1] = 1`,
/// `var[2] = u`, with the public output values placed in the variable slots immediately after `u`.
/// A proof is valid only when this value equals zero.
pub fn logup_sum(
    claim: @CircuitClaim,
    common_lookup_elements: @CommonLookupElements,
    interaction_claim: @CircuitInteractionClaim,
) -> QM31 {
    // component_sum = Σ claimed_sums.
    let component_sum: QM31 = interaction_claim
        .claimed_sum
        .to_fixed_array()
        .span()
        .into_iter()
        .map(|claimed_sum| *claimed_sum)
        .sum();

    // u_sum = the `u` input is yielded at `U_VAR_IDX` with value `U_VALUE = (0, 0, 1, 0)`.
    let u_denom = common_lookup_elements
        .combine(
            [GATE_RELATION_ID, m31(U_VAR_IDX), Zero::zero(), Zero::zero(), One::one(), Zero::zero()]
                .span(),
        );
    let u_sum = u_denom.inverse();

    // output_sum = Σ (1 / combine([GATE_RELATION_ID, addr, a, b, c, d])).
    // Each public output is a circuit variable whose value is yielded into the `Gate` logup
    // relation, keyed by its variable index (addr).
    let mut output_sum: QM31 = Zero::zero();
    let mut addr: M31 = m31(U_VAR_IDX + 1);
    for value in @claim.output_digest.hash.unbox() {
        let [lo, hi] = output_limbs(*value);
        let denom = common_lookup_elements
            .combine([GATE_RELATION_ID, addr, lo, hi, Zero::zero(), Zero::zero()].span());
        output_sum = output_sum + denom.inverse();
        addr += One::one();
    }

    component_sum + u_sum + output_sum
}

/// Returns `[preprocessed_log_sizes, trace_log_sizes, interaction_log_sizes]`, one entry per
/// committed tree, where all three are constants. `tree[0]` is the hardcoded
/// `PREPROCESSED_COLUMN_LOG_SIZES`; `tree[1]` and `tree[2]` repeat each component's log size by its
/// trace/interaction column count, in `ComponentList` order.
pub fn column_log_sizes_per_tree() -> [Span<u32>; N_TREES] {
    let mut trace_log_sizes = array![];
    let mut interaction_log_sizes = array![];
    for (log_size, (n_trace_cols, n_interaction_cols)) in zip_eq(
        COMPONENT_LOG_SIZES.to_fixed_array().span(),
        zip_eq(
            N_TRACE_COLUMNS_PER_COMPONENT.to_fixed_array().span(),
            N_INTERACTION_COLUMNS_PER_COMPONENT.to_fixed_array().span(),
        ),
    ) {
        for _ in 0..*n_trace_cols {
            trace_log_sizes.append(*log_size);
        }
        for _ in 0..*n_interaction_cols {
            interaction_log_sizes.append(*log_size);
        }
    }
    return [
        PREPROCESSED_COLUMN_LOG_SIZES.span(), trace_log_sizes.span(), interaction_log_sizes.span(),
    ];
}
