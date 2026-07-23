use circuit_verifier::circuit_hash::config_words;
use circuits::blake::BLAKE2S_DIGEST_N_WORDS;
use circuits::utils::le_u32s_from_bytes;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

/// Computes the circuit hash. This is the non-circuit version of
/// [`circuit_verifier::circuit_hash::compute_circuit_hash`]. Used by the prover to mix the circuit
/// hash into the channel.
pub fn compute_circuit_hash(
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: Blake2sHash,
) -> Blake2sHash {
    let config_words = config_words(log_blowup_factor, component_log_sizes);
    let root_words: [u32; BLAKE2S_DIGEST_N_WORDS] = le_u32s_from_bytes(preprocessed_root.0);

    let mut hasher = Blake2sHasher::new();
    for word in config_words.into_iter().chain(root_words) {
        hasher.update(&word.to_le_bytes());
    }
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use circuit_verifier::circuit_components::COMPONENT_NAMES;
    use circuit_verifier::circuit_hash::compute_circuit_hash as compute_circuit_hash_in_circuit;
    use circuits::blake::HashValue;
    use circuits::context::TraceContext;
    use circuits::ivalue::IValue;
    use circuits::ops::Guess;
    use stwo::core::fields::qm31::QM31;

    use super::*;

    /// The host `compute_circuit_hash` (mixed into the channel by the prover) must produce the
    /// same digest as the in-circuit `compute_circuit_hash` (recomputed by the verifier circuit),
    /// over the same config. A divergence would silently break the Fiat-Shamir transcript.
    #[test]
    fn host_matches_in_circuit() {
        let component_log_sizes: OrderedHashMap<&'static str, u32> =
            COMPONENT_NAMES.iter().enumerate().map(|(i, name)| (*name, (i % 20) as u32)).collect();
        let log_blowup_factor = 3;
        let preprocessed_root = Blake2sHash(std::array::from_fn(|i| i as u8));

        // Host version, unpacked into eight little-endian u32 words.
        let host: [u32; BLAKE2S_DIGEST_N_WORDS] = le_u32s_from_bytes(
            compute_circuit_hash(&component_log_sizes, log_blowup_factor, preprocessed_root).0,
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
}
