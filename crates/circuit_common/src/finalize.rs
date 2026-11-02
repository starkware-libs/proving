use circuits::blake::blake_g_gate;
use circuits::context::{Context, FinalizedContext};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::wrappers::{M31Wrapper, U32Wrapper};

use crate::N_LANES;

fn pad_qm31_ops(context: &mut Context<impl IValue>, target: usize) {
    let qm31_ops_n_rows = context.circuit.n_qm31_ops_rows();
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
    let zero = U32Wrapper::const_u32(context, 0);
    for _ in n_rows..target {
        circuits::blake::triple_xor(context, zero, zero, zero);
    }
}

fn pad_m31_to_u32(context: &mut Context<impl IValue>, target: usize) {
    let n_rows = context.circuit.m31_to_u32.len();
    assert!(n_rows <= target);
    let zero = M31Wrapper::const_m31(context, 0.into());
    for _ in n_rows..target {
        circuits::blake::m31_to_u32(context, zero);
    }
}

fn pad_blake_g_gate(context: &mut Context<impl IValue>, target: usize) {
    let n_rows = context.circuit.blake_g_gate.len();
    assert!(n_rows <= target);
    let zero = U32Wrapper::const_u32(context, 0);
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
    pad_to_targets(context, &padded_sizes);
}

/// Pads each component to its target size by appending trivial gates. The target sizes are passed
/// in parameter `targets`.
pub fn pad_to_targets(context: &mut FinalizedContext<impl IValue>, targets: &ComponentSizes) {
    let inner_context = &mut context.context;
    pad_eq(inner_context, targets.eq);
    pad_qm31_ops(inner_context, targets.qm31_ops);
    pad_triple_xor(inner_context, targets.triple_xor);
    pad_m31_to_u32(inner_context, targets.m31_to_u32);
    pad_blake_g_gate(inner_context, targets.blake_g_gate);
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

impl ComponentSizes {
    /// Applies `f` to every component's size.
    pub fn map(&self, f: impl Fn(usize) -> usize) -> ComponentSizes {
        ComponentSizes {
            eq: f(self.eq),
            qm31_ops: f(self.qm31_ops),
            m31_to_u32: f(self.m31_to_u32),
            triple_xor: f(self.triple_xor),
            blake_g_gate: f(self.blake_g_gate),
        }
    }

    /// Combines two sets of sizes componentwise with `f`.
    pub fn zip(&self, other: &ComponentSizes, f: impl Fn(usize, usize) -> usize) -> ComponentSizes {
        ComponentSizes {
            eq: f(self.eq, other.eq),
            qm31_ops: f(self.qm31_ops, other.qm31_ops),
            m31_to_u32: f(self.m31_to_u32, other.m31_to_u32),
            triple_xor: f(self.triple_xor, other.triple_xor),
            blake_g_gate: f(self.blake_g_gate, other.blake_g_gate),
        }
    }

    /// The component-wise maximum of two sets of sizes. Padding several circuits to this shared
    /// target is what lets a single AIR verify executions of any of them.
    pub fn elementwise_max(&self, other: &ComponentSizes) -> ComponentSizes {
        self.zip(other, std::cmp::max)
    }
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

/// The raw (non-padded) row count of each AIR component, i.e. [`compute_padded_sizes`] before its
/// power-of-two rounding.
pub fn raw_component_sizes(context: &FinalizedContext<impl IValue>) -> ComponentSizes {
    let circuit = &context.context.circuit;
    ComponentSizes {
        eq: circuit.eq.len(),
        qm31_ops: circuit.n_qm31_ops_rows(),
        m31_to_u32: circuit.m31_to_u32.len(),
        triple_xor: circuit.triple_xor.len(),
        blake_g_gate: circuit.blake_g_gate.len(),
    }
}

pub fn compute_padded_sizes(context: &FinalizedContext<impl IValue>) -> ComponentSizes {
    raw_component_sizes(context).map(padded_size)
}

pub fn padded_size(n_rows: usize) -> usize {
    std::cmp::max(n_rows.next_power_of_two(), N_LANES)
}
