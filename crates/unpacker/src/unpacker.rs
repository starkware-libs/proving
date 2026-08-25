//! Merkle "unpacker" circuit — unbalanced commitment via a fixed, depth-independent topology.
//!
//! Proves a Merkle `root` commits to a whole (possibly unbalanced) tree of [`Node`]s, each pairing
//! a `circuit_hash` — which circuit produced the entry — with a `subtree_hash`. The converse of an
//! authentication-path check: it opens the entire leaf collection, not a single leaf.
//!
//! Binding is up to permutation: leaf slots enter only as an unordered collection and no gate reads
//! a slot's index, so permuting the returned `leaves` — padding included — satisfies the same
//! circuit against the same root. A caller needing a specific leaf *order* must bind it separately.
//!
//! # Node hashing
//!
//! An internal node's `subtree_hash` binds *both* fields of *both* children, mirroring the
//! multiverifier's `(circuit_hash ‖ output)` preimage layout:
//!
//! ```text
//! subtree_hash = blake2s( left.circuit_hash ‖ left.subtree_hash ‖ right.circuit_hash ‖ right.subtree_hash )
//! ```
//!
//! A node's own `circuit_hash` is witness, bound when its parent consumes it; the root's is bound
//! by the caller. So the root commits transitively to every hash in the tree.
//!
//! Every `circuit_hash` is also checked at the slot that produces it, against a caller-supplied
//! whitelist — one for leaves, another for internal nodes. Padding slots are no exception: they
//! carry a real set member, since padding is marked by a zero `subtree_hash` alone.
//!
//! # Design (wiring-by-multiset)
//!
//! Built for a fixed **leaf-slot capacity** `L` (a power of two), handling any `1 ≤ n ≤ L` by
//! padding with the all-zero node `Z`. There are always `L` leaf slots and `M = L - 1` node slots,
//! so the gate structure depends only on `L` — never on `n`, shape, or depth; the caller's
//! [`BinaryTree`] drives witness values only.
//!
//! Each node slot takes two children plus its own `circuit_hash` as witness and applies a
//! **copy-up rule**:
//!
//! ```text
//! out = if is_zero(right) { left } else { Node { circuit_hash, subtree_hash: hash(left, right) } }
//! ```
//!
//! One multiset identity (via [`permute_tuples`]) ties every produced node — a whole
//! `(circuit_hash, subtree_hash)` pair — to its single consumption. The root is the one unconsumed
//! output, held back and **returned** for the caller to bind:
//!
//! ```text
//! inputs  (consumed) = lefts ++ rights              (length 2M = 2(L-1))
//! outputs (produced) = leaves ++ outs[..M-1]        (length L + M - 1 = 2(L-1))
//! ```
//!
//! Each real leaf cancels with its consumption and each padding `Z` is dropped by a node whose
//! `right = Z` (so `out = left`). Both sides reduce to ∅ iff the witness describes a valid Merkle
//! tree over exactly the `n` real leaves.

use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue, blake2s_u32s};
use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, add, eq, guess, mul};
use circuits::wrappers::U32Wrapper;
use itertools::zip_eq;
use stwo::core::fields::qm31::QM31;

use crate::permutation::permute_tuples;
use crate::tree::BinaryTree;

#[cfg(test)]
#[path = "unpacker_test.rs"]
mod test;

/// A tree entry: `circuit_hash` (which circuit produced this position) paired with `subtree_hash`
/// (the Merkle hash of the subtree rooted here). Both are eight-word [`HashValue`]s.
///
/// `T` is the per-element type: [`Var`] inside a [`Context`], or `QM31` for concrete values.
#[derive(Clone)]
pub struct Node<T> {
    pub circuit_hash: HashValue<T>,
    pub subtree_hash: HashValue<T>,
}

impl<Value: IValue> Guess<Value> for Node<Value> {
    type Target = Node<Var>;
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Node {
            circuit_hash: self.circuit_hash.guess(context),
            subtree_hash: self.subtree_hash.guess(context),
        }
    }
}

