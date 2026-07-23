use circuits::blake::HashValue;
use circuits::context::{TraceContext, Var};
use circuits::ivalue::qm31_from_u32s;
use circuits::ops::Guess;
use circuits::wrappers::{M31Wrapper, U32Wrapper};
use rstest::rstest;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

use crate::merkle::{
    AuthPath, AuthPaths, decommit_eval_domain_samples, hash_leaf_m31s, hash_leaf_qm31, hash_node,
    verify_merkle_path,
};
use crate::oods::EvalDomainSamples;

/// Reads the circuit's value of a [`HashValue<Var>`] back as a [`HashValue<QM31>`].
fn get_hash(context: &TraceContext, hash: HashValue<Var>) -> HashValue<QM31> {
    HashValue(std::array::from_fn(|i| U32Wrapper::new_unsafe(context.get(*hash[i].get()))))
}

/// Standard Blake2s hash of the little-endian bytes of `values`, matching `hash_leaf_m31s`.
fn leaf_hash(values: &[M31]) -> Blake2sHash {
    let bytes: Vec<u8> = values.iter().flat_map(|v| v.0.to_le_bytes()).collect();
    Blake2sHasher::hash(&bytes)
}

#[test]
fn hash_leaf_m31s_regression() {
    let mut context = TraceContext::default();

    let m31s = [M31::from(1641251221)];
    let values = m31s.map(|v| M31Wrapper::from(v).guess(&mut context));
    let hash = hash_leaf_m31s(&mut context, &values);
    assert_eq!(get_hash(&context, hash), HashValue::from(leaf_hash(&m31s)));

    let m31s = [1, 1641251221, 1176667027, 568581975].map(M31::from);
    let values = m31s.map(|v| M31Wrapper::from(v).guess(&mut context));
    let hash = hash_leaf_m31s(&mut context, &values);
    assert_eq!(get_hash(&context, hash), HashValue::from(leaf_hash(&m31s)));

    context.validate_circuit();
}

#[test]
fn hash_leaf_qm31_regression() {
    let mut context = TraceContext::default();

    let words = [106879334u32, 2000582330, 760086299, 1036436096];
    let value = qm31_from_u32s(words[0], words[1], words[2], words[3]).guess(&mut context);

    let hash = hash_leaf_qm31(&mut context, value);

    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    assert_eq!(get_hash(&context, hash), HashValue::from(Blake2sHasher::hash(&bytes)));

    context.validate_circuit();
}

#[test]
fn hash_node_regression() {
    let mut context = TraceContext::default();

    let left_hash = Blake2sHash(std::array::from_fn(|i| i as u8));
    let right_hash = Blake2sHash(std::array::from_fn(|i| (i as u8).wrapping_add(50)));
    let left = HashValue::from(left_hash).guess(&mut context);
    let right = HashValue::from(right_hash).guess(&mut context);

    let hash = hash_node(&mut context, &left, &right);

    let expected = Blake2sHasher::concat_and_hash(&left_hash, &right_hash);
    assert_eq!(get_hash(&context, hash), HashValue::from(expected));

    context.validate_circuit();
}

#[rstest]
#[case::success(false, false)]
#[case::wrong_bit(true, false)]
#[case::wrong_root(false, true)]
fn test_merkle_path(#[case] wrong_bit: bool, #[case] wrong_root: bool) {
    let mut context = TraceContext::default();

    // Full leaf and sibling hashes (the tree is committed with the standard Blake2s hasher).
    let leaf_hash = Blake2sHash(std::array::from_fn(|i| i as u8));
    let siblings: [Blake2sHash; 5] =
        std::array::from_fn(|i| Blake2sHash(std::array::from_fn(|j| ((i + 1) * 40 + j) as u8)));

    let auth_path =
        AuthPath(siblings.iter().map(|s| HashValue::from(*s)).collect()).guess(&mut context);
    let leaf = HashValue::from(leaf_hash).guess(&mut context);
    let bits = vec![
        qm31_from_u32s(if wrong_bit { 0 } else { 1 }, 0, 0, 0),
        qm31_from_u32s(1, 0, 0, 0),
        qm31_from_u32s(0, 0, 0, 0),
        qm31_from_u32s(0, 0, 0, 0),
        qm31_from_u32s(1, 0, 0, 0),
    ]
    .guess(&mut context);

    // Recompute the root following the success-case bits [1, 1, 0, 0, 1] (bit = 1 puts
    // the current node on the right, its sibling on the left), then reduce mod `M31::P`.
    let mut node = leaf_hash;
    node = Blake2sHasher::concat_and_hash(&siblings[0], &node);
    node = Blake2sHasher::concat_and_hash(&siblings[1], &node);
    node = Blake2sHasher::concat_and_hash(&node, &siblings[2]);
    node = Blake2sHasher::concat_and_hash(&node, &siblings[3]);
    node = Blake2sHasher::concat_and_hash(&siblings[4], &node);
    // The tree is committed over full 32-bit words, so the recomputed root is compared as a full
    // `HashValue`.
    let root = if wrong_root {
        let mut bytes = node.0;
        bytes[0] ^= 1;
        HashValue::from(Blake2sHash(bytes))
    } else {
        HashValue::from(node)
    };
    let root = root.guess(&mut context);

    verify_merkle_path(&mut context, leaf, &bits, &root, &auth_path);
    let success = !wrong_bit && !wrong_root;
    assert_eq!(context.is_circuit_valid(), success);
}

