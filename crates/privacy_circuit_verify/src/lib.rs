pub mod consts;
#[cfg(test)]
mod tests;
pub mod utils;

use std::error::Error;
use std::sync::Arc;

use anyhow::Result;
use cairo_air::verifier::INTERACTION_POW_BITS;
use cairo_vm::types::program::Program;
use circuit_cairo_verifier::all_components::all_components;
use circuit_cairo_verifier::statement::AUX_DATA_FIXED_LEN;
use circuit_cairo_verifier::verify::{
    CairoVerifierConfig, NON_QUERY_INFO_LEAK, build_cairo_verifier_circuit, get_preprocessed_root,
    verify_fixed_cairo_circuit,
};
use circuit_common::preprocessed::{PreprocessedCircuit, layout_from_component_sizes};
use circuit_registry::{CircuitProofConfig, CircuitRegistry};
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_verifier::components::prelude::PreProcessedColumnId;
use circuit_verifier::statement::{
    INTERACTION_POW_BITS as CIRCUIT_INTERACTION_POW_BITS, all_circuit_components,
    circuit_component_log_sizes,
};
use circuit_verifier::verify::{CircuitConfig, CircuitPublicData, verify_circuit};
use circuits::blake::HashValue;
use circuits::ivalue::NoValue;
use circuits::utils::le_u32s_from_bytes;
use circuits_stark_verifier::proof::ProofConfig;
use itertools::Itertools;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::Blake2Felt252;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::Felt252;
use tracing::{Level, info, span};
pub use utils::{VERSION_BYTES, Version};

use crate::consts::{
    CAIRO_PCS_CONFIG, CIRCUIT_FRI_CONFIG, CIRCUIT_OUTPUT_ADDRESSES, CIRCUIT_PCS_CONFIG,
    LARGE_PROOFS_CIRCUIT_REGISTRY_JSON, LEAF_BOOTLOADER_JSON, MAX_CAIRO_PROOF_UNCOMPRESSED_BYTES,
    MAX_RECURSIVE_PROOF_UNCOMPRESSED_BYTES, PRIVACY_BOOTLOADER_JSON,
    PRIVACY_CIRCUIT_PREPROCESSED_IDS, PRIVACY_CIRCUIT_PREPROCESSED_LOG_SIZES,
    PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT, PRIVACY_TRANSACTION_COMPONENTS,
};
use crate::utils::ProofHeader;

pub struct PrivacyProofOutput {
    /// Proof bytes, laid out as the serialized [`ProofHeader`] of the `privacy-prove` crate that
    /// generated the proof, followed by the compressed proof. The format must be consistent
    /// between the prover and verifier:
    /// - `privacy_prove` / `verify_cairo`
    /// - `privacy_recursive_prove` / `verify_recursive_circuit`
    pub proof: Vec<u8>,
    pub output_preimage: Vec<Felt>,
}

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

pub fn verify_cairo(proof_output: &PrivacyProofOutput) -> Result<(), Box<dyn Error>> {
    let _span = span!(Level::INFO, "verify_privacy_bootloader").entered();

    let verifier_config = get_cairo_verifier_config()?;

    info!("Decompress and deserialize the proof");
    let (_header, compressed_proof) = split_proof_header(&proof_output.proof)?;
    let proof_bytes = decompress_proof(compressed_proof, MAX_CAIRO_PROOF_UNCOMPRESSED_BYTES)?;
    let bootloader_program = get_privacy_bootloader_program()?;
    let program_len = bootloader_program.data_len();
    let n_components = verifier_config.proof_config.n_components();
    let (serialized_aux_data_bytes, serialized_proof_bytes) =
        proof_bytes.split_at((AUX_DATA_FIXED_LEN + program_len + n_components) * 4);
    let serialized_aux_data: Vec<M31> = serialized_aux_data_bytes
        .chunks_exact(4)
        .map(|c| M31::from(u32::from_le_bytes(c.try_into().unwrap())))
        .collect();
    let mut serialized_proof: &[u8] = serialized_proof_bytes;
    let proof =
        deserialize_proof_with_config(&mut serialized_proof, &verifier_config.proof_config)?;
    if !serialized_proof.is_empty() {
        return Err("Proof deserialization failed".into());
    }

    info!("Compute the output");
    let output_hash = compute_privacy_bootloader_output_hash(&proof_output.output_preimage);

    info!("Call the verifier");
    verify_fixed_cairo_circuit(&verifier_config, proof, serialized_aux_data, output_hash)?;

    Ok(())
}

