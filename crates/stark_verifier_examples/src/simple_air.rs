use circuits_stark_verifier::proof_from_stark_proof::pack_public_claim;
use itertools::{Itertools, zip_eq};
use num_traits::One;
use stwo::core::ColumnVec;
use stwo::core::air::Component;
use stwo::core::channel::{Blake2sM31Channel, Channel};
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::proof_of_work::GrindOps;
use stwo::core::vcs_lifted::blake2_merkle::{Blake2sM31MerkleChannel, Blake2sMerkleHasher};
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::backend::simd::column::BaseColumn;
use stwo::prover::backend::simd::m31::{LOG_N_LANES, PackedBaseField};
use stwo::prover::backend::simd::qm31::PackedSecureField;
use stwo::prover::backend::{Col, Column};
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::{CircleEvaluation, PolyOps};
use stwo::prover::{CommitmentSchemeProver, prove_ex};
use stwo_constraint_framework::logup::LookupElements;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;
use stwo_constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, LogupTraceGenerator, RelationEntry,
    TraceLocationAllocator, relation,
};

pub const INTERACTION_POW_BITS: u32 = 8;

#[cfg(test)]
#[path = "simple_air_test.rs"]
pub mod simple_air_test;

pub const LOG_SIZE_SHORT: u32 = 4;
pub const LOG_SIZE_LONG: u32 = 5;
const _: () = assert!(LOG_SIZE_LONG > LOG_SIZE_SHORT);
const _: () = assert!(LOG_SIZE_SHORT >= LOG_N_LANES);

pub const FIB_SEQUENCE_LENGTH: usize = 4;

pub const FIB_PREPROCESSED_COLUMNS: [&str; 2] = ["row_const_short", "row_const_long"];

relation!(SimpleRelation, 2);

pub type SimpleComponent = FrameworkComponent<Eval>;

pub struct FibInput {
    a: PackedBaseField,
    b: PackedBaseField,
}

/// A component that enforces a variation of the Fibonacci sequence:
///    `a_{i + 2} = a_{i + 1}^2 + a_{i}^2 + row_const`.
///
/// The `row_const` is a different constant for each row.
///
/// The last two elements of the sequence are added to the relation.
/// Each row contains a separate sequence of length `N`.
#[derive(Clone)]
pub struct Eval {
    pub lookup_elements: SimpleRelation,
    pub preprocessed_column_id: PreProcessedColumnId,
    pub log_n_instances: u32,
}
impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.log_n_instances
    }
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_n_instances + 1
    }
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let row_const = eval.get_preprocessed_column(self.preprocessed_column_id.clone());
        let mut a = eval.next_trace_mask();
        let mut b = eval.next_trace_mask();
        for _ in 2..FIB_SEQUENCE_LENGTH {
            let c = eval.next_trace_mask();
            eval.add_constraint(c.clone() - (a.square() + b.square() + row_const.clone()));
            a = b;
            b = c;
        }

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            E::EF::one(),
            &[a.clone(), b.clone()],
        ));
        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            E::EF::one(),
            &[a.clone(), b.clone()],
        ));
        eval.add_to_relation(RelationEntry::new(&self.lookup_elements, E::EF::one(), &[a, b]));
        eval.finalize_logup_in_pairs();

        eval
    }
}

/// Generates a trace for the test.
fn generate_trace(
    log_n_instances: u32,
) -> ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
    let inputs = (0..(1 << (log_n_instances - LOG_N_LANES)))
        .map(|i| FibInput {
            a: PackedBaseField::one(),
            b: PackedBaseField::from_array(std::array::from_fn(|j| {
                BaseField::from_u32_unchecked((i * 16 + j) as u32)
            })),
        })
        .collect_vec();

    let mut trace = (0..FIB_SEQUENCE_LENGTH)
        .map(|_| Col::<SimdBackend, BaseField>::zeros(1 << log_n_instances))
        .collect_vec();
    let row_const: BaseColumn = (0..(1 << log_n_instances)).map(|i| i.into()).collect();
    for (vec_index, (input, row_const)) in zip_eq(inputs, row_const.data).enumerate() {
        let mut a = input.a;
        let mut b = input.b;
        trace[0].data[vec_index] = a;
        trace[1].data[vec_index] = b;
        trace.iter_mut().skip(2).for_each(|col| {
            (a, b) = (b, a.square() + b.square() + row_const);
            col.data[vec_index] = b;
        });
    }
    let domain = CanonicCoset::new(log_n_instances).circle_domain();
    trace
        .into_iter()
        .map(|eval| CircleEvaluation::<SimdBackend, _, BitReversedOrder>::new(domain, eval))
        .collect_vec()
}

/// Generates the interaction trace for the test.
pub fn generate_interaction_trace(
    trace: &ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>,
    lookup_elements: &LookupElements<2>,
) -> (ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>, SecureField) {
    let log_instances = trace[0].values.length.ilog2();
    let mut logup_gen = unsafe { LogupTraceGenerator::uninitialized(log_instances) };

    let mut col_gen = logup_gen.new_col();
    for vec_row in 0..(1 << (log_instances - LOG_N_LANES)) {
        let denom0: PackedSecureField = lookup_elements.combine(&[
            trace[FIB_SEQUENCE_LENGTH - 2].values.data[vec_row],
            trace[FIB_SEQUENCE_LENGTH - 1].values.data[vec_row],
        ]);
        let denom1: PackedSecureField = lookup_elements.combine(&[
            trace[FIB_SEQUENCE_LENGTH - 2].values.data[vec_row],
            trace[FIB_SEQUENCE_LENGTH - 1].values.data[vec_row],
        ]);
        col_gen.write_frac(vec_row, denom0 + denom1, denom0 * denom1);
    }
    col_gen.finalize_col();

    let mut col_gen = logup_gen.new_col();
    for vec_row in 0..(1 << (log_instances - LOG_N_LANES)) {
        let denom: PackedSecureField = lookup_elements.combine(&[
            trace[FIB_SEQUENCE_LENGTH - 2].values.data[vec_row],
            trace[FIB_SEQUENCE_LENGTH - 1].values.data[vec_row],
        ]);
        col_gen.write_frac(vec_row, PackedSecureField::one(), denom);
    }
    col_gen.finalize_col();

    logup_gen.finalize_last()
}

