use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use stwo::core::circle::Coset;
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::SecureField;
use stwo::core::fri::{fold_circle_into_line, fold_coset};
use stwo::core::poly::circle::{CanonicCoset, CircleDomain};
use stwo::core::poly::line::LineDomain;
use stwo::prover::backend::CpuBackend;
use stwo::prover::fri::FriOps;
use stwo::prover::line::LineEvaluation;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::secure_column::SecureColumnByCoords;

fn folding_benchmark(c: &mut Criterion) {
    const LOG_SIZE: u32 = 12;
    let domain = LineDomain::new(CanonicCoset::new(LOG_SIZE + 1).half_coset());
    let evals = LineEvaluation::new(
        domain,
        SecureColumnByCoords {
            columns: std::array::from_fn(|i| {
                vec![BaseField::from_u32_unchecked(i as u32); 1 << LOG_SIZE]
            }),
        },
    );
    let alpha = SecureField::from_u32_unchecked(2213980, 2213981, 2213982, 2213983);
    let twiddles = CpuBackend::precompute_twiddles(domain.coset());
    c.bench_function("fold_line", |b| {
        b.iter(|| {
            black_box(CpuBackend::fold_line(black_box(&evals), black_box(&[alpha]), &twiddles));
        })
    });
}

/// The FRI verifier folds, per query and layer, a coset of size `2^fold_step` of the layer's
/// domain.
fn verifier_folding_benchmark(c: &mut Criterion) {
    const LAYER_LOG_SIZE: u32 = 20;
    let alpha = SecureField::from_u32_unchecked(2213980, 2213981, 2213982, 2213983);
    for fold_step in [1, 4] {
        let evals = (0..1 << fold_step)
            .map(|i| SecureField::from_u32_unchecked(i, i + 1, i + 2, i + 3))
            .collect::<Vec<_>>();

        let layer_coset = CanonicCoset::new(LAYER_LOG_SIZE + 1).half_coset();
        let line_domain = LineDomain::new(Coset::new(layer_coset.index_at(12345), fold_step));
        c.bench_function(&format!("verifier fold_coset 2^{fold_step}"), |b| {
            b.iter_batched(
                || evals.clone(),
                |evals| fold_coset(evals, black_box(line_domain), black_box(alpha)),
                BatchSize::SmallInput,
            )
        });

        let layer_domain = CanonicCoset::new(LAYER_LOG_SIZE).circle_domain();
        let circle_domain =
            CircleDomain::new(Coset::new(layer_domain.index_at(12345), fold_step - 1));
        c.bench_function(&format!("verifier fold_circle_into_line 2^{fold_step}"), |b| {
            b.iter(|| {
                fold_circle_into_line(black_box(&evals), black_box(circle_domain), black_box(alpha))
            })
        });
    }
}

criterion_group!(benches, folding_benchmark, verifier_folding_benchmark);
criterion_main!(benches);
