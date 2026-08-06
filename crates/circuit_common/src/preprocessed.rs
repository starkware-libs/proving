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
use stwo::core::vcs::blake2_hash::Blake2sHash;
#[cfg(feature = "prover")]
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
#[cfg(feature = "prover")]
use stwo::prover::CommitmentTreeProver;
#[cfg(feature = "prover")]
use stwo::prover::backend::simd::SimdBackend;
#[cfg(feature = "prover")]
use stwo::prover::backend::simd::m31::PackedM31;
#[cfg(feature = "prover")]
use stwo::prover::backend::{Backend, Col, Column};
#[cfg(feature = "prover")]
use stwo::prover::mempool::BaseColumnPool;
#[cfg(feature = "prover")]
use stwo::prover::poly::BitReversedOrder;
#[cfg(feature = "prover")]
use stwo::prover::poly::circle::{CircleEvaluation, PolyOps};
pub use stwo_cairo_common::preprocessed_columns::blake::BLAKE_SIGMA;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[cfg(feature = "prover")]
use crate::N_LANES;
use crate::Qm31OpsTraceGenerator;
use crate::finalize::{ComponentSizes, pad_context};

#[cfg(feature = "prover")]
#[cfg(test)]
#[path = "preprocessed_test.rs"]
pub mod test;

const N_OP_CODES: usize = 4;

/// Declares one padded AIR component's preprocessed columns, in commitment order: one column per
/// field, under the field's name as the column id.
///
/// The ids and the fields holding the values they label are one list, so no edit can pair a
/// column with the wrong id.
macro_rules! define_preprocessed_columns {
    ($struct_name:ident, $column_ids_array_name:ident, [$($field:ident),+ $(,)?]) => {
        const $column_ids_array_name: &[&str] = &[$(stringify!($field)),+];

        /// Defaults to every column empty, to be filled by the component's gates.
        #[derive(Default)]
        struct $struct_name {
            $($field: Vec<usize>),+
        }

        impl From<[Vec<usize>; $column_ids_array_name.len()]> for $struct_name {
            /// For components whose columns are filled by index rather than by name.
            fn from(columns: [Vec<usize>; $column_ids_array_name.len()]) -> Self {
                let [$($field),+] = columns;
                Self { $($field),+ }
            }
        }

        impl $struct_name {
            /// Pushes every column under its id, in commitment order.
            fn push_to(self, pp_trace: &mut PreProcessedTrace) {
                for (id, column) in
                    zip_eq($column_ids_array_name.iter().copied(), [$(self.$field),+])
                {
                    pp_trace.push_column(PreProcessedColumnId { id: id.to_owned() }, column);
                }
            }
        }
    };
}

define_preprocessed_columns!(EqColumns, EQ_COLUMN_IDS, [eq_in0_address, eq_in1_address]);
define_preprocessed_columns!(
    Qm31OpsColumns,
    QM31_OPS_COLUMN_IDS,
    [
        qm31_ops_add_flag,
        qm31_ops_sub_flag,
        qm31_ops_mul_flag,
        qm31_ops_pointwise_mul_flag,
        qm31_ops_in0_address,
        qm31_ops_in1_address,
        qm31_ops_out_address,
        qm31_ops_mults
    ]
);
define_preprocessed_columns!(
    TripleXorColumns,
    TRIPLE_XOR_COLUMN_IDS,
    [
        triple_xor_input_addr_0,
        triple_xor_input_addr_1,
        triple_xor_input_addr_2,
        triple_xor_output_addr,
        triple_xor_multiplicity
    ]
);
define_preprocessed_columns!(
    M31ToU32Columns,
    M31_TO_U32_COLUMN_IDS,
    [m31_to_u32_input_addr, m31_to_u32_output_addr, m31_to_u32_multiplicity]
);
define_preprocessed_columns!(
    BlakeGGateColumns,
    BLAKE_G_GATE_COLUMN_IDS,
    [
        blake_g_gate_input_addr_a,
        blake_g_gate_input_addr_b,
        blake_g_gate_input_addr_c,
        blake_g_gate_input_addr_d,
        blake_g_gate_input_addr_f0,
        blake_g_gate_input_addr_f1,
        blake_g_gate_output_addr_a,
        blake_g_gate_output_addr_b,
        blake_g_gate_output_addr_c,
        blake_g_gate_output_addr_d,
        blake_g_gate_multiplicity
    ]
);

