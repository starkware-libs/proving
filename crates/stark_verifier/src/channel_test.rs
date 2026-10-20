use circuits::blake::{HashValue, unpack_qm31s_to_u32_words};
use circuits::context::TraceContext;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::Guess;
use circuits::stats::Stats;
use rstest::rstest;
use stwo::core::channel::{Blake2sChannel, Channel as StwoChannel};
use stwo::core::fields::qm31::QM31;

use super::Channel;

#[test]
fn test_mix_commitment_regression() {
    let mut context = TraceContext::default();

    let mut channel = Channel::new(&mut context);
    let root0 = HashValue::from([
        637418335, 1672023491, 980858689, 607764934, 386900718, 430556311, 1187803054, 669301442,
    ])
    .guess(&mut context);
    let root1 = HashValue::from([
        1477561267, 1244239078, 1979857528, 1316512771, 490980261, 2016799283, 79573118, 1350641448,
    ])
    .guess(&mut context);
    channel.mix_commitment(&mut context, &root0);
    let digest0 = channel.digest();
    channel.mix_commitment(&mut context, &root1);
    let digest1 = channel.digest();

    let expected_digest0 = HashValue::from([
        3816148463, 1251290000, 263177925, 2870147445, 484105836, 2288081674, 2827170385,
        4132878725,
    ]);
    for i in 0..8 {
        assert_eq!(context.get(*digest0[i].get()), *expected_digest0[i].get());
    }

    let expected_digest1 = HashValue::from([
        2735032799, 2943602129, 3853563773, 1324500876, 876237970, 3035088689, 2935972668,
        3936037936,
    ]);
    for i in 0..8 {
        assert_eq!(context.get(*digest1[i].get()), *expected_digest1[i].get());
    }

    assert_eq!(
        context.stats,
        Stats {
            add: 16,
            mul: 16,
            guess: 32,
            blake_updates: 2,
            triple_xor: 16,
            // The context constructor marks `u` as an output.
            outputs: 1,
            ..Stats::default()
        }
    );

    context.validate_circuit();
}

#[test]
fn test_mix_qm31s_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        266526289, 1341429509, 1126614795, 1001621831, 1024638884, 1857778419, 1763024470,
        1859929979,
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let felts = [
        context.new_var(qm31_from_u32s(1, 0, 0, 0)),
        context.new_var(qm31_from_u32s(485399786, 1255952693, 1939438763, 1561715227)),
        context.new_var(qm31_from_u32s(1757357815, 8864493, 674769946, 1715431414)),
        context.new_var(qm31_from_u32s(1148846901, 1519172202, 357767101, 2129853554)),
        context.new_var(qm31_from_u32s(0, 0, 0, 0)),
        context.new_var(qm31_from_u32s(0, 0, 0, 0)),
        context.new_var(qm31_from_u32s(0, 0, 0, 0)),
    ];
    channel.mix_qm31s(&mut context, felts);

    let expected_digest = HashValue::from([
        1186703962, 1584594219, 633548839, 1510969779, 1524867388, 1224019906, 1564199416,
        388718964,
    ]);
    let digest = channel.digest();
    for i in 0..8 {
        assert_eq!(context.get(*digest[i].get()), *expected_digest[i].get());
    }
    assert_eq!(channel.n_draws, 0);

    context.validate_circuit();
}

#[test]
fn test_mix_u32s_matches_stwo() {
    // Includes full 32-bit words (>= 2^31) to exercise the path that bypasses `m31_to_u32`.
    let data: [u32; 5] = [1, 0x8000_0000, 0xFFFF_FFFF, 2, 3];

    let mut context = TraceContext::default();
    // A fresh circuit channel starts from a zero digest, matching stwo's default channel.
    let mut channel = Channel::new(&mut context);
    let mut stwo_channel = Blake2sChannel::default();

    // Mixing twice also exercises the non-zero-digest path on the second call.
    for _ in 0..2 {
        let words = data.map(|word| QM31::pack_u32(word).guess(&mut context));
        channel.mix_u32s(&mut context, words.into_iter());
        stwo_channel.mix_u32s(&data);

        let expected = HashValue::from(stwo_channel.digest());
        let digest = channel.digest();
        for i in 0..8 {
            assert_eq!(context.get(*digest[i].get()), *expected[i].get());
        }
    }
    assert_eq!(channel.n_draws, 0);

    context.validate_circuit();
}

#[test]
fn test_draw_qm31_regression() {
    let mut context = TraceContext::default();

    let init_digest =
        [800533588, 1994201536, 2099095392, 678020158, 1950435309, 1607451911, 2421030, 565867237];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let res = channel.draw_qm31(&mut context);
    assert_eq!(context.get(res), qm31_from_u32s(576777365, 1893273620, 1311820968, 1368123472));

    let res2 = channel.draw_qm31(&mut context);
    assert_eq!(context.get(res2), qm31_from_u32s(511638860, 1228809056, 792274960, 1379022579));

    context.validate_circuit();
}

