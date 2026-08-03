use circuit_common::finalize::{ComponentSizes, compute_padded_sizes};

use crate::canonical::{
    CanonicalCircuit, TARGET_PADDING_SIZES, build_unpadded_leaf_context,
    build_unpadded_multiverifier_context,
};
use crate::{LeafInput, PackedNode, RecursiveTreeError, load_leaves};

// ------------------------------------------------------------------------------------------------
// Serde shapes.
// ------------------------------------------------------------------------------------------------

#[test]
fn test_leaf_input_roundtrips() {
    // A representative leaf: `proof` is base64 ("AQID" = bytes [1, 2, 3]); the two digests are
    // eight little-endian `0x`-hex words each, with the injected `output_preimage` flattened in.
    let json = r#"{"output_preimage":["5","11"],"circuit_preprocessed_root":["0x00000000","0x00000001","0x00000002","0x00000003","0x00000004","0x00000005","0x00000006","0x00000007"],"circuit_hash":["0x0000000a","0x0000000b","0x0000000c","0x0000000d","0x0000000e","0x0000000f","0x00000010","0x00000011"],"proof":"AQID"}"#;
    let leaf: LeafInput = serde_json::from_str(json).unwrap();
    assert_eq!(leaf.output_preimage, vec!["5", "11"]);
    assert_eq!(leaf.proof.circuit_preprocessed_root.0[0], 0);
    assert_eq!(leaf.proof.circuit_preprocessed_root.0[7], 7);
    assert_eq!(leaf.proof.circuit_hash.0[0], 0x0a);
    assert_eq!(leaf.proof.proof, vec![1, 2, 3]);
    // The wrapper is flattened: it round-trips through the same flat JSON object,
    // `SerializedLeafProof` fields at top level next to `output_preimage`.
    let back = serde_json::to_string(&leaf).unwrap();
    let leaf2: LeafInput = serde_json::from_str(&back).unwrap();
    assert_eq!(leaf2, leaf);
}

#[test]
fn test_load_leaves_reads_manifest_of_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let leaf_path = tmp.path().join("leaf0.json");
    std::fs::write(
        &leaf_path,
        r#"{"output_preimage":["2"],"circuit_preprocessed_root":["0x0","0x0","0x0","0x0","0x0","0x0","0x0","0x0"],"circuit_hash":["0x0","0x0","0x0","0x0","0x0","0x0","0x0","0x0"],"proof":"AQID"}"#,
    )
    .unwrap();
    let manifest = tmp.path().join("leaves.json");
    std::fs::write(&manifest, format!(r#"{{"leaves":[{:?}]}}"#, leaf_path.to_str().unwrap()))
        .unwrap();
    let leaves = load_leaves(&manifest).unwrap();
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].output_preimage, vec!["2"]);
    assert_eq!(leaves[0].proof.proof, vec![1, 2, 3]);
}

/// [`LeafInput::output_values`] must reproduce the leaf bootloader's output digest `H1` (the
/// cairo0-encoded Blake2s of the preimage) — the value the leaf cairo-verifier circuit emits
/// verbatim as its public output. Golden recomputed independently (matches a real leaf run).
#[test]
fn test_leaf_output_values_derived_from_preimage() {
    let leaf = LeafInput {
        proof: crate::SerializedLeafProof {
            circuit_preprocessed_root: leaf_proof_format::DigestHex([0; 8]),
            circuit_hash: leaf_proof_format::DigestHex([0; 8]),
            proof: vec![],
        },
        output_preimage: [
            "1433852663250257978909904594223798547176815246431631498282706690602142197827",
            "11",
            "13",
            "17",
        ]
        .map(str::to_string)
        .to_vec(),
    };
    assert_eq!(
        leaf.output_values().unwrap(),
        [
            1603116091, 3258597502, 2711032228, 4175407283, 343882323, 1898618121, 1344732087,
            1064799167,
        ]
    );
}

