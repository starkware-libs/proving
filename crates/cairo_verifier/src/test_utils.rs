use std::collections::HashSet;

use cairo_air::CairoProof;
use cairo_air::air::PubMemoryValue;
use cairo_air::flat_claims::FlatClaim;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::proof::ProofConfig;
use indexmap::IndexMap;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::qm31::QM31;
use stwo::core::fri::{FriConfig, FriParams};
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::felt::split_f252;

use crate::all_components::all_components;
use crate::privacy::PRIVACY_TRANSACTION_COMPONENTS;
use crate::statement::{N_OUTPUTS, N_WORDS_PER_OUTPUT_CELL};
use crate::utils::{get_test_data_dir, load_program};
use crate::verify::{
    CairoVerifierConfig, INTERACTION_POW_BITS, get_preprocessed_root,
    prepare_cairo_proof_for_circuit_verifier, verify_fixed_cairo_circuit,
};

#[cfg(test)]
#[path = "output_hash_test.rs"]
mod output_hash_test;
#[cfg(test)]
#[path = "privacy_test.rs"]
mod privacy_test;

/// The log size of the trace produced by the privacy setup. It is equal to the log size of the
/// [`PreProcessedTraceVariant::CanonicalSmall`] preprocessed trace.
const PRIVACY_CAIRO_TRACE_LOG_SIZE: u32 = 20;

/// Returns a [CairoVerifierConfig] for the privacy proof setup with the given log blowup factor.
/// See [CairoVerifierConfig] for the meaning of zk_blinding_amount.
pub fn privacy_cairo_verifier_config(
    log_blowup_factor: u32,
    zk_blinding_amount: Option<usize>,
) -> CairoVerifierConfig {
    let preprocessed_trace_variant = PreProcessedTraceVariant::CanonicalSmall;
    let privacy_set = privacy_components();
    // Build `enabled_bits` (one flag per component in the full list) and `components` (only the
    // enabled entries, as expected by `ProofConfig::new`) in a single pass.
    let (enabled_bits, components): (Vec<bool>, Vec<_>) = all_components::<NoValue>()
        .into_iter()
        .map(|(name, component)| {
            let enabled = privacy_set.contains(name);
            (enabled, enabled.then_some((name, component)))
        })
        .unzip();
    let components: IndexMap<&'static str, Box<dyn CircuitEval<NoValue>>> =
        components.into_iter().flatten().collect();

    // Derive proof config parameters from the log blowup factor, targeting 96-bit security.
    let pcs_config = privacy_pcs_config(PRIVACY_CAIRO_TRACE_LOG_SIZE, log_blowup_factor);

    let proof_config = ProofConfig::new(
        &components,
        preprocessed_trace_variant.n_columns(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );

    let program_path = get_test_data_dir().join("privacy/privacy_simple_bootloader_compiled.json");
    let program = load_program(&program_path);

    CairoVerifierConfig {
        preprocessed_root: get_preprocessed_root(pcs_config.preprocessed_lifting_log_size),
        proof_config,
        enabled_bits,
        program,
        preprocessed_trace_variant,
        zk_blinding_amount,
    }
}

/// Returns [`PRIVACY_TRANSACTION_COMPONENTS`] as a set.
pub fn privacy_components() -> HashSet<&'static str> {
    HashSet::from_iter(PRIVACY_TRANSACTION_COMPONENTS)
}

pub const fn privacy_pcs_config(trace_log_size: u32, log_blowup_factor: u32) -> PcsConfig {
    let (pow_bits, n_queries) = match log_blowup_factor {
        1 => (26, 70),
        2 => (26, 35),
        3 => (27, 23),
        _ => panic!("Unsupported log blowup factor."),
    };
    assert!(
        pow_bits + n_queries as u32 * log_blowup_factor >= 96_u32,
        "The config is not secure enough."
    );
    // Note that `fold_step` is hardcoded to 4.
    let fri_params = FriParams {
        pow_bits,
        log_blowup_factor,
        log_last_layer_degree_bound: 0,
        n_queries,
        fold_step: 4,
    };
    PcsConfig::from_fri_config(FriConfig { params: fri_params, trace_log_size })
}

