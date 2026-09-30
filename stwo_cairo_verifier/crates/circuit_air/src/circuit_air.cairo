use core::box::BoxImpl;
use core::num::traits::Zero;
use stwo_constraint_framework::{
    AirComponent, CommonLookupElements, Component, PreprocessedMaskValuesImpl,
};
use stwo_verifier_core::fields::qm31::{QM31, QM31_EXTENSION_DEGREE};
use stwo_verifier_core::verifier::Air;
use stwo_verifier_core::{ColumnSpan, TreeSpan};
use crate::claims::CircuitInteractionClaim;
use crate::components;
use crate::components::blake_g_gate::AirComponentImpl as BlakeGGateAirComponentImpl;
use crate::components::eq::AirComponentImpl as EqAirComponentImpl;
use crate::components::m_31_to_u_32::AirComponentImpl as M31ToU32AirComponentImpl;
use crate::components::qm_31_ops::AirComponentImpl as Qm31OpsAirComponentImpl;
use crate::components::range_check_16::AirComponentImpl as RangeCheck16AirComponentImpl;
use crate::components::triple_xor::AirComponentImpl as TripleXorAirComponentImpl;
use crate::components::verify_bitwise_xor_12::AirComponentImpl as VerifyBitwiseXor12AirComponentImpl;
use crate::components::verify_bitwise_xor_4::AirComponentImpl as VerifyBitwiseXor4AirComponentImpl;
use crate::components::verify_bitwise_xor_7::AirComponentImpl as VerifyBitwiseXor7AirComponentImpl;
use crate::components::verify_bitwise_xor_8::AirComponentImpl as VerifyBitwiseXor8AirComponentImpl;
use crate::components::verify_bitwise_xor_9::AirComponentImpl as VerifyBitwiseXor9AirComponentImpl;
use crate::multiverifier_consts::COMPONENT_LOG_SIZES;
use crate::per_component::*;
use crate::preprocessed_columns::NUM_PREPROCESSED_COLUMNS;

/// Circuit components, in `crate::per_component` (committed) order.
#[derive(Drop)]
pub struct CircuitAir {
    pub eq: Component<components::eq::Claim>,
    pub qm_31_ops: Component<components::qm_31_ops::Claim>,
    pub triple_xor: Component<components::triple_xor::Claim>,
    pub m_31_to_u_32: Component<components::m_31_to_u_32::Claim>,
    pub blake_g_gate: Component<components::blake_g_gate::Claim>,
    pub verify_bitwise_xor_8: Component<components::verify_bitwise_xor_8::Claim>,
    pub verify_bitwise_xor_12: Component<components::verify_bitwise_xor_12::Claim>,
    pub verify_bitwise_xor_4: Component<components::verify_bitwise_xor_4::Claim>,
    pub verify_bitwise_xor_7: Component<components::verify_bitwise_xor_7::Claim>,
    pub verify_bitwise_xor_9: Component<components::verify_bitwise_xor_9::Claim>,
    pub range_check_16: Component<components::range_check_16::Claim>,
    /// Shared by every component's constraints, so held once here rather than per component.
    pub common_lookup_elements: CommonLookupElements,
}