#[test]
fn test_leaf_output_values_rejects_invalid_felt() {
    let leaf = LeafInput {
        proof: crate::SerializedLeafProof {
            circuit_preprocessed_root: leaf_proof_format::DigestHex([0; 8]),
            circuit_hash: leaf_proof_format::DigestHex([0; 8]),
            proof: vec![],
        },
        output_preimage: vec!["not-a-felt".to_string()],
    };
    match leaf.output_values() {
        Err(RecursiveTreeError::BadLeafOutputs { reason }) => {
            assert!(reason.contains("not-a-felt"))
        }
        other => panic!("expected BadLeafOutputs, got {other:?}"),
    }
}

#[test]
fn test_packed_node_serializes_leaf_and_internal() {
    // A leaf: a `Composite` (the leaf circuit) over `Plain` (the raw preimage reveal).
    let leaf_a = PackedNode::leaf(
        std::array::from_fn(|i| i as u32 + 30),
        vec!["1".to_string(), "2".to_string()],
    );
    // Serializes as `{"Composite": { circuit_hash, subtasks: [{"Plain": {
    // output_preimage }}] }}`.
    let leaf_json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&leaf_a).unwrap()).unwrap();
    assert_eq!(leaf_json["Composite"]["circuit_hash"][0], 30);
    assert_eq!(leaf_json["Composite"]["subtasks"][0]["Plain"]["output_preimage"][0], "1");

    // Internal: a `Composite` over two child subtasks.
    let leaf_b = PackedNode::leaf(std::array::from_fn(|i| i as u32 + 40), vec![]);
    let internal = PackedNode::internal(std::array::from_fn(|i| i as u32 + 50), leaf_a, leaf_b);

    // Round-trips exactly (the recursive-tree reads back its own `root_packed.json`).
    let back: PackedNode =
        serde_json::from_str(&serde_json::to_string(&internal).unwrap()).unwrap();
    assert_eq!(back, internal);
}

// ------------------------------------------------------------------------------------------------
// B-0: lock TARGET_PADDING_SIZES and the homogeneity (padding parity) invariant.
// ------------------------------------------------------------------------------------------------

/// The pinned [`TARGET_PADDING_SIZES`] must be exactly the per-component max (each already rounded
/// up to a power of two by `compute_padded_sizes`) of the unpadded leaf and multiverifier circuits.
/// If this fails, the assertion prints the value the constant should be updated to.
#[test]
fn test_target_padding_sizes_are_consistent() {
    let leaf = compute_padded_sizes(&build_unpadded_leaf_context());
    let multiverifier =
        compute_padded_sizes(&build_unpadded_multiverifier_context(circuit_prover_params()));
    let derived = ComponentSizes {
        eq: leaf.eq.max(multiverifier.eq),
        qm31_ops: leaf.qm31_ops.max(multiverifier.qm31_ops),
        m31_to_u32: leaf.m31_to_u32.max(multiverifier.m31_to_u32),
        triple_xor: leaf.triple_xor.max(multiverifier.triple_xor),
        blake_g_gate: leaf.blake_g_gate.max(multiverifier.blake_g_gate),
    };
    assert_eq!(
        derived, TARGET_PADDING_SIZES,
        "leaf sizes: {leaf}\nmultiverifier sizes: {multiverifier}\nupdate \
         crate::canonical::TARGET_PADDING_SIZES to the derived value above"
    );
}

/// Building the canonical circuit must succeed; in particular the leaf and multiverifier circuits,
/// padded to [`TARGET_PADDING_SIZES`], must share a preprocessed root (checked inside `build`).
#[test]
fn test_canonical_circuit_builds_with_matching_preprocessed_root() {
    CanonicalCircuit::build(circuit_prover_params())
        .expect("canonical circuit should build with matching preprocessed root");
}