// The two components that fill their columns by index rather than by name.
const N_QM31_OPS_PP_COLUMNS: usize = QM31_OPS_COLUMN_IDS.len();
const N_M31_TO_U32_PP_COLUMNS: usize = M31_TO_U32_COLUMN_IDS.len();

/// Bit widths of the bitwise-XOR lookup tables (three columns each; a table of `n` bits has
/// `2^(2n)` rows).
const XOR_TABLE_N_BITS: [u32; 5] = [4, 7, 8, 9, 10];
/// Log size of the sequence column (`0..2^n`, used by `range_check_16`).
const SEQ_LOG_SIZE: u32 = 16;

/// A fixed column's per-row value, as a function of the column's log size.
type ColumnFn = fn(u32, usize) -> usize;

fn seq(_log_size: u32, row: usize) -> usize {
    row
}
// Row `i` of a `bitwise_xor_{n}` table (`2^(2n)` rows, so `n` is half the log size) pairs
// `rhs = i >> n` with `lhs = i & (2^n - 1)`.
fn xor_rhs(log_size: u32, row: usize) -> usize {
    row >> (log_size / 2)
}
fn xor_lhs(log_size: u32, row: usize) -> usize {
    row & ((1 << (log_size / 2)) - 1)
}
fn xor_result(log_size: u32, row: usize) -> usize {
    xor_lhs(log_size, row) ^ xor_rhs(log_size, row)
}

/// The fixed lookup-table columns, in commitment order: the sequence column, then the three
/// columns of the bitwise-XOR table of each bit width.
///
/// Each entry pairs an id and log size with the per-row value function of the column it labels.
fn fixed_columns() -> impl Iterator<Item = (String, u32, ColumnFn)> {
    let xor_columns: [ColumnFn; 3] = [xor_rhs, xor_lhs, xor_result];
    std::iter::once((format!("seq_{SEQ_LOG_SIZE}"), SEQ_LOG_SIZE, seq as ColumnFn)).chain(
        XOR_TABLE_N_BITS.into_iter().flat_map(move |n_bits| {
            xor_columns
                .into_iter()
                .enumerate()
                .map(move |(i, value)| (format!("bitwise_xor_{n_bits}_{i}"), 2 * n_bits, value))
        }),
    )
}

/// The preprocessed-trace layout of a circuit whose components are padded to `sizes` (powers of
/// two), without building it: each column's log size, in commitment order. Matches what
/// [`PreprocessedCircuit::preprocess_circuit`] produces for such a circuit — both take the ids and
/// their order from the same generated lists, and each component's columns are as long as the
/// component.
pub fn layout_from_component_sizes(
    sizes: &ComponentSizes,
) -> OrderedHashMap<PreProcessedColumnId, u32> {
    let mut entries: Vec<(String, u32)> = vec![];
    let mut push = |ids: &[&str], size: usize| {
        assert!(size.is_power_of_two());
        entries.extend(ids.iter().map(|id| (id.to_string(), size.ilog2())));
    };
    // Order of components here must match `PreprocessedCircuit::from_finalized_circuit`.
    push(EQ_COLUMN_IDS, sizes.eq);
    push(QM31_OPS_COLUMN_IDS, sizes.qm31_ops);
    push(TRIPLE_XOR_COLUMN_IDS, sizes.triple_xor);
    push(M31_TO_U32_COLUMN_IDS, sizes.m31_to_u32);
    push(BLAKE_G_GATE_COLUMN_IDS, sizes.blake_g_gate);
    entries.extend(fixed_columns().map(|(id, log_size, _)| (id, log_size)));
    // The same order as `PreProcessedTrace::sort_by_size` (stable, so ties keep insertion order).
    entries.sort_by_key(|(_, log_size)| *log_size);
    entries.into_iter().map(|(id, log_size)| (PreProcessedColumnId { id }, log_size)).collect()
}

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

    Qm31OpsColumns::from(qm31_ops_columns).push_to(pp_trace);
    qm31_ops_trace_generator
}

