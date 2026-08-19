use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use anyhow::Result;
use cairo_air::CairoProof;
use cairo_air::cairo_components::CairoComponents;
use cairo_air::claims::{CairoClaim, lookup_sum};
use cairo_air::relations::CommonLookupElements;
use cairo_air::utils::{ProofFormat, serialize_proof_to_file};
use cairo_air::verifier::{INTERACTION_POW_BITS, verify_cairo_ex};
use num_traits::Zero;
use serde::Serialize;
use stwo::core::channel::{Channel, MerkleChannel};
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::SecureField;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::pcs::utils::InvalidLiftingLogSizeError;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof_of_work::GrindOps;
use stwo::core::utils::MaybeOwned;
use stwo::core::vcs_lifted::blake2_merkle::{Blake2sM31MerkleChannel, Blake2sMerkleChannel};
use stwo::core::vcs_lifted::merkle_hasher::MerkleHasherLifted;
use stwo::prover::backend::BackendForChannel;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::mempool::BaseColumnPool;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::poly::twiddles::TwiddleTree;
use stwo::prover::{CommitmentSchemeProver, CommitmentTreeProver, ProvingError, prove_ex};
use stwo_cairo_adapter::ProverInput;
use stwo_cairo_common::preprocessed_columns::pedersen::{PEDERSEN_TABLE_9, PEDERSEN_TABLE_18};
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::{
    PreProcessedTrace, PreProcessedTraceVariant,
};
use stwo_cairo_common::preprocessed_columns::simd_prelude::CircleEvaluation;
// TODO(yairv): drop this re-export of the params' historical home and have users import them
// from `stwo_cairo_common::prover_params` directly.
pub use stwo_cairo_common::prover_params::{ChannelHash, LiftingSizePolicy, ProverParameters};
use stwo_cairo_serialize::CairoSerialize;
use tracing::{Level, event, span};

use crate::utils::cairo_provers;
use crate::witness::cairo::create_cairo_claim_generator;
use crate::witness::cairo_claim_generator::CairoInteractionClaimGenerator;
use crate::witness::preprocessed_trace::gen_trace;
use crate::witness::utils::witness_trace_cells;

mod json {
    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    pub use serde_json::from_str;
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub use sonic_rs::from_str;
}

fn prove_verify_serialize<MC: MerkleChannel>(
    input: ProverInput,
    verify: bool,
    proof_path: &Path,
    proof_format: ProofFormat,
    proof_params: ProverParameters,
) -> Result<()>
where
    SimdBackend: BackendForChannel<MC>,
    MC::H: MerkleHasherLifted + Serialize,
    <MC::H as MerkleHasherLifted>::Hash: CairoSerialize,
{
    let cairo_proof = prove_cairo::<MC>(input, proof_params)?;
    if verify {
        verify_cairo_ex::<MC>(
            cairo_proof.clone().into(),
            proof_params.include_all_preprocessed_columns,
        )?;
    }
    serialize_proof_to_file(&cairo_proof, proof_path, proof_format)?;
    Ok(())
}

/// Builds the preprocessed Pedersen points table up front, on the calling thread.
///
/// The table is a `LazyLock` whose initializer itself uses rayon. Forcing it before the parallel
/// witness generation avoids a deadlock: if first built from within that section, the worker pool
/// is exhausted and the initializer can't get the threads it needs to finish.
pub fn warm_pedersen_pp_trace(variant: PreProcessedTraceVariant) {
    match variant {
        PreProcessedTraceVariant::Canonical => {
            LazyLock::force(&PEDERSEN_TABLE_18);
        }
        PreProcessedTraceVariant::CanonicalSmall => {
            LazyLock::force(&PEDERSEN_TABLE_9);
        }
        PreProcessedTraceVariant::CanonicalWithoutPedersen => {}
    }
}

