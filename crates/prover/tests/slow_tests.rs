//! Heavy end-to-end proving and constraint-checking tests.
//! Run it explicitly with `--test slow_tests`.

use std::io::Write;
use std::sync::Arc;

use cairo_air::CairoProofForRustVerifier;
use cairo_air::verifier::verify_cairo;
use cairo_vm::types::layout_name::LayoutName;
use itertools::Itertools;
use stwo::core::fri::FriConfig;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo_cairo_adapter::ProverInput;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::{
    PreProcessedTrace, PreProcessedTraceVariant, testing_preprocessed_tree,
};
use stwo_cairo_dev_utils::utils::{get_compiled_cairo_program_path, get_proof_file_path};
use stwo_cairo_dev_utils::vm_utils::{ProgramType, run_and_adapt};
use stwo_cairo_prover::debug_tools::assert_constraints::assert_cairo_constraints;
use stwo_cairo_prover::prover::{ChannelHash, LiftingSizePolicy, ProverParameters, prove_cairo};
use stwo_cairo_prover::witness::preprocessed_trace::generate_preprocessed_commitment_root;
use stwo_cairo_serialize::CairoSerialize;
use tempfile::NamedTempFile;
use test_log::test;

#[test]
fn test_all_cairo_constraints() {
    let compiled_program =
        get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
    let input =
        run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
            .unwrap();
    let pp_tree = Arc::new(testing_preprocessed_tree(24));
    assert_cairo_constraints(input, pp_tree);
}

#[test]
fn test_canonical_preprocessed_root_regression() {
    let log_blowup_factor = 1;
    let expected = Blake2sHash::from(
        hex::decode("a98e22423bf5d235981f0b36d939ae56ef3be2751c58b032b2831e6e24ba0364")
            .expect("Invalid hex string"),
    );

    let root = generate_preprocessed_commitment_root::<Blake2sMerkleChannel>(
        log_blowup_factor,
        PreProcessedTraceVariant::Canonical,
        0,
    );

    assert_eq!(root, expected);
}

#[test]
fn test_small_canonical_preprocessed_root_regression() {
    let log_blowup_factor = 1;
    let expected = Blake2sHash::from(
        hex::decode("068d1166c9f9f0ec247641ca391ee8396170e69343dfcacc632f9638670d2bec")
            .expect("Invalid hex string"),
    );

    let root = generate_preprocessed_commitment_root::<Blake2sMerkleChannel>(
        log_blowup_factor,
        PreProcessedTraceVariant::CanonicalSmall,
        0,
    );

    assert_eq!(root, expected);
}

fn test_proof_stability(path: &str, n_proofs_to_compare: usize) {
    let compiled_program = get_compiled_cairo_program_path(path);
    let input =
        run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
            .unwrap();
    let prover_params = ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        fri_config: FriConfig::default(),
        preprocessed_trace: PreProcessedTraceVariant::Canonical,
        channel_salt: 0,
        store_polynomials_coefficients: false,
        include_all_preprocessed_columns: false,
        opt_n_id_to_big_components: None,
        lifting_size_policy: LiftingSizePolicy::Auto,
    };
    let proofs = (0..n_proofs_to_compare)
        .map(|_| {
            let proof: CairoProofForRustVerifier<_> =
                prove_cairo::<Blake2sMerkleChannel>(input.clone(), prover_params).unwrap().into();
            sonic_rs::to_string(&proof).unwrap()
        })
        .collect_vec();

    assert!(proofs.iter().all_equal());
}

#[test]
fn test_opcodes_proof_stability() {
    test_proof_stability("test_prove_verify_all_opcode_components", 2);
}

#[test]
fn test_builtins_proof_stability() {
    test_proof_stability("test_prove_verify_all_builtins", 2);
}

/// These tests' inputs were generated using cairo-vm with 50 instances of each builtin.
pub mod builtin_tests {
    use stwo::core::pcs::utils::prepare_preprocessed_query_positions;
    use stwo_constraint_framework::ORIGINAL_TRACE_IDX;
    use test_log::test;

    use super::*;

    /// Exercises `LiftingSizePolicy::AtLeastPreprocessed`: every tree, the preprocessed one
    /// included, is lifted to the maximal committed column log size — taken over the
    /// preprocessed trace's columns too.
    #[test]
    fn test_prove_verify_raise_min_lifting_to_max_column() {
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
            lifting_size_policy: LiftingSizePolicy::AtLeastPreprocessed,
        };
        let cairo_proof = prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap();

