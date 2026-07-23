use hashbrown::{HashMap, HashSet};
use indexmap::IndexMap;
use itertools::zip_eq;
use num_traits::{One, Zero};
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::blake::m31_to_u32_into;
use crate::circuit::{Add, Circuit};
use crate::finalize_constants::finalize_constants;
use crate::ivalue::IValue;
use crate::ops::{add_into, output, pointwise_mul_into};
use crate::stats::Stats;

#[cfg(test)]
#[path = "context_test.rs"]
pub mod test;

/// The address of the `u` variable.
pub const U_VAR_IDX: usize = 2;
/// The value of `u`.
pub const U_VALUE: QM31 = QM31::from_m31(M31(0), M31(0), M31(1), M31(0));

/// Represents a variable in a [Circuit].
///
/// A [Var] represents a `QM31` value.
/// In some cases, it may be restricted to an `M31` or a boolean value by adding constraints to the
/// circuit. For example, `x = x * x` will enforce that `x` is either `0` or `1`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Var {
    pub idx: usize,
}
impl std::fmt::Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.idx)
    }
}

/// Represents a variable that is guessed.
#[derive(PartialEq, Eq)]
pub enum GuessVar {
    // The guessed value is in the base field `M31`.
    M31(Var),
    // The guessed value is in the field `QM31`.
    QM31(Var),
    // The guessed value is a 16-bit unsigned integer.
    U16(Var),
}

/// Represents the information required to build a [Circuit].
///
/// Context can be used in two ways:
/// 1. To build just the [Circuit] with no concrete values for the variables.
/// 2. To build the [Circuit] with concrete `QM31` values for the variables.
///
/// In the first case the type `Value` is set to `NoValue`, and in the second case it is set to
/// `QM31`.
pub struct Context<Value: IValue> {
    pub circuit: Circuit,
    constants: IndexMap<QM31, Var>,
    values: Vec<Value>,
    /// Counters tracking how many of each gate/operation were emitted while building the circuit.
    ///
    /// NOTE: These stats are independent of the arithmetization of the circuit — they count
    /// circuit-level operations, not AIR component rows, and do NOT account for all the extra
    /// gates that finalization appends (e.g. the padding gates added by `finalize_context`).
    /// Do not rely on them to size AIR components.
    // TODO(Ilya): Consider removing `stats` entirely.
    pub stats: Stats,
    /// Set of variables that were marked by the code as "unused".
    /// [Self::check_vars_used] checks that these are indeed unused.
    unused_vars: HashSet<usize>,
    /// Set of variables that were marked by the code as "maybe unused". These skip
    /// the checks in [Self::check_vars_used] entirely.
    maybe_unused_vars: HashSet<usize>,
    /// Set of variables that are not the result of a gate, but instead are provided by the prover.
    ///
    /// `None` if the set of guessed variables has already been finalized.
    ///
    /// See [crate::ops::guess].
    pub guessed_vars: Option<Vec<GuessVar>>,
    /// Variables allocated by [Self::reserve] whose values have not been supplied yet. Assignment
    /// happens through method [Self::set_outputs]. Reading a reserved variable via
    /// [Self::get] triggers a debug assertion.
    reserved_vars: Vec<usize>,
    /// Debug only. If true, equality is asserted when adding the `eq` gate; if false, no
    /// assertion is made during construction and equality can be checked later at validation.
    pub assert_eq_on_eval: bool,
    /// Debug only. A map from variable name to the corresponding [Var].
    pub debug_info: HashMap<String, Var>,
}

