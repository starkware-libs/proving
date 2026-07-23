use std::sync::Arc;

use circuits::circuit::{
    Add, BlakeGGate, Circuit, Eq, Gate, M31ToU32, Mul, Permutation, PointwiseMul, Sub, TripleXor,
};
use circuits::context::FinalizedContext;
use circuits::ivalue::IValue;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use itertools::zip_eq;
#[cfg(feature = "prover")]
use stwo::core::fields::m31::BaseField;
#[cfg(feature = "prover")]
use stwo::core::poly::circle::CanonicCoset;
#[cfg(feature = "prover")]
use stwo::prover::backend::simd::m31::PackedM31;
#[cfg(feature = "prover")]
use stwo::prover::backend::{Backend, Col, Column};
#[cfg(feature = "prover")]
use stwo::prover::poly::BitReversedOrder;
#[cfg(feature = "prover")]
use stwo::prover::poly::circle::CircleEvaluation;
pub use stwo_cairo_common::preprocessed_columns::blake::BLAKE_SIGMA;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[cfg(feature = "prover")]
use crate::N_LANES;
use crate::Qm31OpsTraceGenerator;
use crate::finalize::pad_context;

#[cfg(feature = "prover")]
#[cfg(test)]
#[path = "preprocessed_test.rs"]
pub mod test;

const N_QM31_OPS_PP_COLUMNS: usize = 8;
const N_OP_CODES: usize = 4;

#[derive(Copy, Clone)]
enum OpCode {
    Add,
    Sub,
    Mul,
    PointwiseMul,
}

/// Adds the binary operation gates to the qm31 ops preprocessed trace.
fn fill_binary_op_columns<G: Gate>(
    gates: &[G],
    op_code: OpCode,
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; N_QM31_OPS_PP_COLUMNS],
) {
    let op_code_idx = op_code as usize;
    for gate in gates.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        let [out] = gate.yields()[..] else { panic!("Expected 1 yield for gate") };
        (0..N_OP_CODES).for_each(|i| {
            columns[i].push(if i == op_code_idx { 1 } else { 0 });
        });
        columns[4].push(in0);
        columns[5].push(in1);
        columns[6].push(out);
        // TODO(Gali): Consider negating the multiplicities.
        columns[7].push(multiplicities[out]);
    }
}

/// Implements a permutation gate with n inputs and n outputs using 2n Add gates.
///
/// Process:
/// 1. First n gates: Write inputs to permutation wire
///    - `permutation_wire = Add(0, input_i)` for each input i
/// 2. Next n gates: Read outputs from permutation wire
///    - `output_i = Add(0, permutation_wire)` for each output i
///
/// Using the same wire address for all the inputs with multiplicity 1 ensures that the outputs
/// are a permutation of the inputs.
fn fill_permutation_columns(
    gates: &[Permutation],
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; N_QM31_OPS_PP_COLUMNS],
    first_unused_address: usize,
) {
    let add_op_code_idx = OpCode::Add as usize;
    let mut permutation_address = first_unused_address;
    for gate in gates.iter() {
        let inputs = gate.uses();
        let outputs = gate.yields();

        // Set flag to Add opcode.
        (0..N_OP_CODES).for_each(|i| {
            columns[i].extend(std::iter::repeat_n(
                (i == add_op_code_idx) as usize,
                inputs.len() + outputs.len(),
            ));
        });

        // TODO(alonf): Parallelize, and insert the above loop inside.
        for (input, output) in zip_eq(inputs, outputs) {
            // Input row.
            columns[4].push(0);
            columns[5].push(input);
            columns[6].push(permutation_address);
            columns[7].push(1);

            // Output row.
            columns[4].push(0);
            columns[5].push(permutation_address);
            columns[6].push(output);
            columns[7].push(multiplicities[output]);
        }

        permutation_address += 1;
    }
}

/// The circuit gates that map onto the qm31_ops component.
struct Qm31OpsGates<'a> {
    add: &'a [Add],
    sub: &'a [Sub],
    mul: &'a [Mul],
    pointwise_mul: &'a [PointwiseMul],
    permutation: &'a [Permutation],
}

