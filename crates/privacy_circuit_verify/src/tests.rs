use circuit_common::N_RESERVED;
use circuits::utils::le_u32s_from_bytes;
use privacy_common::consts::{
    CAIRO_LOG_BLOWUP_FACTOR, CAIRO_PCS_CONFIG, CAIRO_TRACE_LOG_SIZE, CIRCUIT_LOG_BLOWUP_FACTOR,
    CIRCUIT_PCS_CONFIG, CIRCUIT_TRACE_LOG_SIZE, PRIVACY_CIRCUIT_PREPROCESSED_COLUMNS,
    PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT,
};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;

use crate::{
    get_cairo_preprocessed_circuit, get_cairo_verifier_config, get_proof_config,
    get_recursive_circuit_config,
};

const CONJECTURED_SECURITY_BITS: u32 = 96;

#[test]
fn check_proof_config() {
    let proof_config = get_proof_config(
        &get_recursive_circuit_config(&PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT).unwrap(),
    );
    // All circuit components should be enabled.
    assert!(proof_config.component_shapes.iter().all(|s| s.trace_columns > 0));
}

#[test]
fn check_privacy_recursion_circuit_preprocessed_root() {
    let cairo_verifier_config = get_cairo_verifier_config().unwrap();
    let preprocessed_circuit = get_cairo_preprocessed_circuit(&cairo_verifier_config);
    // The prover commits the preprocessed trace at the config's lifting size, so
    // `preprocessed_root` must lift to the same size for the roots to match.
    let log_blowup_factor = CIRCUIT_PCS_CONFIG.fri_params.log_blowup_factor;
    assert_eq!(
        preprocessed_circuit.trace_log_size() + log_blowup_factor,
        CIRCUIT_PCS_CONFIG.preprocessed_lifting_log_size
    );
    let root_hash =
        preprocessed_circuit.preprocessed_root::<Blake2sMerkleChannel>(log_blowup_factor);
    let raw_words: [u32; 8] = le_u32s_from_bytes(root_hash.0);

    assert_eq!(raw_words, PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT);
}

#[test]
fn check_circuit_verifier_configs() {
    let cairo_verifier_config = get_cairo_verifier_config().unwrap();
    let preprocessed_circuit = get_cairo_preprocessed_circuit(&cairo_verifier_config);

    // Compare fields of the circuit config that are easily computed from the preprocessed circuit
    // to the expected values
    assert_eq!(
        preprocessed_circuit.trace_log_size() + CIRCUIT_LOG_BLOWUP_FACTOR,
        CIRCUIT_PCS_CONFIG.trace_lifting_log_size
    );
    // The output gates, excluding the `u` constant wire, hold the output digest's words.
    assert_eq!(preprocessed_circuit.n_outputs, N_RESERVED);
    let actual_columns: Vec<(String, u32)> = preprocessed_circuit
        .preprocessed_trace
        .log_sizes()
        .into_iter()
        .map(|(id, log_size)| (id.id, log_size))
        .collect();
    let expected_columns: Vec<(String, u32)> = PRIVACY_CIRCUIT_PREPROCESSED_COLUMNS
        .iter()
        .map(|&(id, log_size)| (id.to_string(), log_size))
        .collect();
    assert_eq!(
        actual_columns, expected_columns,
        "Update PRIVACY_CIRCUIT_PREPROCESSED_COLUMNS in consts.rs"
    );

    // Check that the lifting log sizes are correct
    const {
        assert!(
            CAIRO_TRACE_LOG_SIZE + CAIRO_LOG_BLOWUP_FACTOR
                == CAIRO_PCS_CONFIG.trace_lifting_log_size
        )
    };
    const {
        assert!(
            CIRCUIT_TRACE_LOG_SIZE + CIRCUIT_LOG_BLOWUP_FACTOR
                == CIRCUIT_PCS_CONFIG.trace_lifting_log_size
        )
    };

    // Check that the circuit pcs config is secure enough
    assert!(
        CIRCUIT_PCS_CONFIG.fri_params.pow_bits
            + CIRCUIT_PCS_CONFIG.fri_params.n_queries as u32
                * CIRCUIT_PCS_CONFIG.fri_params.log_blowup_factor
            >= CONJECTURED_SECURITY_BITS,
        "The recursive circuit pcs config is not secure enough."
    );

    assert!(
        CAIRO_PCS_CONFIG.fri_params.pow_bits
            + CAIRO_PCS_CONFIG.fri_params.n_queries as u32
                * CAIRO_PCS_CONFIG.fri_params.log_blowup_factor
            >= CONJECTURED_SECURITY_BITS,
        "The cairo circuit pcs config is not secure enough."
    );
}

