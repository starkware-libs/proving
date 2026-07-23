use std::iter::repeat_n;

use circuits::blake::BLAKE2S_DIGEST_N_WORDS;
use circuits::context::{U_VALUE, U_VAR_IDX};
use circuits::ivalue::NoValue;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use num_traits::Zero;
use stwo::core::channel::Channel;
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo_constraint_framework::Relation;

use crate::circuit_components::{N_COMPONENTS, sorted_component_order};
use crate::relations::{CommonLookupElements, GATE_RELATION_ID};
use crate::statement::all_circuit_components;

pub type ComponentLogSize = u32;
pub type ClaimedSum = QM31;

#[derive(Debug, PartialEq)]
pub struct CircuitClaim {
    pub output_values: Vec<QM31>,
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { output_values } = self;

        // mix the output values into the channel.
        channel.mix_felts(output_values);
    }
}

/// Mixes the circuit hash into the channel, committing the transcript to the circuit's identity.
pub fn mix_circuit_hash(channel: &mut impl Channel, circuit_hash: &Blake2sHash) {
    let circuit_hash_words: [u32; BLAKE2S_DIGEST_N_WORDS] = std::array::from_fn(|i| {
        u32::from_le_bytes(circuit_hash.0[i * 4..i * 4 + 4].try_into().unwrap())
    });

    channel.mix_u32s(&circuit_hash_words);
}

/// Returns `[trace_log_sizes, interaction_log_sizes]` for `tree[1]` and `tree[2]`,
/// in the order the prover commits columns, i.e. size-sorted component order (see
/// `sorted_component_order`). Each component contributes its `log_size` repeated by its number of
/// trace and interaction columns respectively.
pub fn column_log_sizes_per_tree(log_sizes: &OrderedHashMap<&'static str, u32>) -> [Vec<u32>; 2] {
    let components = all_circuit_components::<NoValue>();
    let order = sorted_component_order(log_sizes);
    let mut trace = Vec::new();
    let mut interaction = Vec::new();
    for &i in &order {
        let (&name, component) = components.get_index(i).unwrap();
        let log_size = log_sizes[name];
        trace.extend(repeat_n(log_size, component.trace_columns()));
        interaction.extend(repeat_n(log_size, component.interaction_columns()));
    }
    [trace, interaction]
}

pub struct CircuitInteractionElements {
    pub common_lookup_elements: CommonLookupElements,
}
impl CircuitInteractionElements {
    pub fn draw(channel: &mut impl Channel) -> CircuitInteractionElements {
        CircuitInteractionElements { common_lookup_elements: CommonLookupElements::draw(channel) }
    }
}

#[derive(Debug, PartialEq)]
pub struct CircuitInteractionClaim {
    pub claimed_sums: [ClaimedSum; N_COMPONENTS],
}
impl CircuitInteractionClaim {
    /// Mixes the claimed sums into the channel. They are already stored in size-sorted component
    /// order (see `sorted_component_order`), matching the order in which the components' columns
    /// are committed and the order in which the verifier consumes the claimed sums.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { claimed_sums } = self;
        channel.mix_felts(claimed_sums);
    }
}

pub fn lookup_sum(
    claim: &CircuitClaim,
    interaction_claim: &CircuitInteractionClaim,
    interaction_elements: &CircuitInteractionElements,
) -> QM31 {
    let CircuitInteractionClaim { claimed_sums } = interaction_claim;
    let component_sum: QM31 = claimed_sums.iter().sum();

    // Compute the public logup sum from output gates.
    let mut output_sum = QM31::zero();
    let gate_relation_id = M31::from(GATE_RELATION_ID);
    for (i, value) in claim.output_values.iter().enumerate() {
        let values = [
            gate_relation_id,
            M31::from((U_VAR_IDX + 1 + i) as u32),
            value.0.0,
            value.0.1,
            value.1.0,
            value.1.1,
        ];
        let denom: QM31 = interaction_elements.common_lookup_elements.combine(&values);
        output_sum += denom.inverse();
    }
    let u_lookup_term = [
        gate_relation_id,
        M31::from(U_VAR_IDX),
        U_VALUE.0.0,
        U_VALUE.0.1,
        U_VALUE.1.0,
        U_VALUE.1.1,
    ];
    let denom: QM31 = interaction_elements.common_lookup_elements.combine(&u_lookup_term);
    output_sum += denom.inverse();

    component_sum + output_sum
}
