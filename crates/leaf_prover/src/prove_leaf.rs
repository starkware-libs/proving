use std::path::{Path, PathBuf};
use std::sync::Arc;

use cairo_air::verifier::INTERACTION_POW_BITS;
use cairo_program_runner_lib::utils::{
    get_cairo_run_config, get_program, get_program_input_from_path,
};
use cairo_program_runner_lib::{ProgramInput, cairo_run_program};
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::types::program::Program;
use circuit_cairo_verifier::all_components::all_components;
use circuit_cairo_verifier::statement::{MEMORY_VALUES_LIMBS, N_OUTPUTS, N_WORDS_PER_OUTPUT_CELL};
use circuit_cairo_verifier::verify::{
    CairoVerifierConfig, build_and_fill_cairo_verifier_circuit,
    prepare_cairo_proof_for_circuit_verifier,
};
use circuit_common::finalize::pad_to_targets;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    BaseColumnPool, prepare_circuit_proof_for_circuit_verifier, prove_circuit_assignment,
};
use circuit_registry::CircuitRegistry;
use circuit_serialize::serialize::CircuitSerialize;
use circuits::blake::HashValue;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::proof::ProofConfig;
use indexmap::IndexMap;
use leaf_proof_format::{DigestHex, SerializedLeafProof};
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::verifier::PREPROCESSED_TRACE_IDX;
use stwo_cairo_adapter::adapter::adapt;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_prover::prover::{LiftingSizePolicy, prove_cairo};
use stwo_cairo_prover::witness::prelude::{Felt252, QM31};
use tracing::info;

use crate::consts::{
    DISABLED_COMPONENTS_CANONICAL_PREPROCESSED, DISABLED_COMPONENTS_SMALL_PREPROCESSED,
};

/// File-path front end of [`prove_leaf`]: loads the program, program input and circuit registry
/// from disk.
pub fn prove_leaf_from_files(
    program_path: &Path,
    program_input: &Option<PathBuf>,
    circuit_registry_json: &Path,
) -> SerializedLeafProof {
    let program = get_program(program_path)
        .unwrap_or_else(|err| panic!("Cannot get program from {}: {err}", program_path.display()));
    // Infallible: the input file is only wrapped here, not read (that happens inside the run).
    let program_input = get_program_input_from_path(program_input).unwrap();
    let circuit_registry =
        CircuitRegistry::from_path(circuit_registry_json).unwrap_or_else(|err| panic!("{err}"));
    prove_leaf(&program, program_input, &circuit_registry)
}

