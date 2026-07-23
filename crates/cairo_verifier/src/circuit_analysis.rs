//! Analyzes the Cairo-verifier [`circuits::circuit::Circuit`].

use std::ops::Deref;

use circuits::circuit::{Add, BlakeGGate, Circuit, M31ToU32, Mul, PointwiseMul, Sub, TripleXor};
use circuits::context::Var;
use circuits::ivalue::{IValue, qm31_from_u32s};
use hashbrown::{HashMap, HashSet};
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

/// Helper function to construct a QM31 value from 4 u32s.
fn qm(a: u32, b: u32, c: u32, d: u32) -> QM31 {
    qm31_from_u32s(a, b, c, d)
}

/// Simd constants for 1/2.
fn simd_half() -> QM31 {
    qm(1 << 30, 1 << 30, 1 << 30, 1 << 30)
}

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Pointwise,
}

/// A binary arithmetic gate `[out] = [a] op [b]`.
struct Arith {
    out: usize,
    op: Op,
    a: usize,
    b: usize,
}

/// The verifier circuit plus the lookup tables the analysis needs.
/// Derefs to the wrapped [`Circuit`], so its gate vectors (`add`, `eq`, `blake_g_gate`, ...)
/// can be accessed directly.
struct CircuitEx<'a> {
    circuit: &'a Circuit,
    /// All arithmetic gates (add/sub/mul/pointwise).
    arith: Vec<Arith>,
    /// Raw producer of each arithmetic output: `out -> (op, a, b)`.
    producers: HashMap<usize, (Op, usize, usize)>,
    /// Guessed (unconstrained) variables, yielded by the trivial gate `[v] = [v] + [0]`.
    guessed: HashSet<usize>,
    /// Map output -> in for `m31_to_u32` gates.
    m31_to_u32_map: HashMap<usize, usize>,
    /// Map output -> inputs for `blake_g_gate` gates.
    blake_g_map: HashMap<usize, [usize; 6]>,
    /// Map output -> inputs for `triple_xor` gates.
    triple_xor_map: HashMap<usize, [usize; 3]>,
}

impl Deref for CircuitEx<'_> {
    type Target = Circuit;
    fn deref(&self) -> &Circuit {
        self.circuit
    }
}

/// Records one arithmetic gate `[out] = [a] op [b]`.
fn add_arith(c: &mut CircuitEx<'_>, op: Op, a: usize, b: usize, out: usize) {
    c.arith.push(Arith { out, op, a, b });
    c.producers.insert(out, (op, a, b));
    if op == Op::Add && out == a && b == 0 {
        // The trivial yield gate `[v] = [v] + [0]`.
        c.guessed.insert(out);
    }
}

/// Builds the analysis view from the verifier circuit's typed gates. Output gates are ignored
/// (they carry no dependency we need).
fn build_analysis(circuit: &Circuit) -> CircuitEx<'_> {
    let mut c = CircuitEx {
        circuit,
        arith: Vec::new(),
        producers: HashMap::new(),
        guessed: HashSet::new(),
        m31_to_u32_map: HashMap::new(),
        blake_g_map: HashMap::new(),
        triple_xor_map: HashMap::new(),
    };

    for Add { in0, in1, out } in &circuit.add {
        add_arith(&mut c, Op::Add, *in0, *in1, *out);
    }
    for Sub { in0, in1, out } in &circuit.sub {
        add_arith(&mut c, Op::Sub, *in0, *in1, *out);
    }
    for Mul { in0, in1, out } in &circuit.mul {
        add_arith(&mut c, Op::Mul, *in0, *in1, *out);
    }
    for PointwiseMul { in0, in1, out } in &circuit.pointwise_mul {
        add_arith(&mut c, Op::Pointwise, *in0, *in1, *out);
    }

    for M31ToU32 { input, out } in &circuit.m31_to_u32 {
        c.m31_to_u32_map.insert(*out, *input);
    }

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
    } in &circuit.blake_g_gate
    {
        let inputs = [*input_a, *input_b, *input_c, *input_d, *input_f0, *input_f1];
        for o in [*out_a, *out_b, *out_c, *out_d] {
            c.blake_g_map.insert(o, inputs);
        }
    }

    for TripleXor { input_a, input_b, input_c, out } in &circuit.triple_xor {
        c.triple_xor_map.insert(*out, [*input_a, *input_b, *input_c]);
    }

    c
}