pub fn prove_cairo<MC: MerkleChannel>(
    input: ProverInput,
    prover_params: ProverParameters,
) -> Result<CairoProof<MC::H>, ProvingError>
where
    SimdBackend: BackendForChannel<MC>,
{
    let _span = span!(Level::INFO, "prove_cairo").entered();
    let ProverParameters {
        channel_hash: _,
        channel_salt: _,
        fri_config,
        preprocessed_trace: preprocessed_trace_variant,
        store_polynomials_coefficients,
        include_all_preprocessed_columns: _,
        opt_n_id_to_big_components,
        lifting_size_policy,
    } = prover_params;

    let span = span!(Level::INFO, "Write Preprocessed trace").entered();
    let preprocessed_trace = Arc::new(preprocessed_trace_variant.to_preprocessed_trace());
    span.exit();

    warm_pedersen_pp_trace(preprocessed_trace_variant);

    // Run Cairo.
    let cairo_claim_generator = create_cairo_claim_generator(input, preprocessed_trace.clone());
    // Base trace.
    let span = span!(Level::INFO, "Write Base trace").entered();
    let (trace_evals, claim, interaction_generator) =
        cairo_claim_generator.write_trace(opt_n_id_to_big_components);
    span.exit();

    // Calculate max trace and preprocessed trace log size.
    let cairo_air_log_degree_bound = 1;
    assert!(cairo_air_log_degree_bound <= fri_config.log_blowup_factor);
    let trace_domain_log_size =
        claim.log_sizes().iter().flatten().copied().max().unwrap() + fri_config.log_blowup_factor;
    let preprocessed_trace_domain_log_size =
        preprocessed_trace_variant.max_log_trace_size() + fri_config.log_blowup_factor;

    // The heights the trace trees and the preprocessed tree are lifted to, per
    // `LiftingSizePolicy`. The pair goes into the proof's `PcsConfig`, which is what the verifier
    // commits with.
    let (trace_lifting_log_size, preprocessed_lifting_log_size) = match lifting_size_policy {
        LiftingSizePolicy::Auto => (trace_domain_log_size, preprocessed_trace_domain_log_size),
        LiftingSizePolicy::Fixed(size) => (size, size),
        LiftingSizePolicy::AtLeastPreprocessed => {
            let size = trace_domain_log_size.max(preprocessed_trace_domain_log_size);
            (size, size)
        }
    };
    // Each tree is lifted to a height that must dominate its own columns, which
    // `MerkleVerifierLifted::new` asserts on the verifier side.
    for (lifting_log_size, min_lifting_log_size) in [
        (trace_lifting_log_size, trace_domain_log_size),
        (preprocessed_lifting_log_size, preprocessed_trace_domain_log_size),
    ] {
        if lifting_log_size < min_lifting_log_size {
            return Err(ProvingError::InvalidLiftingLogSize(InvalidLiftingLogSizeError {
                lifting_log_size,
                min_lifting_log_size,
            }));
        }
    }

    let pcs_config =
        PcsConfig { fri_config, trace_lifting_log_size, preprocessed_lifting_log_size };
    // The twiddles must cover the tallest tree, whichever of the two it is.
    let max_lifting_log_size = trace_lifting_log_size.max(preprocessed_lifting_log_size);

    let span = span!(Level::INFO, "Precompute Twiddles").entered();
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::try_new(max_lifting_log_size)?.circle_domain().half_coset,
    );
    span.exit();

    let span = span!(Level::INFO, "Compute preprocessed trace commitment").entered();
    let preprocessed_trace_polys =
        SimdBackend::interpolate_columns(gen_trace(preprocessed_trace.clone()), &twiddles);

    let base_column_pool = BaseColumnPool::new();
    let preprocessed_tree = MaybeOwned::Owned(CommitmentTreeProver::<SimdBackend, MC>::new(
        preprocessed_trace_polys,
        fri_config.log_blowup_factor,
        &twiddles,
        store_polynomials_coefficients,
        preprocessed_lifting_log_size,
        &base_column_pool,
    ));
    span.exit();

    prove_cairo_common::<MC>(
        &twiddles,
        &base_column_pool,
        preprocessed_trace,
        preprocessed_tree,
        trace_evals,
        claim,
        interaction_generator,
        prover_params,
        pcs_config,
    )
}