/// Packs the program's output memory cells into the Blake2s digest they represent.
///
/// The program emits its output as a single Blake2s digest stored across `N_OUTPUTS` memory cells,
/// each holding one 128-bit half. Each cell contributes its low `N_WORDS_PER_OUTPUT_CELL` words
/// (little-endian) to the digest; this is the exact inverse of the reconstruction performed by
/// `output_limbs_from_output_hash`, so `CairoStatement::new` recovers the same cells from the
/// returned hash.
///
/// Returns an error if the output isn't exactly `N_OUTPUTS` cells or if any cell doesn't fit in
/// 128 bits.
fn output_hash_from_output_cells(output: &[PubMemoryValue]) -> Result<Blake2sHash, String> {
    if output.len() != N_OUTPUTS {
        return Err(format!(
            "The circuit verifier requires exactly {N_OUTPUTS} output cells, got {}.",
            output.len()
        ));
    }
    let mut output_hash_bytes = [0u8; 32];
    const BYTES_PER_WORD: usize = size_of::<u32>();
    const BYTES_PER_CELL: usize = N_WORDS_PER_OUTPUT_CELL * BYTES_PER_WORD;
    for (memory_cell_bytes, (_id, value)) in
        zip_eq(output_hash_bytes.chunks_exact_mut(BYTES_PER_CELL), output.iter())
    {
        if value[N_WORDS_PER_OUTPUT_CELL..].iter().any(|word| *word != 0) {
            return Err("Each output cell must fit in 128 bits.".to_string());
        }
        for (dst, word) in zip_eq(
            memory_cell_bytes.chunks_exact_mut(BYTES_PER_WORD),
            &value[..N_WORDS_PER_OUTPUT_CELL],
        ) {
            dst.copy_from_slice(&word.to_le_bytes());
        }
    }
    Ok(Blake2sHash(output_hash_bytes))
}

/// Verifies a [CairoProof] with a given set of components. Doesn't add ZK blinding.
pub fn verify_cairo_with_component_set(
    cairo_proof: &CairoProof<Blake2sMerkleHasher>,
    component_set: HashSet<&str>,
) -> Result<FinalizedContext<QM31>, String> {
    let FlatClaim { component_enable_bits, component_log_sizes: _, public_data } =
        cairo_proof.claim.flatten_claim();
    let components: IndexMap<&'static str, Box<dyn CircuitEval<QM31>>> =
        zip_eq(all_components::<QM31>().into_iter(), &component_enable_bits)
            .filter_map(|((component_name, component), &enable_bit)| {
                let component_in_set = component_set.contains(component_name);
                if component_in_set != enable_bit {
                    return Some(Err(format!(
                        "Proof was produced with the wrong components set: expected the component \
                         '{}' to be {} according to the component set, but it is {} in the proof.",
                        component_name,
                        if component_in_set { "enabled" } else { "disabled" },
                        if enable_bit { "enabled" } else { "disabled" }
                    )));
                }
                if enable_bit { Some(Ok((component_name, component))) } else { None }
            })
            .try_collect()?;

    let (proof, serialized_aux_data) = prepare_cairo_proof_for_circuit_verifier(cairo_proof);

    let output_hash = output_hash_from_output_cells(&public_data.public_memory.output)?;

    let program =
        public_data.public_memory.program.iter().map(|(_id, value)| split_f252(*value)).collect();

    let ppt_root = cairo_proof.extended_stark_proof.proof.commitments[0];
    let proof_config = ProofConfig::new(
        &components,
        cairo_proof.preprocessed_trace_variant.n_columns(),
        &cairo_proof.extended_stark_proof.proof.config,
        INTERACTION_POW_BITS,
    );
    let verifier_config = CairoVerifierConfig {
        preprocessed_root: ppt_root.into(),
        proof_config,
        enabled_bits: component_enable_bits,
        program,
        preprocessed_trace_variant: cairo_proof.preprocessed_trace_variant,
        zk_blinding_amount: None,
    };

    verify_fixed_cairo_circuit(&verifier_config, proof, serialized_aux_data, output_hash)
}