// Constant propagation.

fn apply(op: Op, a: QM31, b: QM31) -> QM31 {
    match op {
        Op::Add => a + b,
        Op::Sub => a - b,
        Op::Mul => a * b,
        Op::Pointwise => QM31::pointwise_mul(a, b),
    }
}

/// Propagates the seed constants (`[0]=0`, `[1]=1`, `[2]=u`) through arithmetic and equality
/// gates: an arithmetic output is constant once both inputs are; an equality propagates a known
/// side to the other. Returns each constant variable's QM31 value.
fn propagate_constants(c: &CircuitEx<'_>) -> HashMap<usize, QM31> {
    // input variable -> arithmetic gates consuming it.
    // For example, if the first arithmetic gate (idx=0) is [6] = [4] + [5],
    // then consumers[4] = consumers[5] = [0].
    let mut consumers: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, arith) in c.arith.iter().enumerate() {
        consumers.entry(arith.a).or_default().push(idx);
        if arith.b != arith.a {
            consumers.entry(arith.b).or_default().push(idx);
        }
    }

    // Equality adjacency: for every equality gate, each side points to the other.
    let mut eq_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for eq in &c.eq {
        eq_adj.entry(eq.in0).or_default().push(eq.in1);
        eq_adj.entry(eq.in1).or_default().push(eq.in0);
    }

    // Initialize the values of the seed constants.
    let mut const_values =
        HashMap::from([(0, QM31::zero()), (1, QM31::one()), (2, qm(0, 0, 1, 0))]);
    let mut work = vec![0, 1, 2];
    while let Some(var) = work.pop() {
        let val = const_values[&var];
        // Go over all the variables that are equal to the current variable, register their value
        // and add them to `work`.
        for &other_var in eq_adj.get(&var).into_iter().flatten() {
            register_constant(&mut const_values, &mut work, other_var, val);
        }

        // Go over all the consumers of the current variable, and compute the value of the output.
        for &gate_idx in consumers.get(&var).into_iter().flatten() {
            let arith = &c.arith[gate_idx];

            // If both inputs are constants, registers the output.
            let (Some(&va), Some(&vb)) = (const_values.get(&arith.a), const_values.get(&arith.b))
            else {
                continue;
            };

            register_constant(&mut const_values, &mut work, arith.out, apply(arith.op, va, vb));
        }
    }
    const_values
}

/// Helper function for `propagate_constants`.
/// Registers the value `val` for `var` in `const_values`.
/// If `var` was not seen before, it is added to `work` for further propagation.
/// If it was seen, asserts that the new value matches the old one.
fn register_constant(
    const_values: &mut HashMap<usize, QM31>,
    work: &mut Vec<usize>,
    var: usize,
    val: QM31,
) {
    let insert_res = const_values.insert(var, val);
    if let Some(old_val) = insert_res {
        assert!(old_val == val, "Value mismatch for {var} during constant propagation");
    } else {
        work.push(var);
    }
}

// Idioms.

/// A recognized higher-level meaning of a variable, layered over the raw producers.
#[derive(Clone, Copy)]
enum Idiom {
    // X = 1/Y, from the `div(1, Y)` idiom.
    Inverse { y: usize },
    // One of the bits in `value`'s binary decomposition.
    Bit { value: usize },
}