#[test]
fn test_draw_two_qm31s_regression() {
    let mut context = TraceContext::default();

    let init_digest =
        [800533588, 1994201536, 2099095392, 678020158, 1950435309, 1607451911, 2421030, 565867237];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let res = [channel.draw_qm31(&mut context), channel.draw_qm31(&mut context)];
    assert_eq!(context.get(res[0]), qm31_from_u32s(576777365, 1893273620, 1311820968, 1368123472));
    assert_eq!(context.get(res[1]), qm31_from_u32s(511638860, 1228809056, 792274960, 1379022579));

    let res2 = [channel.draw_qm31(&mut context), channel.draw_qm31(&mut context)];
    assert_eq!(
        context.get(res2[0]),
        qm31_from_u32s(1447209722, 1052793597, 1641866048, 2014085447)
    );
    assert_eq!(
        context.get(res2[1]),
        qm31_from_u32s(2009705394, 2131110291, 1555778018, 2088343029)
    );

    context.validate_circuit();
}

#[test]
fn test_draw_point_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        2072130922, 1322677507, 1508142866, 1010842681, 967226388, 1861793490, 1980108433,
        243066861,
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let pt = channel.draw_point(&mut context);

    assert_eq!(context.get(pt.x), qm31_from_u32s(411390069, 896574898, 574994754, 1474200604));
    assert_eq!(context.get(pt.y), qm31_from_u32s(535573151, 1636024854, 138098982, 28192374));

    context.validate_circuit();
}

#[rstest]
#[case::success(10, 484, true)]
#[case::wrong_n_bits0(11, 484, false)]
#[case::wrong_n_bits1(9, 484, false)]
#[case::wrong_nonce0(10, 483, false)]
#[case::wrong_nonce1(10, 485, false)]
fn test_pow_regression(#[case] n_bits: u32, #[case] nonce: u32, #[case] success: bool) {
    let mut context = TraceContext::default();

    let init_digest =
        [968886948, 725376924, 836084817, 484428276, 1805658819, 300032261, 172116750, 994058243];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let nonce = context.new_var(qm31_from_u32s(nonce, 0, 0, 0));
    channel.pow(&mut context, n_bits, nonce);

    assert_eq!(context.is_circuit_valid(), success);

    if success {
        let digest = channel.digest();
        let expected_digest = HashValue::from([
            504080137, 1083012672, 3095019893, 4068872123, 1307383598, 2664898104, 4196059202,
            2257392334,
        ]);
        for i in 0..8 {
            assert_eq!(context.get(*digest[i].get()), *expected_digest[i].get());
        }
    }
}

/// Verifies that `unpack_qm31s_to_u32_words` followed by `channel.mix_u32s` produces the same
/// channel state as stwo's `Blake2sChannel::mix_felts`.
#[test]
fn test_unpack_then_mix_u32s_matches_mix_felts() {
    use stwo::core::fields::m31::M31;
    use stwo::core::fields::qm31::QM31 as StwoQM31;

    // A few non-trivial QM31 values with all four M31 coordinates nonzero.
    let felts: Vec<StwoQM31> = vec![
        StwoQM31::from_m31_array([M31(42), M31(1337), M31(999999), M31(2147483646)]),
        StwoQM31::from_m31_array([M31(1), M31(0), M31(0), M31(0)]),
        StwoQM31::from_m31_array([
            M31(485399786),
            M31(1255952693),
            M31(1939438763),
            M31(1561715227),
        ]),
    ];

    let mut context = TraceContext::default();
    // Both channels start from a zero digest.
    let mut channel = Channel::new(&mut context);
    let mut stwo_channel = Blake2sChannel::default();

    // Stwo prover-side: mix the felts directly.
    stwo_channel.mix_felts(&felts);

    // Circuit-side: convert QM31s to u32 words, then mix via mix_u32s.
    let circuit_felts: Vec<_> = felts
        .iter()
        .map(|qm31| {
            let [a, b, c, d] = qm31.to_m31_array();
            context.new_var(qm31_from_u32s(a.0, b.0, c.0, d.0))
        })
        .collect();
    let words = unpack_qm31s_to_u32_words(&mut context, circuit_felts);
    channel.mix_u32s(&mut context, words.into_iter());

    let expected = HashValue::from(stwo_channel.digest());
    for i in 0..8 {
        assert_eq!(context.get(*channel.digest()[i].get()), *expected[i].get());
    }

    // Also verify a second round with a non-zero starting digest.
    let felts2: Vec<StwoQM31> = vec![StwoQM31::from_m31_array([
        M31(1757357815),
        M31(8864493),
        M31(674769946),
        M31(1715431414),
    ])];

    stwo_channel.mix_felts(&felts2);

    let circuit_felts2: Vec<_> = felts2
        .iter()
        .map(|qm31| {
            let [a, b, c, d] = qm31.to_m31_array();
            context.new_var(qm31_from_u32s(a.0, b.0, c.0, d.0))
        })
        .collect();
    let words2 = unpack_qm31s_to_u32_words(&mut context, circuit_felts2);
    channel.mix_u32s(&mut context, words2.into_iter());

    let expected2 = HashValue::from(stwo_channel.digest());
    for i in 0..8 {
        assert_eq!(context.get(*channel.digest()[i].get()), *expected2[i].get());
    }

    context.validate_circuit();
}
