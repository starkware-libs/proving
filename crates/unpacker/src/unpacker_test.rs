use circuits::blake::HashValue;
use circuits::circuit::Circuit;
use circuits::context::{Context, TraceContext, Var};
use circuits::ivalue::{IValue, NoValue, qm31_from_u32s};
use circuits::ops::{Guess, eq};
use circuits::wrappers::U32Wrapper;
use rstest::rstest;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

use crate::tree::BinaryTree;
use crate::unpacker::{Node, unpack_recursion_tree};

/// The tree the unpacker consumes: leaves carry a full [`Node`], internal nodes only their supplied
/// circuit hash.
type InputTree = BinaryTree<Node<QM31>, HashValue<QM31>>;

/// Reference single-`QM31` leaf hash, used only to manufacture distinct `subtree_hash` values.
fn hash_leaf_ref(value: QM31) -> Blake2sHash {
    let bytes: Vec<u8> = value.to_m31_array().iter().flat_map(|m| m.0.to_le_bytes()).collect();
    Blake2sHasher::hash(&bytes)
}

/// A distinct, nonzero circuit hash for leaf `i`.
fn leaf_circuit_hash(i: u32) -> HashValue<QM31> {
    let words: [u32; 8] = std::array::from_fn(|k| 0x2000_0000 + i * 8 + k as u32);
    HashValue::from(words)
}

/// A distinct, nonzero circuit hash for the internal node numbered `*counter`, then bumps it.
fn node_circuit_hash(counter: &mut u32) -> HashValue<QM31> {
    let c = *counter;
    *counter += 1;
    let words: [u32; 8] = std::array::from_fn(|k| 0x1000_0000 + c * 8 + k as u32);
    HashValue::from(words)
}

/// Out-of-circuit twin of the node hash: `blake2s` over the children's four eight-word hashes,
/// each word little-endian.
fn hash_node_ref(left: &Node<QM31>, right: &Node<QM31>) -> HashValue<QM31> {
    let mut hasher = Blake2sHasher::new();
    for hash in [&left.circuit_hash, &left.subtree_hash, &right.circuit_hash, &right.subtree_hash] {
        for word in hash.iter() {
            hasher.update(&word.get().unpack_u32().to_le_bytes());
        }
    }
    HashValue::from(hasher.finalize())
}

/// `n` distinct leaf [`Node`]s, each with its own circuit hash and node hash.
fn example_leaves(n_leaves: u32) -> Vec<Node<QM31>> {
    (0..n_leaves)
        .map(|i| {
            let value = qm31_from_u32s(i + 1, i + 2, i + 3, i + 4);
            Node {
                circuit_hash: leaf_circuit_hash(i),
                subtree_hash: HashValue::from(hash_leaf_ref(value)),
            }
        })
        .collect()
}

/// The canonical balanced Merkle tree over `leaves`: pair adjacent siblings, carry a lone odd node
/// up unchanged, hash by [`hash_node_ref`]. Returns the tree and its root — the convention the
/// unpacker circuit must reproduce.
fn build_balanced_tree(leaves: &[Node<QM31>]) -> (InputTree, Node<QM31>) {
    let mut counter = 0u32;
    let mut layer: Vec<(InputTree, Node<QM31>)> =
        leaves.iter().cloned().map(|leaf| (BinaryTree::Leaf(leaf.clone()), leaf)).collect();
    while layer.len() > 1 {
        let mut next: Vec<(InputTree, Node<QM31>)> = layer
            .chunks_exact(2)
            .map(|p| {
                let circuit_hash = node_circuit_hash(&mut counter);
                let subtree_hash = hash_node_ref(&p[0].1, &p[1].1);
                let tree = BinaryTree::Internal(
                    circuit_hash.clone(),
                    Box::new([p[0].0.clone(), p[1].0.clone()]),
                );
                (tree, Node { circuit_hash, subtree_hash })
            })
            .collect();
        if layer.len() % 2 == 1 {
            next.push(layer.last().unwrap().clone());
        }
        layer = next;
    }
    layer.into_iter().next().expect("at least one leaf")
}