/// Proves `program`'s run, verifies that proof with the cairo-verifier circuit, and proves that
/// circuit's execution.
///
/// A Cairo proof whose trace size the registry does not cover is rejected.
pub fn prove_leaf(
    program: &Program,
    program_input: Option<ProgramInput>,
    circuit_registry: &CircuitRegistry,
) -> SerializedLeafProof {
    let cairo_prover_parameters = circuit_registry.cairo_prover_params;
    assert!(
        cairo_prover_parameters.include_all_preprocessed_columns,
        "The prover parameters must set include_all_preprocessed_columns=true because the \
         verifier circuit expects a constant number of preprocessed columns"
    );
    assert!(
        matches!(
            cairo_prover_parameters.lifting_size_policy,
            LiftingSizePolicy::AtLeastPreprocessed
        ),
        "The prover parameters must set lifting_size_policy=AtLeastPreprocessed because the \
         circuit-cairo-verifier only supports verifying proofs where the lifting size is >= the \
         preprocessed trace height"
    );

    // Execute & prove the input Cairo program.

    let cairo_run_config = get_cairo_run_config(
        // we don't use dynamic layout in stwo.
        &None,
        LayoutName::all_cairo_stwo,
        // proof_mode.
        true,
        // in stwo when proof_mode==true, trace padding is redundant work.
        true,
        // allow_missing_builtins - ignored when proof_mode == true.
        true,
        // we don't need to relocate memory in the VM because we later call the adapter that does
        // relocation.
        false,
    )
    .unwrap();
    let runner = cairo_run_program(program, program_input, cairo_run_config, None).unwrap();
    info!("Program execution done");

    let prover_input = adapt(&runner).unwrap();
    let output_addresses = prover_input.builtin_segments.output.unwrap();

    let program_output_u256s: Vec<[u32; 8]> = (output_addresses.begin_addr
        ..output_addresses.stop_ptr)
        .map(|addr| prover_input.memory.get(addr.try_into().unwrap()).as_u256())
        .collect();
    let n_outputs = program_output_u256s.len();
    info!("Adapter done. Program created {n_outputs} outputs.");

    assert_eq!(
        n_outputs, N_OUTPUTS,
        "The circuit cairo verifier expects exactly {N_OUTPUTS} output cells but the program \
         produced {n_outputs}."
    );
    // The program emits its output as the raw 256-bit Blake2s digest, one 128-bit half per output
    // cell. Concatenate the low `N_WORDS_PER_OUTPUT_CELL` words of each cell into the 8-word
    // digest.
    let mut digest_words = [0u32; N_OUTPUTS * N_WORDS_PER_OUTPUT_CELL];
    for (chunk, cell) in
        digest_words.chunks_exact_mut(N_WORDS_PER_OUTPUT_CELL).zip(program_output_u256s.iter())
    {
        chunk.copy_from_slice(&cell[..N_WORDS_PER_OUTPUT_CELL]);
    }
    let output_hash = Blake2sHash::from(digest_words);
    let proof =
        prove_cairo::<Blake2sM31MerkleChannel>(prover_input, cairo_prover_parameters).unwrap();
    info!("Cairo proving done");

    let preprocessed_root = proof.extended_stark_proof.proof.commitments[PREPROCESSED_TRACE_IDX];

    let pcs_config = proof.extended_stark_proof.proof.config;

    // `AtLeastPreprocessed`, asserted above, lifts every tree to a single, shared height, which the
    // circuit verifier requires and which the trace size is then read off of.
    assert_eq!(
        pcs_config.trace_lifting_log_size, pcs_config.preprocessed_lifting_log_size,
        "The verifier circuit expects every tree, the preprocessed one included, lifted to the \
         same size"
    );
    let trace_log_size =
        pcs_config.trace_lifting_log_size - pcs_config.fri_config.log_blowup_factor;
    let registry_entry =
        circuit_registry.leaf_verifier(trace_log_size).unwrap_or_else(|err| panic!("{err}"));
    let circuit_proof_config =
        circuit_registry.config(&registry_entry.config).unwrap_or_else(|err| panic!("{err}"));

    let verifier_config = leaf_verifier_config(
        cairo_prover_parameters.preprocessed_trace,
        &pcs_config,
        Arc::from(program_felts(program)),
        preprocessed_root.into(),
    );

    // Verify that the Cairo proof has the expected trace width (if not - this is an
    // indication that the program doesn't use all components).
    for (trace_idx, trace_name) in ["preprocessed", "base", "interaction"].iter().enumerate() {
        let expected_columns = verifier_config.proof_config.n_columns_per_trace()[trace_idx];
        let columns_in_proof = proof.extended_stark_proof.proof.queried_values[trace_idx].len();
        assert!(
            columns_in_proof == expected_columns,
            "Expected {expected_columns} columns in {trace_name} trace, but proof has \
             {columns_in_proof}"
        );
    }

    let (proof_for_circuit, serialized_aux_data) =
        prepare_cairo_proof_for_circuit_verifier(&proof, &verifier_config.enabled_bits);

    let mut context = build_and_fill_cairo_verifier_circuit(
        &verifier_config,
        proof_for_circuit,
        serialized_aux_data,
        output_hash,
    );

    // Pad to the registry's shared target, giving every circuit it lists one shape — what lets
    // a single multiverifier circuit verify any of them.
    pad_to_targets(&mut context, &circuit_proof_config.target_sizes());

    info!(
        "Verifier config:
    program: ({} felts)
    n_outputs: {}
    Cairo preprocessed trace: {:?}
    Cairo preprocessed trace root: {:?}
    Proof pow bits: {}
    Proof FRI config: {:?}",
        verifier_config.program.len(),
        n_outputs,
        verifier_config.preprocessed_trace_variant,
        verifier_config.preprocessed_root,
        verifier_config.proof_config.fri.pow_bits,
        verifier_config.proof_config.fri,
    );
    assert!(context.is_circuit_valid(), "The verifier circuit rejected the proof!");
    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut context);

    // Prove the execution of the verifier circuit.

    // The padded circuit fixes the circuit proof's lifting size.
    let circuit_prover_pcs_config = PcsConfig::from_fri_and_trace_size(
        circuit_proof_config.fri_config,
        preprocessed_circuit.trace_log_size,
    );
    let base_column_pool = BaseColumnPool::new();
    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed_circuit,
        &base_column_pool,
        circuit_prover_pcs_config,
    )
    .unwrap();
    info!("Circuit proving done");
    let circuit_preprocessed_root =
        circuit_proof.stark_proof.proof.commitments[PREPROCESSED_TRACE_IDX].0;
    // The hash the prover mixed into the channel; identifies the circuit together with the config
    // its preprocessed root is interpreted under.
    let circuit_hash = DigestHex::from(circuit_proof.circuit_hash.0);
    info!("Circuit preprocessed root: {:?}", circuit_preprocessed_root);
    info!("Circuit hash: {:?}", circuit_hash);

    // The proven circuit must be the one the registry describes; no verifier of these circuits
    // trusts an unrecognized circuit.
    assert_eq!(
        circuit_hash, registry_entry.circuit_hash,
        "The proven circuit's hash differs from the registry's leaf verifier for trace log size \
         {trace_log_size}."
    );

    // Convert the proof to our output format.

    let (proof_qm31s, _public_data) = prepare_circuit_proof_for_circuit_verifier(circuit_proof);

    let mut proof_bytes: Vec<u8> = vec![];
    proof_qm31s.serialize(&mut proof_bytes);

    SerializedLeafProof {
        circuit_preprocessed_root: circuit_preprocessed_root.into(),
        circuit_hash,
        proof: proof_bytes,
    }
}

