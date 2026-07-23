use std::collections::HashMap;

use itertools::chain;
use stwo::core::fields::qm31::QM31;

use crate::blake::blake2s_g;
use crate::ivalue::IValue;

#[cfg(test)]
#[path = "circuit_test.rs"]
pub mod test;

fn check_eq(a: QM31, b: QM31) -> Result<(), String> {
    if a != b {
        return Err(format!("{a} != {b}"));
    }
    Ok(())
}

pub trait Gate: std::fmt::Debug {
    fn check(&self, values: &[QM31]) -> Result<(), String>;

    /// Returns the variables that are "used" by the gate (in the context of lookup terms).
    fn uses(&self) -> Vec<usize>;

    /// Returns the variables that are "yielded" by the gate (in the context of lookup terms).
    fn yields(&self) -> Vec<usize>;
}

/// Represents an addition gate in the circuit: `[in0] + [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Add {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Add {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] + values[self.in1], values[self.out])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Add {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] + [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a subtraction gate in the circuit: `[in0] - [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Sub {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Sub {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] - values[self.in1], values[self.out])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] - [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a multiplication gate in the circuit: `[in0] * [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Mul {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Mul {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] * values[self.in1], values[self.out])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Mul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] * [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a pointwise multiplication gate in the circuit.
/// See [IValue::pointwise_mul] for more details.
#[derive(PartialEq, Eq)]
pub struct PointwiseMul {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for PointwiseMul {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(QM31::pointwise_mul(values[self.in0], values[self.in1]), values[self.out])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for PointwiseMul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] x [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents an equality gate in the circuit: `[in0] = [in1]`.
#[derive(PartialEq, Eq)]
pub struct Eq {
    pub in0: usize,
    pub in1: usize,
}
impl Gate for Eq {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0], values[self.in1])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![]
    }
}

impl std::fmt::Debug for Eq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}]", self.in0, self.in1)
    }
}

/// Represents a triple XOR gate: `out = a ^ b ^ c` (u32 XOR).
/// Inputs must be encoded as `(u16, u16, 0, 0)` in QM31.
#[derive(PartialEq, Eq)]
pub struct TripleXor {
    pub input_a: usize,
    pub input_b: usize,
    pub input_c: usize,
    pub out: usize,
}
impl Gate for TripleXor {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        for (i, &input) in [self.input_a, self.input_b, self.input_c].iter().enumerate() {
            let [a, b, c, d] = values[input].to_m31_array().map(|m| m.0);
            if a > 0xffff || b > 0xffff || c != 0 || d != 0 {
                return Err(format!(
                    "TripleXor: input {i} is not of the form (u16, u16, 0, 0), got {}",
                    values[input]
                ));
            }
        }
        let a = values[self.input_a].unpack_u32();
        let b = values[self.input_b].unpack_u32();
        let c = values[self.input_c].unpack_u32();
        check_eq(values[self.out], QM31::pack_u32(a ^ b ^ c))
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.input_a, self.input_b, self.input_c]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for TripleXor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] = TripleXor([{}], [{}], [{}])",
            self.out, self.input_a, self.input_b, self.input_c
        )
    }
}

/// Represents an M31ToU32 gate in the circuit: `(x & 0xFFFF, x >> 16, 0, 0) = M31ToU32(x, 0, 0,
/// 0)`.
#[derive(PartialEq, Eq)]
pub struct M31ToU32 {
    pub input: usize,
    pub out: usize,
}
impl Gate for M31ToU32 {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        let input = values[self.input];

        // Assert that the input is M31.
        let [_, b, c, d] = input.to_m31_array().map(|m| m.0);
        if b != 0 || c != 0 || d != 0 {
            return Err(format!("M31ToU32: input is not M31, got {input}"));
        }
        let expected_output = input.m31_to_u32();
        check_eq(values[self.out], expected_output)
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.input]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for M31ToU32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = m31_to_u32([{}])", self.out, self.input)
    }
}

/// Represents a permutation gate in the circuit.
/// The gate enforces that the input values as a multi-set are equal to the output values
/// as a multi-set.
#[derive(PartialEq, Eq)]
pub struct Permutation {
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
}
impl Gate for Permutation {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        let mut multi_set = HashMap::new();

        for input in &self.inputs {
            *multi_set.entry(values[*input]).or_insert(0) += 1;
        }

        for output in &self.outputs {
            *multi_set.entry(values[*output]).or_insert(0) -= 1;
        }

        for count in multi_set.values() {
            if *count != 0 {
                return Err("Permutation is not valid".to_string());
            }
        }

        Ok(())
    }

    fn uses(&self) -> Vec<usize> {
        self.inputs.to_vec()
    }

    fn yields(&self) -> Vec<usize> {
        self.outputs.to_vec()
    }
}

impl std::fmt::Debug for Permutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}) = ({:?})", self.outputs, self.inputs)
    }
}

/// Represents a gate that receives an input wire and "marks" it as output.
#[derive(PartialEq, Eq)]
pub struct Output {
    pub in0: usize,
}
impl Gate for Output {
    fn check(&self, _values: &[QM31]) -> Result<(), String> {
        Ok(())
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0]
    }

    fn yields(&self) -> Vec<usize> {
        vec![]
    }
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "output [{}]", self.in0)
    }
}