/// A *caterpillar*: a maximally unbalanced left-leaning spine, e.g. `(((a,b),c),d)` — depth 3 where
/// a balanced tree is depth 2. The worst case for the depth-independence claim, and a shape a flat
/// leaf list cannot express. Returns the tree and its root, like [`build_balanced_tree`].
fn build_caterpillar_tree(leaves: &[Node<QM31>]) -> (InputTree, Node<QM31>) {
    let mut counter = 0u32;
    let mut tree = BinaryTree::Leaf(leaves[0].clone());
    let mut acc = leaves[0].clone();
    for leaf in &leaves[1..] {
        let circuit_hash = node_circuit_hash(&mut counter);
        let subtree_hash = hash_node_ref(&acc, leaf);
        tree = BinaryTree::Internal(
            circuit_hash.clone(),
            Box::new([tree, BinaryTree::Leaf(leaf.clone())]),
        );
        acc = Node { circuit_hash, subtree_hash };
    }
    (tree, acc)
}

/// Binds two hash values equal word-by-word.
fn bind_hash_eq<Value: IValue>(
    context: &mut Context<Value>,
    a: &HashValue<Var>,
    b: &HashValue<Var>,
) {
    for (x, y) in a.iter().zip(b.iter()) {
        eq(context, *x.get(), *y.get());
    }
}

/// Binds two nodes equal, modelling the caller's obligation to constrain the returned root to the
/// claimed commitment.
fn bind_node_eq<Value: IValue>(context: &mut Context<Value>, a: &Node<Var>, b: &Node<Var>) {
    bind_hash_eq(context, &a.circuit_hash, &b.circuit_hash);
    bind_hash_eq(context, &a.subtree_hash, &b.subtree_hash);
}

/// Re-encodes a [`HashValue<QM31>`] as a `HashValue<Value>`, so one build runs under both.
fn hash_as<Value: IValue>(hash: &HashValue<QM31>) -> HashValue<Value> {
    HashValue(std::array::from_fn(|i| U32Wrapper::new_unsafe(Value::from_qm31(*hash[i].get()))))
}

/// Re-encodes a concrete [`Node<QM31>`] as a `Node<Value>`.
fn node_as<Value: IValue>(node: &Node<QM31>) -> Node<Value> {
    Node { circuit_hash: hash_as(&node.circuit_hash), subtree_hash: hash_as(&node.subtree_hash) }
}

/// Runs the commitment circuit over `tree`, binds its resolved root to the claimed `root`, and
/// returns whether the circuit is satisfied. Checks the *finalized* circuit, so the `x + 0 = x`
/// gates [`Context::finalize`] adds for guessed variables are validated too.
fn verify_commitment(capacity: usize, root: Node<QM31>, tree: InputTree) -> bool {
    let mut context = TraceContext::default();
    let root_var = root.guess(&mut context);
    let computed = unpack_recursion_tree(&mut context, &tree, capacity);
    bind_node_eq(&mut context, &computed.root, &root_var);
    context.finalize(false).is_circuit_valid()
}

#[rstest]
#[case::one_leaf(1)]
#[case::two_leaves(2)]
#[case::three_leaves(3)]
#[case::four_leaves(4)]
#[case::five_leaves(5)]
#[case::seven_leaves(7)]
#[case::eight_leaves(8)]
fn verify_commitment_succeeds_balanced_capacity(#[case] n_leaves: u32) {
    let (tree, root) = build_balanced_tree(&example_leaves(n_leaves));
    let capacity = (n_leaves as usize).next_power_of_two();
    assert!(verify_commitment(capacity, root, tree));
}