        let config = cairo_proof.extended_stark_proof.proof.config;
        let max_log_trace_size =
            cairo_proof.claim.log_sizes().iter().flatten().fold(
                PreProcessedTraceVariant::CanonicalSmall.max_log_trace_size(),
                |max, &size| max.max(size),
            );
        let max_column_log_size =
            max_log_trace_size + std::cmp::max(1, config.fri_config.log_blowup_factor);
        assert_eq!(config.trace_lifting_log_size, max_column_log_size);
        assert_eq!(config.preprocessed_lifting_log_size, max_column_log_size);

        verify_cairo::<Blake2sMerkleChannel>(cairo_proof.into()).unwrap();
    }

    /// Exercises the path where the lifting log size exceeds `pp_log_size`, producing
    /// *unsorted* preprocessed query positions that the Merkle verifier must
    /// sort.
    ///
    /// `channel_salt = 12` with `n_queries = 1000` is a seed where the folded positions
    /// come out unsorted.
    #[test]
    fn test_prove_verify_large_trace_canonical_small() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_large_trace_canonical_small");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::stwo_no_ecop, None)
                .unwrap();
        let prover_params = ProverParameters {
            channel_hash: ChannelHash::Blake2s,
            fri_config: FriConfig::new(10, 0, 1, 1000, 1),
            preprocessed_trace: PreProcessedTraceVariant::CanonicalSmall,
            channel_salt: 12,
            store_polynomials_coefficients: false,
            include_all_preprocessed_columns: false,
            opt_n_id_to_big_components: None,
            lifting_size_policy: LiftingSizePolicy::Auto,
        };
        let cairo_proof = prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap();
        // Check that this seed produces unsorted preprocessed query positions.
        let unsorted_query_positions =
            cairo_proof.extended_stark_proof.aux.unsorted_query_locations.clone();
        let log_sizes = cairo_proof.claim.log_sizes();
        let max_trace_log_size = log_sizes[ORIGINAL_TRACE_IDX].iter().max().unwrap();
        let max_pp_log_size = PreProcessedTraceVariant::CanonicalSmall.max_log_trace_size();
        let log_blowup_factor =
            cairo_proof.extended_stark_proof.proof.config.fri_config.log_blowup_factor;
        assert!(
            !prepare_preprocessed_query_positions(
                &unsorted_query_positions.into_iter().sorted().collect_vec(),
                *max_trace_log_size + log_blowup_factor,
                max_pp_log_size + log_blowup_factor,
            )
            .is_sorted()
        );

        verify_cairo::<Blake2sMerkleChannel>(cairo_proof.into()).unwrap();
    }

    #[test]
    fn test_bitwise_builtin_constraints() {
        let compiled_program = get_compiled_cairo_program_path("test_prove_verify_bitwise_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(testing_preprocessed_tree(20)));
    }

    #[test]
    fn test_mul_mod_builtin_constraints() {
        let compiled_program = get_compiled_cairo_program_path("test_prove_verify_mul_mod_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(testing_preprocessed_tree(20)));
    }

    #[test]
    fn test_pedersen_builtin_constraints() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_pedersen_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(PreProcessedTrace::canonical()));
    }

    #[test]
    fn test_poseidon_builtin_constraints() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_poseidon_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(testing_preprocessed_tree(20)));
    }

    #[test]
    fn test_range_check_bits_96_builtin_constraints() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_range_check_bits_96_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(testing_preprocessed_tree(20)));
    }

    #[test]
    fn test_range_check_bits_128_builtin_constraints() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_range_check_bits_128_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        assert_cairo_constraints(input, Arc::new(testing_preprocessed_tree(20)));
    }

    #[test]
    fn test_poseidon_aggregator() {
        let prover_params = ProverParameters {
            channel_hash: ChannelHash::Blake2s,
            fri_config: FriConfig::default(),
            preprocessed_trace: PreProcessedTraceVariant::Canonical,
            channel_salt: 0,
            store_polynomials_coefficients: false,
            include_all_preprocessed_columns: false,
            opt_n_id_to_big_components: None,
            lifting_size_policy: LiftingSizePolicy::Auto,
        };

        // Run poseidon builtin with 15 different instances.
        let compiled_program_a =
            get_compiled_cairo_program_path("test_prove_verify_poseidon_builtin");
        let input_a =
            run_and_adapt(&compiled_program_a, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        let proof_a = prove_cairo::<Blake2sMerkleChannel>(input_a, prover_params).unwrap();
        let poseidon_builtin_size_a = 2u32.pow(
            proof_a
                .claim
                .poseidon_builtin
                .expect("Poseidon builtin is not present in the claim")
                .log_size,
        );
        assert!(
            poseidon_builtin_size_a == 16,
            "Expected program to contain 15 poseidon instances, which then padded to the next \
             power of two"
        );

        let poseidon_aggregator_log_size_a = proof_a
            .claim
            .poseidon_aggregator
            .expect("Poseidon context is not present in the claim")
            .log_size;

        // Run poseidon builtin with 15 different instances, each one 30 times.
        let compiled_program_b = get_compiled_cairo_program_path("test_poseidon_aggregator");
        let input_b =
            run_and_adapt(&compiled_program_b, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        let proof_b = prove_cairo::<Blake2sMerkleChannel>(input_b, prover_params).unwrap();
        let poseidon_builtin_size_b = 2u32.pow(
            proof_b
                .claim
                .poseidon_builtin
                .expect("Poseidon builtin is not present in the claim")
                .log_size,
        );
        assert!(
            poseidon_builtin_size_b == 512,
            "Expected program to contain 15*30 poseidon instances, which then padded to the next \
             power of two"
        );

        let poseidon_aggregator_log_size_b = proof_b
            .claim
            .poseidon_aggregator
            .expect("Poseidon context is not present in the claim")
            .log_size;

        assert_eq!(
            poseidon_aggregator_log_size_a, poseidon_aggregator_log_size_b,
            "Poseidon aggregator log size should be the same for both proof because it uses \
             multiplicity"
        );
    }

    #[test]
    fn test_pedersen_aggregator() {
        let prover_params = ProverParameters {
            channel_hash: ChannelHash::Blake2s,
            fri_config: FriConfig::default(),
            preprocessed_trace: PreProcessedTraceVariant::Canonical,
            channel_salt: 0,
            store_polynomials_coefficients: false,
            include_all_preprocessed_columns: false,
            opt_n_id_to_big_components: None,
            lifting_size_policy: LiftingSizePolicy::Auto,
        };

        // Run pedersen builtin with 15 different instances.
        let compiled_program_a =
            get_compiled_cairo_program_path("test_prove_verify_pedersen_builtin");
        let input_a =
            run_and_adapt(&compiled_program_a, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        let proof_a = prove_cairo::<Blake2sMerkleChannel>(input_a, prover_params).unwrap();
        let pedersen_builtin_size_a = 2u32.pow(
            proof_a
                .claim
                .pedersen_builtin
                .expect("Pedersen builtin is not present in the claim")
                .log_size,
        );
        assert!(
            pedersen_builtin_size_a == 16,
            "Expected program to contain 15 pedersen instances, which then padded to the next \
             power of two"
        );

        let pedersen_aggregator_log_size_a = proof_a
            .claim
            .pedersen_aggregator_window_bits_18
            .expect("Pedersen context is not present in the claim")
            .log_size;

        // Run pedersen builtin with 15 different instances, each one 30 times.
        let compiled_program_b = get_compiled_cairo_program_path("test_pedersen_aggregator");
        let input_b =
            run_and_adapt(&compiled_program_b, ProgramType::Json, LayoutName::all_cairo_stwo, None)
                .unwrap();
        let proof_b = prove_cairo::<Blake2sMerkleChannel>(input_b, prover_params).unwrap();
        let pedersen_builtin_size_b = 2u32.pow(
            proof_b
                .claim
                .pedersen_builtin
                .expect("Pedersen builtin is not present in the claim")
                .log_size,
        );
        assert!(
            pedersen_builtin_size_b == 512,
            "Expected program to contain 15*30 pedersen instances, which then padded to the next \
             power of two"
        );

        let pedersen_aggregator_log_size_b = proof_b
            .claim
            .pedersen_aggregator_window_bits_18
            .expect("Pedersen context is not present in the claim")
            .log_size;

        assert_eq!(
            pedersen_aggregator_log_size_a, pedersen_aggregator_log_size_b,
            "Pedersen aggregator log size should be the same for both proof because it uses \
             multiplicity"
        );
    }
}

/// Asserts that all supported builtins are present in the input.
/// Panics if any of the builtins is missing.
fn assert_all_builtins_in_input(input: &ProverInput) {
    let empty_builtins: Vec<_> = input
        .builtin_segments
        .get_counts()
        .into_iter()
        .filter(|(_, count)| *count == 0)
        .map(|(name, _)| name)
        .collect();

    if !empty_builtins.is_empty() {
        panic!("Builtins missing in the input: {empty_builtins:?}");
    }
}

#[test_log::test]
fn test_prove_verify_all_opcode_components() {
    let compiled_program =
        get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
    let input =
        run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
            .unwrap();
    for (opcode, n_instances) in &input.state_transitions.casm_states_by_opcode.counts() {
        assert!(*n_instances > 0, "{opcode} isn't used in E2E full-Cairo opcode test");
    }
    let prover_params = ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        fri_config: FriConfig::default(),
        preprocessed_trace: PreProcessedTraceVariant::CanonicalWithoutPedersen,
        channel_salt: 0,
        store_polynomials_coefficients: true,
        include_all_preprocessed_columns: false,
        opt_n_id_to_big_components: None,
        lifting_size_policy: LiftingSizePolicy::Auto,
    };
    let cairo_proof = prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap();
    verify_cairo::<Blake2sMerkleChannel>(cairo_proof.into()).unwrap();
}

#[test]
fn test_prove_all_opcode_components_proof_regression() {
    let compiled_program =
        get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
    let input =
        run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
            .unwrap();
    let prover_params = ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        fri_config: FriConfig::new(26, 0, 1, 70, 3),
        preprocessed_trace: PreProcessedTraceVariant::Canonical,
        channel_salt: 0,
        store_polynomials_coefficients: false,
        include_all_preprocessed_columns: false,
        opt_n_id_to_big_components: None,
        lifting_size_policy: LiftingSizePolicy::Auto,
    };
    let cairo_proof = prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap();
    let mut proof_file = NamedTempFile::new().unwrap();
    let mut serialized: Vec<starknet_ff::FieldElement> = Vec::new();
    CairoSerialize::serialize(&cairo_proof, &mut serialized);
    let proof_hex: Vec<String> = serialized.into_iter().map(|felt| format!("0x{felt:x}")).collect();
    proof_file.write_all(sonic_rs::to_string_pretty(&proof_hex).unwrap().as_bytes()).unwrap();

    let expected_proof_file = get_proof_file_path("test_prove_verify_all_opcode_components");
    if std::env::var("FIX_PROOF").is_ok() {
        std::fs::copy(proof_file.path(), &expected_proof_file)
            .expect("Failed to overwrite expected proof file");
    }

    // Compare the contents of proof_file and expected_proof_file
    let proof_file_contents =
        std::fs::read_to_string(proof_file.path()).expect("Failed to read generated proof file");
    let expected_proof_contents =
        std::fs::read_to_string(&expected_proof_file).expect("Failed to read expected proof file");
    assert!(
        proof_file_contents == expected_proof_contents,
        "Generated proof file does not match the expected proof file"
    );

    verify_cairo::<Blake2sMerkleChannel>(cairo_proof.into()).unwrap();
}

