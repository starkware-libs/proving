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

/// Verifies a 1-1 (privacy) circuit proof.
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
