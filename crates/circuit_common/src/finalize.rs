use circuits::blake::{blake_g_gate, m31_to_u32, triple_xor};
use circuits::circuit::Circuit;
use circuits::context::{Context, FinalizedContext, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{add_into, eq};
use rand_chacha::rand_core::{RngCore, SeedableRng};

use crate::N_LANES;

fn pad_qm31_ops(context: &mut Context<impl IValue>, target: usize) {
    let qm31_ops_n_rows = qm31_ops_n_rows(&context.circuit);
    assert!(qm31_ops_n_rows <= target);
    let one = context.one();
    for _ in qm31_ops_n_rows..target {
        // Use 1 + 1 to work around + zero optimization.
        eval!(context, (one) + (one));
    }
}

fn pad_eq(context: &mut Context<impl IValue>, target: usize) {
    let eq_n_rows = context.circuit.eq.len();
    assert!(eq_n_rows <= target);
    let zero = context.zero();
    for _ in eq_n_rows..target {
        circuits::ops::eq(context, zero, zero);
    }
}

fn pad_triple_xor(context: &mut Context<impl IValue>, target: usize) {
    let n_rows = context.circuit.triple_xor.len();
    assert!(n_rows <= target);
    let zero = context.zero();
    for _ in n_rows..target {
        circuits::blake::triple_xor(context, zero, zero, zero);
    }
}

fn pad_m31_to_u32(context: &mut Context<impl IValue>, target: usize) {
    let n_rows = context.circuit.m31_to_u32.len();
    assert!(n_rows <= target);
    let zero = context.zero();
    for _ in n_rows..target {
        circuits::blake::m31_to_u32(context, zero);
    }
}

fn pad_blake_g_gate(context: &mut Context<impl IValue>, target: usize) {
    let n_rows = context.circuit.blake_g_gate.len();
    assert!(n_rows <= target);
    let zero = context.zero();
    for _ in n_rows..target {
        blake_g_gate(context, zero, zero, zero, zero, zero, zero);
    }
}

/// Pads the components to the next power of two of each component.
/// Appends gates to the context so that the resulting AIR components have length equal to a power
/// of two.
// TODO(Gali): Have it under a trait.
// TODO(Ilya): Make it pub(crate).
pub fn pad_context(context: &mut FinalizedContext<impl IValue>) {
    // Padding the components to a power of two.
    let padded_sizes = compute_padded_sizes(context);
    pad_to_targets(context, padded_sizes);
}

/// Pads each component to its target size by appending trivial gates. The target sizes are passed
/// in parameter `targets`.
pub fn pad_to_targets(context: &mut FinalizedContext<impl IValue>, targets: ComponentSizes) {
    let inner_context = &mut context.context;
    pad_eq(inner_context, targets.eq);
    pad_qm31_ops(inner_context, targets.qm31_ops);
    pad_triple_xor(inner_context, targets.triple_xor);
    pad_m31_to_u32(inner_context, targets.m31_to_u32);
    pad_blake_g_gate(inner_context, targets.blake_g_gate);
}

/// Adds random values to the qm31_ops component via trivial.
/// the padding is
/// x + 0 = x
/// 0 + y = y
/// 0 + z = z
fn qm31_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let zero = context.zero();
    let value_x = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let x = context.new_var(IValue::from_qm31(value_x));
    add_into(context, x, zero, x);
    let value_y = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let y = context.new_var(IValue::from_qm31(value_y));
    add_into(context, zero, y, y);
    let value_z = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let z = context.new_var(IValue::from_qm31(value_z));
    add_into(context, z, zero, z);
}

/// Adds a random value to the eq component via a trivial `x == x` equality, blinding the eq trace.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
fn eq_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let zero = context.zero();
    let value = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let x = context.new_var(IValue::from_qm31(value));
    // yield x.
    add_into(context, x, zero, x);
    eq(context, x, x);
}

/// Creates a fresh variable holding a random `u32` value, encoded as `(low_u16, high_u16, 0, 0)`,
/// and yields it via a trivial `x + 0 = x` add gate so it can be used as a gate input.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
fn random_u32_var(context: &mut Context<impl IValue>, rng: &mut impl RngCore) -> Var {
    let zero = context.zero();
    let x = context.new_var(IValue::pack_u32(rng.next_u32()));
    add_into(context, x, zero, x);
    x
}