#[test]
fn test_prove_verify_all_builtins_non_default_fri_config() {
    let compiled_program = get_compiled_cairo_program_path("test_prove_verify_all_builtins");
    let input =
        run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
            .unwrap();
    let prover_params = ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        fri_config: FriConfig::new(26, 0, 1, 70, 1),
        preprocessed_trace: PreProcessedTraceVariant::Canonical,
        channel_salt: 0,
        store_polynomials_coefficients: false,
        include_all_preprocessed_columns: false,
        opt_n_id_to_big_components: None,
        lifting_size_policy: LiftingSizePolicy::Auto,
    };
    let cairo_proof = prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap();
    verify_cairo::<Blake2sMerkleChannel>(cairo_proof.into()).unwrap();
}

/// Tests for the Pedersen `PartialEcMul` fast deduction.
#[generic_tests::define]
mod pedersen_fast_deduction {
    use starknet_curve::curve_params::{PEDERSEN_P0, PEDERSEN_P1, PEDERSEN_P2, SHIFT_POINT};
    use starknet_types_core::curve::ProjectivePoint;
    use starknet_types_core::felt::Felt;
    use stwo_cairo_common::prover_types::cpu::M31;
    use stwo_cairo_prover::witness::fast_deduction::pedersen::PartialEcMul;