/// The committed circuit prover params file — the `PcsConfig` the tree's tests and goldens are
/// generated with, fed to both `leaf_prover` and the tree binary exactly as the backend feeds
/// them one shared file.
fn circuit_prover_params_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_data/circuit_prover_params.json")
}

/// The committed circuit prover params, deserialized.
fn circuit_prover_params() -> stwo::core::pcs::PcsConfig {
    serde_json::from_str(&std::fs::read_to_string(circuit_prover_params_path()).unwrap()).unwrap()
}

// ------------------------------------------------------------------------------------------------
// End-to-end folds (gated behind the `slow-tests` feature; run in RELEASE mode, one test at a
// time: `cargo test --release --features slow-tests -- --test-threads=1`).
//
// `fold_two_leaves` / `fold_three_leaves_with_carry` duplicate the pre-generated golden
// `leaf_prover` output at `test_data/goldens/four_leaves/leaf.json` as N leaves and fold them —
// every fold builds and proves a real multiverifier circuit over two valid child proofs. The
// leaves go through the full `LeafInput` path, so the multiverifier also attests
// `LeafInput::output_values`'s derivation against the real fixture proof.
//
// `test_golden_four_leaves_e2e` is the true end-to-end: it runs `leaf_prover` itself on the leaf
// simple bootloader (executing the simple-output task), injects the dumped hashed-output preimage
// the way the backend does, folds 4 copies, and asserts the artifacts match the committed goldens.
// Run it with `FIX=1` to regenerate every golden:
//
//   FIX=1 cargo test -p stwo-run-and-prove-recursive-tree --release --features slow-tests \
//     --lib -- test_golden_four_leaves_e2e --test-threads=1
//
// The two compiled programs in `test_data/` (`leaf_simple_bootloader_compiled.json`,
// `simple_output_compiled.json`) are inputs, not goldens; they are compiled from the main starkware
// repo via `bazel run
// //src/services/gps/bin/rust/test:compile_cairo_run_programs_with_rust_hints_script`.
// ------------------------------------------------------------------------------------------------

#[cfg(feature = "slow-tests")]
mod e2e {
    use std::path::{Path, PathBuf};

    use blake2::{Blake2s256, Digest};
    use circuit_prover::circuit_hash::preprocessed_circuit_hash;
    use leaf_prover::prove_leaf::prove_leaf_from_files;
    use num_bigint::BigUint;

    use super::{circuit_prover_params, circuit_prover_params_path};
    use crate::canonical::CanonicalCircuit;
    use crate::fold::digest_bytes_to_words;
    use crate::{LeafInput, PackedNode, stwo_run_and_prove_recursive_tree};

