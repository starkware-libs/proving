use circuit_common::N_RESERVED;
use circuits::blake::{HashValue, blake2s_u32s, unpack_qm31s_to_u32_words};
use circuits::context::{Context, FinalizedContext};
use circuits::ivalue::IValue;
use circuits::ops::Guess;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::proof::{Proof, ProofConfig};
use circuits_stark_verifier::statement::Statement;
use circuits_stark_verifier::verify::verify;
use itertools::Itertools;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::statement::{CircuitStatement, INTERACTION_POW_BITS};

pub struct CircuitPublicData {
    pub output_values: Vec<QM31>,
}

#[derive(Debug, PartialEq)]
pub struct CircuitConfig {
    pub config: PcsConfig,
    /// The number of output gates of the circuit, excluding the output gate of the `u` constant
    /// (at address [`circuits::context::U_VAR_IDX`]).
    pub n_outputs: usize,
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
    pub preprocessed_root: HashValue<QM31>,
}

/// Builds the circuit that verifies a proof of execution of another circuit.
///
/// The circuit:
/// 1. Builds a [`CircuitStatement`] from the output addresses, output values, preprocessed column
///    log sizes, and preprocessed root.
/// 2. Guesses the proof values into the circuit and runs the STARK verification.
/// 3. Hashes the preprocessed root together with all output values except the last (`u`) via Blake,
///    and copies the resulting unreduced digest into the reserved output wires (`3..3 +
///    N_RESERVED`). To ensure soundness in a recursive setup, the outer-most verifier (assumed
///    honest) must reconstruct the whole chain of output hashes computed during the recursive
///    steps.
/// 4. Finalizes constants and guessed variables.
pub fn build_verification_circuit<Value: IValue>(
    circuit_config: CircuitConfig,
    proof: Proof<Value>,
    public_data: CircuitPublicData,
) -> Result<FinalizedContext<Value>, String> {
    let mut context = Context::new(N_RESERVED);
    let output_values = public_data
        .output_values
        .iter()
        .map(|value| Value::from_qm31(*value).guess(&mut context))
        .collect_vec();
    let statement = CircuitStatement::new(&mut context, &circuit_config, &output_values);

    let proof_config = ProofConfig::new(
        statement.get_components(),
        circuit_config.preprocessed_column_log_sizes.len(),
        &circuit_config.config,
        INTERACTION_POW_BITS,
    );
    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &proof_config, &statement);

    // Deal with the outputs: hash the preprocessed root and all the output values except `u` (= the
    // last one). This is fine for soundness because `u` is checked as part of the logup sum.
    let preprocessed_root = statement.get_preprocessed_root(&mut context);
    let output_preimage: Vec<_> = preprocessed_root
        .into_iter()
        .chain(unpack_qm31s_to_u32_words(&mut context, output_values))
        .collect();
    let n_bytes = 4 * output_preimage.len();
    let output_hash = blake2s_u32s(&mut context, output_preimage, n_bytes);
    context.set_outputs(&output_hash.iter().map(|word| *word.get()).collect_vec());

    let context = context.finalize(false);
    #[cfg(test)]
    context.circuit().check_yields();

    Ok(context)
}

pub fn verify_circuit(
    circuit_config: CircuitConfig,
    proof: Proof<QM31>,
    public_data: CircuitPublicData,
) -> Result<FinalizedContext<QM31>, String> {
    let context = build_verification_circuit(circuit_config, proof, public_data)?;
    #[cfg(test)]
    context.check_vars_used();

    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}