#[rstest]
#[case::success(None, false)]
#[case::wrong_bit(None, true)]
#[case::wrong_root0(Some(0), false)]
#[case::wrong_root1(Some(1), false)]
#[case::wrong_root2(Some(2), false)]
#[case::wrong_root3(Some(3), false)]
fn test_decommit_eval_domain_samples(#[case] wrong_root: Option<usize>, #[case] wrong_bit: bool) {
    use std::collections::HashMap;

    let mut context = TraceContext::default();

    // 1 preprocessed trace with a single column + 3 traces with no columns.
    let eval_domain_samples =
        EvalDomainSamples::from_m31s(vec![vec![vec![M31::from(1)]], vec![], vec![], vec![]]);

    // 2 traces with no columns.
    let opt_column_log_sizes_by_trace: HashMap<usize, Vec<Var>> =
        HashMap::from([(1, vec![]), (2, vec![])]);

    // Sibling hashes are unreduced: the tree is committed with the standard (non-`M31`-reduced)
    // Blake2s hasher, so each auth-path sibling is a full 32-byte hash.
    let sibling0 = Blake2sHash(std::array::from_fn(|i| i as u8));
    let sibling1 = Blake2sHash(std::array::from_fn(|i| (i as u8).wrapping_add(100)));
    let auth_paths = AuthPaths {
        data: vec![
            vec![AuthPath(vec![HashValue::from(sibling0)])],
            vec![AuthPath(vec![HashValue::from(sibling1)])],
            vec![AuthPath(vec![HashValue::from(sibling1)])],
            vec![AuthPath(vec![HashValue::from(sibling1)])],
        ],
    };
    let bits: Vec<Vec<QM31>> = vec![vec![(if wrong_bit { 1 } else { 0 }).into()]];

    // Recompute the roots with the standard Blake2s; the tree is committed over full 32-bit words,
    // so each root is compared as a full `HashValue` (no `M31` reduction).
    let leaf0 = Blake2sHasher::hash(&M31::from(1).0.to_le_bytes());
    let empty_leaf = Blake2sHasher::hash(&[]);
    let node0 = Blake2sHasher::concat_and_hash(&leaf0, &sibling0);
    let node1 = Blake2sHasher::concat_and_hash(&empty_leaf, &sibling1);

    let mut node_hashes = [node0, node1, node1, node1];
    if let Some(wrong_root) = wrong_root {
        let mut bytes = node_hashes[wrong_root].0;
        bytes[0] ^= 1;
        node_hashes[wrong_root] = Blake2sHash(bytes);
    }
    let roots = node_hashes.map(HashValue::from);

    let eval_domain_samples_vars = eval_domain_samples.guess(&mut context);
    let auth_paths_vars = auth_paths.guess(&mut context);
    let bits_vars = bits.guess(&mut context);
    let roots_vars = roots.guess(&mut context);
    let n_queries = 1;
    decommit_eval_domain_samples(
        &mut context,
        n_queries,
        &opt_column_log_sizes_by_trace,
        &eval_domain_samples_vars,
        &auth_paths_vars,
        &bits_vars,
        &roots_vars,
    );

    let success = wrong_root.is_none() && !wrong_bit;
    assert_eq!(context.is_circuit_valid(), success);
}