pub fn prove_cairo_with_precompute<'a, MC: MerkleChannel>(
    base_column_pool: &BaseColumnPool<SimdBackend>,
    twiddles: &TwiddleTree<SimdBackend>,
    preprocessed_trace: Arc<PreProcessedTrace>,
    preprocessed_tree: MaybeOwned<'a, CommitmentTreeProver<SimdBackend, MC>>,
    input: ProverInput,
    prover_params: ProverParameters,
) -> Result<CairoProof<MC::H>, ProvingError>
where
    SimdBackend: BackendForChannel<MC>,
{
    let _span = span!(Level::INFO, "prove_cairo").entered();

    // The preprocessed tree and twiddles are already computed, so the lifting size can no longer
    // be adjusted to the trace. `Fixed` lifts every tree, the preprocessed one included, to the
    // same height.
    let LiftingSizePolicy::Fixed(lifting_log_size) = prover_params.lifting_size_policy else {
        panic!("Only LiftingSizePolicy::Fixed is supported with a precomputed preprocessed tree");
    };
    let pcs_config =
        PcsConfig::from_fri_and_lifting_size(prover_params.fri_config, lifting_log_size);

    // Run Cairo.
    let cairo_claim_generator = create_cairo_claim_generator(input, preprocessed_trace.clone());
    // Base trace.
    let span = span!(Level::INFO, "Write Base trace").entered();
    let (trace_evals, claim, interaction_generator) =
        cairo_claim_generator.write_trace(prover_params.opt_n_id_to_big_components);
    span.exit();

    prove_cairo_common::<MC>(
        twiddles,
        base_column_pool,
        preprocessed_trace,
        preprocessed_tree,
        trace_evals,
        claim,
        interaction_generator,
        prover_params,
        pcs_config,
    )
}