fn generate_seq_column(
    log_size: u32,
) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
    let col = Col::<SimdBackend, BaseField>::from_iter((0..(1 << log_size)).map(BaseField::from));
    CircleEvaluation::new(CanonicCoset::new(log_size).circle_domain(), col)
}

#[allow(clippy::type_complexity)]
/// Creates a proof for the simple AIR. See documentation in [Eval].
pub fn create_proof() -> (
    Vec<Box<dyn Component>>,
    Vec<QM31>,
    PcsConfig,
    ExtendedStarkProof<Blake2sMerkleHasher>,
    u64,
    u32,
) {
    create_proof_with_fold_step(1)
}

#[allow(clippy::type_complexity)]
/// Creates a proof for the simple AIR with a configurable FRI fold step.
pub fn create_proof_with_fold_step(
    fold_step: u32,
) -> (
    Vec<Box<dyn Component>>,
    Vec<QM31>,
    PcsConfig,
    ExtendedStarkProof<Blake2sMerkleHasher>,
    u64,
    u32,
) {
    let mut config = PcsConfig::default();
    config.fri_config.log_blowup_factor = 2;
    config.fri_config.fold_step = fold_step;
    let lifting_log_size = LOG_SIZE_LONG + config.fri_config.log_blowup_factor;
    config.min_lifting_log_size = lifting_log_size;
    // Precompute twiddles.
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(lifting_log_size).circle_domain().half_coset,
    );

    // Setup protocol.
    let prover_channel = &mut Blake2sM31Channel::default();

    // Mix channel salt. Note that we first reduce it modulo `M31::P`, then cast it as QM31.
    let channel_salt = 0_u32;
    prover_channel.mix_felts(&[channel_salt.into()]);
    config.mix_into(prover_channel);

    let mut commitment_scheme =
        CommitmentSchemeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(config, &twiddles);
    commitment_scheme.set_store_polynomials_coefficients();

    // Preprocessed trace
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals(vec![
        generate_seq_column(LOG_SIZE_SHORT),
        generate_seq_column(LOG_SIZE_LONG),
    ]);
    tree_builder.commit(prover_channel);

    // Mix the public claim into the channel.
    // Public claim is empty.
    let public_claim = pack_public_claim(&[]);
    prover_channel.mix_felts(&public_claim);

    let trace_1 = generate_trace(LOG_SIZE_LONG);
    let trace_2 = generate_trace(LOG_SIZE_SHORT);
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals([trace_1.clone(), trace_2.clone()].concat());
    tree_builder.commit(prover_channel);

    let interaction_pow_nonce = SimdBackend::grind(prover_channel, INTERACTION_POW_BITS);
    prover_channel.mix_u64(interaction_pow_nonce);

    // Draw lookup element.
    let lookup_elements = SimpleRelation::draw(prover_channel);

    let (interaction_trace_1, claimed_sum_1) =
        generate_interaction_trace(&trace_1, &lookup_elements.0);
    let (interaction_trace_2, claimed_sum_2) =
        generate_interaction_trace(&trace_2, &lookup_elements.0);

    let claimed_sums = vec![claimed_sum_1, claimed_sum_2];
    prover_channel.mix_felts(&claimed_sums);
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals([interaction_trace_1, interaction_trace_2].concat());
    tree_builder.commit(prover_channel);

    let short_preprocessed_column = PreProcessedColumnId { id: "row_const_short".into() };
    let long_preprocessed_column = PreProcessedColumnId { id: "row_const_long".into() };

    // Allocate the preprocessed columns in ascending size order.
    let mut trace_location_allocator = TraceLocationAllocator::new_with_preprocessed_columns(
        &FIB_PREPROCESSED_COLUMNS
            .iter()
            .map(|id| PreProcessedColumnId { id: id.to_string() })
            .collect_vec(),
    );

    // Prove constraints.
    let component_1 = SimpleComponent::new(
        &mut trace_location_allocator,
        Eval {
            lookup_elements: lookup_elements.clone(),
            preprocessed_column_id: long_preprocessed_column.clone(),
            log_n_instances: LOG_SIZE_LONG,
        },
        claimed_sum_1,
    );
    let component_2 = SimpleComponent::new(
        &mut trace_location_allocator,
        Eval {
            lookup_elements: lookup_elements.clone(),
            preprocessed_column_id: short_preprocessed_column,
            log_n_instances: LOG_SIZE_SHORT,
        },
        claimed_sum_2,
    );

    let proof = prove_ex::<SimdBackend, Blake2sM31MerkleChannel>(
        &[&component_1, &component_2],
        prover_channel,
        commitment_scheme,
        true,
    )
    .unwrap();

    let components: Vec<Box<dyn Component>> = vec![Box::new(component_1), Box::new(component_2)];
    (components, claimed_sums, config, proof, interaction_pow_nonce, channel_salt)
}