/// Returns every bit-constrained variable: variable with a constraint forcing `v == v²`
/// (hence v in {0, 1}). One constraint of the form `[s] = [v] x [v]` and another constraint which
/// is either:
///   (a) a direct equality `[v] = [s]`; or
///   (b) the difference `[d] = [v] - [s]`, pointwise-masked by a constant, constrained to zero.
///
/// As some of the coordinates of the constant in (b) may be zero, the check here is not full.
/// This is acceptable for this test.
fn find_bits(c: &CircuitEx<'_>, const_values: &HashMap<usize, QM31>) -> HashSet<usize> {
    // A map `v` -> `s` from `[s] = [v] x [v]`.
    let mut square_of: HashMap<usize, usize> = HashMap::new();
    // A map `(a, b)` -> `[a] - [b]`.
    let mut sub_out: HashMap<(usize, usize), usize> = HashMap::new();
    // A map `var` -> outputs of `var x const`.
    let mut masked: HashMap<usize, Vec<usize>> = HashMap::new();

    for arith in &c.arith {
        match arith.op {
            Op::Pointwise if arith.a == arith.b => {
                square_of.entry(arith.a).or_insert(arith.out);
            }
            Op::Pointwise if const_values.get(&arith.b).is_some_and(|c| !c.is_zero()) => {
                masked.entry(arith.a).or_default().push(arith.out);
            }
            Op::Sub => {
                sub_out.entry((arith.a, arith.b)).or_insert(arith.out);
            }
            _ => {}
        }
    }

    let eq_set: HashSet<(usize, usize)> = c.eq.iter().map(|e| (e.in0, e.in1)).collect();
    // Variables constrained to equal the zero constant.
    let eq_zero: HashSet<usize> =
        c.eq.iter().filter_map(|e| (e.in1 == 0).then_some(e.in0)).collect();
    // Whether `[d]` is constrained to zero through a pointwise mask `[d] x [const]`.
    let zero_checked =
        |d: usize| masked.get(&d).is_some_and(|outs| outs.iter().any(|m| eq_zero.contains(m)));

    // Keep the squared values whose `v == v²` is actually constrained.
    square_of
        .iter()
        .filter_map(|(&v, &s)| {
            let constrained =
                eq_set.contains(&(v, s)) || sub_out.get(&(v, s)).is_some_and(|&d| zero_checked(d));
            constrained.then_some(v)
        })
        .collect()
}

/// Detects bit decompositions. A value is split lsb-first by repeated halving:
/// `[rnext] = ([r] - [b]) x half` with `b` a guessed bit. Fills `idiom` with `Bit`.
fn detect_bits(
    c: &CircuitEx<'_>,
    const_values: &HashMap<usize, QM31>,
    idiom: &mut HashMap<usize, Idiom>,
) {
    let bits = find_bits(c, const_values);

    // chain: r -> (b, (r - b) x (1/2)).
    // The variable representing (r - b) x (1/2) is added to `is_next`.
    let mut chain: HashMap<usize, (usize, usize)> = HashMap::new();
    let mut is_next: HashSet<usize> = HashSet::new();
    for g in &c.arith {
        if !(g.op == Op::Pointwise && const_values.get(&g.b) == Some(&simd_half())) {
            continue;
        }
        let diff = g.a;
        if let Some(&(Op::Sub, r, b)) = c.producers.get(&diff)
            && c.guessed.contains(&b)
        {
            chain.insert(r, (b, g.out));
            is_next.insert(g.out);
        }
    }

    for &value in chain.keys() {
        // If it's not the root of a chain, skip it.
        if is_next.contains(&value) {
            continue;
        }
        // Every bit on the chain, and the final running value, must be bit-constrained.
        let mut cur = value;
        while let Some(&(b, next)) = chain.get(&cur) {
            assert!(bits.contains(&b), "chain bit [{b}] (value [{value}]) is not a bit");
            idiom.insert(b, Idiom::Bit { value });
            cur = next;
        }
        assert!(
            bits.contains(&cur),
            "top of bit-decomposition chain (value [{value}]) [{cur}] is not a bit"
        );
        idiom.insert(cur, Idiom::Bit { value });
    }
}

/// Detects the `div(1, Y)` idiom: `[1] = [X] * [Y]` with `X` guessed, so `X = 1/Y`. Fills `idiom`
/// with `Inverse`.
fn detect_inverses(
    c: &CircuitEx<'_>,
    const_values: &HashMap<usize, QM31>,
    idiom: &mut HashMap<usize, Idiom>,
) {
    for g in &c.arith {
        if g.op != Op::Mul || const_values.get(&g.out) != Some(&QM31::one()) {
            continue;
        }
        let (output, input) = if c.guessed.contains(&g.b) {
            (g.b, g.a)
        } else if c.guessed.contains(&g.a) {
            (g.a, g.b)
        } else {
            continue;
        };
        idiom.entry(output).or_insert(Idiom::Inverse { y: input });
    }
}

