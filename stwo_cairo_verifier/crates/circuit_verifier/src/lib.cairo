use stwo_circuit_air::multiverifier_consts::CIRCUIT_FRI_PARAMS;
use stwo_circuit_air::{CircuitProof, compute_circuit_hash, get_verification_output, verify_circuit};
use stwo_verifier_core::Hash;

#[executable]
fn main(proof: CircuitProof) -> Hash {
    // Take the commitments and the span of `output_values` here, because `verify_circuit`
    // consumes the proof later.
    let commitments: @Box<[Hash; 4]> = proof
        .stark_proof
        .commitment_scheme_proof
        .commitments
        .try_into()
        .unwrap();
    let output_values = proof.claim.output_values.span();

    // Compute the circuit hash. The blowup factor is the circuit's hardcoded one, the same one
    // `verify_circuit` verifies against, and not a value the proof carries.
    let [preprocessed_commitment, _, _, _] = commitments.unbox();
    let circuit_hash = compute_circuit_hash(
        CIRCUIT_FRI_PARAMS.log_blowup_factor, preprocessed_commitment,
    );

    // Verify the circuit proof; panics on an invalid proof.
    verify_circuit(:proof, :circuit_hash);

    // Return the verification output.
    get_verification_output(:circuit_hash, :output_values)
}