    #[test]
    fn test_deduce_output<const NUM_WINDOWS: usize>() {
        let window_bits = 252 / NUM_WINDOWS;

        let chain = M31::from_u32_unchecked(1234);
        let round = M31::from_u32_unchecked((NUM_WINDOWS + 1) as u32);
        let mut m_shifted = [M31::from_u32_unchecked(0); NUM_WINDOWS];
        m_shifted[0] = M31::from_u32_unchecked(56);
        m_shifted[1] = M31::from_u32_unchecked(99);
        let accumulator = [PEDERSEN_P1.x().into(), PEDERSEN_P1.y().into()];

        let (new_chain, new_round, (new_m_shifted, new_accumulator)) =
            PartialEcMul::<NUM_WINDOWS>::deduce_output(chain, round, (m_shifted, accumulator));

        let mut expected_new_m_shifted = [M31::from_u32_unchecked(0); NUM_WINDOWS];
        expected_new_m_shifted[0] = M31::from_u32_unchecked(99);

        let p1 = ProjectivePoint::from_affine(PEDERSEN_P1.x(), PEDERSEN_P1.y()).unwrap();
        let p2 = ProjectivePoint::from_affine(PEDERSEN_P2.x(), PEDERSEN_P2.y()).unwrap();
        let shift_point = ProjectivePoint::from_affine(SHIFT_POINT.x(), SHIFT_POINT.y()).unwrap();
        let expected_new_accumulator =
            (p1 + &p2 * Felt::from(56 << window_bits) - shift_point).to_affine().unwrap();

        assert_eq!(new_chain, chain);
        assert_eq!(new_round, round + M31::from_u32_unchecked(1));
        assert_eq!(new_m_shifted, expected_new_m_shifted);
        assert_eq!(expected_new_accumulator.x(), new_accumulator[0].into());
        assert_eq!(expected_new_accumulator.y(), new_accumulator[1].into());
    }

