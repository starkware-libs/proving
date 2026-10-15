use std::error::Error;
use std::sync::Arc;

use anyhow::Result;
use cairo_air::verifier::INTERACTION_POW_BITS;
use circuit_cairo_verifier::all_components::all_components;
use circuit_cairo_verifier::verify::{
    CairoVerifierConfig, NON_QUERY_INFO_LEAK, build_cairo_verifier_circuit, get_preprocessed_root,
};
use circuit_common::preprocessed::{PreprocessedCircuit, layout_from_component_sizes};
use circuit_registry::CircuitProofConfig;
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_verifier::statement::{
    all_circuit_components, circuit_component_log_sizes, circuit_verifier_proof_config,
};
use circuit_verifier::verify::{CircuitConfig, CircuitPublicData, verify_circuit};
use circuits::blake::HashValue;
use circuits::ivalue::NoValue;
use circuits::utils::le_u32s_from_bytes;
use circuits_stark_verifier::proof::ProofConfig;
use privacy_common::consts::{
    CAIRO_PCS_CONFIG, CIRCUIT_FRI_CONFIG, CIRCUIT_PCS_CONFIG,
    MAX_RECURSIVE_PROOF_UNCOMPRESSED_BYTES, PRIVACY_CIRCUIT_PREPROCESSED_COLUMNS,
    PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT, PRIVACY_TRANSACTION_COMPONENTS,
};
use privacy_common::utils::ProofHeader;
use privacy_common::{
    compute_privacy_bootloader_output_hash, get_large_proofs_circuit_registry,
    get_privacy_bootloader_program,
};
use stwo::core::fields::qm31::QM31;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::Felt252;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;
use tracing::{Level, info, span};

#[cfg(test)]
mod tests;

// Re-exported for external consumers (the sequencer uses `PrivacyProofOutput` and
// `utils::ProofHeader` through this crate).
pub use privacy_common::{PrivacyProofOutput, utils};

/// Splits the version-prefixed proof bytes into the embedded header and the remaining
/// compressed proof bytes.
pub(crate) fn split_proof_header(proof: &[u8]) -> Result<(ProofHeader, &[u8]), Box<dyn Error>> {
    let (header_bytes, compressed_proof) = proof
        .split_first_chunk::<{ ProofHeader::SIZE }>()
        .ok_or("Proof is too short to contain a header")?;
    Ok((ProofHeader::deserialize(header_bytes), compressed_proof))
}

pub(crate) fn decompress_proof(
    compressed: &[u8],
    max_bytes: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(zstd::bulk::decompress(compressed, max_bytes)?)
}

pub fn verify_recursive_circuit(proof_output: &PrivacyProofOutput) -> Result<(), Box<dyn Error>> {
    let _span = span!(Level::INFO, "verify_privacy_circuit").entered();
    let (header, compressed_proof) = split_proof_header(&proof_output.proof)?;

    let preprocessed_root = le_u32s_from_bytes(header.preprocessed_root);
    // Compare `preprocessed_root` to the known roots to get the corresponding config.
    // If `preprocessed_root` is invalid, we return early with an error.
    // TODO(Leo): consider moving the preprocessed_root validation outside of
    // `get_recursive_circuit_config`.
    let circuit_config = get_recursive_circuit_config(&preprocessed_root)?;
    let proof_config = get_proof_config(&circuit_config);

    info!("Decompress and deserialize the proof");
    let proof_bytes = decompress_proof(compressed_proof, MAX_RECURSIVE_PROOF_UNCOMPRESSED_BYTES)?;
    let mut serialized_proof: &[u8] = &proof_bytes;
    let proof = deserialize_proof_with_config(&mut serialized_proof, &proof_config)?;
    if !serialized_proof.is_empty() {
        return Err("Proof deserialization failed".into());
    }

    info!("Compute the output digest");
    // The cairo-verifier circuit outputs the program's Blake2s digest words directly at the
    // reserved output wires (see its `set_outputs`), so reproduce them via the same
    // `HashValue::<QM31>::from(_)` conversion the circuit uses on the digest.
    let output_hash = compute_privacy_bootloader_output_hash(&proof_output.output_preimage);
    let output_digest = HashValue::<QM31>::from(output_hash);

    info!("Call the verifier");
    verify_circuit(
        circuit_config,
        // We know that `preprocessed_root` is valid because we compared it to the known roots
        // when getting the circuit config.
        preprocessed_root.into(),
        proof,
        CircuitPublicData { output_digest },
    )?;

    Ok(())
}

