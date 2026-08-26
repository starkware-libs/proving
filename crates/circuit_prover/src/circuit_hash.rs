use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::circuit_components::PerComponent;
use circuit_verifier::circuit_hash::config_words;
use circuit_verifier::statement::{all_circuit_components, circuit_component_log_sizes};
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::Hasher;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;

/// Computes the circuit hash: `H(log_blowup_factor || component_log_sizes || preprocessed_root)`,
/// where `H` is the hash function the circuit is proven with, so that the circuit's identity is
/// committed to under the same hash function as the rest of the transcript.
///
/// This is the non-circuit version of [`circuit_verifier::circuit_hash::compute_circuit_hash`],
/// used by the prover to mix the circuit hash into the channel.
pub fn compute_circuit_hash<H: Hasher>(
    component_log_sizes: &PerComponent<u32>,
    log_blowup_factor: u32,
    preprocessed_root: H::Hash,
) -> H::Hash {
    let config_words = config_words(log_blowup_factor, component_log_sizes);

    H::hash_u32s_followed_by_digest(&config_words, preprocessed_root)
}

/// The circuit hash identifying `preprocessed_circuit` when proven with `log_blowup_factor`.
///
/// Currently hardcoded to use Blake2s.
pub fn preprocessed_circuit_hash(
    preprocessed_circuit: &PreprocessedCircuit,
    log_blowup_factor: u32,
) -> Blake2sHash {
    let component_log_sizes = circuit_component_log_sizes(
        &all_circuit_components::<QM31>(),
        &preprocessed_circuit.preprocessed_trace.log_sizes(),
    );
    compute_circuit_hash::<Blake2sMerkleHasher>(
        &component_log_sizes,
        log_blowup_factor,
        preprocessed_circuit.preprocessed_root(log_blowup_factor),
    )
}

#[cfg(test)]
mod tests {
    use circuit_verifier::circuit_hash::compute_circuit_hash as compute_circuit_hash_in_circuit;
    use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue};
    use circuits::context::TraceContext;
    use circuits::ivalue::IValue;
    use circuits::ops::Guess;
    use circuits::utils::le_u32s_from_bytes;
    use expect_test::expect;
    use stwo::core::fields::qm31::QM31;

    use super::*;

    /// The host `compute_circuit_hash` (mixed into the channel by the prover) must produce the
    /// same digest as the in-circuit `compute_circuit_hash` (recomputed by the verifier circuit),
    /// over the same config. A divergence would silently break the Fiat-Shamir transcript.
    #[test]
    fn host_matches_in_circuit() {
        let component_log_sizes = PerComponent {
            eq: 0,
            qm31_ops: 1,
            triple_xor: 2,
            m_31_to_u_32: 3,
            blake_g_gate: 4,
            verify_bitwise_xor_8: 5,
            verify_bitwise_xor_12: 6,
            verify_bitwise_xor_4: 7,
            verify_bitwise_xor_7: 8,
            verify_bitwise_xor_9: 9,
            range_check_16: 10,
        };
        let log_blowup_factor = 3;
        let preprocessed_root = Blake2sHash(std::array::from_fn(|i| i as u8));

        // Host version, unpacked into eight little-endian u32 words.
        let host: [u32; BLAKE2S_DIGEST_N_WORDS] = le_u32s_from_bytes(
            compute_circuit_hash::<Blake2sMerkleHasher>(
                &component_log_sizes,
                log_blowup_factor,
                preprocessed_root,
            )
            .into(),
        );

        // In-circuit version: build in a fresh context and read back the output words.
        let mut context = TraceContext::default();
        let root = HashValue::<QM31>::from(preprocessed_root).guess(&mut context);
        let hash = compute_circuit_hash_in_circuit(
            &mut context,
            &component_log_sizes,
            log_blowup_factor,
            &root,
        );
        let in_circuit: [u32; BLAKE2S_DIGEST_N_WORDS] =
            std::array::from_fn(|i| context.get(*hash[i].get()).unpack_u32());

        assert_eq!(host, in_circuit);
    }

    #[test]
    fn poseidon252_root() {
        use starknet_ff::FieldElement as FieldElement252;
        use stwo::core::vcs_lifted::poseidon252_merkle::Poseidon252MerkleHasher;

        let component_log_sizes = PerComponent {
            eq: 20,
            qm31_ops: 23,
            triple_xor: 20,
            m_31_to_u_32: 21,
            blake_g_gate: 23,
            verify_bitwise_xor_8: 16,
            verify_bitwise_xor_12: 20,
            verify_bitwise_xor_4: 8,
            verify_bitwise_xor_7: 14,
            verify_bitwise_xor_9: 18,
            range_check_16: 16,
        };
        let hash = compute_circuit_hash::<Poseidon252MerkleHasher>(
            &component_log_sizes,
            1,
            FieldElement252::from_hex_be(
                "0x0172b6763ef45133e1d1d1a507f14fe24702c221cdbbc7ca7c0e5d654b008d27",
            )
            .unwrap(),
        );

        expect!["1e153175973f9466ee2f662e1f782c10a3d0b6bfa92d64c2b78c53fd35195ca"]
            .assert_eq(&format!("{hash:x}"));
    }
}
