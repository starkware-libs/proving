use circuits::blake::HashValue;
use circuits::context::TraceContext;
use circuits::ivalue::qm31_from_u32s;
use circuits::ops::Guess;
use rstest::rstest;
use stwo::core::circle::Coset;
use stwo::core::fields::qm31::SecureField;
use stwo::core::fri::fold_coset as stwo_fold_coset;
use stwo::core::poly::line::LineDomain;
use stwo::core::utils::bit_reverse_index;

use super::fri_commit;
use crate::channel::Channel;
use crate::fri::{fold_coset, validate_query_position_in_coset};
use crate::fri_proof::FriCommitProof;

#[test]
fn test_fri_commit_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(375224163, 1270824854, 44060607, 991529112),
        qm31_from_u32s(1068130924, 1630210318, 1632828025, 1983481471),
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let proof = FriCommitProof {
        layer_commitments: vec![
            HashValue::from([
                370372302, 356302922, 2040089875, 232934191, 1279830905, 1240360672, 1788604172,
                465814885,
            ]),
            HashValue::from([
                1558212721, 609186473, 1554074721, 1956195301, 1243917617, 135256448, 1193318416,
                1792104990,
            ]),
            HashValue::from([
                1017503040, 1411053946, 1805475392, 1906875756, 2035075097, 617472393, 571220918,
                1577790110,
            ]),
            HashValue::from([
                1290083578, 670256590, 203247471, 492011214, 353269841, 1619070080, 770215254,
                1663098736,
            ]),
        ],
        last_layer_coefs: vec![qm31_from_u32s(1802004671, 1018373769, 131996621, 1575090881)],
    };
    let proof_vars = proof.guess(&mut context);

    let alphas = fri_commit(&mut context, &mut channel, &proof_vars);
    assert_eq!(alphas.len(), 4);
    assert_eq!(context.get(alphas[0]), qm31_from_u32s(2047550788, 23895068, 1676134944, 263598239));
    assert_eq!(
        context.get(alphas[1]),
        qm31_from_u32s(1988032363, 1739489633, 826507892, 1797301629)
    );
    assert_eq!(
        context.get(alphas[2]),
        qm31_from_u32s(1957504342, 848565442, 1129943791, 1937962621)
    );
    assert_eq!(context.get(alphas[3]), qm31_from_u32s(1748651123, 2133979933, 232524784, 85583628));

    assert_eq!(
        context.get(channel.digest().0),
        qm31_from_u32s(968886948, 725376924, 836084817, 484428276)
    );
    assert_eq!(
        context.get(channel.digest().1),
        qm31_from_u32s(1805658819, 300032261, 172116750, 994058243)
    );

    context.validate_circuit();
}

#[test]
fn test_fold_coset() {
    let mut context = TraceContext::default();
    let coset_log_size = 3_u32;

    let coset_values_qm31: Vec<_> = (0..(1 << coset_log_size))
        .map(|i| qm31_from_u32s(4 * i, 4 * i + 1, 4 * i + 2, 4 * i + 3))
        .collect();
    let coset_values: Vec<_> = coset_values_qm31.iter().map(|x| context.constant(*x)).collect();

    let alpha_qm31 = qm31_from_u32s(98, 76, 54, 32);
    let mut alpha_pow = alpha_qm31;
    let mut alphas = Vec::with_capacity(coset_log_size as usize);
    for _ in 0..coset_log_size {
        alphas.push(context.constant(alpha_pow));
        alpha_pow = alpha_pow * alpha_pow;
    }

    let mut fold_domain = LineDomain::new(Coset::half_odds(coset_log_size));
    let mut twiddles_per_fold = Vec::with_capacity(coset_log_size as usize);
    for i in 0..coset_log_size {
        let twiddles_len = 1 << (coset_log_size - i - 1);
        let twiddles = (0..twiddles_len)
            .map(|k| {
                let j = 2 * k;
                let x = fold_domain.at(bit_reverse_index(j, fold_domain.log_size()));
                context.constant(SecureField::from(x.inverse()))
            })
            .collect::<Vec<_>>();
        twiddles_per_fold.push(twiddles);
        fold_domain = fold_domain.double();
    }

    let actual = fold_coset(&mut context, &coset_values, &twiddles_per_fold, &alphas);
    let expected = stwo_fold_coset(
        coset_values_qm31,
        LineDomain::new(Coset::half_odds(coset_log_size)),
        alpha_qm31,
    );

    assert_eq!(context.get(actual), expected);
    context.validate_circuit();
}

#[rstest]
#[case::success(true)]
#[case::failure(false)]
fn test_validate_query_position_in_coset(#[case] success: bool) {
    let mut context = TraceContext::default();
    let mut const_var = |value: u32| context.constant(qm31_from_u32s(value, 0, 0, 0));

    // 3 queries, layer step = 2 (hence coset size = 2^2).
    let fri_coset_per_query = vec![
        vec![const_var(10), const_var(11), const_var(12), const_var(13)],
        vec![const_var(20), const_var(21), const_var(22), const_var(23)],
        vec![const_var(30), const_var(31), const_var(32), const_var(33)],
    ];

    let bits = vec![
        vec![const_var(0), const_var(1), const_var(0)],
        vec![const_var(0), const_var(0), const_var(1)],
    ];

    let mut fri_data = vec![const_var(10), const_var(21), const_var(32)];
    if !success {
        fri_data[2] = const_var(33);
    }

    validate_query_position_in_coset(&mut context, &fri_coset_per_query, &fri_data, &bits);
    assert_eq!(context.is_circuit_valid(), success);
}
