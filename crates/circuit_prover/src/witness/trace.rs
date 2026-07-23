use std::mem::MaybeUninit;
use std::sync::Arc;

use circuit_common::Qm31OpsTraceGenerator;
use circuit_common::preprocessed::PreProcessedTrace;
use circuit_verifier::circuit_claim::{
    CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements,
};
use circuit_verifier::circuit_components::{
    COMPONENT_NAMES, ComponentList, PerComponent, sorted_component_order,
};
use circuits::context::U_VAR_IDX;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use itertools::Itertools;
use rayon::scope;
use stwo::core::channel::MerkleChannel;
use stwo::core::fields::qm31::QM31;
use stwo::prover::TreeBuilder;
use stwo::prover::backend::BackendForChannel;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::poly::twiddles::TwiddleTree;

use crate::witness::components::{
    blake_g_gate, eq, m_31_to_u_32, qm31_ops, range_check_16, triple_xor, verify_bitwise_xor_4,
    verify_bitwise_xor_7, verify_bitwise_xor_8, verify_bitwise_xor_9, verify_bitwise_xor_12,
};

pub struct TraceGenerator {
    pub qm31_ops_trace_generator: Qm31OpsTraceGenerator,
}

pub fn write_trace<MC: MerkleChannel>(
    context_values: &[QM31],
    preprocessed_trace: Arc<PreProcessedTrace>,
    n_outputs: usize,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
    trace_generator: &TraceGenerator,
    twiddles: &TwiddleTree<SimdBackend>,
) -> (CircuitClaim, OrderedHashMap<&'static str, u32>, CircuitInteractionClaimGenerator)
where
    SimdBackend: BackendForChannel<MC>,
{
    let preprocessed_trace_ref = preprocessed_trace.as_ref();

    // Parent scope: eq/qm31_ops traces run as spawns alongside everything else.
    let mut eq_result = MaybeUninit::uninit();
    let mut qm31_ops_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_8_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_12_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_4_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_7_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_9_state = MaybeUninit::uninit();
    let mut range_check_16_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_8_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_12_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_4_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_7_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_9_result = MaybeUninit::uninit();
    let mut range_check_16_result = MaybeUninit::uninit();
    let mut triple_xor_trace_data = MaybeUninit::uninit();
    let mut triple_xor_claim_icg = MaybeUninit::uninit();
    let mut triple_xor_polys_result = MaybeUninit::uninit();
    let mut blake_g_gate_trace_data = MaybeUninit::uninit();
    let mut blake_g_gate_claim_icg = MaybeUninit::uninit();
    let mut blake_g_gate_polys_result = MaybeUninit::uninit();
    let mut m_31_to_u_32_trace_data = MaybeUninit::uninit();
    let mut m_31_to_u_32_claim_icg = MaybeUninit::uninit();
    let mut m_31_to_u_32_polys_result = MaybeUninit::uninit();
    scope(|s| {
        // Eq and qm31_ops traces run for the full duration of the parent scope.
        s.spawn(|_| {
            let (trace, log_size, lookup_data) = qm31_ops::write_trace(
                context_values,
                preprocessed_trace_ref,
                &trace_generator.qm31_ops_trace_generator,
            );
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            qm31_ops_result.write((polys, log_size, lookup_data));
        });
        s.spawn(|_| {
            let (trace, log_size, lookup_data) =
                eq::write_trace(context_values, preprocessed_trace_ref);
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            eq_result.write((polys, log_size, lookup_data));
        });

        // Initialize claim generators in a subscope.
        scope(|s| {
            s.spawn(|_| {
                verify_bitwise_xor_8_state
                    .write(verify_bitwise_xor_8::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                verify_bitwise_xor_12_state
                    .write(verify_bitwise_xor_12::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                verify_bitwise_xor_4_state
                    .write(verify_bitwise_xor_4::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                verify_bitwise_xor_7_state
                    .write(verify_bitwise_xor_7::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                verify_bitwise_xor_9_state
                    .write(verify_bitwise_xor_9::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                range_check_16_state
                    .write(range_check_16::ClaimGenerator::new(preprocessed_trace.clone()));
            });
        });

        // SAFETY: Claim generators were initialized by the subscope above.
        // Move xor/range-check states out of MaybeUninit into owned locals so we can
        // borrow them for blake_gate/blake_g/triple_xor_32, then move them into spawns.
        let verify_bitwise_xor_8_state = unsafe { verify_bitwise_xor_8_state.assume_init() };
        let verify_bitwise_xor_12_state = unsafe { verify_bitwise_xor_12_state.assume_init() };
        let verify_bitwise_xor_4_state = unsafe { verify_bitwise_xor_4_state.assume_init() };
        let verify_bitwise_xor_7_state = unsafe { verify_bitwise_xor_7_state.assume_init() };
        let verify_bitwise_xor_9_state = unsafe { verify_bitwise_xor_9_state.assume_init() };
        let range_check_16_state = unsafe { range_check_16_state.assume_init() };

        // triple_xor gate, m31_to_u32, and blake_g_gate mutate
        // xor/range-check states through shared refs, so they must complete before those
        // states are consumed.
        scope(|s| {
            s.spawn(|_| {
                let (trace, claim, icg) = triple_xor::write_trace(
                    context_values,
                    preprocessed_trace_ref,
                    &verify_bitwise_xor_8_state,
                );
                triple_xor_trace_data.write(trace);
                triple_xor_claim_icg.write((claim, icg));
            });
            s.spawn(|_| {
                let (trace, claim, icg) = m_31_to_u_32::write_trace(
                    context_values,
                    preprocessed_trace_ref,
                    &range_check_16_state,
                );
                m_31_to_u_32_trace_data.write(trace);
                m_31_to_u_32_claim_icg.write((claim, icg));
            });
            s.spawn(|_| {
                let (trace, claim, icg) = blake_g_gate::write_trace(
                    context_values,
                    preprocessed_trace_ref,
                    &verify_bitwise_xor_8_state,
                    &verify_bitwise_xor_12_state,
                    &verify_bitwise_xor_4_state,
                    &verify_bitwise_xor_9_state,
                    &verify_bitwise_xor_7_state,
                );
                blake_g_gate_trace_data.write(trace);
                blake_g_gate_claim_icg.write((claim, icg));
            });
        });

        // SAFETY: The subscope above guarantees triple_xor, m_31_to_u_32, and blake_g_gate have
        // finished mutating xor/range-check states. Interpolations and xor/range-check
        // write_trace calls can now run in parallel.
        s.spawn(|_| {
            let trace = unsafe { triple_xor_trace_data.assume_init() };
            triple_xor_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
        });
        s.spawn(|_| {
            let trace = unsafe { m_31_to_u_32_trace_data.assume_init() };
            m_31_to_u_32_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
        });
        s.spawn(|_| {
            let trace = unsafe { blake_g_gate_trace_data.assume_init() };
            blake_g_gate_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_8_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            verify_bitwise_xor_8_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_12_state.write_trace();
            // verify_bitwise_xor_12 returns Vec<CircleEvaluation> directly, not ComponentTrace.
            let polys = SimdBackend::interpolate_columns(trace, twiddles);
            verify_bitwise_xor_12_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_4_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            verify_bitwise_xor_4_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_7_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            verify_bitwise_xor_7_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_9_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            verify_bitwise_xor_9_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = range_check_16_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            range_check_16_result.write((polys, interaction_claim_gen));
        });
    });

    // SAFETY: All MaybeUninit values were initialized by the scope above.
    let (
        (eq_polys, eq_log_size, eq_lookup_data),
        (qm31_ops_polys, qm31_ops_log_size, qm31_ops_lookup_data),
        triple_xor_polys,
        (triple_xor_claim, triple_xor_interaction_claim_gen),
        m_31_to_u_32_polys,
        (m_31_to_u_32_claim, m_31_to_u_32_interaction_claim_gen),
        blake_g_gate_polys,
        (blake_g_gate_claim, blake_g_gate_interaction_claim_gen),
        (verify_bitwise_xor_8_polys, verify_bitwise_xor_8_interaction_claim_gen),
        (verify_bitwise_xor_12_polys, verify_bitwise_xor_12_interaction_claim_gen),
        (verify_bitwise_xor_4_polys, verify_bitwise_xor_4_interaction_claim_gen),
        (verify_bitwise_xor_7_polys, verify_bitwise_xor_7_interaction_claim_gen),
        (verify_bitwise_xor_9_polys, verify_bitwise_xor_9_interaction_claim_gen),
        (range_check_16_polys, range_check_16_interaction_claim_gen),
    ) = unsafe {
        (
            eq_result.assume_init(),
            qm31_ops_result.assume_init(),
            triple_xor_polys_result.assume_init(),
            triple_xor_claim_icg.assume_init(),
            m_31_to_u_32_polys_result.assume_init(),
            m_31_to_u_32_claim_icg.assume_init(),
            blake_g_gate_polys_result.assume_init(),
            blake_g_gate_claim_icg.assume_init(),
            verify_bitwise_xor_8_result.assume_init(),
            verify_bitwise_xor_12_result.assume_init(),
            verify_bitwise_xor_4_result.assume_init(),
            verify_bitwise_xor_7_result.assume_init(),
            verify_bitwise_xor_9_result.assume_init(),
            range_check_16_result.assume_init(),
        )
    };

    // Per-component log sizes, in `COMPONENT_NAMES` (i.e. `ComponentList`) order.
    let log_sizes: OrderedHashMap<&'static str, u32> = COMPONENT_NAMES
        .into_iter()
        .zip_eq([
            eq_log_size,
            qm31_ops_log_size,
            triple_xor_claim.log_size,
            m_31_to_u_32_claim.log_size,
            blake_g_gate_claim.log_size,
            crate::circuit_air::components::verify_bitwise_xor_8::LOG_SIZE,
            crate::circuit_air::components::verify_bitwise_xor_12::LOG_SIZE,
            crate::circuit_air::components::verify_bitwise_xor_4::LOG_SIZE,
            crate::circuit_air::components::verify_bitwise_xor_7::LOG_SIZE,
            crate::circuit_air::components::verify_bitwise_xor_9::LOG_SIZE,
            crate::circuit_air::components::range_check_16::LOG_SIZE,
        ])
        .collect();

    // Commit the component columns in size-sorted order (see `sorted_component_order`) so the
    // committed column layout coincides with the natural component order, letting the verifier
    // skip the in-circuit query-column sort during decommitment.
    let component_trace_polys = [
        eq_polys,
        qm31_ops_polys,
        triple_xor_polys,
        m_31_to_u_32_polys,
        blake_g_gate_polys,
        verify_bitwise_xor_8_polys,
        verify_bitwise_xor_12_polys,
        verify_bitwise_xor_4_polys,
        verify_bitwise_xor_7_polys,
        verify_bitwise_xor_9_polys,
        range_check_16_polys,
    ];
    let mut component_trace_polys = component_trace_polys.map(Some);
    for &i in sorted_component_order(&log_sizes).iter() {
        tree_builder.extend_polys(component_trace_polys[i].take().unwrap());
    }

    let output_values = ((U_VAR_IDX + 1)..(U_VAR_IDX + 1 + n_outputs))
        .map(|addr| context_values[addr])
        .collect_vec();

    (
        CircuitClaim { output_values },
        log_sizes,
        CircuitInteractionClaimGenerator {
            eq_lookup_data,
            qm31_ops_lookup_data,
            triple_xor: triple_xor_interaction_claim_gen,
            m_31_to_u_32: m_31_to_u_32_interaction_claim_gen,
            blake_g_gate: blake_g_gate_interaction_claim_gen,
            verify_bitwise_xor_8: verify_bitwise_xor_8_interaction_claim_gen,
            verify_bitwise_xor_12: verify_bitwise_xor_12_interaction_claim_gen,
            verify_bitwise_xor_4: verify_bitwise_xor_4_interaction_claim_gen,
            verify_bitwise_xor_7: verify_bitwise_xor_7_interaction_claim_gen,
            verify_bitwise_xor_9: verify_bitwise_xor_9_interaction_claim_gen,
            range_check_16: range_check_16_interaction_claim_gen,
        },
    )
}

pub struct CircuitInteractionClaimGenerator {
    pub eq_lookup_data: eq::LookupData,
    pub qm31_ops_lookup_data: qm31_ops::LookupData,
    pub triple_xor: triple_xor::InteractionClaimGenerator,
    pub m_31_to_u_32: m_31_to_u_32::InteractionClaimGenerator,
    pub blake_g_gate: blake_g_gate::InteractionClaimGenerator,
    pub verify_bitwise_xor_8: verify_bitwise_xor_8::InteractionClaimGenerator,
    pub verify_bitwise_xor_12: verify_bitwise_xor_12::InteractionClaimGenerator,
    pub verify_bitwise_xor_4: verify_bitwise_xor_4::InteractionClaimGenerator,
    pub verify_bitwise_xor_7: verify_bitwise_xor_7::InteractionClaimGenerator,
    pub verify_bitwise_xor_9: verify_bitwise_xor_9::InteractionClaimGenerator,
    pub range_check_16: range_check_16::InteractionClaimGenerator,
}

pub fn write_interaction_trace<MC: MerkleChannel>(
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    circuit_interaction_claim_generator: CircuitInteractionClaimGenerator,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
    interaction_elements: &CircuitInteractionElements,
    twiddles: &TwiddleTree<SimdBackend>,
) -> CircuitInteractionClaim
where
    SimdBackend: BackendForChannel<MC>,
{
    // Extract log sizes before parallel section.
    let eq_log_size = component_log_sizes[ComponentList::Eq.name()];
    let qm31_ops_log_size = component_log_sizes[ComponentList::Qm31Ops.name()];

    // Write all interaction traces in parallel, including interpolation. The slots below start
    // default-initialized; each spawned task writes into its own disjoint field. Destructuring
    // `circuit_interaction_claim_generator` below forces every task to run, so no field is left
    // at its default.
    let mut all_polys = PerComponent::default();
    let mut claimed_sums = PerComponent::default();
    scope(|s| {
        let CircuitInteractionClaimGenerator {
            eq_lookup_data,
            qm31_ops_lookup_data,
            triple_xor,
            m_31_to_u_32,
            blake_g_gate,
            verify_bitwise_xor_8,
            verify_bitwise_xor_12,
            verify_bitwise_xor_4,
            verify_bitwise_xor_7,
            verify_bitwise_xor_9,
            range_check_16,
        } = circuit_interaction_claim_generator;

        s.spawn(|_| {
            let (trace, claimed_sum) = eq::write_interaction_trace(
                eq_log_size,
                eq_lookup_data,
                &interaction_elements.common_lookup_elements,
            );
            all_polys.eq = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.eq = claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claimed_sum) = qm31_ops::write_interaction_trace(
                qm31_ops_log_size,
                qm31_ops_lookup_data,
                &interaction_elements.common_lookup_elements,
            );
            all_polys.qm31_ops = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.qm31_ops = claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) =
                triple_xor.write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.triple_xor = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.triple_xor = claim.claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) =
                m_31_to_u_32.write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.m_31_to_u_32 = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.m_31_to_u_32 = claim.claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) =
                blake_g_gate.write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.blake_g_gate = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.blake_g_gate = claim.claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) = verify_bitwise_xor_8
                .write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.verify_bitwise_xor_8 = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.verify_bitwise_xor_8 = claim.claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) = verify_bitwise_xor_12
                .write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.verify_bitwise_xor_12 = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.verify_bitwise_xor_12 = claim.claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) = verify_bitwise_xor_4
                .write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.verify_bitwise_xor_4 = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.verify_bitwise_xor_4 = claim.claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) = verify_bitwise_xor_7
                .write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.verify_bitwise_xor_7 = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.verify_bitwise_xor_7 = claim.claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) = verify_bitwise_xor_9
                .write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.verify_bitwise_xor_9 = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.verify_bitwise_xor_9 = claim.claimed_sum;
        });
        s.spawn(|_| {
            let (trace, claim) = range_check_16
                .write_interaction_trace(&interaction_elements.common_lookup_elements);
            all_polys.range_check_16 = SimdBackend::interpolate_columns(trace, twiddles);
            claimed_sums.range_check_16 = claim.claimed_sum;
        });
    });

    // Commit the interaction columns in the same size-sorted order as the base trace columns
    // (see `sorted_component_order`), and reorder the claimed sums to match so that
    // `CircuitInteractionClaim` stores them in committed (size-sorted) order.
    let order = sorted_component_order(component_log_sizes);
    let mut all_polys = all_polys.into_array();
    for &i in &order {
        tree_builder.extend_polys(std::mem::take(&mut all_polys[i]));
    }
    let claimed_sums = claimed_sums.into_array();
    let sorted_claimed_sums = std::array::from_fn(|sorted_idx| claimed_sums[order[sorted_idx]]);

    CircuitInteractionClaim { claimed_sums: sorted_claimed_sums }
}