/// `n < L`: many padding zeros must be dropped without changing the root.
#[rstest]
#[case::one_pad4(1, 4)]
#[case::one_pad8(1, 8)]
#[case::two_pad8(2, 8)]
#[case::three_pad8(3, 8)]
#[case::five_pad16(5, 16)]
#[case::seven_pad16(7, 16)]
#[case::six_pad8(6, 8)]
fn verify_commitment_succeeds_with_slack_capacity(#[case] n_leaves: u32, #[case] capacity: usize) {
    let (tree, root) = build_balanced_tree(&example_leaves(n_leaves));
    assert!(verify_commitment(capacity, root, tree));
}

/// Repeated leaf nodes: the multiset soundness argument is multiplicity-aware.
#[test]
fn verify_commitment_succeeds_with_duplicate_leaves() {
    let a = Node {
        circuit_hash: leaf_circuit_hash(0),
        subtree_hash: HashValue::from(hash_leaf_ref(qm31_from_u32s(7, 7, 7, 7))),
    };
    let b = Node {
        circuit_hash: leaf_circuit_hash(1),
        subtree_hash: HashValue::from(hash_leaf_ref(qm31_from_u32s(9, 8, 7, 6))),
    };
    // A real leaf adjacent to its duplicate, and a node repeated non-adjacently.
    let leaves = vec![a.clone(), a.clone(), b.clone(), a.clone(), b];
    let (tree, root) = build_balanced_tree(&leaves);
    assert!(verify_commitment(8, root, tree));
}

#[test]
fn verify_commitment_succeeds_for_caterpillar() {
    let (tree, root) = build_caterpillar_tree(&example_leaves(4));
    assert!(verify_commitment(4, root, tree));
}

/// Shape is bound through `root`: a caterpillar witness reconstructs a different root, so it cannot
/// verify against the *balanced* root of the same leaves.
#[test]
fn verify_commitment_fails_on_wrong_shape() {
    let leaves = example_leaves(4);
    let (_, balanced_root) = build_balanced_tree(&leaves);
    // A caterpillar witness cannot reproduce the balanced root of the same leaves.
    let (caterpillar, _) = build_caterpillar_tree(&leaves);
    assert!(!verify_commitment(4, balanced_root, caterpillar));
}

#[test]
fn verify_commitment_fails_on_wrong_root() {
    let (tree, root) = build_balanced_tree(&example_leaves(4));
    // Flip one bit of the root's node hash.
    let mut node_words: [u32; 8] = std::array::from_fn(|i| root.subtree_hash[i].get().unpack_u32());
    node_words[0] ^= 1;
    let wrong_root =
        Node { circuit_hash: root.circuit_hash, subtree_hash: HashValue::from(node_words) };
    assert!(!verify_commitment(4, wrong_root, tree));
}

#[test]
fn verify_commitment_fails_on_tampered_leaf() {
    let (_, root) = build_balanced_tree(&example_leaves(4));

    let mut tampered = example_leaves(4);
    tampered[2].subtree_hash = HashValue::from(hash_leaf_ref(qm31_from_u32s(100, 100, 100, 100)));
    let (tampered_tree, _) = build_balanced_tree(&tampered);

    // Tampered leaves reconstruct a different root than the one committed to.
    assert!(!verify_commitment(4, root, tampered_tree));
}

/// Dropping a real leaf — padding in its place — must not verify against the genuine root.
#[test]
fn verify_commitment_fails_when_real_leaf_dropped() {
    let (_, root) = build_balanced_tree(&example_leaves(4));
    let (dropped_tree, _) = build_balanced_tree(&example_leaves(3));
    // Three real leaves at capacity 4: the fourth slot pads, so the circuit reconstructs the root
    // of [l0, l1, l2] — not the four-leaf root.
    assert!(!verify_commitment(4, root, dropped_tree));
}