/// Variables reachable from `root` through `+`/`-`/`*`, skipping constants.
/// Every reachable variable is returned, including inner nodes.
fn add_mul_cone(
    c: &CircuitEx<'_>,
    const_values: &HashMap<usize, QM31>,
    root: usize,
) -> HashSet<usize> {
    let mut res = HashSet::new();
    let mut seen = HashSet::new();
    let mut stack = vec![root];
    while let Some(v) = stack.pop() {
        if const_values.contains_key(&v) {
            continue;
        }
        let is_first_visit = seen.insert(v);
        if !is_first_visit {
            continue;
        }
        // `v != a` keeps a guessed leaf `[v] = [v] + [0]` from expanding into itself.
        if let Some(&(Op::Add | Op::Sub | Op::Mul, a, b)) = c.producers.get(&v)
            && v != a
        {
            stack.push(a);
            stack.push(b);
            res.insert(v);
        } else {
            // A leaf.
            res.insert(v);
        }
    }
    res
}

// Groundedness.

/// Computes groundedness against a challenge. A value is considered grounded to a challenge if it
/// is in the challenge's dependency closure or it is a deterministic function of such values.
/// The challenge is not considered grounded to itself.
struct Ground<'a> {
    c: &'a CircuitEx<'a>,
    const_values: &'a HashMap<usize, QM31>,
    idiom: &'a HashMap<usize, Idiom>,
}

/// Cache for [`Ground::grounded`] for a single challenge.
struct GroundingCache {
    data: HashMap<usize, bool>,
}

impl Ground<'_> {
    /// Returns variables that `root` depends on, including `root`.
    /// Note that it's possible that only a subset of the variables will be returned.
    fn closure(&self, root: usize) -> HashSet<usize> {
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        while let Some(v) = stack.pop() {
            let is_first_visit = seen.insert(v);
            if is_first_visit {
                stack.extend(self.gate_inputs(v).unwrap_or_default());
            }
        }
        seen
    }

    /// Returns the input variables of the gate that produces `v`, or `None` if not known.
    /// These inputs are considered grounded to `v`.
    fn gate_inputs(&self, v: usize) -> Option<Vec<usize>> {
        if let Some(&(_, a, b)) = self.c.producers.get(&v) {
            Some(vec![a, b])
        } else if let Some(&inp) = self.c.m31_to_u32_map.get(&v) {
            Some(vec![inp])
        } else if let Some(&inputs) = self.c.blake_g_map.get(&v) {
            Some(inputs.to_vec())
        } else if let Some(&inputs) = self.c.triple_xor_map.get(&v) {
            Some(inputs.to_vec())
        } else {
            None
        }
    }

    /// Returns a list of variables that `v` deterministically depends on, or `None` on failure.
    /// If every one of them is grounded, so is `v`.
    ///
    /// This differs from [`Self::gate_inputs`] for idioms. For example, a bit is grounded once its
    /// base value is, yet if the bit is grounded it doesn't follow that the value is.
    /// A guess is considered a leaf.
    fn grounding_inputs(&self, v: usize) -> Option<Vec<usize>> {
        match self.idiom.get(&v) {
            Some(Idiom::Bit { value }) => Some(vec![*value]),
            Some(Idiom::Inverse { y }) => Some(vec![*y]),
            None => {
                if self.c.guessed.contains(&v) {
                    None
                } else {
                    self.gate_inputs(v)
                }
            }
        }
    }

    /// A cache for [`Self::grounded`] for a single challenge: its dependency closure and
    /// the constants are grounded (`true`); the challenge itself is not (`false`).
    fn init_grounding_cache(&self, challenge: usize) -> GroundingCache {
        let closure = self.closure(challenge);
        let mut cache: HashMap<usize, bool> = closure.iter().map(|&v| (v, true)).collect();
        cache.extend(self.const_values.keys().map(|&v| (v, true)));
        cache.insert(challenge, false);
        GroundingCache { data: cache }
    }

    /// Whether `root` is a deterministic function of the challenge's dependencies. `cache` must be
    /// seeded by [`Self::init_grounding_cache`]; it caches results for that challenge across calls.
    fn grounded(&self, root: usize, cache: &mut GroundingCache) -> bool {
        let mut stack = vec![root];
        let cache = &mut cache.data;
        while let Some(v) = stack.pop() {
            if cache.contains_key(&v) {
                continue;
            }
            let Some(inputs) = self.grounding_inputs(v) else {
                cache.insert(v, false);
                continue;
            };
            // Check if there are children that were not resolved yet.
            let pending: Vec<usize> =
                inputs.iter().copied().filter(|i| !cache.contains_key(i)).collect();
            if pending.is_empty() {
                // All children were resolved. Update the status of `v`.
                let grounded = inputs.iter().all(|i| cache[i]);
                cache.insert(v, grounded);
            } else {
                // Re-add v to the stack. We will return to it once all its children were resolved.
                stack.push(v);
                stack.extend(pending);
            }
        }
        cache[&root]
    }
}

