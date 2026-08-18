use criterion::{Criterion, criterion_group, criterion_main};
use num_traits::{One, Zero};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::{BaseField, M31};
use stwo::core::fields::qm31::SecureField;
use stwo::core::fields::{FieldExpOps, batch_inverse_in_place};
use stwo::prover::backend::simd::cm31::PackedCM31;
use stwo::prover::backend::simd::m31::{N_LANES, PackedBaseField, PackedM31};
use stwo::prover::backend::simd::qm31::{PackedQM31, batch_inverse_packed_qm31};

pub const N_ELEMENTS: usize = 1 << 16;
pub const N_STATE_ELEMENTS: usize = 8;

pub fn m31_operations_bench(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(0);
    let elements: Vec<M31> = (0..N_ELEMENTS).map(|_| rng.random()).collect();
    let mut state: [M31; N_STATE_ELEMENTS] = rng.random();

    c.bench_function("M31 mul", |b| {
        b.iter(|| {
            for elem in &elements {
                for _ in 0..128 {
                    for state_elem in &mut state {
                        *state_elem *= *elem;
                    }
                }
            }
        })
    });

    c.bench_function("M31 add", |b| {
        b.iter(|| {
            for elem in &elements {
                for _ in 0..128 {
                    for state_elem in &mut state {
                        *state_elem += *elem;
                    }
                }
            }
        })
    });
}

pub fn cm31_operations_bench(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(0);
    let elements: Vec<CM31> = (0..N_ELEMENTS).map(|_| rng.random()).collect();
    let mut state: [CM31; N_STATE_ELEMENTS] = rng.random();

    c.bench_function("CM31 mul", |b| {
        b.iter(|| {
            for elem in &elements {
                for _ in 0..128 {
                    for state_elem in &mut state {
                        *state_elem *= *elem;
                    }
                }
            }
        })
    });

    c.bench_function("CM31 add", |b| {
        b.iter(|| {
            for elem in &elements {
                for _ in 0..128 {
                    for state_elem in &mut state {
                        *state_elem += *elem;
                    }
                }
            }
        })
    });
}

pub fn qm31_operations_bench(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(0);
    let elements: Vec<SecureField> = (0..N_ELEMENTS).map(|_| rng.random()).collect();
    let mut state: [SecureField; N_STATE_ELEMENTS] = rng.random();

    c.bench_function("SecureField mul", |b| {
        b.iter(|| {
            for elem in &elements {
                for _ in 0..128 {
                    for state_elem in &mut state {
                        *state_elem *= *elem;
                    }
                }
            }
        })
    });

    c.bench_function("SecureField add", |b| {
        b.iter(|| {
            for elem in &elements {
                for _ in 0..128 {
                    for state_elem in &mut state {
                        *state_elem += *elem;
                    }
                }
            }
        })
    });
}

pub fn simd_m31_operations_bench(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(0);
    let elements: Vec<PackedBaseField> = (0..N_ELEMENTS / N_LANES).map(|_| rng.random()).collect();
    let mut states = vec![PackedBaseField::broadcast(BaseField::one()); N_STATE_ELEMENTS];

    c.bench_function("mul_simd", |b| {
        b.iter(|| {
            for elem in elements.iter() {
                for _ in 0..128 {
                    for state in states.iter_mut() {
                        *state *= *elem;
                    }
                }
            }
        })
    });

    c.bench_function("add_simd", |b| {
        b.iter(|| {
            for elem in elements.iter() {
                for _ in 0..128 {
                    for state in states.iter_mut() {
                        *state += *elem;
                    }
                }
            }
        })
    });

    c.bench_function("sub_simd", |b| {
        b.iter(|| {
            for elem in elements.iter() {
                for _ in 0..128 {
                    for state in states.iter_mut() {
                        *state -= *elem;
                    }
                }
            }
        })
    });
}

pub fn simd_batch_inverse_bench(c: &mut Criterion) {
    const N_PACKED_ELEMENTS: usize = 1 << 16;
    let mut rng = SmallRng::seed_from_u64(0);

    // The base field reference point: Montgomery's batch inverse with nothing to descend to.
    let m31s: Vec<PackedM31> = (0..N_PACKED_ELEMENTS).map(|_| rng.random()).collect();
    let mut m31_dst = vec![PackedM31::zero(); N_PACKED_ELEMENTS];
    c.bench_function("PackedM31 batch_inverse", |b| {
        b.iter(|| batch_inverse_in_place(&m31s, &mut m31_dst))
    });

    let cm31s: Vec<PackedCM31> =
        (0..N_PACKED_ELEMENTS).map(|_| PackedCM31::from_array(rng.random())).collect();
    c.bench_function("PackedCM31 batch_inverse", |b| b.iter(|| PackedCM31::batch_inverse(&cm31s)));

    let qm31s: Vec<PackedQM31> = (0..N_PACKED_ELEMENTS).map(|_| rng.random()).collect();
    let mut qm31_dst = vec![PackedQM31::zero(); N_PACKED_ELEMENTS];
    c.bench_function("PackedQM31 batch_inverse", |b| {
        b.iter(|| batch_inverse_packed_qm31(&qm31s, &mut qm31_dst))
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = m31_operations_bench, cm31_operations_bench, qm31_operations_bench,
        simd_m31_operations_bench, simd_batch_inverse_bench);
criterion_main!(benches);