impl Node<Var> {
    /// The node's sixteen words in `circuit_hash ‖ subtree_hash` order — the atomic tuple permuted
    /// by [`permute_tuples`] and hashed by [`hash_node`].
    fn words(&self) -> Vec<U32Wrapper<Var>> {
        self.circuit_hash.iter().chain(self.subtree_hash.iter()).copied().collect()
    }
}

/// The leaf-slot capacity and the two whitelists `circuit_hash`es are checked against.
///
/// All three fix the emitted circuit: two trees built with equal `RecursionTreeParams` emit
/// byte-identical circuits, and any change here changes the circuit. Both set lengths are
/// gate-count parameters.
///
/// The fields are private and validated in [`RecursionTreeParams::new`], so holding an instance
/// already guarantees a power-of-two capacity and two non-empty whitelists.
pub struct RecursionTreeParams<'a> {
    capacity: usize,
    leaf_set: &'a [HashValue<QM31>],
    internal_node_set: &'a [HashValue<QM31>],
}

impl<'a> RecursionTreeParams<'a> {
    /// Panics unless `capacity` is a power of two and both whitelists are non-empty.
    pub fn new(
        capacity: usize,
        leaf_set: &'a [HashValue<QM31>],
        internal_node_set: &'a [HashValue<QM31>],
    ) -> Self {
        assert!(capacity.is_power_of_two(), "capacity must be a power of two, got {capacity}");
        assert!(!leaf_set.is_empty(), "the allowed leaf circuit-hash set must be non-empty");
        assert!(
            !internal_node_set.is_empty(),
            "the allowed internal-node circuit-hash set must be non-empty"
        );
        Self { capacity, leaf_set, internal_node_set }
    }

    /// Leaf-slot capacity `L`, a power of two and an upper bound on the leaf count.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Whitelist of circuit hashes for leaf slots.
    pub fn leaf_set(&self) -> &'a [HashValue<QM31>] {
        self.leaf_set
    }

    /// Whitelist of circuit hashes for internal node slots — one per registry multiverifier.
    pub fn internal_node_set(&self) -> &'a [HashValue<QM31>] {
        self.internal_node_set
    }
}

/// Output of [`unpack_recursion_tree`], holding the computed root [`Node`] together with the
/// padded leaf nodes.
pub struct UnpackedRecursionTree {
    pub root: Node<Var>,
    pub leaves: Vec<Node<Var>>,
}

/// Word-wise `if_zero + selector * (if_one - if_zero)`. Assumes `selector` is `0` or `1`.
fn select_hash<Value: IValue>(
    context: &mut Context<Value>,
    selector: Var,
    if_zero: &HashValue<Var>,
    if_one: &HashValue<Var>,
) -> HashValue<Var> {
    HashValue(std::array::from_fn(|i| {
        let diff = circuits::eval!(context, (*if_one[i].get()) - (*if_zero[i].get()));
        let result = circuits::eval!(context, (*if_zero[i].get()) + ((selector) * (diff)));
        U32Wrapper::new_unsafe(result)
    }))
}

/// Reads the concrete value of a [`HashValue<Var>`] back out of the context.
fn hash_value_of<Value: IValue>(
    context: &Context<Value>,
    hash: &HashValue<Var>,
) -> HashValue<Value> {
    HashValue(std::array::from_fn(|i| U32Wrapper::new_unsafe(context.get(*hash[i].get()))))
}

/// Reads the concrete value of a [`Node<Var>`] back out of the context.
fn node_value_of<Value: IValue>(context: &Context<Value>, node: &Node<Var>) -> Node<Value> {
    Node {
        circuit_hash: hash_value_of(context, &node.circuit_hash),
        subtree_hash: hash_value_of(context, &node.subtree_hash),
    }
}

/// Guesses a fresh all-zero hash `Z`.
///
/// The words are *distinct guessed variables*, not the canonical zero constant (`Var { idx: 0 }`):
/// [`add`]/[`mul`] constant-fold the canonical zero away, so padding built from it would emit fewer
/// gates than a real slot and leak `n` into the gate count.
///
/// Soundness is unaffected — this `subtree_hash` is unconstrained witness, but any non-zero value a
/// prover puts here makes the slot a produced node the multiset must consume, so it is hashed in
/// and changes the root. (The padding slot's `circuit_hash` is *not* free: like every other slot's,
/// it is checked against the whitelist.)
fn guess_zero_hash<Value: IValue>(context: &mut Context<Value>) -> HashValue<Var> {
    guess_hash_value(context, &zero_hash())
}