fn prove_cairo_common<'a, MC: MerkleChannel>(
    twiddles: &TwiddleTree<SimdBackend>,
    base_column_pool: &BaseColumnPool<SimdBackend>,
    preprocessed_trace: Arc<PreProcessedTrace>,
    preprocessed_tree: MaybeOwned<'a, CommitmentTreeProver<SimdBackend, MC>>,
    trace_evals: Vec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>,
    claim: CairoClaim,
    interaction_generator: CairoInteractionClaimGenerator,
    prover_params: ProverParameters,
    pcs_config: PcsConfig,
) -> Result<CairoProof<MC::H>, ProvingError>
where
    SimdBackend: BackendForChannel<MC>,
{
    let ProverParameters {
        channel_hash: _,
        channel_salt,
        fri_config: _,
        preprocessed_trace: preprocessed_trace_variant,
        store_polynomials_coefficients,
        include_all_preprocessed_columns,
        opt_n_id_to_big_components: _,
        lifting_size_policy: _,
    } = prover_params;

    // Setup protocol.
    let channel = &mut MC::C::default();

    // Mix channel salt. Note that we first reduce it modulo `M31::P`, then cast it as QM31.
    channel.mix_felts(&[channel_salt.into()]);
    // Mix PCS config.
    pcs_config.mix_into(channel);
    let mut commitment_scheme = CommitmentSchemeProver::<SimdBackend, MC>::with_memory_pool(
        pcs_config,
        twiddles,
        base_column_pool,
    );
    if store_polynomials_coefficients {
        commitment_scheme.set_store_polynomials_coefficients();
    }

    // Add the preprocessed trace commitment that was computed earlier to the commitment scheme.
    commitment_scheme.commit_tree(preprocessed_tree, channel);

    // Base trace.
    claim.mix_into::<MC>(channel);
    let span = span!(Level::INFO, "Compute base trace commitment").entered();
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals(trace_evals);
    tree_builder.commit(channel);
    span.exit();

    // Draw interaction elements.
    let interaction_pow = SimdBackend::grind(channel, INTERACTION_POW_BITS);
    channel.mix_u64(interaction_pow);
    let interaction_elements = CommonLookupElements::draw(channel);

    // Interaction trace.
    let span = span!(Level::INFO, "Write interaction trace").entered();
    let (interaction_trace_evals, interaction_claim) =
        interaction_generator.write_interaction_trace(&interaction_elements);
    span.exit();

    tracing::info!("Witness trace cells: {:?}", witness_trace_cells(&claim, &preprocessed_trace));
    // Validate lookup argument.
    debug_assert_eq!(
        lookup_sum(&claim, &interaction_elements, &interaction_claim),
        SecureField::zero()
    );
    interaction_claim.mix_into(channel);

    let span = span!(Level::INFO, "Compute interaction trace commitment").entered();
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals(interaction_trace_evals);
    tree_builder.commit(channel);
    span.exit();

    // Component provers.
    let component_builder = CairoComponents::new(
        &claim,
        &interaction_elements,
        &interaction_claim,
        &preprocessed_trace.ids(),
    );

    // TODO(Ohad): move to a testing routine.
    #[cfg(feature = "relation-tracker")]
    {
        use crate::debug_tools::relation_tracker::track_and_summarize_cairo_relations;
        let summary = track_and_summarize_cairo_relations(
            &commitment_scheme,
            &component_builder,
            &claim.public_data,
        );
        tracing::info!("Relations summary: {:?}", summary);
    }

    let components = cairo_provers(&component_builder);

    // Prove stark.
    let span = span!(Level::INFO, "Prove STARKs").entered();
    let proof = prove_ex::<SimdBackend, _>(
        &components,
        channel,
        commitment_scheme,
        include_all_preprocessed_columns,
    )?;
    span.exit();

    event!(name: "component_info", Level::DEBUG, "Components: {}", component_builder);

    Ok(CairoProof {
        claim,
        interaction_pow,
        interaction_claim,
        extended_stark_proof: proof,
        channel_salt,
        preprocessed_trace_variant,
    })
}

