use core::box::BoxImpl;
use crate::channel::blake2s::{Blake2sChannel, ChannelTrait, check_leading_zeros};
use crate::fields::qm31::qm31_const;
use crate::vcs::blake2s_hasher::Blake2sHash;

#[test]
fn test_mix_felts_with_1_felt() {
    let mut channel: Blake2sChannel = Default::default();

    channel.mix_felts([qm31_const::<1, 2, 3, 4>()].span());

    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(
        channel.digest.hash.unbox(),
        [
            1586304710, 1167332849, 1688630032, 429142330, 4001363212, 2013799503, 180553907,
            2044853257,
        ],
    );
}

#[test]
fn test_mix_felts_with_2_felts() {
    let mut channel: Blake2sChannel = Default::default();

    channel.mix_felts([qm31_const::<1, 2, 3, 4>(), qm31_const::<5, 6, 7, 8>()].span());

    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(
        channel.digest.hash.unbox(),
        [
            1835698174, 2969628929, 1758616107, 158303712, 3820231193, 179192886, 4063347398,
            3332297509,
        ],
    );
}

#[test]
fn test_mix_felts_with_3_felts() {
    let mut channel: Blake2sChannel = Default::default();

    channel
        .mix_felts(
            [qm31_const::<1, 2, 3, 4>(), qm31_const::<5, 6, 7, 8>(), qm31_const::<9, 10, 11, 12>()]
                .span(),
        );

    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(
        channel.digest.hash.unbox(),
        [
            2116479765, 3227507660, 1737697798, 2518684651, 1068812914, 1858078313, 1722202885,
            2198022752,
        ],
    );
}

#[test]
fn test_mix_felts_with_4_felts() {
    let mut channel: Blake2sChannel = Default::default();

    channel
        .mix_felts(
            [
                qm31_const::<1, 2, 3, 4>(), qm31_const::<5, 6, 7, 8>(),
                qm31_const::<9, 10, 11, 12>(), qm31_const::<13, 14, 15, 16>(),
            ]
                .span(),
        );

    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(
        channel.digest.hash.unbox(),
        [
            940149128, 1354728945, 2816315586, 1690943110, 210254904, 3746481728, 1339132640,
            3760408575,
        ],
    );
}

#[test]
fn test_mix_felts_with_5_felts() {
    let mut channel: Blake2sChannel = Default::default();

    channel
        .mix_felts(
            [
                qm31_const::<1, 2, 3, 4>(), qm31_const::<5, 6, 7, 8>(),
                qm31_const::<9, 10, 11, 12>(), qm31_const::<13, 14, 15, 16>(),
                qm31_const::<17, 18, 19, 20>(),
            ]
                .span(),
        );

    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(
        channel.digest.hash.unbox(),
        [
            3425911356, 1462327982, 3241135902, 4212900065, 3145879221, 3413011910, 3946733048,
            4081152200,
        ],
    );
}

/// Reference values were generated from the Rust `Blake2sChannel::mix_u32s`
/// (https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs)
/// with input `[1, 2, ..., n]` on a default channel.
fn assert_mix_u32s(words: Span<u32>, expected: [u32; 8]) {
    let mut channel: Blake2sChannel = Default::default();
    channel.mix_u32s(words);
    assert_eq!(channel.digest.hash.unbox(), expected);
}

#[test]
fn test_mix_u32s_len_0() {
    assert_mix_u32s(
        array![].span(),
        [
            2841512754, 3258672542, 1104909237, 1309331760, 1275804413, 1738990018, 1905092395,
            2905849311,
        ],
    );
}

#[test]
fn test_mix_u32s_len_1() {
    assert_mix_u32s(
        array![1].span(),
        [
            19235923, 1604041799, 1672672341, 2742337121, 3871249661, 1300042830, 4050860166,
            769478617,
        ],
    );
}

#[test]
fn test_mix_u32s_len_7() {
    assert_mix_u32s(
        array![1, 2, 3, 4, 5, 6, 7].span(),
        [
            1952582910, 3463395335, 4218083501, 3678229254, 4010827797, 227638702, 2211456986,
            2204729745,
        ],
    );
}

#[test]
fn test_mix_u32s_len_8() {
    // First-block-exactly-filled boundary: `half` (digest) + 8 words → single finalize.
    assert_mix_u32s(
        array![1, 2, 3, 4, 5, 6, 7, 8].span(),
        [
            1835698174, 2969628929, 1758616107, 158303712, 3820231193, 179192886, 4063347398,
            3332297509,
        ],
    );
}