    fn goldens_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/goldens/four_leaves")
    }

    /// The golden `leaf_prover` output the cheaper e2e folds duplicate their leaves from: the leaf
    /// simple bootloader running a simple-output task, proven with the canonical-small setup (the
    /// `cairo_prover_params_canonical_small.json` parameters and the committed
    /// `circuit_prover_params.json`;
    /// see `generate_leaf`). Regenerated by `test_golden_four_leaves_e2e` under `FIX=1`.
    fn golden_leaf() -> LeafInput {
        let path = goldens_dir().join("leaf.json");
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// The simple-output task's output, and the leaf bootloader input driving it — the same values
    /// the goldens were generated with.
    const LEAF_TASK_OUTPUT: [u32; 3] = [11, 13, 17];

    /// The leaf bootloader input JSON: one simple-output `RunProgramTask` (blake program hash) plus
    /// the hashed-output preimage dump path.
    fn leaf_bl_input_json(dump_path: &Path) -> String {
        let task_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/simple_output_compiled.json");
        serde_json::to_string_pretty(&serde_json::json!({
            "tasks": [{
                "type": "RunProgramTask",
                "path": task_path.to_str().unwrap(),
                "program_input": {"output": LEAF_TASK_OUTPUT},
                "program_hash_function": "blake",
            }],
            "fact_topologies_path": null,
            "single_page": true,
            "output_preimage_dump_path": dump_path.to_str().unwrap(),
        }))
        .unwrap()
    }

    /// True-e2e leaf generation: runs `leaf_prover` on the leaf simple bootloader (executing the
    /// simple-output task) with the canonical-small parameters, then wraps the produced
    /// `SerializedLeafProof` into a `LeafInput` with the dumped hashed-output preimage exactly as
    /// the backend does (hex felts from the dump file, re-encoded as decimal strings).
    fn generate_leaf(dir: &Path) -> LeafInput {
        let dump_path = dir.join("leaf_preimage.json");
        let input_path = dir.join("leaf_bl_input.json");
        std::fs::write(&input_path, leaf_bl_input_json(&dump_path)).unwrap();

        let leaf = prove_leaf_from_files(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("test_data/leaf_simple_bootloader_compiled.json"),
            &Some(input_path),
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../leaf_prover/tests/data/cairo_prover_params_canonical_small.json"),
            &circuit_prover_params_path(),
        );
        let dumped: Vec<String> =
            serde_json::from_str(&std::fs::read_to_string(&dump_path).unwrap()).unwrap();
        let output_preimage = dumped
            .iter()
            .map(|hex| {
                BigUint::parse_bytes(hex.trim_start_matches("0x").as_bytes(), 16)
                    .expect("dump entries are hex felts")
                    .to_string()
            })
            .collect();
        LeafInput { proof: leaf, output_preimage }
    }

    /// The leaf circuit's `circuit_hash` (its trust-anchor identity) — read straight from the
    /// leaf proof.
    fn leaf_circuit_hash(leaf: &LeafInput) -> [u32; circuit_common::N_RESERVED] {
        leaf.proof.circuit_hash.0
    }

    fn init_tracing() {
        let _ = tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).try_init();
    }

    /// Out-of-circuit `circuits::blake::blake2s_u32s`: Blake2s-256 over the little-endian bytes of
    /// `words`, digest read back as eight little-endian u32 words (mirrors verify_test's helper).
    fn blake2s_u32s_host(words: &[u32]) -> [u32; circuit_common::N_RESERVED] {
        let mut hasher = Blake2s256::new();
        for word in words {
            hasher.update(word.to_le_bytes());
        }
        let hash: [u8; 32] = hasher.finalize().into();
        std::array::from_fn(|i| u32::from_le_bytes(hash[i * 4..i * 4 + 4].try_into().unwrap()))
    }

    /// The canonical multiverifier's `circuit_hash` — the identity every internal node carries.
    fn multiverifier_circuit_hash(
        canonical: &CanonicalCircuit,
    ) -> [u32; circuit_common::N_RESERVED] {
        let hash = preprocessed_circuit_hash(
            &canonical.preprocessed_multiverifier,
            canonical.shared_config.pcs_config.fri_config.log_blowup_factor,
        );
        digest_bytes_to_words(&hash.0)
    }

    /// An expected tree node: the raw u32 output words + `circuit_hash` (both needed to hash a
    /// parent) alongside the `PackedNode` subtree the binary should emit for it.
    struct ExpectedNode {
        output_words: [u32; circuit_common::N_RESERVED],
        circuit_hash: [u32; circuit_common::N_RESERVED],
        packed: PackedNode,
    }

    fn expected_leaf(leaf: &LeafInput) -> ExpectedNode {
        ExpectedNode {
            output_words: leaf.output_values().unwrap(),
            circuit_hash: leaf_circuit_hash(leaf),
            packed: PackedNode::leaf(leaf_circuit_hash(leaf), leaf.output_preimage.clone()),
        }
    }

    /// Mirrors `reduce_pair`'s output: the multiverifier hashes, for each child, its `circuit_hash`
    /// (8 words) followed by its raw output words (8 words), and keeps the resulting eight-word
    /// digest as its own output.
    fn expected_reduce(
        left: ExpectedNode,
        right: ExpectedNode,
        multiverifier_circuit_hash: [u32; circuit_common::N_RESERVED],
    ) -> ExpectedNode {
        let mut preimage = Vec::new();
        for child in [&left, &right] {
            preimage.extend_from_slice(&child.circuit_hash);
            preimage.extend_from_slice(&child.output_words);
        }
        let output_words = blake2s_u32s_host(&preimage);
        ExpectedNode {
            output_words,
            circuit_hash: multiverifier_circuit_hash,
            packed: PackedNode::internal(multiverifier_circuit_hash, left.packed, right.packed),
        }
    }

    /// Folds `n` identical leaves exactly as the binary does (pair adjacent, carry the odd one up)
    /// and returns the expected root node. `multiverifier_circuit_hash` is the `circuit_hash` every
    /// internal node carries (see [`multiverifier_circuit_hash`]).
    fn expected_root(
        leaf: &LeafInput,
        n: usize,
        multiverifier_circuit_hash: [u32; circuit_common::N_RESERVED],
    ) -> ExpectedNode {
        let mut layer: Vec<ExpectedNode> = (0..n).map(|_| expected_leaf(leaf)).collect();
        while layer.len() > 1 {
            let mut next = Vec::with_capacity(layer.len().div_ceil(2));
            let mut pairs = layer.into_iter();
            while let Some(left) = pairs.next() {
                match pairs.next() {
                    Some(right) => {
                        next.push(expected_reduce(left, right, multiverifier_circuit_hash))
                    }
                    None => next.push(left),
                }
            }
            layer = next;
        }
        layer.pop().expect("at least one leaf")
    }

    /// Folds `n` identical copies of `leaf` with the binary entry point, and asserts the fold's
    /// shape stats plus the produced root proof, root outputs, and full `packed_output` tree match
    /// what we recompute independently (topology + values) from the identical leaves.
    fn dupe_and_fold(leaf: &LeafInput, n: usize, dir: &Path) {
        init_tracing();
        let leaves: Vec<LeafInput> = vec![leaf.clone(); n];
        let stats = stwo_run_and_prove_recursive_tree(
            leaves,
            circuit_prover_params(),
            &dir.join("root.proof"),
            &dir.join("root_outputs.json"),
            &dir.join("root_packed.json"),
        )
        .unwrap();

        // A balanced two-to-one tree with odd-carry: depth = ceil(log2 n), reductions = n - 1.
        assert_eq!(stats.n_leaves, n, "leaf count mismatch");
        assert_eq!(stats.n_layers, n.next_power_of_two().ilog2() as usize, "layer count mismatch");
        assert_eq!(stats.n_pair_reductions, n - 1, "reduction count mismatch");

        // The root proof is the Cairo circuit verifier's `--arguments-file` stream: a JSON array
        // of hex-string felts (the final fold is proven with the standard Blake2s channel and
        // serialized via `prepare_circuit_proof_for_cairo_verifier`).
        let root_felts: Vec<String> =
            serde_json::from_str(&std::fs::read_to_string(dir.join("root.proof")).unwrap())
                .expect("root proof parses as a JSON array of hex felts");
        assert!(
            !root_felts.is_empty() && root_felts.iter().all(|f| f.starts_with("0x")),
            "root proof felts must be 0x-prefixed hex strings"
        );

        // Independently recompute the whole tree (topology + hashed output values) from the leaves.
        let canonical = CanonicalCircuit::build(circuit_prover_params()).unwrap();
        let multiverifier_circuit_hash = multiverifier_circuit_hash(&canonical);
        let expected = expected_root(leaf, n, multiverifier_circuit_hash);

        let actual_outputs: Vec<u32> =
            serde_json::from_str(&std::fs::read_to_string(dir.join("root_outputs.json")).unwrap())
                .unwrap();
        assert_eq!(actual_outputs, expected.output_words.to_vec(), "root output values mismatch");

        let actual_packed: PackedNode =
            serde_json::from_str(&std::fs::read_to_string(dir.join("root_packed.json")).unwrap())
                .unwrap();
        assert_eq!(
            actual_packed, expected.packed,
            "packed output tree (topology + values) mismatch"
        );
    }

    /// Parses the same JSON file from two directories and asserts value equality (formatting- and
    /// byte-layout-agnostic).
    fn assert_same_json<T: serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(
        actual_dir: &Path,
        golden_dir: &Path,
        file: &str,
    ) {
        let parse = |dir: &Path| -> T {
            serde_json::from_str(&std::fs::read_to_string(dir.join(file)).unwrap())
                .unwrap_or_else(|e| panic!("{file} does not parse: {e}"))
        };
        assert_eq!(
            parse(actual_dir),
            parse(golden_dir),
            "{file} does not match the committed golden; run with FIX=1 to regenerate"
        );
    }

    #[test]
    fn test_fold_two_leaves() {
        let tmp = tempfile::tempdir().unwrap();
        dupe_and_fold(&golden_leaf(), 2, tmp.path());
    }

    #[test]
    fn test_fold_three_leaves_with_carry() {
        let tmp = tempfile::tempdir().unwrap();
        dupe_and_fold(&golden_leaf(), 3, tmp.path());
    }

    /// True end-to-end: `leaf_prover` over the leaf simple bootloader (running the simple-output
    /// task), backend-style preimage injection, 4-leaf fold, and comparison against the committed
    /// goldens at `test_data/goldens/four_leaves/`. When run with the `FIX` env var set, it
    /// regenerates the goldens (including the machine-specific manual-CLI-repro inputs) instead
    /// of asserting.
    ///
    /// Note the circuit hashes have no dedicated trust-list golden: the trust anchors are the
    /// `circuit_params --registry` artifacts published to the artifacts bucket per commit, and
    /// hash drift is still caught here via the `root_packed.json` golden (every `Composite` node
    /// carries its `circuit_hash`).
    #[test]
    fn test_golden_four_leaves_e2e() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let leaf = generate_leaf(dir);
        dupe_and_fold(&leaf, 4, dir);

        let goldens = goldens_dir();
        if std::env::var("FIX").is_ok() {
            std::fs::write(goldens.join("leaf.json"), serde_json::to_string_pretty(&leaf).unwrap())
                .unwrap();
            for file in
                ["leaf_preimage.json", "root.proof", "root_outputs.json", "root_packed.json"]
            {
                std::fs::copy(dir.join(file), goldens.join(file)).unwrap();
            }
            // Manual-CLI-repro inputs (machine-specific absolute paths).
            std::fs::write(
                goldens.join("leaf_bl_input.json"),
                leaf_bl_input_json(&goldens.join("leaf_preimage.json")),
            )
            .unwrap();
            let leaf_path = goldens.join("leaf.json");
            std::fs::write(
                goldens.join("manifest.json"),
                serde_json::to_string_pretty(
                    &serde_json::json!({"leaves": vec![leaf_path.to_str().unwrap(); 4]}),
                )
                .unwrap(),
            )
            .unwrap();
            return;
        }

        // Regression: the freshly generated artifacts must match the committed goldens.
        assert_eq!(
            leaf,
            golden_leaf(),
            "freshly proven leaf does not match the golden leaf.json; run with FIX=1 to regenerate"
        );
        assert_same_json::<Vec<String>>(dir, &goldens, "leaf_preimage.json");
        assert_same_json::<Vec<String>>(dir, &goldens, "root.proof");
        assert_same_json::<Vec<u32>>(dir, &goldens, "root_outputs.json");
        assert_same_json::<PackedNode>(dir, &goldens, "root_packed.json");
    }
}