    #[test]
    fn test_deduce_output_high_window<const NUM_WINDOWS: usize>() {
        let window_bits = 252 / NUM_WINDOWS;
        let bits_in_last_window = window_bits - 4;
        let chain = M31::from_u32_unchecked(1234);
        let round = M31::from_u32_unchecked((NUM_WINDOWS - 1) as u32);
        let mut m_shifted = [M31::from_u32_unchecked(0); NUM_WINDOWS];
        m_shifted[0] = M31::from_u32_unchecked(((2 << bits_in_last_window) + 5) as u32);
        m_shifted[1] = M31::from_u32_unchecked(99);
        let accumulator = [PEDERSEN_P1.x().into(), PEDERSEN_P1.y().into()];

        let (new_chain, new_round, (new_m_shifted, new_accumulator)) =
            PartialEcMul::<NUM_WINDOWS>::deduce_output(chain, round, (m_shifted, accumulator));

        let mut expected_new_m_shifted = [M31::from_u32_unchecked(0); NUM_WINDOWS];
        expected_new_m_shifted[0] = M31::from_u32_unchecked(99);

        let p0 = ProjectivePoint::from_affine(PEDERSEN_P0.x(), PEDERSEN_P0.y()).unwrap();
        let p1 = ProjectivePoint::from_affine(PEDERSEN_P1.x(), PEDERSEN_P1.y()).unwrap();
        let shift_point = ProjectivePoint::from_affine(SHIFT_POINT.x(), SHIFT_POINT.y()).unwrap();
        let shifted_p0 = &p0
            * (Felt::from(1u128 << (window_bits * (NUM_WINDOWS / 2)))
                * Felt::from(1u128 << (window_bits * (NUM_WINDOWS / 2 - 1))));
        let expected_new_accumulator =
            (&p1 * Felt::from(3) + &shifted_p0 * Felt::from(5) - shift_point).to_affine().unwrap();

        assert_eq!(new_chain, chain);
        assert_eq!(new_round, round + M31::from_u32_unchecked(1));
        assert_eq!(new_m_shifted, expected_new_m_shifted);
        assert_eq!(expected_new_accumulator.x(), new_accumulator[0].into());
        assert_eq!(expected_new_accumulator.y(), new_accumulator[1].into());
    }

    // This test is actually fast but is included here to avoid duplication of the setup.
    #[instantiate_tests(<28>)]
    mod small_window {}

    #[instantiate_tests(<14>)]
    mod large_window {}
}

#[test]
fn test_prove_verify_all_builtins() {
    let compiled_program = get_compiled_cairo_program_path("test_prove_verify_all_builtins");
    let input =
        run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::all_cairo_stwo, None)
            .unwrap();
    assert_all_builtins_in_input(&input);
    let prover_params = ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        fri_config: FriConfig::default(),
        preprocessed_trace: PreProcessedTraceVariant::Canonical,
        channel_salt: 0,
        store_polynomials_coefficients: false,
        include_all_preprocessed_columns: false,
        opt_n_id_to_big_components: None,
        lifting_size_policy: LiftingSizePolicy::Auto,
    };
    let cairo_proof = prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap();
    verify_cairo::<Blake2sMerkleChannel>(cairo_proof.into()).unwrap();
}