/// The all-zero `subtree_hash` that marks a padding leaf slot.
fn zero_hash() -> HashValue<QM31> {
    HashValue::from([0u32; BLAKE2S_DIGEST_N_WORDS])
}

/// Guesses a concrete [`HashValue<QM31>`] as witness, lifting each word through
/// [`IValue::from_qm31`]. Each word is range-constrained like any other [`HashValue`] guess.
fn guess_hash_value<Value: IValue>(
    context: &mut Context<Value>,
    hash: &HashValue<QM31>,
) -> HashValue<Var> {
    let value: HashValue<Value> = HashValue(std::array::from_fn(|i| {
        U32Wrapper::new_unsafe(Value::from_qm31(*hash[i].get()))
    }));
    value.guess(context)
}

/// Guesses a concrete [`Node<QM31>`] (both hashes) as witness. See [`guess_hash_value`].
fn guess_node<Value: IValue>(context: &mut Context<Value>, node: &Node<QM31>) -> Node<Var> {
    Node {
        circuit_hash: guess_hash_value(context, &node.circuit_hash),
        subtree_hash: guess_hash_value(context, &node.subtree_hash),
    }
}

/// Returns a `0/1` selector that is `1` iff `node`'s `subtree_hash` is all-zero — the padding
/// marker. The `circuit_hash` is deliberately *not* read: padding slots carry a real whitelist
/// member there, so that a slot's membership check needs no exemption.
///
/// The eight `subtree_hash` words are summed into `acc`. Every word is range-constrained to a
/// `u32` packing `(low_u16, high_u16, 0, 0)` (see [`HashValue::guess`]), so each coordinate of the
/// sum stays below `8 · 2^16 < M31::P` and cannot wrap: `acc == 0` iff every word is zero.
///
/// `is_zero` is pinned to `[acc == 0]` by the standard gadget with witness `inv_or_zero`:
/// * `acc * is_zero == 0` forces it false whenever `acc != 0`;
/// * `acc * inv_or_zero + is_zero == 1` forces it true whenever `acc == 0`.
///
/// That makes it a deterministic function of `node`, so a prover cannot steer the copy-up rule.
///
/// Note the guarantee this gives up against summing all sixteen words: a *whitelisted* leaf whose
/// `subtree_hash` a prover sets to zero is still dropped, and still appears in the returned
/// `leaves`. Callers must treat a zero `subtree_hash` as padding.
fn is_zero_node<Value: IValue>(context: &mut Context<Value>, node: &Node<Var>) -> Var {
    let zero = context.zero();
    let one = context.one();

    let words = &node.subtree_hash;
    let acc = words.iter().skip(1).fold(*words[0].get(), |acc, w| add(context, acc, *w.get()));

    let acc_val = context.get(acc);
    let is_zero = acc_val == Value::from_qm31(qm31_from_u32s(0, 0, 0, 0));
    let (is_zero_val, inv_or_zero_val) = if is_zero {
        (Value::from_qm31(qm31_from_u32s(1, 0, 0, 0)), Value::from_qm31(qm31_from_u32s(0, 0, 0, 0)))
    } else {
        (
            Value::from_qm31(qm31_from_u32s(0, 0, 0, 0)),
            Value::from_qm31(qm31_from_u32s(1, 0, 0, 0)) / acc_val,
        )
    };
    let is_zero = guess(context, is_zero_val);
    let inv_or_zero = guess(context, inv_or_zero_val);

    // acc * is_zero == 0
    let acc_is_zero = mul(context, acc, is_zero);
    eq(context, acc_is_zero, zero);
    // acc * inv_or_zero + is_zero == 1
    let acc_inv_or_zero = mul(context, acc, inv_or_zero);
    let acc_inv_or_zero_plus_is_zero = add(context, acc_inv_or_zero, is_zero);
    eq(context, acc_inv_or_zero_plus_is_zero, one);

    is_zero
}