pub fn verify_recursive_circuit(proof_output: &PrivacyProofOutput) -> Result<(), Box<dyn Error>> {
    let _span = span!(Level::INFO, "verify_privacy_circuit").entered();
    let (header, compressed_proof) = split_proof_header(&proof_output.proof)?;

    let preprocessed_root = le_u32s_from_bytes(header.preprocessed_root);
    let circuit_config = get_recursive_circuit_config(&preprocessed_root)?;
    let proof_config = get_proof_config(&preprocessed_root)?;

    info!("Decompress and deserialize the proof");
    let proof_bytes = decompress_proof(compressed_proof, MAX_RECURSIVE_PROOF_UNCOMPRESSED_BYTES)?;
    let mut serialized_proof: &[u8] = &proof_bytes;
    let proof = deserialize_proof_with_config(&mut serialized_proof, &proof_config)?;
    if !serialized_proof.is_empty() {
        return Err("Proof deserialization failed".into());
    }

    info!("Compute the output values");
    // The cairo-verifier circuit outputs the program's Blake2s digest words directly at the
    // reserved output wires (see its `set_outputs`), so reproduce them via the same
    // `HashValue::<QM31>::from(_)` conversion the circuit uses on the digest. The `u` anchor wire
    // is appended internally by the verifier, so it must not be part of `output_values`.
    let output_hash = compute_privacy_bootloader_output_hash(&proof_output.output_preimage);
    let output_values: Vec<QM31> =
        HashValue::<QM31>::from(output_hash).iter().map(|w| *w.get()).collect();

    info!("Call the verifier");
    verify_circuit(circuit_config, proof, CircuitPublicData { output_values })?;

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
    let mut program_entries = vec![];
    for value in bootloader_program.iter_data() {
        let value = value.get_int().ok_or("Failed to get value")?;
        program_entries.push(Felt252::from(value).get_limbs());
    }

    let cairo_lifting_log_size: u32 = cairo_proof_config.log_evaluation_domain_size() as u32;
    let preprocessed_trace_variant = PreProcessedTraceVariant::CanonicalSmall;

    Ok(CairoVerifierConfig {
        proof_config: cairo_proof_config,
        enabled_bits,
        program: Arc::from(program_entries.as_slice()),
        preprocessed_root: get_preprocessed_root(cairo_lifting_log_size),
        preprocessed_trace_variant,
        zk_blinding_amount: Some(CIRCUIT_FRI_CONFIG.n_queries + NON_QUERY_INFO_LEAK),
    })
}

pub fn get_privacy_bootloader_program() -> Result<Program, Box<dyn Error>> {
    let bootloader_program = Program::from_bytes(PRIVACY_BOOTLOADER_JSON, Some("main"))?;
    Ok(bootloader_program)
}

pub fn get_leaf_bootloader_program() -> Result<Program, Box<dyn Error>> {
    let bootloader_program = Program::from_bytes(LEAF_BOOTLOADER_JSON, Some("main"))?;
    Ok(bootloader_program)
}

/// Computes the Blake2s digest that the privacy bootloader emits as its output memory cells.
///
/// The bootloader hashes the felt-encoded `output_preimage` with Blake2s (the same encoding as
/// [`Blake2Felt252::encode_felt252_data_and_calc_blake_hash`]) and stores the raw 256-bit digest
/// across its output cells. This returns that digest directly (before any packing/reduction into
/// a felt), which is the `output_hash` the circuit cairo verifier expects.
pub fn compute_privacy_bootloader_output_hash(output_preimage: &[Felt]) -> Blake2sHash {
    let u32_words = Blake2Felt252::encode_felts_to_u32s(output_preimage);
    let byte_stream: Vec<u8> = u32_words.iter().flat_map(|word| word.to_le_bytes()).collect();
    Blake2sHasher::hash(&byte_stream)
}