/// Adds the preprocessed columns of qm31_ops component to the preprocessed trace. If the component
/// is empty, no columns are added. Preprocessed columns are in the following format:
/// | add_flag | sub_flag | mul_flag | pointwise_mul_flag | in0_address | in1_address | out_address | mults |
///
/// `n_vars` is the total number of circuit variables, used as the first unused wire address for the
/// permutation columns.
fn add_qm31_ops_to_preprocessed_trace(
    gates: Qm31OpsGates<'_>,
    n_vars: usize,
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) -> Qm31OpsTraceGenerator {
    let Qm31OpsGates { add, sub, mul, pointwise_mul, permutation } = gates;
    let mut qm31_ops_columns: [_; N_QM31_OPS_PP_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_binary_op_columns(add, OpCode::Add, multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(sub, OpCode::Sub, multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(mul, OpCode::Mul, multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(
        pointwise_mul,
        OpCode::PointwiseMul,
        multiplicities,
        &mut qm31_ops_columns,
    );
    let qm31_ops_trace_generator =
        Qm31OpsTraceGenerator { first_permutation_row: qm31_ops_columns[0].len() };

    fill_permutation_columns(permutation, multiplicities, &mut qm31_ops_columns, n_vars);

    let ids = [
        "qm31_ops_add_flag",
        "qm31_ops_sub_flag",
        "qm31_ops_mul_flag",
        "qm31_ops_pointwise_mul_flag",
        "qm31_ops_in0_address",
        "qm31_ops_in1_address",
        "qm31_ops_out_address",
        "qm31_ops_mults",
    ];
    for (id, column) in zip_eq(ids, qm31_ops_columns) {
        pp_trace.push_column(PreProcessedColumnId { id: id.to_owned() }, column);
    }
    qm31_ops_trace_generator
}

/// Adds the preprocessed columns of eq component to the preprocessed trace. If the component
/// is empty, no columns are added. Preprocessed columns are in the following format:
/// | in0_address | in1_address |
fn add_eq_to_preprocessed_trace(eq: &[Eq], pp_trace: &mut PreProcessedTrace) {
    let mut eq_in0_address = vec![];
    let mut eq_in1_address = vec![];
    for Eq { in0, in1 } in eq {
        eq_in0_address.push(*in0);
        eq_in1_address.push(*in1);
    }

    pp_trace.push_column(PreProcessedColumnId { id: "eq_in0_address".to_owned() }, eq_in0_address);
    pp_trace.push_column(PreProcessedColumnId { id: "eq_in1_address".to_owned() }, eq_in1_address);
}

/// Adds TripleXor gates to the preprocessed trace. Preprocessed columns are in the format:
/// | input_addr_0 | input_addr_1 | input_addr_2 | output_addr | multiplicity |
fn add_triple_xor_to_preprocessed_trace(
    triple_xor: &[TripleXor],
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let mut triple_xor_input_addr_0 = vec![];
    let mut triple_xor_input_addr_1 = vec![];
    let mut triple_xor_input_addr_2 = vec![];
    let mut triple_xor_output_addr = vec![];
    let mut triple_xor_multiplicity = vec![];
    for TripleXor { input_a, input_b, input_c, out } in triple_xor.iter() {
        triple_xor_input_addr_0.push(*input_a);
        triple_xor_input_addr_1.push(*input_b);
        triple_xor_input_addr_2.push(*input_c);
        triple_xor_output_addr.push(*out);
        triple_xor_multiplicity.push(multiplicities[*out]);
    }

    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_input_addr_0".to_owned() },
        triple_xor_input_addr_0,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_input_addr_1".to_owned() },
        triple_xor_input_addr_1,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_input_addr_2".to_owned() },
        triple_xor_input_addr_2,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_output_addr".to_owned() },
        triple_xor_output_addr,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_multiplicity".to_owned() },
        triple_xor_multiplicity,
    );
}

const N_M31_TO_U32_PP_COLUMNS: usize = 3;

/// Adds M31ToU32 gates to preprocessed trace columns.
/// | input_address | output_address | multiplicity |
fn fill_m31_to_u32_columns(
    gates: &[M31ToU32],
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; N_M31_TO_U32_PP_COLUMNS],
) {
    for M31ToU32 { input, out } in gates.iter() {
        columns[0].push(*input);
        columns[1].push(*out);
        columns[2].push(multiplicities[*out]);
    }
}

fn add_m31_to_u32_to_preprocessed_trace(
    m31_to_u32: &[M31ToU32],
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let mut columns: [_; N_M31_TO_U32_PP_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_m31_to_u32_columns(m31_to_u32, multiplicities, &mut columns);

    let ids = ["m31_to_u32_input_addr", "m31_to_u32_output_addr", "m31_to_u32_multiplicity"];
    for (id, column) in zip_eq(ids, columns) {
        pp_trace.push_column(PreProcessedColumnId { id: id.to_owned() }, column);
    }
}

/// Adds BlakeGGate gates to the preprocessed trace. Preprocessed columns are in the format:
/// | input_addr_a | input_addr_b | input_addr_c | input_addr_d | input_addr_f0 | input_addr_f1 |
/// | output_addr_a | output_addr_b | output_addr_c | output_addr_d | multiplicity |
fn add_blake_g_gate_to_preprocessed_trace(
    blake_g_gate: &[BlakeGGate],
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let mut blake_g_gate_input_addr_a = vec![];
    let mut blake_g_gate_input_addr_b = vec![];
    let mut blake_g_gate_input_addr_c = vec![];
    let mut blake_g_gate_input_addr_d = vec![];
    let mut blake_g_gate_input_addr_f0 = vec![];
    let mut blake_g_gate_input_addr_f1 = vec![];
    let mut blake_g_gate_output_addr_a = vec![];
    let mut blake_g_gate_output_addr_b = vec![];
    let mut blake_g_gate_output_addr_c = vec![];
    let mut blake_g_gate_output_addr_d = vec![];
    let mut blake_g_gate_multiplicity = vec![];
    for BlakeGGate {
        input_a,
        input_b,
        input_c,
        input_d,
        input_f0,
        input_f1,
        out_a,
        out_b,
        out_c,
        out_d,
    } in blake_g_gate.iter()
    {
        blake_g_gate_input_addr_a.push(*input_a);
        blake_g_gate_input_addr_b.push(*input_b);
        blake_g_gate_input_addr_c.push(*input_c);
        blake_g_gate_input_addr_d.push(*input_d);
        blake_g_gate_input_addr_f0.push(*input_f0);
        blake_g_gate_input_addr_f1.push(*input_f1);
        blake_g_gate_output_addr_a.push(*out_a);
        blake_g_gate_output_addr_b.push(*out_b);
        blake_g_gate_output_addr_c.push(*out_c);
        blake_g_gate_output_addr_d.push(*out_d);

        // All four outputs of a Blake G gate share one multiplicity column. In the Blake
        // construction, each G output is consumed exactly once (by another G step or by the
        // triple-XOR).
        let mult = multiplicities[*out_a];
        for y in [out_b, out_c, out_d] {
            assert_eq!(
                multiplicities[*y], mult,
                "BlakeGGate output multiplicities must be identical"
            );
        }
        blake_g_gate_multiplicity.push(mult);
    }

    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_a".to_owned() },
        blake_g_gate_input_addr_a,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_b".to_owned() },
        blake_g_gate_input_addr_b,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_c".to_owned() },
        blake_g_gate_input_addr_c,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_d".to_owned() },
        blake_g_gate_input_addr_d,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_f0".to_owned() },
        blake_g_gate_input_addr_f0,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_f1".to_owned() },
        blake_g_gate_input_addr_f1,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_output_addr_a".to_owned() },
        blake_g_gate_output_addr_a,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_output_addr_b".to_owned() },
        blake_g_gate_output_addr_b,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_output_addr_c".to_owned() },
        blake_g_gate_output_addr_c,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_output_addr_d".to_owned() },
        blake_g_gate_output_addr_d,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_multiplicity".to_owned() },
        blake_g_gate_multiplicity,
    );
}

/// A collection of preprocessed columns, whose values are publicly acknowledged, and independent of
/// the proof.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreProcessedTrace {
    columns: OrderedHashMap<PreProcessedColumnId, Vec<usize>>,
}

impl PreProcessedTrace {
    fn push_column(&mut self, id: PreProcessedColumnId, column: Vec<usize>) {
        assert!(
            self.columns.insert(id.clone(), column).is_none(),
            "Duplicate preprocessed column id: {id:?}"
        );
    }

    /// Sorts preprocessed columns by size (ascending), preserving original order for ties.
    fn sort_by_size(&mut self) {
        // `IndexMap::sort_by` is a stable sort, so ties keep insertion order.
        self.columns.sort_by(|_, c1, _, c2| c1.len().cmp(&c2.len()));
    }
    /// Adds preprocessed columns that are fixed lookup tables, independent of the circuit's gates.
    ///
    /// Unlike the per-component helpers (e.g. `add_eq_to_preprocessed_trace`), these columns do not
    /// depend on the circuit gates or their multiplicities. They provide constant lookup
    /// infrastructure referenced by certain components:
    /// - `seq_16`: the sequence `0..2^16`, used by `range_check_16`.
    /// - `bitwise_xor_{n}_{0,1,2}`: for each bit-width n in {4, 7, 8, 9, 10}, three columns holding
    ///   all ordered pairs of n-bit values (`lhs`, `rhs`) and their XOR, used by the
    ///   `VerifyBitwiseXor` components.
    fn add_fixed_preprocessed_columns(pp_trace: &mut PreProcessedTrace) {
        // Seq column of size 2^16, needed by range_check_16.
        pp_trace.push_column(
            PreProcessedColumnId { id: "seq_16".to_owned() },
            (0..1_usize << 16).collect(),
        );

        // Bitwise XOR columns of sizes 2^4, 2^7, 2^8, 2^9, 2^10.
        let bitwise_xor: Vec<Vec<usize>> = [4, 7, 8, 9, 10]
            .into_iter()
            .flat_map(|n_bits| gen_xor_columns(n_bits).into_iter())
            .collect();
        let xor_col_ids = [
            "bitwise_xor_4_0",
            "bitwise_xor_4_1",
            "bitwise_xor_4_2",
            "bitwise_xor_7_0",
            "bitwise_xor_7_1",
            "bitwise_xor_7_2",
            "bitwise_xor_8_0",
            "bitwise_xor_8_1",
            "bitwise_xor_8_2",
            "bitwise_xor_9_0",
            "bitwise_xor_9_1",
            "bitwise_xor_9_2",
            "bitwise_xor_10_0",
            "bitwise_xor_10_1",
            "bitwise_xor_10_2",
        ];
        for (id, column) in zip_eq(xor_col_ids, bitwise_xor) {
            pp_trace.push_column(PreProcessedColumnId { id: id.to_owned() }, column);
        }
    }

    pub fn log_sizes(&self) -> OrderedHashMap<PreProcessedColumnId, u32> {
        self.columns.iter().map(|(id, column)| (id.clone(), column.len().ilog2())).collect()
    }

    pub fn ids(&self) -> Vec<PreProcessedColumnId> {
        self.columns.keys().cloned().collect()
    }

    pub fn n_columns(&self) -> usize {
        self.columns.len()
    }

    #[cfg(feature = "prover")]
    pub fn get_trace<B: Backend>(&self) -> Vec<CircleEvaluation<B, BaseField, BitReversedOrder>> {
        let to_evaluation = |vec: &[usize]| {
            let col = Col::<B, BaseField>::from_iter(vec.iter().cloned().map(BaseField::from));
            CircleEvaluation::new(CanonicCoset::new(col.len().ilog2()).circle_domain(), col)
        };

        self.columns.values().map(|c| to_evaluation(c)).collect()
    }

    pub fn get_column(&self, id: &PreProcessedColumnId) -> &Vec<usize> {
        self.columns.get(id).unwrap_or_else(|| panic!("Missing preprocessed column {id:?}"))
    }

    #[cfg(feature = "prover")]
    pub fn get_packed_column(&self, id: &PreProcessedColumnId) -> Vec<PackedM31> {
        let column = self.get_column(id);
        column
            .chunks_exact(N_LANES)
            .map(|c| PackedM31::from_array(std::array::from_fn(|i| BaseField::from(c[i]))))
            .collect::<Vec<_>>()
    }
}

/// A finalized circuit ready for proving: its fixed preprocessed trace together with the
/// parameters derived from the circuit's structure.
#[derive(Debug, PartialEq)]
pub struct PreprocessedCircuit {
    /// The fixed preprocessed trace columns, shared (via `Arc`) between the prover and the
    /// components that read them during witness generation.
    pub preprocessed_trace: Arc<PreProcessedTrace>,
    /// Log2 of the circuit's base trace size, this is the largest preprocessed column log size.
    pub trace_log_size: u32,
    /// Index of the first permutation row in the qm31_ops component, i.e. the number of
    /// (non-permutation) binary-op rows that precede the permutation rows.
    pub first_permutation_row: usize,
    /// Number of public output values of the circuit (excluding the output gate of the `u` wire).
    pub n_outputs: usize,
}

impl PreprocessedCircuit {
    /// Finalizes the context, then builds the preprocessed circuit.
    pub fn preprocess_circuit(context: &mut FinalizedContext<impl IValue>) -> Self {
        pad_context(context);
        Self::from_finalized_circuit(context.circuit())
    }

    /// Builds the preprocessed circuit data (trace + params) from a finalized circuit.
    pub fn from_finalized_circuit(circuit: &Circuit) -> Self {
        let mut pp_trace = PreProcessedTrace::default();

        // Adjust multiplicities to account for the use of the constant 0 in the permutation gate
        // implementation. See `fill_permutation_columns` for details.
        let mut multiplicities = circuit.compute_multiplicities().0;
        let additional_zero_multiplicity: usize =
            circuit.permutation.iter().map(|gate| gate.inputs.len() + gate.outputs.len()).sum();
        multiplicities[0] += additional_zero_multiplicity;

        let Circuit {
            n_vars,
            add,
            sub,
            mul,
            pointwise_mul,
            permutation,
            eq,
            triple_xor,
            m31_to_u32,
            blake_g_gate,
            output,
        } = circuit;

        // Add Eq columns.
        add_eq_to_preprocessed_trace(eq, &mut pp_trace);
        // Add QM31 operations columns.
        let qm31_ops_trace_generator = add_qm31_ops_to_preprocessed_trace(
            Qm31OpsGates { add, sub, mul, pointwise_mul, permutation },
            *n_vars,
            &multiplicities,
            &mut pp_trace,
        );
        // Add TripleXor columns.
        add_triple_xor_to_preprocessed_trace(triple_xor, &multiplicities, &mut pp_trace);
        // Add M31ToU32 columns.
        add_m31_to_u32_to_preprocessed_trace(m31_to_u32, &multiplicities, &mut pp_trace);
        // Add BlakeGGate columns.
        add_blake_g_gate_to_preprocessed_trace(blake_g_gate, &multiplicities, &mut pp_trace);

        PreProcessedTrace::add_fixed_preprocessed_columns(&mut pp_trace);
        pp_trace.sort_by_size();

        // The log trace size is The largest preprocessed column log size.
        let trace_log_size = pp_trace.log_sizes().values().copied().max().unwrap();

        Self {
            preprocessed_trace: Arc::new(pp_trace),
            trace_log_size,
            first_permutation_row: qm31_ops_trace_generator.first_permutation_row,
            // Discard the output gate of the `u` wire.
            n_outputs: output.len() - 1,
        }
    }
}

/// Generates three columns of size (2^n_bits)^2. The first two columns are all ordered pairs of
/// n-bit values, and the third column contains the bitwise XOR of each pair.
fn gen_xor_columns(n_bits: usize) -> [Vec<usize>; 3] {
    let size = 1_usize << (2 * n_bits);
    let mask = (1_usize << n_bits) - 1;
    let mut columns: [Vec<usize>; 3] = std::array::from_fn(|_| vec![0; size]);
    // `i` is used for bit arithmetic (lhs/rhs), not merely to index the columns.
    #[allow(clippy::needless_range_loop)]
    for i in 0..size {
        let lhs = i & mask;
        let rhs = i >> n_bits;
        columns[0][i] = rhs;
        columns[1][i] = lhs;
        columns[2][i] = lhs ^ rhs;
    }
    columns
}