/// Guesses `len` flags that are `1` at `hot` and `0` elsewhere, constrained to be genuinely
/// one-hot: `flag² == flag` pins each to `{0, 1}` and `Σ flags == 1` admits exactly one (`len < P`,
/// so the sum cannot wrap).
fn guess_one_hot<Value: IValue>(context: &mut Context<Value>, len: usize, hot: usize) -> Vec<Var> {
    let one = context.one();
    let zero_value = Value::from_qm31(qm31_from_u32s(0, 0, 0, 0));
    let one_value = Value::from_qm31(qm31_from_u32s(1, 0, 0, 0));

    let flags: Vec<Var> = (0..len)
        .map(|i| {
            let flag = guess(context, if i == hot { one_value } else { zero_value });
            let squared = mul(context, flag, flag);
            eq(context, squared, flag);
            flag
        })
        .collect();

    // The fold starts at the canonical zero, which `add` folds away.
    let zero = context.zero();
    let total = flags.iter().fold(zero, |acc, &flag| add(context, acc, flag));
    eq(context, total, one);

    flags
}

/// A [`guess_one_hot`] flag per candidate selects the match, and word-wise
/// `hash[j] == Σ_i is_match[i] * allowed[i][j]` collapses to that candidate's word.
///
/// The comparison is per word, never a summed difference: word differences can be negative and
/// cancel across words, so [`is_zero_node`]'s no-wrap argument does not survive a subtraction.
///
/// The gates do not depend on *which* candidate matches, so the circuit stays fixed across
/// witnesses. A single-candidate set takes the cheaper [`constrain_hash_eq`] path.
fn constrain_hash_in_set<Value: IValue>(
    context: &mut Context<Value>,
    hash: &HashValue<Var>,
    allowed: &[HashValue<QM31>],
) {
    assert!(!allowed.is_empty(), "the allowed circuit-hash set must be non-empty");
    if let [only] = allowed {
        return constrain_hash_eq(context, hash, only);
    }
    // Witness only: the index of the matching candidate.
    let matching = allowed
        .iter()
        .position(|candidate| {
            candidate
                .iter()
                .enumerate()
                .all(|(j, word)| context.get(*hash[j].get()) == Value::from_qm31(*word.get()))
        })
        .expect("circuit hash is not in the allowed set");

    let is_match = guess_one_hot(context, allowed.len(), matching);

    // Bind each word to the matched candidate's word.
    for j in 0..BLAKE2S_DIGEST_N_WORDS {
        let mut selected = {
            let word = context.constant(*allowed[0][j].get());
            mul(context, is_match[0], word)
        };
        for (&flag, candidate) in zip_eq(&is_match, allowed).skip(1) {
            let word = context.constant(*candidate[j].get());
            let term = mul(context, flag, word);
            selected = add(context, selected, term);
        }
        eq(context, *hash[j].get(), selected);
    }
}

/// Constrains `hash` to equal the single constant `expected`.
///
/// The one-candidate specialization of [`constrain_hash_in_set`]: with nothing to select between,
/// the one-hot flags and their booleanity constraints all vanish, leaving one equality per word.
fn constrain_hash_eq<Value: IValue>(
    context: &mut Context<Value>,
    hash: &HashValue<Var>,
    expected: &HashValue<QM31>,
) {
    for j in 0..BLAKE2S_DIGEST_N_WORDS {
        let word = context.constant(*expected[j].get());
        eq(context, *hash[j].get(), word);
    }
}

/// Hashes `left.circuit_hash ‖ left.subtree_hash ‖ right.circuit_hash ‖ right.subtree_hash` — four
/// eight-word hashes, 128 bytes. Each word is already a Blake2s output, so the 32 words feed
/// straight in as message words with no unpacking.
fn hash_node<Value: IValue>(
    context: &mut Context<Value>,
    left: &Node<Var>,
    right: &Node<Var>,
) -> HashValue<Var> {
    let mut words: Vec<U32Wrapper<Var>> = Vec::with_capacity(32);
    words.extend(left.words());
    words.extend(right.words());
    blake2s_u32s(context, words, 128)
}