// Sum tree and classification.

/// Expands the add/sub tree at `root` into its leaf summands (left to right). A leaf is any
/// variable not produced by a genuine `+`/`-` gate; zero-valued leaves are dropped.
fn find_summands(
    c: &CircuitEx<'_>,
    const_values: &HashMap<usize, QM31>,
    root: usize,
) -> Vec<usize> {
    let mut summands = Vec::new();
    let mut stack = vec![root];
    while let Some(v) = stack.pop() {
        if let Some(&(Op::Add | Op::Sub, a, b)) = c.producers.get(&v)
            && v != a
            && v != b
        {
            // Push `b` first, so the left child expands first.
            stack.push(b);
            stack.push(a);
        } else if const_values.get(&v) != Some(&QM31::zero()) {
            // A non-zero leaf.
            summands.push(v);
        }
    }
    summands
}

/// Validates a logup summand `s`, asserting the expected groundedness of its values.
///
/// Arguments:
/// - `composition_vars`: the `composition_eval` cone.
/// - `int_z`: the `interaction_z` variable index.
/// - `int_z_cache`: Cache for [`Ground::grounded`] for `int_z`.
/// - `composition_coef_cache`: Cache for [`Ground::grounded`] for `composition_coef`.
fn validate_logup_summand(
    ground: &Ground<'_>,
    int_z: usize,
    int_z_cache: &mut GroundingCache,
    composition_coef_cache: &mut GroundingCache,
    composition_vars: &HashSet<usize>,
    s: usize,
) {
    let c = ground.c;

    // Case I: s = 1 / (X - int_z).
    if let Some(&Idiom::Inverse { y: denom }) = ground.idiom.get(&s)
        && let Some(&(Op::Sub, x, y)) = c.producers.get(&denom)
    {
        assert!(y == int_z, "Summand [{s}] is 1/(X-Y) with Y=[{y}], expected Y=[{int_z}].");
        // Check that X is grounded to int_z.
        assert!(
            ground.grounded(x, int_z_cache),
            "Summand [{s}] = 1 / ([{x}] - [{y}]) is not grounded."
        );
        return;
    }

    // Case II: s is a guess.
    if c.guessed.contains(&s) {
        // Make sure s participates in the constraints.
        assert!(composition_vars.contains(&s));
        // Make sure s is grounded to composition_coef.
        assert!(ground.grounded(s, composition_coef_cache));
        return;
    }

    panic!("Summand [{s}] is not a valid logup summand.");
}

/// Analyzes the circuit and writes the classified logup-sum summands to `summands.txt`.
pub fn analyze(circuit: &Circuit, debug_info: &HashMap<String, Var>) {
    let c = build_analysis(circuit);
    let const_values = propagate_constants(&c);

    // Recognize idioms layered over the raw producers.
    let mut idiom: HashMap<usize, Idiom> = HashMap::new();
    detect_bits(&c, &const_values, &mut idiom);
    detect_inverses(&c, &const_values, &mut idiom);

    let ground = Ground { c: &c, const_values: &const_values, idiom: &idiom };

    // Seed each challenge's `grounded` cache with its dependency closure.
    let int_z = debug_info["interaction_z"].idx;
    let mut int_z_cache = ground.init_grounding_cache(int_z);

    let composition_coef = debug_info["composition_polynomial_coeff"].idx;
    let mut composition_coef_cache = ground.init_grounding_cache(composition_coef);

    let logup_sum = debug_info["logup_sum"].idx;
    let summands = find_summands(&c, &const_values, logup_sum);

    let composition_eval = debug_info["composition_eval"].idx;
    let composition_vars = add_mul_cone(&c, &const_values, composition_eval);

    for s in summands {
        validate_logup_summand(
            &ground,
            int_z,
            &mut int_z_cache,
            &mut composition_coef_cache,
            &composition_vars,
            s,
        );
    }
}