/// Creates a fresh variable holding a random `M31` value, encoded as `(x, 0, 0, 0)`, and yields it
/// via a trivial `x + 0 = x` add gate so it can be used as a gate input.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
fn random_m31_var(context: &mut Context<impl IValue>, rng: &mut impl RngCore) -> Var {
    let zero = context.zero();
    let x = context.new_var(IValue::from_qm31(qm31_from_u32s(rng.next_u32(), 0, 0, 0)));
    add_into(context, x, zero, x);
    x
}

/// Adds a random row to the triple_xor component, blinding its trace.
fn triple_xor_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let a = random_u32_var(context, rng);
    let b = random_u32_var(context, rng);
    let c = random_u32_var(context, rng);
    triple_xor(context, a, b, c);
}

/// Adds a random row to the m31_to_u32 component, blinding its trace.
fn m31_to_u32_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let input = random_m31_var(context, rng);
    m31_to_u32(context, input);
}

/// Adds a random row to the blake_g_gate component, blinding its trace.
fn blake_g_gate_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let a = random_u32_var(context, rng);
    let b = random_u32_var(context, rng);
    let c = random_u32_var(context, rng);
    let d = random_u32_var(context, rng);
    let f0 = random_u32_var(context, rng);
    let f1 = random_u32_var(context, rng);
    blake_g_gate(context, a, b, c, d, f0, f1);
}

/// Adds ZK blinding to the circuit by adding random rows to all the witness components: qm31_ops,
/// eq, triple_xor, m31_to_u32, and blake_g_gate. The fresh input variables are yielded through
/// trivial `x + 0 = x` add gates in the qm31_ops component.
pub fn add_zk_blinding(
    context: &mut FinalizedContext<impl IValue>,
    seed_bytes: [u8; 32],
    n_padding: usize,
) {
    let context = &mut context.context;
    let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed_bytes);
    for _ in 0..n_padding {
        qm31_zk_blinding(context, &mut rng);
        eq_zk_blinding(context, &mut rng);
        triple_xor_zk_blinding(context, &mut rng);
        m31_to_u32_zk_blinding(context, &mut rng);
        blake_g_gate_zk_blinding(context, &mut rng);
    }
}

/// Helper struct containing the sizes of the AIR components of a circuit.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentSizes {
    pub eq: usize,
    pub qm31_ops: usize,
    pub m31_to_u32: usize,
    pub triple_xor: usize,
    pub blake_g_gate: usize,
}

// Prints the sizes and the log base 2 rounded up.
impl std::fmt::Display for ComponentSizes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "eq={:>2} (log: {}),  qm31_ops={:>2} (log: {}), m31_to_u32={:>2} (log: {}), \
             triple_xor={:>2} (log: {}), blake_g_gate={:>2} (log: {})",
            self.eq,
            self.eq.next_power_of_two().ilog2(),
            self.qm31_ops,
            self.qm31_ops.next_power_of_two().ilog2(),
            self.m31_to_u32,
            self.m31_to_u32.next_power_of_two().ilog2(),
            self.triple_xor,
            self.triple_xor.next_power_of_two().ilog2(),
            self.blake_g_gate,
            self.blake_g_gate.next_power_of_two().ilog2(),
        ))
    }
}

pub fn compute_padded_sizes(context: &FinalizedContext<impl IValue>) -> ComponentSizes {
    let circuit = &context.context.circuit;
    let qm31_ops_n_rows = qm31_ops_n_rows(circuit);
    ComponentSizes {
        eq: padded_size(circuit.eq.len()),
        qm31_ops: padded_size(qm31_ops_n_rows),
        m31_to_u32: padded_size(circuit.m31_to_u32.len()),
        triple_xor: padded_size(circuit.triple_xor.len()),
        blake_g_gate: padded_size(circuit.blake_g_gate.len()),
    }
}

/// Computes the number of rows in the qm31_ops AIR component.
pub fn qm31_ops_n_rows(circuit: &Circuit) -> usize {
    circuit.add.len()
        + circuit.sub.len()
        + circuit.mul.len()
        + circuit.pointwise_mul.len()
        + circuit.permutation.iter().map(|p| p.inputs.len() + p.outputs.len()).sum::<usize>()
}

pub fn padded_size(n_rows: usize) -> usize {
    std::cmp::max(n_rows.next_power_of_two(), N_LANES)
}
