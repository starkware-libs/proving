use circuit_common::N_RESERVED;
use circuit_verifier::statement::CircuitStatement;
use circuit_verifier::verify::CircuitConfig;
use circuits::blake::{HashValue, blake2s_u32s};
use circuits::context::{Context, FinalizedContext};
use circuits::ivalue::IValue;
use circuits::ops::Guess;
use circuits::wrappers::U32Wrapper;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::proof::{Proof, ProofConfig};
use circuits_stark_verifier::verify::verify;
use itertools::{Itertools, chain};
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[cfg(test)]
#[path = "verify_test.rs"]
mod verify_test;

#[cfg(test)]
#[path = "backward_compatibility_test.rs"]
mod backward_compatibility_test;

/// A circuit proof together with metadata needed to build the [`CircuitStatement`] against which
/// the proof will be verified. The multiverifier expects that output values of an input circuit
/// consist of `N_RESERVED` QM31 (the unreduced output hash) + 1 QM31 equal to
/// [`circuits::context::U_VALUE`] coming from the constant finalization mechanism (see
/// [`circuits::finalize_constants`]).
pub struct MultiverifierInput<Value: IValue> {
    /// A circuit proof.
    pub proof: Proof<Value>,
    /// The preprocessed root of the circuit associated to `proof`.
    pub preprocessed_root: HashValue<QM31>,
    /// The output values of the circuit (excluding the value of the `u` wire at address
    /// [`circuits::context::U_VAR_IDX`]). The multiverifier only supports verification of circuits
    /// whose output is the unreduced Blake2s digest (`N_RESERVED` words).
    pub output_values: [u32; N_RESERVED],
}

/// Configurations shared by the circuits being verified by the multiverifier and their proofs.
/// These values are static and fixed in advance.
pub struct SharedConfig {
    pub pcs_config: PcsConfig,
    pub proof_config: ProofConfig,
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
}

/// Builds a circuit that verifies two circuit proofs.
///
/// For each [`MultiverifierInput`], the function reconstructs the inner
/// [`CircuitStatement`] from `shared_config` and the per-proof
/// `preprocessed_root` and `outputs`, and runs the STARK verifier.
///
/// After both proofs are verified, each circuit's [`CircuitStatement::circuit_hash`] (the hash of
/// its shared config and preprocessed root) is concatenated with the inner circuit's output values,
/// and both circuits' words are hashed together. The resulting unreduced Blake2s digest is written
/// into the outer circuit's `N_RESERVED` reserved output variables.
/// The circuit is then finalized.
///
/// Both proofs must have been produced with the same [`SharedConfig`].
pub fn build_multiverifier_circuit<Value: IValue>(
    input0: MultiverifierInput<Value>,
    input1: MultiverifierInput<Value>,
    shared_config: &SharedConfig,
) -> FinalizedContext<Value> {
    let mut context = Context::new(N_RESERVED);

    let mut outer_verifier_output_preimage = vec![];
    // Verify sequentially the two proofs.
    for multiverifier_input in [input0, input1] {
        let MultiverifierInput { proof, preprocessed_root, output_values } = multiverifier_input;

        let circuit_config = CircuitConfig {
            config: shared_config.pcs_config,
            n_outputs: N_RESERVED,
            preprocessed_column_log_sizes: shared_config.preprocessed_column_log_sizes.clone(),
            preprocessed_root,
        };
        let output_values = output_values
            .iter()
            .map(|value| U32Wrapper::new_unsafe(Value::pack_u32(*value)).guess(&mut context))
            .collect_vec();
        let output_value_vars = output_values.iter().map(|w| *w.get()).collect_vec();
        let statement = CircuitStatement::new(&mut context, &circuit_config, &output_value_vars);
        let proof_vars = proof.guess(&mut context);

        verify(&mut context, &proof_vars, &shared_config.proof_config, &statement);
        // Append (circuit_hash, outputs) to the preimage of the outer hash.
        outer_verifier_output_preimage
            .extend(chain!(statement.circuit_hash.iter().copied(), output_values));
    }
    // The payload to be hashed is, for each of the two circuits A and B, the eight 32-bit words of
    // its circuit hash followed by its `N_RESERVED` raw output words:
    // [
    //      circuit_hashA (8 words), outputsA (N_RESERVED words),
    //      circuit_hashB (8 words), outputsB (N_RESERVED words),
    // ]
    // where A, B are the two circuits being verified.
    let n_bytes = 4 * outer_verifier_output_preimage.len();
    let output_hash = blake2s_u32s(&mut context, outer_verifier_output_preimage, n_bytes);
    // Copy the unreduced digest words into the reserved variables.
    context.set_outputs(&output_hash.iter().map(|word| *word.get()).collect_vec());

    let context = context.finalize(false);
    #[cfg(test)]
    context.circuit().check_yields();
    context
}