/// Represents a Blake2s G function gate in the circuit: G(a, b, c, d, f0, f1) = (a', b', c', d').
/// Inputs must be encoded as `(u16, u16, 0, 0)` in QM31.
#[derive(PartialEq, Eq)]
pub struct BlakeGGate {
    pub input_a: usize,
    pub input_b: usize,
    pub input_c: usize,
    pub input_d: usize,
    pub input_f0: usize,
    pub input_f1: usize,
    pub out_a: usize,
    pub out_b: usize,
    pub out_c: usize,
    pub out_d: usize,
}
impl Gate for BlakeGGate {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        for (i, &input) in
            [self.input_a, self.input_b, self.input_c, self.input_d, self.input_f0, self.input_f1]
                .iter()
                .enumerate()
        {
            let [a, b, c, d] = values[input].to_m31_array().map(|m| m.0);
            if a > 0xffff || b > 0xffff || c != 0 || d != 0 {
                return Err(format!(
                    "BlakeGGate: input {i} is not of the form (u16, u16, 0, 0), got {}",
                    values[input]
                ));
            }
        }

        let a = values[self.input_a].unpack_u32();
        let b = values[self.input_b].unpack_u32();
        let c = values[self.input_c].unpack_u32();
        let d = values[self.input_d].unpack_u32();
        let f0 = values[self.input_f0].unpack_u32();
        let f1 = values[self.input_f1].unpack_u32();

        let (a, b, c, d) = blake2s_g(a, b, c, d, f0, f1);

        check_eq(values[self.out_a], QM31::pack_u32(a))?;
        check_eq(values[self.out_b], QM31::pack_u32(b))?;
        check_eq(values[self.out_c], QM31::pack_u32(c))?;
        check_eq(values[self.out_d], QM31::pack_u32(d))
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.input_a, self.input_b, self.input_c, self.input_d, self.input_f0, self.input_f1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out_a, self.out_b, self.out_c, self.out_d]
    }
}

impl std::fmt::Debug for BlakeGGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "([{}],[{}],[{}],[{}]) = BlakeGGate([{}],[{}],[{}],[{}],[{}],[{}])",
            self.out_a,
            self.out_b,
            self.out_c,
            self.out_d,
            self.input_a,
            self.input_b,
            self.input_c,
            self.input_d,
            self.input_f0,
            self.input_f1,
        )
    }
}

/// Represents a circuit.
#[derive(Default, PartialEq, Eq)]
pub struct Circuit {
    pub n_vars: usize,
    pub add: Vec<Add>,
    pub sub: Vec<Sub>,
    pub mul: Vec<Mul>,
    pub pointwise_mul: Vec<PointwiseMul>,
    pub eq: Vec<Eq>,
    pub triple_xor: Vec<TripleXor>,
    pub m31_to_u32: Vec<M31ToU32>,
    pub blake_g_gate: Vec<BlakeGGate>,
    pub permutation: Vec<Permutation>,
    pub output: Vec<Output>,
}

impl Circuit {
    /// Returns an iterator over all the gates in the circuit.
    pub fn all_gates(&self) -> impl Iterator<Item = &dyn Gate> {
        let Circuit {
            n_vars: _,
            add,
            sub,
            mul,
            pointwise_mul,
            eq,
            triple_xor,
            m31_to_u32,
            blake_g_gate,
            permutation,
            output,
        } = self;
        chain!(
            add.iter().map(|g| g as &dyn Gate),
            sub.iter().map(|g| g as &dyn Gate),
            mul.iter().map(|g| g as &dyn Gate),
            pointwise_mul.iter().map(|g| g as &dyn Gate),
            eq.iter().map(|g| g as &dyn Gate),
            blake_g_gate.iter().map(|g| g as &dyn Gate),
            triple_xor.iter().map(|g| g as &dyn Gate),
            m31_to_u32.iter().map(|g| g as &dyn Gate),
            permutation.iter().map(|g| g as &dyn Gate),
            output.iter().map(|g| g as &dyn Gate),
        )
    }

    /// Number of rows in the QM31Ops AIR component for this circuit.
    ///
    /// NOTE: This assumes a specific arithmetization of the circuit, in which the arithmetic
    /// gates (add/sub/mul/pointwise_mul) and the permutation gates are all packed into a single
    /// QM31Ops component — one row per arithmetic gate plus one row per permutation input/output.
    /// The `Circuit` itself is otherwise independent of the arithmetization.
    pub fn n_qm31_ops_rows(&self) -> usize {
        self.add.len()
            + self.sub.len()
            + self.mul.len()
            + self.pointwise_mul.len()
            + self.permutation.iter().map(|p| p.inputs.len() + p.outputs.len()).sum::<usize>()
    }

    /// Checks if the circuit is satisfied by the given values.
    pub fn check(&self, values: &[QM31]) -> Result<(), String> {
        for gate in self.all_gates() {
            gate.check(values)?;
        }
        Ok(())
    }

    /// Returns the number of uses and number of yields for each variable (in the context of lookup
    /// terms).
    pub fn compute_multiplicities(&self) -> (Vec<usize>, Vec<usize>) {
        let mut n_uses = vec![0; self.n_vars];
        let mut n_yields = vec![0; self.n_vars];

        for gate in self.all_gates() {
            for use_var in gate.uses() {
                n_uses[use_var] += 1;
            }
            for yield_var in gate.yields() {
                n_yields[yield_var] += 1;
            }
        }

        (n_uses, n_yields)
    }

    /// Verifies that each variable appears exactly once as a yield.
    pub fn check_yields(&self) {
        for (idx, n_yields) in self.compute_multiplicities().1.iter().enumerate() {
            assert!(*n_yields == 1, "Variable {idx} appears {n_yields} times as a yield");
        }
    }
}

impl std::fmt::Debug for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for gate in self.all_gates() {
            writeln!(f, "{gate:?}")?;
        }
        Ok(())
    }
}