/// Generates proof given the Cairo VM output and prover config/parameters.
/// Serializes the proof as JSON and write to the output path.
/// Verifies the proof in case the respective flag is set.
pub fn create_and_serialize_proof(
    input: ProverInput,
    verify: bool,
    proof_path: PathBuf,
    proof_format: ProofFormat,
    proof_params_json: Option<PathBuf>,
) -> Result<()> {
    let proof_params = if let Some(proof_params_json) = proof_params_json {
        json::from_str(&read_to_string(&proof_params_json)?)?
    } else {
        // The default prover parameters for prod use (96 bits of security).
        // The formula is `security_bits = pow_bits + log_blowup_factor * n_queries`.
        ProverParameters {
            channel_hash: ChannelHash::Blake2s,
            channel_salt: 0,
            fri_config: FriConfig {
                // Stay within 500ms on M3.
                pow_bits: 26,
                log_last_layer_degree_bound: 0,
                // Blowup factor > 1 significantly degrades proving speed.
                // Can be in range [1, 16].
                log_blowup_factor: 1,
                // The more FRI queries, the larger the proof.
                // Proving time is not affected much by increasing this value.
                n_queries: 70,
                fold_step: 1,
            },
            preprocessed_trace: PreProcessedTraceVariant::Canonical,
            store_polynomials_coefficients: false,
            include_all_preprocessed_columns: false,
            opt_n_id_to_big_components: None,
            lifting_size_policy: LiftingSizePolicy::Auto,
        }
    };

    match proof_params.channel_hash {
        ChannelHash::Blake2s => {
            prove_verify_serialize::<Blake2sMerkleChannel>(
                input,
                verify,
                &proof_path,
                proof_format,
                proof_params,
            )?;
        }
        ChannelHash::Blake2sM31 => {
            prove_verify_serialize::<Blake2sM31MerkleChannel>(
                input,
                verify,
                &proof_path,
                proof_format,
                proof_params,
            )?;
        }
        #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
        ChannelHash::Poseidon252 => {
            unimplemented!("Poseidon252 is not supported for wasm targets");
        }
        #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
        ChannelHash::Poseidon252 => {
            use stwo::core::vcs_lifted::poseidon252_merkle::Poseidon252MerkleChannel;
            prove_verify_serialize::<Poseidon252MerkleChannel>(
                input,
                verify,
                &proof_path,
                proof_format,
                proof_params,
            )?;
        }
    };

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use cairo_air::verifier::verify_cairo;
    use cairo_vm::types::layout_name::LayoutName;
    use stwo::core::fri::FriConfig;
    use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
    use stwo_cairo_adapter::ExecutionResources;
    use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTrace;
    use stwo_cairo_dev_utils::utils::get_compiled_cairo_program_path;
    use stwo_cairo_dev_utils::vm_utils::{ProgramType, run_and_adapt};

    use crate::debug_tools::assert_constraints::assert_cairo_constraints;

    /// The adapter counts the distinct aggregator inputs straight from the builtin segment, with no
    /// trace generation. All four programs hash 15 distinct inputs, a different count means the
    /// adapter and the generated builtin component disagree on what an aggregator input is.
    #[test]
    fn test_unique_aggregator_inputs_count_distinct_hash_inputs() {
        for (program, builtin) in [
            ("test_prove_verify_pedersen_builtin", "pedersen_builtin"),
            ("test_pedersen_aggregator", "pedersen_builtin"),
            ("test_prove_verify_poseidon_builtin", "poseidon_builtin"),
            ("test_poseidon_aggregator", "poseidon_builtin"),
        ] {
            let input = run_and_adapt(
                &get_compiled_cairo_program_path(program),
                ProgramType::Json,
                LayoutName::all_cairo_stwo,
                None,
            )
            .unwrap();

            let execution_resources = ExecutionResources::from_prover_input(&input);

            assert_eq!(
                execution_resources.unique_aggregator_inputs[builtin], 15,
                "{program}: unexpected number of unique {builtin} aggregator inputs, out of {} \
                 instances",
                execution_resources.builtin_instance_counter[builtin]
            );
        }
    }

    use crate::prover::{
        ChannelHash, LiftingSizePolicy, PreProcessedTraceVariant, ProverParameters, prove_cairo,
    };

    #[test]
    fn test_all_cairo_constraints_small_ppt() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        let pp_tree = Arc::new(PreProcessedTrace::canonical_small());
        assert_cairo_constraints(input, pp_tree);
    }

    // TODO(Ohad): fine-grained constraints tests.
    #[test]
    fn test_cairo_constraints() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(PreProcessedTrace::canonical_without_pedersen()));
    }

    #[test]
    fn test_add_mod_builtin_constraints() {
        let compiled_program = get_compiled_cairo_program_path("test_prove_verify_add_mod_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(PreProcessedTrace::canonical_without_pedersen()));
    }

    #[test]
    fn test_pedersen_narrow_windows_builtin_constraints() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_pedersen_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(PreProcessedTrace::canonical_small()));
    }

    #[test]
    fn test_prove_verify_pedersen_canonical_small() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_pedersen_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::stwo_no_ecop, None)
                .unwrap();
        let prover_params = ProverParameters {
            channel_hash: ChannelHash::Blake2s,
            fri_config: FriConfig::default(),
            preprocessed_trace: PreProcessedTraceVariant::CanonicalSmall,
            channel_salt: 0,
            store_polynomials_coefficients: false,
            include_all_preprocessed_columns: false,
            opt_n_id_to_big_components: None,
            lifting_size_policy: LiftingSizePolicy::Auto,
        };
        let cairo_proof = prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap();
        verify_cairo::<Blake2sMerkleChannel>(cairo_proof.into()).unwrap();
    }
}