/// Applies the copy-up rule:
/// `out = is_zero(right) ? left : Node { circuit_hash, subtree_hash: hash(left, right) }`.
///
/// [`hash_node`] is always emitted — the topology is fixed — and [`is_zero_node`] selects the
/// copied-up `left` (both fields) on padding edges.
///
/// The supplied `circuit_hash` must lie in `internal_node_set` unconditionally: a copy-up slot
/// discards it (the select takes `left.circuit_hash`), so padding can simply carry a real set
/// member.
fn handle_node<Value: IValue>(
    context: &mut Context<Value>,
    circuit_hash: &HashValue<Var>,
    left: &Node<Var>,
    right: &Node<Var>,
    internal_node_set: &[HashValue<QM31>],
) -> Node<Var> {
    let hashed = hash_node(context, left, right);
    let right_is_zero = is_zero_node(context, right);
    constrain_hash_in_set(context, circuit_hash, internal_node_set);
    // selector = 1 -> copy `left` (both fields); selector = 0 -> use the supplied `circuit_hash`
    // and the genuine hash.
    Node {
        circuit_hash: select_hash(context, right_is_zero, circuit_hash, &left.circuit_hash),
        subtree_hash: select_hash(context, right_is_zero, &hashed, &left.subtree_hash),
    }
}

/// One emitted node slot: the two child nodes it consumes and the node it produces. Across all
/// slots these are the two sides of the multiset identity.
struct NodeSlot {
    left: Node<Var>,
    right: Node<Var>,
    out: Node<Var>,
}

/// Emits one node slot: guesses fresh `left`/`right`/`circuit_hash` witnesses, derives `out` by the
/// copy-up rule, and records the slot. Children tie to their producers only through the multiset,
/// never topologically.
fn push_node<Value: IValue>(
    context: &mut Context<Value>,
    circuit_hash: &HashValue<QM31>,
    left: &Node<Var>,
    right: &Node<Var>,
    internal_node_set: &[HashValue<QM31>],
    slots: &mut Vec<NodeSlot>,
) -> Node<Var> {
    let left = node_value_of(context, left).guess(context);
    let right = node_value_of(context, right).guess(context);
    let circuit_hash = guess_hash_value(context, circuit_hash);
    let out = handle_node(context, &circuit_hash, &left, &right, internal_node_set);
    slots.push(NodeSlot { left, right, out: out.clone() });
    out
}

/// Emits `tree`'s internal nodes depth-first into `slots` (`n - 1` for `n` leaves, any shape),
/// returning the resolved root.
///
/// Leaves are drawn from `leaves`, guessed up front, in this walk's order. Guessing them inline
/// would interleave leaf and node allocations shape-dependently; emitting only nodes here keeps the
/// variable layout identical across all tree shapes of a given `capacity`.
fn build_tree<Value: IValue>(
    context: &mut Context<Value>,
    tree: &BinaryTree<Node<QM31>, HashValue<QM31>>,
    leaves: &mut impl Iterator<Item = Node<Var>>,
    internal_node_set: &[HashValue<QM31>],
    slots: &mut Vec<NodeSlot>,
) -> Node<Var> {
    match tree {
        BinaryTree::Leaf(_) => leaves.next().expect("fewer guessed leaves than the tree holds"),
        BinaryTree::Internal(circuit_hash, children) => {
            let left = build_tree(context, &children[0], leaves, internal_node_set, slots);
            let right = build_tree(context, &children[1], leaves, internal_node_set, slots);
            push_node(context, circuit_hash, &left, &right, internal_node_set, slots)
        }
    }
}

