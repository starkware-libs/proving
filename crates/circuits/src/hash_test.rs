use expect_test::expect;
use stwo::core::vcs::blake2_hash::Blake2sHash;

use crate::context::TraceContext;
use crate::hash::HashValue;
use crate::ivalue::IValue;
use crate::ops::Guess;
use crate::test_utils::finalize_guessed_vars;
use crate::utils::bytes_from_le_u32s;

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