/// Returns, for each component in `all_components()` order, whether it is enabled in the privacy
/// transaction.
fn get_cairo_enabled_bits() -> Vec<bool> {
    all_components::<NoValue>()
        .keys()
        .map(|name| PRIVACY_TRANSACTION_COMPONENTS.contains(name))
        .collect()
}

pub fn get_cairo_proof_config() -> ProofConfig {
    let enabled_components = all_components::<NoValue>()
        .into_iter()
        .filter(|(name, _)| PRIVACY_TRANSACTION_COMPONENTS.contains(name))
        .collect();

    ProofConfig::new(
        &enabled_components,
        PreProcessedTraceVariant::CanonicalSmall.n_columns(),
        &CAIRO_PCS_CONFIG,
        INTERACTION_POW_BITS,
    )
}

pub fn get_cairo_verifier_config() -> Result<CairoVerifierConfig, Box<dyn Error>> {
    let cairo_proof_config = get_cairo_proof_config();
    let enabled_bits = get_cairo_enabled_bits();

    let bootloader_program = get_privacy_bootloader_program()?;
    let program_entries = bootloader_program
        .iter_data()
        .map(|value| {
            value.get_int().map(|felt| Felt252::from(felt).get_limbs()).ok_or("Failed to get value")
        })
        .collect::<Result<Vec<_>, _>>()?;

    let cairo_lifting_log_size = cairo_proof_config.log_evaluation_domain_size();
    let preprocessed_trace_variant = PreProcessedTraceVariant::CanonicalSmall;

    Ok(CairoVerifierConfig {
        proof_config: cairo_proof_config,
        enabled_bits,
        program: Arc::from(program_entries.as_slice()),
        preprocessed_root: get_preprocessed_root(cairo_lifting_log_size),
        preprocessed_trace_variant,
        zk_blinding_amount: Some(CIRCUIT_FRI_CONFIG.params.n_queries + NON_QUERY_INFO_LEAK),
    })
}

pub fn get_recursive_circuit_config(
    preprocessed_root: &[u32; 8],
) -> Result<CircuitConfig, Box<dyn Error>> {
    if *preprocessed_root == PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT {
        let preprocessed_column_log_sizes = PRIVACY_CIRCUIT_PREPROCESSED_COLUMNS
            .iter()
            .map(|&(id, log_size)| (PreProcessedColumnId { id: id.to_string() }, log_size))
            .collect();
        Ok(CircuitConfig { config: CIRCUIT_PCS_CONFIG, preprocessed_column_log_sizes })
    } else {
        Ok(circuit_config_from_circuit_proof_config(&leaf_verifier_config(preprocessed_root)?))
    }
}

pub fn get_proof_config(circuit_config: &CircuitConfig) -> ProofConfig {
    circuit_verifier_proof_config(
        &circuit_config.preprocessed_column_log_sizes,
        &circuit_config.config,
    )
}

fn circuit_config_from_circuit_proof_config(
    circuit_proof_config: &CircuitProofConfig,
) -> CircuitConfig {
    // Compute the trace size
    let preprocessed_column_log_sizes = layout_from_component_sizes(
        &(circuit_proof_config.component_log_sizes).to_component_sizes(),
    );
    let component_sizes = circuit_component_log_sizes(
        &all_circuit_components::<NoValue>(),
        &preprocessed_column_log_sizes,
    )
    .into_array();
    let trace_size = component_sizes.iter().max().unwrap();

    let config = PcsConfig::from_fri_config(FriConfig {
        params: circuit_proof_config.fri_params,
        trace_log_size: *trace_size,
    });
    CircuitConfig { config, preprocessed_column_log_sizes }
}

// The config used to prove the leaf verifier with the given preprocessed root.
fn leaf_verifier_config(
    preprocessed_root: &[u32; 8],
) -> Result<CircuitProofConfig, Box<dyn Error>> {
    let circuit_registry = get_large_proofs_circuit_registry();
    let leaf_verifier = circuit_registry
        .leaf_verifiers
        .iter()
        .find(|lv| lv.preprocessed_root.0 == *preprocessed_root)
        .ok_or_else(|| format!("Unknown preprocessed root {:?}", preprocessed_root))?;
    Ok(circuit_registry.config(&leaf_verifier.config)?.clone())
}

pub fn get_cairo_preprocessed_circuit(
    cairo_verifier_config: &CairoVerifierConfig,
) -> PreprocessedCircuit {
    let mut novalue_context = build_cairo_verifier_circuit(cairo_verifier_config);
    PreprocessedCircuit::preprocess_circuit(&mut novalue_context)
}
