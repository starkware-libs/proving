use circuits::context::{Context, TraceContext};
use circuits::ivalue::{NoValue, qm31_from_u32s};
use circuits::ops::Guess;
use circuits::test_utils::{finalize_guessed_vars, simd_from_u32s};
use circuits::wrappers::M31Wrapper;
use expect_test::expect;
use stwo::core::circle::CirclePoint;
use stwo::core::fields::m31::M31;

use crate::oods::{
    EvalDomainSamples, OodsResponse, compute_fri_input, empty_eval_domain_samples,
    extract_expected_composition_eval,
};
use crate::select_queries::select_queries;

#[test]
fn test_eval_domain_samples_guess_circuit() {
    let mut context = Context::<NoValue>::default();
    let res = empty_eval_domain_samples(&[2], 1).guess(&mut context);
    expect!["EvalDomainSamples { data: [[[M31([3])], [M31([4])]]] }"]
        .assert_eq(&format!("{res:?}"));
    finalize_guessed_vars(&mut context);
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
            (0 + 0i) + (1 + 0i)u: [2],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [3] = [3] x [1]
        [4] = [4] x [1]
        output [2]

    "#]]
    .assert_debug_eq(&context.circuit);
}

#[test]
fn extract_expected_composition_eval_regression() {
    let mut context = TraceContext::default();

    let composition_eval_at_oods = [
        qm31_from_u32s(1508389461, 2095170364, 1242839621, 121914987),
        qm31_from_u32s(2074471118, 525791636, 1741315353, 560542608),
        qm31_from_u32s(1544603421, 1313779258, 1591174380, 2142352248),
        qm31_from_u32s(376285896, 1645064251, 1972412846, 145104793),
        qm31_from_u32s(425315367, 0, 0, 0),
        qm31_from_u32s(1670393541, 0, 0, 0),
        qm31_from_u32s(833801100, 0, 0, 0),
        qm31_from_u32s(374213131, 0, 0, 0),
    ]
    .guess(&mut context);

    let oods_point = CirclePoint {
        x: qm31_from_u32s(1343313724, 1951183646, 1685075959, 888698585).guess(&mut context),
        y: qm31_from_u32s(674655034, 1516640953, 569857337, 1549701521).guess(&mut context),
    };

    let log_evaluation_domain_size = 5;
    let expected_composition_eval = extract_expected_composition_eval(
        &mut context,
        &composition_eval_at_oods,
        oods_point,
        log_evaluation_domain_size,
    );
    assert_eq!(
        context.get(expected_composition_eval),
        qm31_from_u32s(443798542, 633915785, 595028408, 165661052)
    );

    context.validate_circuit();
}

#[test]
fn test_compute_fri_input_regression() {
    let pt = CirclePoint {
        x: qm31_from_u32s(1977529453, 822446523, 1665855107, 812402677),
        y: qm31_from_u32s(310991198, 1931472985, 1200685911, 1588389778),
    };

    let mut context = TraceContext::default();
    let pt_var = pt.guess(&mut context);

    let oods_responses = vec![
        OodsResponse {
            trace_idx: 0,
            column_idx: 0,
            pt: pt_var,
            value: context.new_var(qm31_from_u32s(1, 0, 0, 0)),
        },
        OodsResponse {
            trace_idx: 0,
            column_idx: 1,
            pt: pt_var,
            value: context.new_var(qm31_from_u32s(2065172982, 64209128, 2018861108, 1995226139)),
        },
        OodsResponse {
            trace_idx: 0,
            column_idx: 2,
            pt: pt_var,
            value: context.new_var(qm31_from_u32s(2038440027, 1469156040, 504751706, 1024643555)),
        },
    ];

    let input = simd_from_u32s(&mut context, vec![0, 21, 26]);
    let log_domain_size = 5;
    let queries = select_queries(&mut context, &input, log_domain_size);

    let trace_queries_vars: Vec<Vec<M31>> = vec![
        vec![1.into(), 1.into(), 1.into()],
        vec![863170483.into(), 58834968.into(), 1606816039.into()],
        vec![1430398088.into(), 1532221375.into(), 264974634.into()],
    ];

    let trace_queries = EvalDomainSamples {
        data: vec![
            trace_queries_vars
                .iter()
                .map(|v| v.iter().map(|v2| M31Wrapper::from(*v2).guess(&mut context)).collect())
                .collect(),
        ],
    };

    let alpha = context.new_var(qm31_from_u32s(1058706599, 1486409878, 1052004241, 54096853));

    let res = compute_fri_input(&mut context, &oods_responses, &queries, &trace_queries, alpha);

    assert_eq!(res.len(), 3);
    assert_eq!(context.get(res[0]), qm31_from_u32s(1799948512, 769698546, 2025315394, 642248561));
    assert_eq!(context.get(res[1]), qm31_from_u32s(628725928, 1812037401, 1687426883, 1599536169));
    assert_eq!(context.get(res[2]), qm31_from_u32s(2027161127, 718656514, 538553980, 959876533));

    context.validate_circuit();
}