pub fn get_recursive_circuit_config(
    preprocessed_root: &[u32; 8],
) -> Result<CircuitConfig, Box<dyn Error>> {
    if *preprocessed_root == PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT {
        let preprocessed_column_log_sizes = PRIVACY_CIRCUIT_PREPROCESSED_IDS
            .iter()
            .zip_eq(PRIVACY_CIRCUIT_PREPROCESSED_LOG_SIZES.iter())
            .map(|(&id, &log_size)| (PreProcessedColumnId { id: id.to_string() }, log_size))
            .collect();
        Ok(CircuitConfig {
            config: CIRCUIT_PCS_CONFIG,
            // `n_outputs` counts only the real output gates (the hash at addresses 3 and 4); the
            // `u` anchor wire (address 2, also in `CIRCUIT_OUTPUT_ADDRESSES`) is
            // appended by the verifier.
            n_outputs: CIRCUIT_OUTPUT_ADDRESSES.len() - 1,
            preprocessed_column_log_sizes,
            preprocessed_root: PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT.into(),
        })
    } else {
        let circuit_registry = get_large_proofs_circuit_registry();
        let leaf_verifier = circuit_registry
            .leaf_verifiers
            .iter()
            .find(|lv| lv.preprocessed_root.0 == *preprocessed_root)
            .ok_or_else(|| format!("Unknown preprocessed root {:?}", preprocessed_root))?;

        let config = leaf_verifier_config(preprocessed_root)?;
        let preprocessed_column_log_sizes =
            layout_from_component_sizes(&(&config.component_log_sizes).into());
        Ok(CircuitConfig {
            config: pcs_config_from_circuit_proof_config(&config),
            n_outputs: CIRCUIT_OUTPUT_ADDRESSES.len() - 1,
            preprocessed_column_log_sizes,
            preprocessed_root: leaf_verifier.preprocessed_root.0.into(),
        })
    }
}

pub fn get_proof_config(preprocessed_root: &[u32; 8]) -> Result<ProofConfig, Box<dyn Error>> {
    let components = all_circuit_components::<QM31>();
    if *preprocessed_root == PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT {
        Ok(ProofConfig::new(
            &components,
            PRIVACY_CIRCUIT_PREPROCESSED_IDS.len(),
            &CIRCUIT_PCS_CONFIG,
            CIRCUIT_INTERACTION_POW_BITS,
        ))
    } else {
        let config = leaf_verifier_config(preprocessed_root)?;
        let n_preprocessed_columns =
            layout_from_component_sizes(&(&config.component_log_sizes).into()).len();
        let pcs_config = pcs_config_from_circuit_proof_config(&config);
        Ok(ProofConfig::new(
            &components,
            n_preprocessed_columns,
            &pcs_config,
            CIRCUIT_INTERACTION_POW_BITS,
        ))
    }
}

fn pcs_config_from_circuit_proof_config(circuit_proof_config: &CircuitProofConfig) -> PcsConfig {
    // Compute the trace size
    let preprocessed_column_log_sizes =
        layout_from_component_sizes(&(&circuit_proof_config.component_log_sizes).into());
    let component_sizes = circuit_component_log_sizes(
        &all_circuit_components::<NoValue>(),
        &preprocessed_column_log_sizes,
    )
    .into_array();
    let trace_size = component_sizes.iter().max().unwrap();

    // Build the PcsConfig
    PcsConfig::from_fri_and_trace_size(circuit_proof_config.fri_config, *trace_size)
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

pub fn get_large_proofs_circuit_registry() -> CircuitRegistry {
    serde_json::from_str(LARGE_PROOFS_CIRCUIT_REGISTRY_JSON).unwrap()
}
