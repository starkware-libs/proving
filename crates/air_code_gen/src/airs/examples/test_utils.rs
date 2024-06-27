use std::iter::zip;

use itertools::Itertools;
use rand::Rng;
use stwo_prover::core::air::accumulation::{
    DomainEvaluationAccumulator, PointEvaluationAccumulator,
};
use stwo_prover::core::air::{AirProver, AirTraceWriter, ComponentProver, ComponentTrace};
use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
use stwo_prover::core::backend::{Backend, CpuBackend};
use stwo_prover::core::channel::{Blake2sChannel, Channel};
use stwo_prover::core::circle::{CirclePoint, SECURE_FIELD_CIRCLE_ORDER};
use stwo_prover::core::fields::m31::{BaseField, P};
use stwo_prover::core::fields::qm31::SecureField;
use stwo_prover::core::pcs::TreeVec;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::prover::{prove, verify};
use stwo_prover::core::vcs::blake2_hash::Blake2sHash;
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use stwo_prover::core::vcs::ops::MerkleOps;
use stwo_prover::core::InteractionElements;

/// Asserts that the component constraints are satisfied on the trace.
/// Should only be used for testing.
pub fn assert_cpu_constraints(
    component: &dyn ComponentProver<CpuBackend>,
    trace: Vec<CpuCircleEvaluation<BaseField, BitReversedOrder>>,
) {
    let mut rng = rand::thread_rng();

    // Evaluate component trace.
    let trace_polys = trace
        .clone()
        .into_iter()
        .map(|eval| eval.interpolate())
        .collect_vec();
    let eval_domain =
        CanonicCoset::new(component.max_constraint_log_degree_bound()).circle_domain();
    let trace_evals = trace_polys
        .iter()
        .map(|poly| poly.evaluate(eval_domain))
        .collect_vec();

    let component_trace = ComponentTrace {
        polys: TreeVec(vec![trace_polys.iter().collect()]),
        evals: TreeVec(vec![trace_evals.iter().collect()]),
    };

    // Accumulate constraints to get the constraint polynomial.
    let random_coeff = SecureField::from_u32_unchecked(
        rng.gen_range(0..P),
        rng.gen_range(0..P),
        rng.gen_range(0..P),
        rng.gen_range(0..P),
    );
    let mut composition_polynomial_acc = DomainEvaluationAccumulator::<CpuBackend>::new(
        random_coeff,
        component.max_constraint_log_degree_bound(),
        component.n_constraints(),
    );
    component.evaluate_constraint_quotients_on_domain(
        &component_trace,
        &mut composition_polynomial_acc,
        &InteractionElements::default(),
    );
    let composition_polynomial = composition_polynomial_acc.finalize();

    // Evaluate constraints at a random point.
    let oods_point = CirclePoint::get_point(rng.gen_range(0..SECURE_FIELD_CIRCLE_ORDER));
    let oods_mask_points = component.mask_points(oods_point);
    let oods_mask_values = zip(&oods_mask_points[0], &component_trace.polys[0])
        .map(|(col_points, col)| {
            col_points
                .iter()
                .map(|point| col.eval_at_point(*point))
                .collect()
        })
        .collect();
    let mut oods_mask_accumulator = PointEvaluationAccumulator::new(random_coeff);
    component.evaluate_constraint_quotients_at_point(
        oods_point,
        &oods_mask_values,
        &mut oods_mask_accumulator,
        &InteractionElements::default(),
    );

    assert_eq!(
        oods_mask_accumulator.finalize(),
        composition_polynomial.eval_at_point(oods_point)
    );
}

pub fn test_prove<B: Backend + MerkleOps<Blake2sMerkleHasher>>(
    air: &(impl AirProver<B> + AirTraceWriter<B>),
    trace: Vec<CircleEvaluation<B, BaseField, BitReversedOrder>>,
) {
    // TODO(ShaharS): Mix channel `initial_seed` with the private input.
    let initial_seed = Blake2sHash::default();
    let prover_channel = &mut Blake2sChannel::new(initial_seed);
    let verifier_channel = &mut Blake2sChannel::new(initial_seed);

    let proof = prove(air, prover_channel, trace).unwrap();

    verify(proof, air, verifier_channel).unwrap();
}
