use circuits::context::TraceContext;
use circuits::ops::Guess;

use super::*;
use crate::circuit_components::PerComponent;

/// Golden test for computing the circuit hash.
#[test]
fn compute_circuit_hash_matches_golden() {
    let component_log_sizes: OrderedHashMap<&'static str, u32> = PerComponent {
        eq: 17,
        qm31_ops: 21,
        triple_xor: 17,
        m_31_to_u_32: 18,
        blake_g_gate: 20,
        verify_bitwise_xor_8: 16,
        verify_bitwise_xor_12: 20,
        verify_bitwise_xor_4: 8,
        verify_bitwise_xor_7: 14,
        verify_bitwise_xor_9: 18,
        range_check_16: 16,
    }
    .into_named_iter()
    .collect();

    let log_blowup_factor = 3;
    let preprocessed_root: [u32; BLAKE2S_DIGEST_N_WORDS] = std::array::from_fn(|i| i as u32);

    // Compute the circuit hash in a fresh context and read back the eight concrete output words.
    let mut context = TraceContext::default();
    let root = HashValue::<QM31>::from(preprocessed_root).guess(&mut context);
    let hash = compute_circuit_hash(&mut context, &component_log_sizes, log_blowup_factor, &root);
    let in_circuit: [u32; BLAKE2S_DIGEST_N_WORDS] =
        std::array::from_fn(|i| context.get(*hash[i].get()).unpack_u32());

    let expected: [u32; BLAKE2S_DIGEST_N_WORDS] = [
        0xa8810641, 0x52391285, 0x90b37fd2, 0x905b887a, 0x7db7dc81, 0xa7c3a731, 0xd0d46b34,
        0x8fa6a471,
    ];

    assert_eq!(in_circuit, expected);
}
