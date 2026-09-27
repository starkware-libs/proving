use stwo_constraint_framework::{
    CommonLookupElements, LookupElementsTrait, RelationUsesDict, accumulate_relation_uses,
};
use stwo_verifier_core::TreeArray;
use stwo_verifier_core::channel::{Channel, ChannelTrait};
use stwo_verifier_core::fields::qm31::{QM31, QM31Serde, QM31Trait};
use stwo_verifier_utils::zip_eq::zip_eq;
use crate::components;
use crate::multiverifier_consts::COMPONENT_LOG_SIZES;
use crate::per_component::{
    N_INTERACTION_COLUMNS_PER_COMPONENT, N_TRACE_COLUMNS_PER_COMPONENT, PerComponent,
    PerComponentTrait,
};
use crate::prelude::{Invertible, M31, Zero, m31};
use crate::relations::GATE_RELATION_ID;

/// Variable index of the public input `u`.
const U_VAR_IDX: u32 = 2;

#[derive(Clone, Drop, Serde)]
pub struct CircuitPublicData {
    pub output_values: Array<QM31>,
}

#[generate_trait]
pub impl CircuitPublicDataImpl of CircuitPublicDataTrait {
    fn mix_into(self: @CircuitPublicData, ref channel: Channel) {
        channel.mix_felts(self.output_values.span());
    }
}

#[derive(Drop, Serde)]
pub struct CircuitClaim {
    pub public_data: CircuitPublicData,
}

#[generate_trait]
pub impl CircuitClaimImpl of CircuitClaimTrait {
    fn mix_into(self: @CircuitClaim, ref channel: Channel) {
        self.public_data.mix_into(ref channel);
    }
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
///   - `u_sum` is the `u` term at `U_VAR_IDX` with value `U_VALUE = (0, 0, 1, 0)`.
///   - `output_sum` is the sum over all public output values;
///
/// Assumes that the circuit lays out its variables in a fixed order:`var[0] = 0`, `var[1] = 1`,
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
        .combine([GATE_RELATION_ID, m31(U_VAR_IDX), m31(0), m31(0), m31(1), m31(0)].span());
    let u_sum = u_denom.inverse();

    // output_sum = Σ (1 / combine([GATE_RELATION_ID, addr, a, b, c, d])).
    // Each public output is a circuit variable whose value is yielded into the `Gate` logup
    // relation, keyed by its variable index (addr).
    let mut output_sum: QM31 = Zero::zero();
    let mut addr: M31 = m31(U_VAR_IDX + 1);
    for value in claim.public_data.output_values.span() {
        let [a, b, c, d] = QM31Trait::to_fixed_array(*value);
        let denom = common_lookup_elements.combine([GATE_RELATION_ID, addr, a, b, c, d].span());
        output_sum = output_sum + denom.inverse();
        addr += m31(1);
    }

    component_sum + u_sum + output_sum
}

/// Builds `[preprocessed (empty placeholder), trace, interaction]` column log sizes from the
/// hardcoded per-component log sizes, repeating each component's log size by its trace/interaction
/// column count, in `ComponentList` order. The preprocessed placeholder is discarded by the caller,
/// which commits the preprocessed tree using the hardcoded `PREPROCESSED_COLUMN_LOG_SIZES`.
pub fn column_log_sizes_per_tree() -> TreeArray<Span<u32>> {
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
    array![array![].span(), trace_log_sizes.span(), interaction_log_sizes.span()]
}

/// Accumulates lookup-relation uses across components from the hardcoded `COMPONENT_LOG_SIZES`.
/// Only the variable-size components export `RELATION_USES_PER_ROW`; fixed-size components
/// (lookup tables) use no relations.
pub fn accumulate_circuit_relation_uses(ref relation_uses: RelationUsesDict) {
    accumulate_relation_uses(
        ref relation_uses, components::eq::RELATION_USES_PER_ROW.span(), COMPONENT_LOG_SIZES.eq,
    );
    accumulate_relation_uses(
        ref relation_uses,
        components::triple_xor::RELATION_USES_PER_ROW.span(),
        COMPONENT_LOG_SIZES.triple_xor,
    );
    accumulate_relation_uses(
        ref relation_uses,
        components::m_31_to_u_32::RELATION_USES_PER_ROW.span(),
        COMPONENT_LOG_SIZES.m_31_to_u_32,
    );
    accumulate_relation_uses(
        ref relation_uses,
        components::blake_g_gate::RELATION_USES_PER_ROW.span(),
        COMPONENT_LOG_SIZES.blake_g_gate,
    );
    accumulate_relation_uses(
        ref relation_uses,
        components::qm31_ops::RELATION_USES_PER_ROW.span(),
        COMPONENT_LOG_SIZES.qm31_ops,
    );
}