#[test]
fn test_mix_u32s_len_9() {
    // First compress + 1-word tail combined with fresh `half` after compress.
    assert_mix_u32s(
        array![1, 2, 3, 4, 5, 6, 7, 8, 9].span(),
        [
            2205585776, 3004939095, 3067768628, 4208416691, 3824924742, 608382508, 2495807392,
            3349281227,
        ],
    );
}

#[test]
fn test_mix_u32s_len_15() {
    assert_mix_u32s(
        array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].span(),
        [
            2753343649, 3381002282, 3322866444, 1593947330, 1863381790, 282458799, 1460609626,
            137829626,
        ],
    );
}

#[test]
fn test_mix_u32s_len_16() {
    // Compress block 0, then finalize `[new_half, zeros]` via the "None" branch after compress.
    assert_mix_u32s(
        array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16].span(),
        [
            940149128, 1354728945, 2816315586, 1690943110, 210254904, 3746481728, 1339132640,
            3760408575,
        ],
    );
}

#[test]
fn test_mix_u32s_len_17() {
    assert_mix_u32s(
        array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17].span(),
        [
            2376835750, 451513028, 2400139659, 1427462776, 2379020428, 895384260, 3462557408,
            298945784,
        ],
    );
}

#[test]
fn test_mix_u32s_len_23() {
    assert_mix_u32s(
        array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
            .span(),
        [
            1985876374, 4211915189, 1616506157, 2006328026, 2371588099, 853383987, 3828210318,
            1464037221,
        ],
    );
}

#[test]
fn test_mix_u32s_len_24() {
    // Two full blocks: compress block 0, finalize block 1 exactly.
    assert_mix_u32s(
        array![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ]
            .span(),
        [
            520731041, 1083958258, 1138152617, 3056749115, 586997436, 3101777590, 2358708943,
            863043355,
        ],
    );
}

#[test]
fn test_mix_u32s_len_25() {
    // Two full compresses, then 1-word tail forms partial final block on its own.
    assert_mix_u32s(
        array![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25,
        ]
            .span(),
        [
            2909607053, 1561944424, 1334308686, 2526443189, 613570545, 3119021913, 2423688026,
            2836315110,
        ],
    );
}

#[test]
fn test_mix_u64() {
    let mut channel: Blake2sChannel = Default::default();

    channel.mix_u64(0x1111222233334444);

    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(
        channel.digest.hash.unbox(),
        [
            0xc13f9ebc, 0x97884ed2, 0x59336d95, 0x24977332, 0xcdca6b9d, 0x74924d22, 0x4abae704,
            0xce6edc77,
        ],
    );
}

#[test]
fn test_check_proof_of_work() {
    let digest = Blake2sHash { hash: BoxImpl::new([0b1000, 0, 0, 0, 0, 0, 0, 0]) };

    let res = check_leading_zeros(digest, 3);

    assert!(res);
}

#[test]
fn test_check_proof_of_work_with_invalid_n_bits() {
    let digest = Blake2sHash { hash: BoxImpl::new([0b1000, 0, 0, 0, 0, 0, 0, 0]) };

    let res = check_leading_zeros(digest, 4);

    assert!(!res);
}

#[test]
fn test_blake_u32s() {
    let mut channel: Blake2sChannel = Default::default();

    let result = channel.draw_u32s();
    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(
        result,
        array![
            1508103417, 49928118, 1851109195, 649450964, 1514800545, 4236765031, 523819246,
            4066564620,
        ]
            .span(),
    );
}

#[test]
fn test_draw_secure_felt() {
    let mut channel: Blake2sChannel = Default::default();

    let felt = channel.draw_secure_felt();

    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(felt, qm31_const::<1508103417, 49928118, 1851109195, 649450964>());
}

#[test]
fn test_draw_secure_felts() {
    let mut channel: Blake2sChannel = Default::default();

    let felts = channel.draw_secure_felts(8);

    // Tested against values produced from Rust code.
    // https://github.com/starkware-libs/stwo/blob/dev/crates/prover/src/core/channel/blake2s.rs
    assert_eq!(
        felts,
        array![
            qm31_const::<1508103417, 49928118, 1851109195, 649450964>(),
            qm31_const::<1514800545, 2089281384, 523819246, 1919080973>(),
            qm31_const::<1769619091, 1335149496, 2007506569, 1426464368>(),
            qm31_const::<853727757, 1673676888, 635879929, 1327640380>(),
            qm31_const::<1751831125, 1559795173, 442209472, 1280396692>(),
            qm31_const::<1528893536, 644814910, 503157674, 286565543>(),
            qm31_const::<1839261536, 1778992654, 807858428, 143171319>(),
            qm31_const::<128590986, 1618375851, 213276683, 76892197>(),
        ],
    );
}