/// Adds the preprocessed columns of eq component to the preprocessed trace. If the component
/// is empty, no columns are added. Preprocessed columns are in the following format:
/// | in0_address | in1_address |
fn add_eq_to_preprocessed_trace(eq: &[Eq], pp_trace: &mut PreProcessedTrace) {
    let mut columns = EqColumns::default();
    for Eq { in0, in1 } in eq {
        columns.eq_in0_address.push(*in0);
        columns.eq_in1_address.push(*in1);
    }

    columns.push_to(pp_trace);
}

/// Adds TripleXor gates to the preprocessed trace. Preprocessed columns are in the format:
/// | input_addr_0 | input_addr_1 | input_addr_2 | output_addr | multiplicity |
fn add_triple_xor_to_preprocessed_trace(
    triple_xor: &[TripleXor],
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let mut columns = TripleXorColumns::default();
    for TripleXor { input_a, input_b, input_c, out } in triple_xor.iter() {
        columns.triple_xor_input_addr_0.push(*input_a);
        columns.triple_xor_input_addr_1.push(*input_b);
        columns.triple_xor_input_addr_2.push(*input_c);
        columns.triple_xor_output_addr.push(*out);
        columns.triple_xor_multiplicity.push(multiplicities[*out]);
    }

    columns.push_to(pp_trace);
}

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

    M31ToU32Columns::from(columns).push_to(pp_trace);
}

/// Adds BlakeGGate gates to the preprocessed trace. Preprocessed columns are in the format:
/// | input_addr_a | input_addr_b | input_addr_c | input_addr_d | input_addr_f0 | input_addr_f1 |
/// | output_addr_a | output_addr_b | output_addr_c | output_addr_d | multiplicity |
fn add_blake_g_gate_to_preprocessed_trace(
    blake_g_gate: &[BlakeGGate],
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let mut columns = BlakeGGateColumns::default();
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
        columns.blake_g_gate_input_addr_a.push(*input_a);
        columns.blake_g_gate_input_addr_b.push(*input_b);
        columns.blake_g_gate_input_addr_c.push(*input_c);
        columns.blake_g_gate_input_addr_d.push(*input_d);
        columns.blake_g_gate_input_addr_f0.push(*input_f0);
        columns.blake_g_gate_input_addr_f1.push(*input_f1);
        columns.blake_g_gate_output_addr_a.push(*out_a);
        columns.blake_g_gate_output_addr_b.push(*out_b);
        columns.blake_g_gate_output_addr_c.push(*out_c);
        columns.blake_g_gate_output_addr_d.push(*out_d);

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
        columns.blake_g_gate_multiplicity.push(mult);
    }

    columns.push_to(pp_trace);
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
    /// - `bitwise_xor_{n}_{0,1,2}`: three columns per bit width n, where `_0` and `_1` run over
    ///   every ordered pair of n-bit values (`rhs`, `lhs`) and `_2` holds their XOR, used by the
    ///   `VerifyBitwiseXor` components.
    fn add_fixed_preprocessed_columns(pp_trace: &mut PreProcessedTrace) {
        for (id, log_size, value) in fixed_columns() {
            let column = (0..1_usize << log_size).map(|row| value(log_size, row)).collect();
            pp_trace.push_column(PreProcessedColumnId { id }, column);
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

    /// The Merkle root of this circuit's preprocessed trace, committed exactly as
    /// [`stwo::prover::CommitmentTreeProver`] does inside the circuit prover, so the result equals
    /// tree 0's commitment in a proof of this circuit.
    ///
    /// `log_blowup_factor` is the circuit prover's blowup (the one in the proof's `PcsConfig`),
    /// which also fixes the lifting log size at `trace_log_size + log_blowup_factor`.
    #[cfg(feature = "prover")]
    pub fn preprocessed_root(&self, log_blowup_factor: u32) -> Blake2sHash {
        let lifting_log_size = self.trace_log_size + log_blowup_factor;
        let twiddles = SimdBackend::precompute_twiddles(
            CanonicCoset::new(lifting_log_size).circle_domain().half_coset,
        );
        let preprocessed_trace = self.preprocessed_trace.get_trace::<SimdBackend>();
        let preprocessed_trace_polys =
            SimdBackend::interpolate_columns(preprocessed_trace, &twiddles);
        let preprocessed_tree = CommitmentTreeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(
            preprocessed_trace_polys,
            log_blowup_factor,
            &twiddles,
            true,
            lifting_log_size,
            &BaseColumnPool::<SimdBackend>::new(),
        );
        preprocessed_tree.commitment.root()
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