impl<Value: IValue> Context<Value> {
    /// Creates a new context with `n_reserved` wires at addresses `3 + i`, for `i ∈ [0,
    /// n_reserved)`.
    pub fn new(n_reserved: usize) -> Self {
        let mut context = Context::default();
        for _ in 0..n_reserved {
            context.reserve();
        }
        context
    }

    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }

    pub fn constants(&self) -> &IndexMap<QM31, Var> {
        &self.constants
    }

    pub fn zero(&self) -> Var {
        Var { idx: 0 }
    }

    pub fn one(&self) -> Var {
        Var { idx: 1 }
    }

    pub fn u(&self) -> Var {
        Var { idx: U_VAR_IDX }
    }

    /// Creates a new variable.
    pub fn new_var(&mut self, value: Value) -> Var {
        let idx = self.circuit.n_vars;
        self.circuit.n_vars += 1;
        self.values.push(value);
        Var { idx }
    }

    /// Returns the value of a variable.
    pub fn get(&self, var: Var) -> Value {
        assert!(!self.reserved_vars.contains(&var.idx), "read of reserved variable",);
        self.values[var.idx]
    }

    /// Allocates a fresh variable with no concrete value yet.
    ///
    /// Reading the value before assignment triggers a debug assertion in [`Self::get`].
    pub fn reserve(&mut self) -> Var {
        let reserved = self.new_var(Value::placeholder());
        self.reserved_vars.push(reserved.idx);
        reserved
    }

    /// Given a vector of variables, it copies their values into the reserved variables and marks
    /// the reserved variables as outputs. It also adds an `Add` gate to yield and constrain the
    /// reserved vars.
    ///
    /// Panics if `vars` has a different length than the number of the currently reserved variables.
    pub fn set_outputs(&mut self, vars: &[Var]) {
        for (reserved, var) in zip_eq(std::mem::take(&mut self.reserved_vars), vars) {
            let value = self.get(*var);
            self.values[reserved] = value;
            self.circuit.add.push(Add { in0: var.idx, in1: self.zero().idx, out: reserved });
            output(self, Var { idx: reserved });
        }
    }

    pub fn constant(&mut self, value: QM31) -> Var {
        if let Some(var) = self.constants.get(&value) {
            *var
        } else {
            let var = self.new_var(Value::from_qm31(value));
            self.constants.insert(value, var);
            var
        }
    }

    /// Marks a variable as unused.
    ///
    /// See [Self::check_vars_used].
    pub fn mark_as_unused(&mut self, var: Var) {
        assert!(self.unused_vars.insert(var.idx));
    }

    /// Marks a variable as "maybe unused".
    ///
    /// See [Self::check_vars_used].
    pub fn mark_as_maybe_unused(&mut self, var: &Var) {
        assert!(self.maybe_unused_vars.insert(var.idx));
    }

    /// Checks that all the variables that were defined are used in the circuit by some gate.
    ///
    /// This is a sanity check for the correction of the circuit.
    ///
    /// Variables that were marked as unused by the code are excluded.
    pub fn check_vars_used(&self) {
        let var_uses = self.circuit.compute_multiplicities().0;
        for (idx, uses) in var_uses.iter().enumerate() {
            let unused = *uses == 0;
            let marked_as_unused = self.unused_vars.contains(&idx);
            if !unused && marked_as_unused {
                panic!("Variable {idx} is used but marked as unused");
            }
            if unused && !(marked_as_unused || self.maybe_unused_vars.contains(&idx)) {
                panic!("Variable {idx} is unused but not marked as unused");
            }
        }
    }

    /// Finalizes the set of guessed variables by adding a trivial constraint for each guessed
    /// variable.
    ///
    /// Each gate in the circuit has lookups for its inputs (use lookups) and outputs (yield
    /// lookups).
    /// For the lookup constraints to hold and be sound, we need to make sure that each variable
    /// appears exactly once as a yield lookup.
    /// For guessed value, add a trivial constraint so that the new variable appears once as
    /// a yield.
    ///
    /// This is `pub(crate)` and only called internally by [`Self::finalize`]. Tests that need to
    /// run this step in isolation go through [`crate::test_utils::finalize_guessed_vars`].
    pub(crate) fn finalize_guessed_vars(&mut self) {
        // TODO(Leo): move the assertion to a new finalize method which calls finalize_constants and
        // finalize_guessed_vars.
        assert!(
            self.reserved_vars.is_empty(),
            "Some reserved variables were never assigned (idxs: {:?})",
            self.reserved_vars,
        );
        for guessed_var in self.guessed_vars.take().unwrap() {
            match guessed_var {
                // An M31 guess must be constrained to the base field. Pointwise-multiplying by
                // `one = (1, 0, 0, 0)` forces the upper three M31 limbs to zero, and yields `var`.
                GuessVar::M31(var) => {
                    pointwise_mul_into(self, var, self.one(), var);
                }
                // A QM31 guess is unrestricted; the trivial `var + 0 = var` gate only yields `var`.
                GuessVar::QM31(var) => {
                    add_into(self, var, self.zero(), var);
                }
                GuessVar::U16(var) => {
                    // A U16 guess must be constrained to a 16-bit unsigned integer.
                    // Asserting `(x, 0, 0, 0) == (x & 0xFFFF, x >> 16, 0, 0)` forces `x >> 16`
                    // to be zero, i.e. `x` to be a 16-bit unsigned integer.
                    m31_to_u32_into(self, var, var);
                }
            }
        }
    }

    pub fn finalize(mut self, check_vars_used: bool) -> FinalizedContext<Value> {
        finalize_constants(&mut self);
        if check_vars_used {
            self.check_vars_used();
        }
        self.finalize_guessed_vars();
        FinalizedContext { context: self }
    }
}

impl<Value: IValue> Default for Context<Value> {
    fn default() -> Self {
        let mut res = Self {
            circuit: Circuit::default(),
            constants: IndexMap::new(),
            values: vec![],
            stats: Stats::default(),
            unused_vars: HashSet::new(),
            maybe_unused_vars: HashSet::new(),
            guessed_vars: Some(vec![]),
            reserved_vars: vec![],
            assert_eq_on_eval: false,
            debug_info: HashMap::new(),
        };
        // Register zero, one, and u as the first constants.
        res.constant(QM31::zero());
        res.constant(QM31::one());
        let u_var = res.constant(U_VALUE); // u at idx 2
        assert_eq!(u_var.idx, U_VAR_IDX);
        // Add `u` to the outputs.
        output(&mut res, u_var);
        res
    }
}

/// A context with real QM31 values.
pub type TraceContext = Context<QM31>;

impl TraceContext {
    /// Returns `true` if the values satisfy the circuit.
    pub fn is_circuit_valid(&self) -> bool {
        self.circuit.check(self.values()).is_ok()
    }

    /// Validates that the values satisfy the circuit.
    pub fn validate_circuit(&self) {
        self.circuit.check(self.values()).unwrap();
    }

    pub fn enable_assert_eq_on_eval(&mut self) {
        self.assert_eq_on_eval = true;
    }
}

pub struct FinalizedContext<Value: IValue> {
    pub context: Context<Value>,
}

impl<Value: IValue> FinalizedContext<Value> {
    pub fn circuit(&self) -> &Circuit {
        &self.context.circuit
    }

    pub fn stats(&self) -> &Stats {
        &self.context.stats
    }

    pub fn values(&self) -> &Vec<Value> {
        self.context.values()
    }

    pub fn check_vars_used(&self) {
        self.context.check_vars_used();
    }

    pub fn get(&self, var: Var) -> Value {
        self.context.get(var)
    }
}

impl FinalizedContext<QM31> {
    pub fn is_circuit_valid(&self) -> bool {
        self.context.is_circuit_valid()
    }

    pub fn validate_circuit(&self) {
        self.context.validate_circuit();
    }
}