/// Proves that [`UnpackedRecursionTree::leaves`] are — as a multiset — the leaves of a recursion
/// tree whose Merkle root is [`UnpackedRecursionTree::root`], and that every `circuit_hash` in it
/// names a circuit the caller allows. Their order and the padding positions are *not* bound; see
/// the module docs.
///
/// `tree` supplies the leaf values, each internal node's `circuit_hash`, and the shape; the emitted
/// circuit depends only on [`RecursionTreeParams`].
///
/// Each `circuit_hash` is checked at the slot that *produces* it — a leaf's against
/// [`RecursionTreeParams::leaf_set`], an internal node's against
/// [`RecursionTreeParams::internal_node_set`]. Padding slots are checked too, against an arbitrary
/// set member.
///
/// That marker is the caller's obligation: a slot whose `subtree_hash` is zero is dropped by
/// copy-up yet still appears in the returned `leaves`, so a prover can place a whitelisted
/// `circuit_hash` beside a zero `subtree_hash` and have that entry show up uncommitted. **Callers
/// must treat a zero `subtree_hash` in `leaves` as padding** rather than as a committed leaf.
pub fn unpack_recursion_tree<Value: IValue>(
    context: &mut Context<Value>,
    tree: &BinaryTree<Node<QM31>, HashValue<QM31>>,
    params: &RecursionTreeParams<'_>,
) -> UnpackedRecursionTree {
    let (capacity, leaf_set, internal_node_set) =
        (params.capacity(), params.leaf_set(), params.internal_node_set());
    let leaf_values = tree.leaves();
    let n = leaf_values.len();
    assert!(n >= 1, "a Merkle tree must have at least one leaf");
    assert!(n <= capacity, "got {n} leaves but capacity is only {capacity}");

    // Guess all `capacity` leaf nodes up front (real then padding) — see `build_tree` for why.
    let real_leaves: Vec<Node<Var>> =
        leaf_values.iter().map(|node| guess_node(context, node)).collect();
    let pads: Vec<Node<Var>> = (n..capacity)
        .map(|_| Node {
            circuit_hash: guess_hash_value(context, &leaf_set[0]),
            subtree_hash: guess_zero_hash(context),
        })
        .collect();
    let leaf_slots: Vec<Node<Var>> = real_leaves.iter().chain(pads.iter()).cloned().collect();

    // Every leaf slot names an allowed circuit, padding included — padding is marked by a zero
    // `subtree_hash`, not by a zero `circuit_hash`. Checking all `capacity` slots in fixed order
    // keeps the emitted gates independent of `n`.
    for leaf in &leaf_slots {
        constrain_hash_in_set(context, &leaf.circuit_hash, leaf_set);
    }

    let mut slots: Vec<NodeSlot> = Vec::new();

    // Emit the tree's `n - 1` internal nodes, then drop each padding `Z` with a copy-up node
    // (`right = Z`, so `out = left`), chaining the real root up: `capacity - 1` uniform slots
    // either way. A copy-up node discards its `circuit_hash`, so any `internal_node_set` member
    // does.
    let pad_circuit_hash = &internal_node_set[0];
    let mut leaves = real_leaves.iter().cloned();
    let mut chain = build_tree(context, tree, &mut leaves, internal_node_set, &mut slots);
    assert!(leaves.next().is_none(), "the tree walk did not consume every guessed leaf");
    for pad in &pads {
        chain = push_node(context, pad_circuit_hash, &chain, pad, internal_node_set, &mut slots);
    }

    // The consumed children must equal, as a multiset, the leaves plus the outputs — minus the
    // root, the one produced node with no consumption, which is popped off and returned for the
    // caller to bind. Each node is permuted as a whole sixteen-word pair, tying both fields.
    let inputs: Vec<Vec<U32Wrapper<Var>>> = slots
        .iter()
        .map(|slot| slot.left.words())
        .chain(slots.iter().map(|slot| slot.right.words()))
        .collect();
    let mut outputs: Vec<Node<Var>> =
        leaf_slots.iter().cloned().chain(slots.iter().map(|slot| slot.out.clone())).collect();
    let root = outputs.pop().expect("a Merkle tree must have at least one leaf");
    let output_tuples: Vec<Vec<U32Wrapper<Var>>> = outputs.iter().map(Node::words).collect();
    permute_tuples(context, &inputs, &output_tuples);
    UnpackedRecursionTree { root, leaves: leaf_slots }
}