/// The components of the circuit that verifies the Cairo proof.
pub struct LeafVerifierComponents {
    /// Map from component name to the circuit evaluator that verifies it.
    pub components: IndexMap<&'static str, Box<dyn CircuitEval<QM31>>>,
    /// One bit per possible component: `true` if the component is enabled (present).
    pub enabled_bits: Vec<bool>,
}

/// Builds the [`CairoVerifierConfig`] of the leaf circuit that verifies a Cairo proof of the
/// given preprocessed-trace variant: picks the disabled component set from the variant and
/// assembles the component list, enabled bits and [`ProofConfig`].
///
/// `pcs_config` is the verified Cairo proof's PCS config with an explicit `lifting_log_size`
/// (`trace_log_size + log_blowup_factor`), and `preprocessed_root` the expected root of that
/// proof's preprocessed trace.
/// The components a Cairo program proven with the given preprocessed-trace variant is expected NOT
/// to use.
fn disabled_components(variant: PreProcessedTraceVariant) -> &'static [&'static str] {
    match variant {
        PreProcessedTraceVariant::Canonical => &DISABLED_COMPONENTS_CANONICAL_PREPROCESSED,
        PreProcessedTraceVariant::CanonicalSmall => &DISABLED_COMPONENTS_SMALL_PREPROCESSED,
        _ => panic!("Unsupported preprocessed trace {variant:?}"),
    }
}

pub fn leaf_verifier_config(
    preprocessed_trace_variant: PreProcessedTraceVariant,
    pcs_config: &PcsConfig,
    program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    preprocessed_root: HashValue<QM31>,
) -> CairoVerifierConfig {
    let LeafVerifierComponents { components, enabled_bits } =
        leaf_verifier_components(disabled_components(preprocessed_trace_variant));
    let proof_config = ProofConfig::new(
        &components,
        preprocessed_trace_variant.n_columns(),
        pcs_config,
        INTERACTION_POW_BITS,
    );
    CairoVerifierConfig {
        proof_config,
        enabled_bits,
        program,
        preprocessed_root,
        preprocessed_trace_variant,
    }
}

/// Creates the component list and enabled bits for the circuit that verifies the Cairo proof.
/// The set of components is constant (all possible components for the given preprocessed trace,
/// minus `disabled_components`) to keep the verifier circuit stable. The trace is expected to
/// contain all the components in this set.
pub fn leaf_verifier_components(disabled_components: &[&str]) -> LeafVerifierComponents {
    let mut components: IndexMap<&'static str, Box<dyn CircuitEval<QM31>>> = IndexMap::default();
    let mut enabled_bits = vec![];
    for (name, component) in all_components::<QM31>() {
        if disabled_components.contains(&name) {
            enabled_bits.push(false);
        } else {
            components.insert(name, component);
            enabled_bits.push(true);
        }
    }
    LeafVerifierComponents { components, enabled_bits }
}

fn program_felts(program: &Program) -> Vec<[M31; MEMORY_VALUES_LIMBS]> {
    let mut program_felts = vec![];
    for value in program.iter_data() {
        let value = value.get_int().unwrap();
        program_felts.push(Felt252::from(value).get_limbs());
    }
    program_felts
}
