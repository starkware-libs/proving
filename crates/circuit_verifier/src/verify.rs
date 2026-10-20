use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::IValue;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::proof::Proof;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::multiverifier::{MultiverifierInput, build_multiverifier_circuit, shared_config};

pub struct CircuitPublicData<Value: IValue> {
    /// The verified circuit's output: the unreduced Blake2s digest held at its
    /// [`circuit_common::N_RESERVED`] reserved output wires. The output gate of the `u` constant
    /// (at address [`circuits::context::U_VAR_IDX`]) is appended by the verifier and is not part
    /// of this.
    pub output_digest: HashValue<Value>,
}

#[derive(Debug, PartialEq)]
pub struct CircuitConfig {
    pub config: PcsConfig,
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
}

<<<<<<< HEAD
/// Builds the circuit that verifies a proof of execution of another circuit.
///
/// The circuit:
/// 1. Builds a [`CircuitStatement`] from the verified circuit's output digest, preprocessed column
///    log sizes, and preprocessed root.
/// 2. Guesses the proof values into the circuit and runs the STARK verification.
/// 3. Hashes the preprocessed root together with the verified circuit's output digest via Blake,
///    and copies the resulting unreduced digest into the reserved output wires (`3..3 +
///    N_RESERVED`). To ensure soundness in a recursive setup, the outer-most verifier (assumed
///    honest) must reconstruct the whole chain of output hashes computed during the recursive
///    steps.
/// 4. Finalizes constants and guessed variables.
pub fn build_verification_circuit<Value: IValue>(
    circuit_config: CircuitConfig,
    preprocessed_root: HashValue<Value>,
    proof: Proof<Value>,
    public_data: CircuitPublicData<Value>,
) -> Result<FinalizedContext<Value>, String> {
    let mut context = Context::new(N_RESERVED);
    let output_digest = public_data.output_digest.guess(&mut context);
    // Guess the preprocessed root. The guessed wires enter the hash that will be output by this
    // verifier. To ensure soundness in a recursive setup, it is *critical* that this hash is
    // reconstructed by the last verifier, which we can assume honest.
    let preprocessed_root = preprocessed_root.guess(&mut context);
    let statement =
        CircuitStatement::new(&mut context, &circuit_config, preprocessed_root, output_digest);

    let proof_config = circuit_verifier_proof_config(
        &circuit_config.preprocessed_column_log_sizes,
        &circuit_config.config,
    );
    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &proof_config, &statement);

    // Deal with the outputs: hash the preprocessed root and all the output values except `u` (= the
    // last one). This is fine for soundness because `u` is checked as part of the logup sum.
    let preprocessed_root = statement.get_preprocessed_root(&mut context);
    let output_preimage: Vec<_> =
        preprocessed_root.iter().copied().chain(statement.output_digest.iter().copied()).collect();
    let n_bytes = 4 * output_preimage.len();
    let output_hash = blake2s_u32s(&mut context, output_preimage, n_bytes);
    context.set_outputs(&output_hash.iter().map(|word| *word.raw()).collect_vec());

    let context = context.finalize(false);
    #[cfg(test)]
    context.circuit().check_yields();

    Ok(context)
}

||||||| 167116f40
/// Builds the circuit that verifies a proof of execution of another circuit.
///
/// The circuit:
/// 1. Builds a [`CircuitStatement`] from the verified circuit's output digest, preprocessed column
///    log sizes, and preprocessed root.
/// 2. Guesses the proof values into the circuit and runs the STARK verification.
/// 3. Hashes the preprocessed root together with the verified circuit's output digest via Blake,
///    and copies the resulting unreduced digest into the reserved output wires (`3..3 +
///    N_RESERVED`). To ensure soundness in a recursive setup, the outer-most verifier (assumed
///    honest) must reconstruct the whole chain of output hashes computed during the recursive
///    steps.
/// 4. Finalizes constants and guessed variables.
pub fn build_verification_circuit<Value: IValue>(
    circuit_config: CircuitConfig,
    preprocessed_root: HashValue<Value>,
    proof: Proof<Value>,
    public_data: CircuitPublicData<Value>,
) -> Result<FinalizedContext<Value>, String> {
    let mut context = Context::new(N_RESERVED);
    let output_digest = public_data.output_digest.guess(&mut context);
    // Guess the preprocessed root. The guessed wires enter the hash that will be output by this
    // verifier. To ensure soundness in a recursive setup, it is *critical* that this hash is
    // reconstructed by the last verifier, which we can assume honest.
    let preprocessed_root = preprocessed_root.guess(&mut context);
    let statement =
        CircuitStatement::new(&mut context, &circuit_config, preprocessed_root, output_digest);

    let proof_config = circuit_verifier_proof_config(
        &circuit_config.preprocessed_column_log_sizes,
        &circuit_config.config,
    );
    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &proof_config, &statement);

    // Deal with the outputs: hash the preprocessed root and all the output values except `u` (= the
    // last one). This is fine for soundness because `u` is checked as part of the logup sum.
    let preprocessed_root = statement.get_preprocessed_root(&mut context);
    let output_preimage: Vec<_> =
        preprocessed_root.iter().copied().chain(statement.output_digest.iter().copied()).collect();
    let n_bytes = 4 * output_preimage.len();
    let output_hash = blake2s_u32s(&mut context, output_preimage, n_bytes);
    context.set_outputs(&output_hash.iter().map(|word| *word.get()).collect_vec());

    let context = context.finalize(false);
    #[cfg(test)]
    context.circuit().check_yields();

    Ok(context)
}

=======
/// Verifies a 1-1 (privacy) circuit proof.
>>>>>>> origin/sharp-8.0
pub fn verify_circuit(
    circuit_config: CircuitConfig,
    preprocessed_root: HashValue<QM31>,
    proof: Proof<QM31>,
    public_data: CircuitPublicData<QM31>,
) -> Result<FinalizedContext<QM31>, String> {
    let CircuitConfig { config: pcs_config, preprocessed_column_log_sizes } = circuit_config;
    let shared = shared_config(preprocessed_column_log_sizes, pcs_config);
    let input =
        MultiverifierInput { proof, preprocessed_root, output_digest: public_data.output_digest };
    let context = build_multiverifier_circuit(vec![input], &shared);
    #[cfg(test)]
    context.check_vars_used();

    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}