/// The full finalized [`Circuit`] — gates *and* variable wiring — for `n` balanced leaves at
/// `capacity`. `Value` only drives whether witness values are tracked, never the emitted gates.
fn build_circuit<Value: IValue>(n: usize, capacity: usize) -> Circuit {
    let mut ctx = Context::<Value>::default();
    let (tree, root) = build_balanced_tree(&example_leaves(n as u32));
    // The value-carrying build must satisfy the binding equality, so bind to the real root. The
    // topology does not depend on that value, so the `NoValue` build emits the same gates.
    let root_var = node_as::<Value>(&root).guess(&mut ctx);
    let computed = unpack_recursion_tree(&mut ctx, &tree, capacity);
    bind_node_eq(&mut ctx, &computed.root, &root_var);
    ctx.finalize(false).context.circuit
}

/// The circuit must not depend on witness values: `QM31` and `NoValue` builds must emit identical
/// gates and wiring.
#[rstest]
#[case(3, 8)]
#[case(5, 8)]
#[case(1, 4)]
fn structure_is_witness_independent(#[case] n: usize, #[case] capacity: usize) {
    let with_values = build_circuit::<QM31>(n, capacity);
    let without_values = build_circuit::<NoValue>(n, capacity);
    assert_eq!(with_values, without_values);
}

/// The circuit is *fixed*: since every value is guessed internally (padding with guessed, not
/// constant, zeros), the emitted circuit — wiring included — is byte-identical for every `n` at a
/// given `capacity`.
#[rstest]
#[case(8)]
#[case(16)]
fn circuit_is_fixed_across_n(#[case] capacity: usize) {
    let reference = build_circuit::<QM31>(1, capacity);
    for n in 2..=capacity {
        assert_eq!(
            build_circuit::<QM31>(n, capacity),
            reference,
            "circuit for n={n} differs from n=1 at capacity={capacity}"
        );
    }
}

/// Also independent of tree *shape*: a balanced tree and a caterpillar over the same leaves emit a
/// byte-identical circuit — "shape is wiring, not topology".
#[test]
fn circuit_is_fixed_across_shape() {
    let leaves = example_leaves(4);
    let (balanced, _) = build_balanced_tree(&leaves);
    let (caterpillar, _) = build_caterpillar_tree(&leaves);

    let circuit_of = |tree: &InputTree| {
        let mut ctx = TraceContext::default();
        unpack_recursion_tree(&mut ctx, tree, 4);
        ctx.finalize(false).context.circuit
    };
    assert_eq!(circuit_of(&balanced), circuit_of(&caterpillar));
}

/// A valid circuit must satisfy its constraints *and* the soundness-critical lookup invariant that
/// every variable is yielded exactly once ([`Circuit::check_yields`]).
///
/// The stronger `check_vars_used` (every variable also *consumed*) does not hold: `permute_tuples`
/// leaves dead tag variables behind. That is a property of the shared utility, not a soundness
/// concern.
#[test]
fn valid_circuit_passes_yield_and_constraint_checks() {
    let (tree, root) = build_balanced_tree(&example_leaves(5));

    let mut context = TraceContext::default();
    let root_var = root.guess(&mut context);
    let computed = unpack_recursion_tree(&mut context, &tree, 8);
    bind_node_eq(&mut context, &computed.root, &root_var);

    let finalized = context.finalize(false);
    finalized.validate_circuit();
    finalized.circuit().check_yields();
}

#[test]
#[should_panic(expected = "power of two")]
fn non_power_of_two_capacity_panics() {
    let mut context = TraceContext::default();
    let (tree, _) = build_balanced_tree(&example_leaves(3));
    unpack_recursion_tree(&mut context, &tree, 3);
}

#[test]
#[should_panic(expected = "capacity")]
fn too_many_leaves_panics() {
    let mut context = TraceContext::default();
    let (tree, _) = build_balanced_tree(&example_leaves(5));
    unpack_recursion_tree(&mut context, &tree, 4);
}
