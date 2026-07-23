use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::sync::Arc;

use cairo_air::CairoProof;
use cairo_air::flat_claims::FlatClaim;
use cairo_air::utils::{binary_deserialize_from_file, binary_serialize_to_file};
use cairo_air::verifier::INTERACTION_POW_BITS;
use cairo_vm::types::layout_name::LayoutName;
use circuit_common::N_RESERVED;
use circuits::blake::HashValue;
use circuits::context::{Context, FinalizedContext};
use circuits::ivalue::NoValue;
use circuits::ops::Guess;
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use circuits_stark_verifier::statement::Statement;
use circuits_stark_verifier::verify::verify;
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::{Blake2sM31MerkleChannel, Blake2sMerkleHasher};
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::{
    PreProcessedTraceVariant, SMALL_MAX_SEQUENCE_LOG_SIZE,
};
use stwo_cairo_dev_utils::utils::get_compiled_cairo_program_path;
use stwo_cairo_dev_utils::vm_utils::{ProgramType, run_and_adapt};
use stwo_cairo_prover::prover::{ChannelHash, ProverParameters, prove_cairo};

use crate::all_components::all_components;
use crate::statement::{AUX_DATA_FIXED_LEN, CairoStatement, MEMORY_VALUES_LIMBS};
use crate::utils::get_proof_file_path;
use crate::verify::{enabled_components, get_preprocessed_root, verify_cairo_with_component_set};

/// Circuit Verifies a [CairoProof].
pub fn verify_cairo(
    proof: &CairoProof<Blake2sMerkleHasher>,
) -> Result<FinalizedContext<QM31>, String> {
    let FlatClaim { component_enable_bits, component_log_sizes: _, public_data: _ } =
        proof.claim.flatten_claim();

    let components: HashSet<&str> =
        enabled_components::<QM31>(&component_enable_bits).into_keys().collect();

    verify_cairo_with_component_set(proof, components)
}

#[test]
fn test_verify() {
    let mut pcs_config = PcsConfig::default();
    pcs_config.fri_config.fold_step = 4;
    pcs_config.min_lifting_log_size =
        SMALL_MAX_SEQUENCE_LOG_SIZE + pcs_config.fri_config.log_blowup_factor;

    let mut novalue_context: Context<NoValue> = Context::new(N_RESERVED);
    let program_len = 128;
    let program: Arc<[[M31; MEMORY_VALUES_LIMBS]]> =
        std::iter::repeat_n([M31::zero(); MEMORY_VALUES_LIMBS], program_len).collect();
    // Remove the pedersen points table component since it requires long preprocessed columns, which
    // are not supported.
    let pedersen_points_index =
        all_components::<NoValue>().get_full("pedersen_points_table_window_bits_18").unwrap().0;

    let mut enabled_bits = vec![true; all_components::<NoValue>().len()];
    enabled_bits[pedersen_points_index] = false;
    let n_components = enabled_bits.iter().filter(|b| **b).count();

    let serialized_aux_data = vec![M31::zero(); AUX_DATA_FIXED_LEN + program_len + n_components];
    let statement = CairoStatement::new(
        &mut novalue_context,
        serialized_aux_data,
        HashValue::no_value(),
        program,
        enabled_bits,
        get_preprocessed_root(20 + pcs_config.fri_config.log_blowup_factor),
        PreProcessedTraceVariant::CanonicalSmall,
    );

    let config = ProofConfig::new(
        statement.get_components(),
        statement.preprocessed_trace_variant.n_columns(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );

    let empty_proof = empty_proof(&config);

    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &statement);
    let novalue_context = novalue_context.finalize(true);
    novalue_context.circuit().check_yields();

    println!("Stats: {:?}", novalue_context.stats());
}

/// Builds the verifier circuit for `all_opcode_components`.
fn all_opcode_components_context() -> FinalizedContext<QM31> {
    let proof_path = get_proof_file_path("all_opcode_components");
    let preprocessed_trace_variant = PreProcessedTraceVariant::Canonical;
    let low_blowup_factor = 1;
    let trace_log_size = 25;

    if std::env::var("FIX_PROOF").is_ok() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        let prover_params = ProverParameters {
            channel_hash: ChannelHash::Blake2sM31,
            pcs_config: PcsConfig {
                pow_bits: 26,
                // Fold step = 4.
                fri_config: FriConfig::new(0, low_blowup_factor, 70, 4),
                min_lifting_log_size: trace_log_size + low_blowup_factor,
            },
            preprocessed_trace: preprocessed_trace_variant,
            channel_salt: 0,
            store_polynomials_coefficients: true,
            include_all_preprocessed_columns: true,
            opt_n_id_to_big_components: None,
            raise_min_lifting_to_max_column: false,
        };
        let cairo_proof = prove_cairo::<Blake2sM31MerkleChannel>(input, prover_params).unwrap();

        let proof_file =
            OpenOptions::new().create(true).write(true).truncate(true).open(&proof_path).unwrap();
        binary_serialize_to_file(&cairo_proof, &proof_file).unwrap();
    }

    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();
    verify_cairo(&cairo_proof).unwrap()
}

#[test]
fn test_verify_all_opcodes() {
    let context = all_opcode_components_context();
    println!("Stats: {:?}", context.stats());
}

#[test]
fn analyze_all_opcode_components() {
    let context = all_opcode_components_context();
    crate::circuit_analysis::analyze(context.circuit(), &context.context.debug_info);
}