#[cfg(feature = "slow-tests")]
pub mod slow_tests {
    use std::path::PathBuf;

    use cairo_vm::vm::runners::cairo_pie::CairoPie;
    use circuit_params::assert_committed_registry_is_up_to_date;
    use privacy_common::consts::{
        MAX_RECURSIVE_PROOF_UNCOMPRESSED_BYTES, RECURSIVE_PROOF_UNCOMPRESSED_BYTES,
    };
    use privacy_prove::{prepare_recursive_prover_precomputes, privacy_recursive_prove};
    use tracing_subscriber::fmt;

    use crate::{
        PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT, get_proof_config, get_recursive_circuit_config,
    };

    #[test]
    fn check_recursive_circuit_proof_deserializes() {
        let _ = fmt().with_max_level(tracing::Level::INFO).try_init();

        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let pie_path = project_dir.join("../privacy_prove/test_data/privacy_tx_cairo_pie.zip");
        let pie = CairoPie::read_zip_file(&pie_path).unwrap();
        let precomputes = prepare_recursive_prover_precomputes().unwrap();
        let proof_output = privacy_recursive_prove(pie, precomputes).unwrap();

        let proof_config = get_proof_config(
            &get_recursive_circuit_config(&PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT).unwrap(),
        );
        let (_header, compressed_proof) = crate::split_proof_header(&proof_output.proof).unwrap();
        let proof_bytes =
            crate::decompress_proof(compressed_proof, MAX_RECURSIVE_PROOF_UNCOMPRESSED_BYTES)
                .unwrap();
        let mut serialized_proof: &[u8] = &proof_bytes;
        circuit_serialize::deserialize::deserialize_proof_with_config(
            &mut serialized_proof,
            &proof_config,
        )
        .unwrap();
        assert!(serialized_proof.is_empty());
    }

    #[test]
    fn check_max_recursive_proof_uncompressed_size() {
        let _ = fmt().with_max_level(tracing::Level::INFO).try_init();

        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let pie_path = project_dir.join("../privacy_prove/test_data/privacy_tx_cairo_pie.zip");
        let pie = CairoPie::read_zip_file(&pie_path).unwrap();
        let precomputes = prepare_recursive_prover_precomputes().unwrap();
        let proof_output = privacy_recursive_prove(pie, precomputes).unwrap();

        let (_header, compressed_proof) = crate::split_proof_header(&proof_output.proof).unwrap();
        let proof_bytes = zstd::decode_all(compressed_proof).unwrap();
        assert_eq!(
            proof_bytes.len(),
            RECURSIVE_PROOF_UNCOMPRESSED_BYTES,
            "Update RECURSIVE_PROOF_UNCOMPRESSED_BYTES in consts.rs to {}",
            proof_bytes.len()
        );
    }

    #[test]
    fn large_proofs_registry_matches_definition() {
        const LARGE_REGISTRY_PATH: &str =
            "crates/privacy_common/large_proofs_circuit_registry.json";
        const LARGE_REGISTRY_DEFINITION: &str =
            "circuit_registry_definitions/privacy_large_proofs/definition.json";

        assert_committed_registry_is_up_to_date(LARGE_REGISTRY_DEFINITION, LARGE_REGISTRY_PATH);
    }
}
