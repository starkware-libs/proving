use blake2::{Blake2s256, Digest};
use expect_test::expect;
use itertools::zip_eq;
use rstest::rstest;
use stwo::core::vcs::blake2_hash::Blake2sHash;

use crate::blake::{HashValue, blake2s, blake2s_u32s, unpack_qm31s_to_u32_words};
use crate::context::TraceContext;
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::{Guess, eq, guess};
use crate::stats::Stats;
use crate::test_utils::finalize_guessed_vars;
use crate::utils::bytes_from_le_u32s;

#[rstest]
#[case::success(false)]
#[case::wrong_output(true)]
fn test_blake(#[case] wrong_output: bool) {
    let mut context = TraceContext::default();

    let input = [
        qm31_from_u32s(1, 2, 3, 4),
        qm31_from_u32s(5, 6, 7, 8),
        qm31_from_u32s(9, 10, 11, 12),
        qm31_from_u32s(13, 14, 15, 16),
        qm31_from_u32s(17, 0, 0, 0),
    ]
    .guess(&mut context);

    let mut hasher = Blake2s256::new();
    hasher.update([
        1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0,
        0, 0, 9, 0, 0, 0, 10, 0, 0, 0, 11, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 15, 0,
        0, 0, 16, 0, 0, 0, 17, 0,
    ]);
    let mut expected_hash: [u8; 32] = hasher.finalize().into();
    if wrong_output {
        expected_hash[0] += 1;
    }

    let input_u32s = unpack_qm31s_to_u32_words(&mut context, input);

    let output = blake2s_u32s(&mut context, input_u32s, 66);
    for i in 0..8 {
        use stwo::core::fields::qm31::QM31;

        let out_i = guess(
            &mut context,
            *QM31::pack_u32(u32::from_le_bytes(
                expected_hash[4 * i..4 * i + 4].try_into().unwrap(),
            ))
            .get(),
        );
        eq(&mut context, *output.0[i].get(), out_i);
    }

    assert_eq!(
        context.stats,
        Stats {
            equals: 8,
            add: 0,
            sub: 0,
            mul: 15,
            inv: 0,
            div: 0,
            pointwise_mul: 20,
            guess: 13,
            blake_updates: 2,
            permutation_inputs: 0,
            // The context constructor marks `u` as an output.
            outputs: 1,
            triple_xor: 16,
            m31_to_u32: 20
        }
    );

    let context = context.finalize(false);
    context.circuit().check_yields();

    assert_eq!(context.is_circuit_valid(), !wrong_output);
}

#[test]
fn test_blake2s() {
    let mut context = TraceContext::default();

    let message: [u32; 16] = [
        930933030, 1766240503, 3660871006, 388409270, 1948594622, 3119396969, 3924579183,
        2089920034, 3857888532, 929304360, 1810891574, 860971754, 1822893775, 2008495810,
        2958962335, 2340515744,
    ];
    let n_bytes = 64;

    let input_values = [
        qm31_from_u32s(message[0], message[1], message[2], message[3]),
        qm31_from_u32s(message[4], message[5], message[6], message[7]),
        qm31_from_u32s(message[8], message[9], message[10], message[11]),
        qm31_from_u32s(message[12], message[13], message[14], message[15]),
    ];

    let expected = IValue::blake2s(&input_values, n_bytes);

    let input = input_values.guess(&mut context);
    let output = blake2s(&mut context, &input, n_bytes);

    for (out_word, exp_word) in zip_eq(output.iter(), expected.iter()) {
        let exp = guess(&mut context, *exp_word.raw());
        eq(&mut context, *out_word.raw(), exp);
    }

    let context = context.finalize(false);
    context.circuit().check_yields();
    assert!(context.is_circuit_valid());
}

#[test]
fn test_hash_value_guess() {
    let mut context = TraceContext::default();

    // Eight raw Blake2s output words, chosen so each exercises both 16-bit limbs (including the
    // boundary values `0x0000` and `0xFFFF` in either limb).
    let words: [u32; 8] = [
        0x0000_0001,
        0x1234_5678,
        0xFFFF_FFFF,
        0xDEAD_BEEF,
        0x0000_FFFF,
        0xFFFF_0000,
        0xCAFE_BABE,
        0x8000_0001,
    ];
    let hash_value = HashValue::from(Blake2sHash(bytes_from_le_u32s(words)));
    let guessed = hash_value.guess(&mut context);

    // Each guessed word must round-trip back to the original raw u32 in full.
    for (i, &word) in words.iter().enumerate() {
        assert_eq!(context.get(*guessed.0[i].raw()).unpack_u32(), word);
    }

    finalize_guessed_vars(&mut context);

    expect![[r#"
        [7] = [3] + [6]
        [11] = [8] + [10]
        [15] = [12] + [14]
        [19] = [16] + [18]
        [23] = [20] + [22]
        [27] = [24] + [26]
        [31] = [28] + [30]
        [35] = [32] + [34]
        [6] = [4] * [5]
        [10] = [9] * [5]
        [14] = [13] * [5]
        [18] = [17] * [5]
        [22] = [21] * [5]
        [26] = [25] * [5]
        [30] = [29] * [5]
        [34] = [33] * [5]
        [3] = m31_to_u32([3])
        [4] = m31_to_u32([4])
        [8] = m31_to_u32([8])
        [9] = m31_to_u32([9])
        [12] = m31_to_u32([12])
        [13] = m31_to_u32([13])
        [16] = m31_to_u32([16])
        [17] = m31_to_u32([17])
        [20] = m31_to_u32([20])
        [21] = m31_to_u32([21])
        [24] = m31_to_u32([24])
        [25] = m31_to_u32([25])
        [28] = m31_to_u32([28])
        [29] = m31_to_u32([29])
        [32] = m31_to_u32([32])
        [33] = m31_to_u32([33])
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}
