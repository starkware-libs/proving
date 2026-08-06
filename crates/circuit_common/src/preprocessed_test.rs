use circuits::circuit::{
    Add, BlakeGGate, Circuit, Eq, M31ToU32, Mul, Output, PointwiseMul, Sub, TripleXor,
};
use expect_test::expect;
use itertools::Itertools;
use stwo::prover::backend::Column;
use stwo::prover::backend::simd::SimdBackend;

use crate::finalize::{ComponentSizes, qm31_ops_n_rows};
use crate::preprocessed::{PreprocessedCircuit, layout_from_component_sizes};

/// A small circuit with power-of-two row counts in every component: 2 eq, 8 qm31_ops (binary
/// only), and 16 each of triple_xor, m31_to_u32 and blake_g_gate.
fn sample_circuit() -> Circuit {
    let mut circuit = Circuit::default();
    circuit.add.push(Add { in0: 0, in1: 1, out: 2 });
    circuit.add.push(Add { in0: 3, in1: 4, out: 5 });
    circuit.sub.push(Sub { in0: 6, in1: 7, out: 8 });
    circuit.sub.push(Sub { in0: 9, in1: 10, out: 11 });
    circuit.mul.push(Mul { in0: 12, in1: 13, out: 14 });
    circuit.mul.push(Mul { in0: 15, in1: 16, out: 17 });
    circuit.pointwise_mul.push(PointwiseMul { in0: 18, in1: 19, out: 20 });
    circuit.pointwise_mul.push(PointwiseMul { in0: 21, in1: 22, out: 23 });
    circuit.eq.push(Eq { in0: 0, in1: 1 });
    circuit.eq.push(Eq { in0: 0, in1: 2 });
    for i in 0..16 {
        circuit.triple_xor.push(TripleXor { input_a: 0, input_b: 1, input_c: 2, out: 56 + i });
    }
    for i in 0..16 {
        let o = 88 + 4 * i;
        circuit.blake_g_gate.push(BlakeGGate {
            input_a: 0,
            input_b: 1,
            input_c: 2,
            input_d: 3,
            input_f0: 4,
            input_f1: 5,
            out_a: o,
            out_b: o + 1,
            out_c: o + 2,
            out_d: o + 3,
        });
    }
    for i in 0..16 {
        circuit.m31_to_u32.push(M31ToU32 { input: 0, out: 72 + i });
    }
    circuit.n_vars = 152;
    // A circuit must always have at least one output.
    circuit.output.push(Output { in0: 0 });
    circuit
}

#[test]
fn test_preprocess_circuit() {
    let preprocessed_trace = PreprocessedCircuit::from_finalized_circuit(&sample_circuit())
        .preprocessed_trace
        .get_trace::<SimdBackend>();

    assert_eq!(preprocessed_trace.len(), 45);
    let lengths = preprocessed_trace.iter().map(|column| column.values.len()).collect_vec();
    expect![[r#"
        [
            2,
            2,
            8,
            8,
            8,
            8,
            8,
            8,
            8,
            8,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            256,
            256,
            256,
            16384,
            16384,
            16384,
            65536,
            65536,
            65536,
            65536,
            262144,
            262144,
            262144,
            1048576,
            1048576,
            1048576,
        ]
    "#]]
    .assert_debug_eq(&lengths);
}

/// [`layout_from_component_sizes`] must reproduce a real preprocessed trace's layout — ids, log
/// sizes and commitment order (`OrderedHashMap` equality is order-sensitive) — for a circuit
/// whose components have exactly those sizes.
#[test]
fn test_layout_from_component_sizes_matches_preprocessed_trace() {
    let circuit = sample_circuit();
    let sizes = ComponentSizes {
        eq: circuit.eq.len(),
        qm31_ops: qm31_ops_n_rows(&circuit),
        m31_to_u32: circuit.m31_to_u32.len(),
        triple_xor: circuit.triple_xor.len(),
        blake_g_gate: circuit.blake_g_gate.len(),
    };
    let real = PreprocessedCircuit::from_finalized_circuit(&circuit).preprocessed_trace;
    assert_eq!(real.log_sizes(), layout_from_component_sizes(&sizes));
}