#[generate_trait]
pub impl CircuitAirNewImpl of CircuitAirNewTrait {
    /// Builds the circuit components. Component log sizes are not part of the claim; they are
    /// the hardcoded `COMPONENT_LOG_SIZES`, one entry per component. The circuit is fixed-size,
    /// so every component is present.
    fn new(
        common_lookup_elements: CommonLookupElements, interaction_claim: @CircuitInteractionClaim,
    ) -> CircuitAir {
        // Each component's interaction claim is its single `claimed_sum`, and its log size is the
        // matching field of `COMPONENT_LOG_SIZES`.
        let CircuitInteractionClaim { claimed_sum } = interaction_claim;
        let PerComponent {
            eq: eq_claimed_sum,
            qm_31_ops: qm_31_ops_claimed_sum,
            triple_xor: triple_xor_claimed_sum,
            m_31_to_u_32: m_31_to_u_32_claimed_sum,
            blake_g_gate: blake_g_gate_claimed_sum,
            verify_bitwise_xor_8: verify_bitwise_xor_8_claimed_sum,
            verify_bitwise_xor_12: verify_bitwise_xor_12_claimed_sum,
            verify_bitwise_xor_4: verify_bitwise_xor_4_claimed_sum,
            verify_bitwise_xor_7: verify_bitwise_xor_7_claimed_sum,
            verify_bitwise_xor_9: verify_bitwise_xor_9_claimed_sum,
            range_check_16: range_check_16_claimed_sum,
        } = *claimed_sum;
        let PerComponent {
            eq: eq_log_size,
            qm_31_ops: qm_31_ops_log_size,
            triple_xor: triple_xor_log_size,
            m_31_to_u_32: m_31_to_u_32_log_size,
            blake_g_gate: blake_g_gate_log_size,
            verify_bitwise_xor_8: _,
            verify_bitwise_xor_12: _,
            verify_bitwise_xor_4: _,
            verify_bitwise_xor_7: _,
            verify_bitwise_xor_9: _,
            range_check_16: _,
        } = COMPONENT_LOG_SIZES;

        CircuitAir {
            eq: Component {
                claim: components::eq::Claim { log_size: eq_log_size }, claimed_sum: eq_claimed_sum,
            },
            qm_31_ops: Component {
                claim: components::qm_31_ops::Claim { log_size: qm_31_ops_log_size },
                claimed_sum: qm_31_ops_claimed_sum,
            },
            triple_xor: Component {
                claim: components::triple_xor::Claim { log_size: triple_xor_log_size },
                claimed_sum: triple_xor_claimed_sum,
            },
            m_31_to_u_32: Component {
                claim: components::m_31_to_u_32::Claim { log_size: m_31_to_u_32_log_size },
                claimed_sum: m_31_to_u_32_claimed_sum,
            },
            blake_g_gate: Component {
                claim: components::blake_g_gate::Claim { log_size: blake_g_gate_log_size },
                claimed_sum: blake_g_gate_claimed_sum,
            },
            verify_bitwise_xor_8: Component {
                claim: components::verify_bitwise_xor_8::Claim {},
                claimed_sum: verify_bitwise_xor_8_claimed_sum,
            },
            verify_bitwise_xor_12: Component {
                claim: components::verify_bitwise_xor_12::Claim {},
                claimed_sum: verify_bitwise_xor_12_claimed_sum,
            },
            verify_bitwise_xor_4: Component {
                claim: components::verify_bitwise_xor_4::Claim {},
                claimed_sum: verify_bitwise_xor_4_claimed_sum,
            },
            verify_bitwise_xor_7: Component {
                claim: components::verify_bitwise_xor_7::Claim {},
                claimed_sum: verify_bitwise_xor_7_claimed_sum,
            },
            verify_bitwise_xor_9: Component {
                claim: components::verify_bitwise_xor_9::Claim {},
                claimed_sum: verify_bitwise_xor_9_claimed_sum,
            },
            range_check_16: Component {
                claim: components::range_check_16::Claim {},
                claimed_sum: range_check_16_claimed_sum,
            },
            common_lookup_elements,
        }
    }
}

pub impl CircuitAirImpl of Air<CircuitAir> {
    fn eval_composition_polynomial_at_point(
        self: @CircuitAir, mask_values: TreeSpan<ColumnSpan<Span<QM31>>>, random_coeff: QM31,
    ) -> QM31 {
        let mut sum = Zero::zero();

        let [
            preprocessed_mask_values,
            mut trace_mask_values,
            mut interaction_trace_mask_values,
            _composition_trace_mask_values,
        ]: [ColumnSpan<Span<QM31>>; QM31_EXTENSION_DEGREE] =
            (*mask_values
            .try_into()
            .unwrap())
            .unbox();

        let mut preprocessed_mask_values = PreprocessedMaskValuesImpl::new(
            preprocessed_mask_values, NUM_PREPROCESSED_COLUMNS,
        );

        // Evaluate components in committed order — this must match the order in which the prover
        // commits trace/interaction columns, since each component consumes its columns from the
        // front of the mask spans.
        let CircuitAir {
            eq,
            qm_31_ops,
            triple_xor,
            m_31_to_u_32,
            blake_g_gate,
            verify_bitwise_xor_8,
            verify_bitwise_xor_12,
            verify_bitwise_xor_4,
            verify_bitwise_xor_7,
            verify_bitwise_xor_9,
            range_check_16,
            common_lookup_elements,
        } = self;

        eq
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        qm_31_ops
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        triple_xor
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        m_31_to_u_32
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        blake_g_gate
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        verify_bitwise_xor_8
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        verify_bitwise_xor_12
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        verify_bitwise_xor_4
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        verify_bitwise_xor_7
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        verify_bitwise_xor_9
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );
        range_check_16
            .evaluate_constraints_at_point(
                ref sum,
                ref preprocessed_mask_values,
                ref trace_mask_values,
                ref interaction_trace_mask_values,
                random_coeff,
                common_lookup_elements,
            );

        // Sanity check that the components consumed every trace and interaction-trace column.
        assert!(trace_mask_values.is_empty());
        assert!(interaction_trace_mask_values.is_empty());
        sum
    }
}
