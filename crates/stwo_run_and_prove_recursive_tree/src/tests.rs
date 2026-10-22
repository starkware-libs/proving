use circuit_registry::CircuitRegistry;

use crate::canonical::CanonicalCircuit;
use crate::{
    LeafInput, LeafOutputHashType, LeafOutputPreimage, PackedNode, RecursiveTreeError, load_leaves,
};

// ------------------------------------------------------------------------------------------------
// Serde shapes.
// ------------------------------------------------------------------------------------------------

#[test]
fn test_leaf_input_roundtrips() {
    // A representative leaf: `proof` is base64 ("AQID" = bytes [1, 2, 3]); the two digests are
    // eight little-endian `0x`-hex words each, with the injected `output` next to them.
    let json = r#"{"output":{"hash_type":"blake2s","preimage":["5","11"]},"circuit_preprocessed_root":["0x00000000","0x00000001","0x00000002","0x00000003","0x00000004","0x00000005","0x00000006","0x00000007"],"circuit_hash":["0x0000000a","0x0000000b","0x0000000c","0x0000000d","0x0000000e","0x0000000f","0x00000010","0x00000011"],"proof":"AQID"}"#;
    let leaf: LeafInput = serde_json::from_str(json).unwrap();
    assert_eq!(
        leaf.output,
        LeafOutputPreimage {
            hash_type: LeafOutputHashType::Blake2s,
            preimage: vec!["5".to_string(), "11".to_string()],
        }
    );
    assert_eq!(leaf.proof.circuit_preprocessed_root.0[0], 0);
    assert_eq!(leaf.proof.circuit_preprocessed_root.0[7], 7);
    assert_eq!(leaf.proof.circuit_hash.0[0], 0x0a);
    assert_eq!(leaf.proof.proof, vec![1, 2, 3]);
    // The wrapper is flattened: it round-trips through the same JSON object, `SerializedLeafProof`
    // fields at top level next to `output`.
    let back = serde_json::to_string(&leaf).unwrap();
    let leaf2: LeafInput = serde_json::from_str(&back).unwrap();
    assert_eq!(leaf2, leaf);

    // A poseidon hash type round-trips the same way.
    let poseidon_json = json.replace(r#""hash_type":"blake2s""#, r#""hash_type":"poseidon""#);
    let poseidon_leaf: LeafInput = serde_json::from_str(&poseidon_json).unwrap();
    assert_eq!(poseidon_leaf.output.hash_type, LeafOutputHashType::Poseidon);
    assert_eq!(poseidon_leaf.output.preimage, leaf.output.preimage);
    let poseidon_back = serde_json::to_string(&poseidon_leaf).unwrap();
    assert_eq!(serde_json::from_str::<LeafInput>(&poseidon_back).unwrap(), poseidon_leaf);
}

#[test]
fn test_load_leaves_reads_manifest_of_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let leaf_path = tmp.path().join("leaf0.json");
    std::fs::write(
        &leaf_path,
        r#"{"output":{"hash_type":"blake2s","preimage":["2"]},"circuit_preprocessed_root":["0x0","0x0","0x0","0x0","0x0","0x0","0x0","0x0"],"circuit_hash":["0x0","0x0","0x0","0x0","0x0","0x0","0x0","0x0"],"proof":"AQID"}"#,
    )
    .unwrap();
    let manifest = tmp.path().join("leaves.json");
    std::fs::write(&manifest, format!(r#"{{"leaves":[{:?}]}}"#, leaf_path.to_str().unwrap()))
        .unwrap();
    let leaves = load_leaves(&manifest).unwrap();
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].output.preimage, ["2"]);
    assert_eq!(leaves[0].proof.proof, vec![1, 2, 3]);
}

/// [`LeafInput::output_digest`] must reproduce the leaf bootloader's output digest `H1` (the
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
        output: LeafOutputPreimage {
            hash_type: LeafOutputHashType::Blake2s,
            preimage: [
                "1433852663250257978909904594223798547176815246431631498282706690602142197827",
                "11",
                "13",
                "17",
            ]
            .map(str::to_string)
            .to_vec(),
        },
    };
    assert_eq!(
        leaf.output_digest().unwrap(),
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
        output: LeafOutputPreimage {
            hash_type: LeafOutputHashType::Blake2s,
            preimage: vec!["not-a-felt".to_string()],
        },
    };
    match leaf.output_digest() {
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
    let internal = PackedNode::Composite {
        circuit_hash: std::array::from_fn(|i| i as u32 + 50),
        subtasks: vec![leaf_a, leaf_b],
    };

    // Round-trips exactly (the recursive-tree reads back its own `root_packed.json`).
    let back: PackedNode =
        serde_json::from_str(&serde_json::to_string(&internal).unwrap()).unwrap();
    assert_eq!(back, internal);
}

/// Building the canonical circuit must succeed; in particular the leaf and multiverifier circuits,
/// padded to the registry's target sizes, must share a preprocessed root, and the multiverifier
/// must hash to the registry's entry (both checked inside `build`).
#[test]
fn test_canonical_circuit_builds_with_matching_preprocessed_root() {
    CanonicalCircuit::build(&circuit_registry())
        .expect("canonical circuit should build and match the registry");
}

/// The circuit registry file used for tests. Output of `circuit_params --registry` (see
/// `circuit_params`' cli test, which regenerates it).
fn circuit_registry_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/circuit_registry.json")
}

/// The committed circuit registry, deserialized.
fn circuit_registry() -> CircuitRegistry {
    CircuitRegistry::from_path(&circuit_registry_path()).unwrap()
}

// ------------------------------------------------------------------------------------------------
// Program-hash snapshots of the compiled programs the tests run. A snapshot pins the exact
// program a fixture holds; regenerating a fixture must reproduce the hash (or knowingly change it
// here).
//
// Each fixture is built (with debug info) via its standard BUILD target in
// https://github.com/starkware-industries/starkware (see the per-test comments), then stripped of
// debug info to keep the checked-in file small — from the starkware repo root:
//   bazel build <target>
//   jq --indent 4 '.debug_info = null' "$(bazel info bazel-bin)/<artifact>.json" > <fixture>.json
// ------------------------------------------------------------------------------------------------

/// The Blake program-hash chain of a compiled program, as the decimal string the snapshots pin.
fn program_hash(path: &std::path::Path) -> String {
    use cairo_program_runner_lib::compiled_program_hash;
    use cairo_program_runner_lib::types::HashFunc;

    compiled_program_hash(path, HashFunc::Blake)
        .expect("Failed to compute the program hash.")
        .to_string()
}

// The leaf simple bootloader the e2e folds run, shared with the privacy flow (which pins the same
// hash): built from starkware#41232's
// //src/starkware/cairo/bootloaders/simple_bootloader:leaf_simple_bootloader_program.
#[test]
fn test_leaf_simple_bootloader_program_hash_snapshot() {
    let expected = expect_test::expect![
        "182007537687310579697542984331198756973092309284201401073437552481857403195"
    ];
    expected.assert_eq(&program_hash(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../cairo-program-runner-lib/resources/compiled_programs/bootloaders/\
         leaf_simple_bootloader_compiled.json",
    )));
}

// Built at dev commit "c9180c2c9d9dd6df31fcdafcb86ede8a44402a73" from
// //src/starkware/cairo/bootloaders/bootloader:simple_output_program.
#[test]
fn test_simple_output_program_hash_snapshot() {
    let expected = expect_test::expect![
        "1433852663250257978909904594223798547176815246431631498282706690602142197827"
    ];
    expected.assert_eq(&program_hash(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data/simple_output_compiled.json"),
    ));
}

// Built from starkware-industries/starkware#41234, not yet on dev, from
// //src/starkware/cairo/bootloaders/simple_bootloader:aggregator_wrapper_program.
#[test]
fn test_aggregator_wrapper_program_hash_snapshot() {
    let expected = expect_test::expect![
        "828474924517741145847165707602666455661714061056494467766548351125989539667"
    ];
    expected.assert_eq(&program_hash(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data/aggregator_wrapper_compiled.json"),
    ));
}

// Built at the pre-merge starkware branch "yair/aggregator_simple_bootloader" (commit
// "4c56ac0e689") from
// //src/starkware/cairo/bootloaders/applicative_bootloader/test_utils:concat_aggregator_program.
#[test]
fn test_concat_aggregator_program_hash_snapshot() {
    let expected = expect_test::expect![
        "1554726174270058396665791700878496117514229730010818533966500003869418874662"
    ];
    expected.assert_eq(&program_hash(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data/concat_aggregator_compiled.json"),
    ));
}

// ------------------------------------------------------------------------------------------------
// End-to-end folds (gated behind the `slow-tests` feature; run in RELEASE mode, one test at a
// time: `cargo test --release --features slow-tests -- --test-threads=1`).
//
// `fold_two_leaves` / `fold_three_leaves_with_carry` duplicate the pre-generated golden
// `leaf_prover` output at `test_data/goldens/four_leaves/leaf.json` as N leaves and fold them —
// every fold builds and proves a real multiverifier circuit over two valid child proofs. The
// leaves go through the full `LeafInput` path, so the multiverifier also attests
// `LeafInput::output_digest`'s derivation against the real fixture proof.
//
// `test_golden_four_leaves_e2e` is the true end-to-end: it runs `leaf_prover` itself on the leaf
// simple bootloader (executing the simple-output task), injects the dumped hashed-output preimage
// the way the backend does, folds 4 copies, and asserts the artifacts match the committed goldens.
// Run it with `FIX=1` to regenerate every golden:
//
//   FIX=1 cargo test -p stwo-run-and-prove-recursive-tree --release --features slow-tests \
//     --lib -- test_golden_four_leaves_e2e --test-threads=1
//
// The compiled programs are inputs, not goldens: `simple_output_compiled.json` and
// `aggregator_wrapper_compiled.json` in `test_data/`, and the leaf simple bootloader shared with
// the privacy flow at `crates/cairo-program-runner-lib/resources/compiled_programs/bootloaders/`.
// They are compiled from the main starkware repo via `bazel run
// //src/services/gps/bin/rust/test:compile_cairo_run_programs_with_rust_hints_script`.
// ------------------------------------------------------------------------------------------------

#[cfg(feature = "slow-tests")]
mod e2e {
    use std::num::NonZeroUsize;
    use std::path::{Path, PathBuf};

    use blake2::{Blake2s256, Digest};
    use circuit_prover::circuit_hash::preprocessed_circuit_hash;
    use leaf_prover::prove_leaf::prove_leaf_from_files;
    use num_bigint::BigUint;
    use stwo::core::fields::qm31::QM31;

    use super::{circuit_registry, circuit_registry_path};
    use crate::canonical::CanonicalCircuit;
    use crate::fold::digest_bytes_to_words;
    use crate::{
        LeafInput, LeafOutputHashType, LeafOutputPreimage, PackedNode, ProofFormat,
        stwo_run_and_prove_recursive_tree,
    };

    fn goldens_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/goldens/four_leaves")
    }

    /// The golden `leaf_prover` output the cheaper e2e folds duplicate their leaves from: the leaf
    /// simple bootloader running a simple-output task, proven against the committed
    /// canonical-small registry (see `generate_leaf`). Regenerated by
    /// `test_golden_four_leaves_e2e` under `FIX=1`.
    fn golden_leaf() -> LeafInput {
        let path = goldens_dir().join("leaf.json");
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// The simple-output task's output, and the leaf bootloader input driving it — the same values
    /// the goldens were generated with.
    const LEAF_TASK_OUTPUT: [u32; 3] = [11, 13, 17];

    /// The leaf bootloader input JSON: one simple-output `RunProgramTask` (blake program hash) plus
    /// the hashed-output preimage dump path.
    fn leaf_bl_input_json(dump_path: &Path, task_output: &[u32]) -> String {
        let task_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/simple_output_compiled.json");
        serde_json::to_string_pretty(&serde_json::json!({
            "tasks": [{
                "type": "RunProgramTask",
                "path": task_path.to_str().unwrap(),
                "program_input": {"output": task_output},
                "program_hash_function": "blake",
            }],
            "fact_topologies_path": null,
            "single_page": true,
            "output_preimage_dump_path": dump_path.to_str().unwrap(),
        }))
        .unwrap()
    }

    /// True-e2e leaf generation: runs `leaf_prover` on the leaf simple bootloader (executing the
    /// simple-output task with `task_output`) against the registry's parameters, then wraps the
    /// produced `SerializedLeafProof` into a `LeafInput` with the dumped hashed-output preimage
    /// exactly as the backend does (hex felts from the dump file, re-encoded as decimal strings).
    fn generate_leaf(dir: &Path, task_output: &[u32]) -> LeafInput {
        let dump_path = dir.join("leaf_preimage.json");
        let input_path = dir.join("leaf_bl_input.json");
        std::fs::write(&input_path, leaf_bl_input_json(&dump_path, task_output)).unwrap();

        let leaf = prove_leaf_from_files(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../cairo-program-runner-lib/resources/compiled_programs/bootloaders/\
                 leaf_simple_bootloader_compiled.json",
            ),
            &Some(input_path),
            &circuit_registry_path(),
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
        LeafInput {
            proof: leaf,
            output: LeafOutputPreimage {
                hash_type: LeafOutputHashType::Blake2s,
                preimage: output_preimage,
            },
        }
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
        output_digest: [u32; circuit_common::N_RESERVED],
        circuit_hash: [u32; circuit_common::N_RESERVED],
        packed: PackedNode,
    }

    fn expected_leaf(leaf: &LeafInput) -> ExpectedNode {
        ExpectedNode {
            output_digest: leaf.output_digest().unwrap(),
            circuit_hash: leaf_circuit_hash(leaf),
            packed: PackedNode::leaf(leaf_circuit_hash(leaf), leaf.output.preimage.clone()),
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
            preimage.extend_from_slice(&child.output_digest);
        }
        let output_digest = blake2s_u32s_host(&preimage);
        ExpectedNode {
            output_digest,
            circuit_hash: multiverifier_circuit_hash,
            packed: PackedNode::Composite {
                circuit_hash: multiverifier_circuit_hash,
                subtasks: vec![left.packed, right.packed],
            },
        }
    }

    /// Mirrors `reduce_root_single`: the child's contribution enters the fold preimage twice,
    /// and the packed node carries the child once.
    fn expected_self_fold(
        child: ExpectedNode,
        multiverifier_circuit_hash: [u32; circuit_common::N_RESERVED],
    ) -> ExpectedNode {
        let mut preimage = Vec::new();
        for _ in 0..2 {
            preimage.extend_from_slice(&child.circuit_hash);
            preimage.extend_from_slice(&child.output_digest);
        }
        ExpectedNode {
            output_digest: blake2s_u32s_host(&preimage),
            circuit_hash: multiverifier_circuit_hash,
            packed: PackedNode::Composite {
                circuit_hash: multiverifier_circuit_hash,
                subtasks: vec![child.packed],
            },
        }
    }

    /// Folds `leaves` exactly as the binary does (pair adjacent, carry the odd one up; a single
    /// leaf self-folds) and returns the expected root node. `multiverifier_circuit_hash` is the
    /// `circuit_hash` every internal node carries (see [`multiverifier_circuit_hash`]).
    fn expected_root(
        leaves: &[LeafInput],
        multiverifier_circuit_hash: [u32; circuit_common::N_RESERVED],
    ) -> ExpectedNode {
        if leaves.len() == 1 {
            return expected_self_fold(expected_leaf(&leaves[0]), multiverifier_circuit_hash);
        }
        let mut layer: Vec<ExpectedNode> = leaves.iter().map(expected_leaf).collect();
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

    /// Folds the leaves with [`stwo_run_and_prove_recursive_tree`], validating the shape stats,
    /// root outputs and packed tree against an independent recomputation. Returns the
    /// packed tree.
    fn fold_and_check(
        leaves: &[LeafInput],
        dir: &Path,
        canonical: &CanonicalCircuit,
        root_proof_format: ProofFormat,
    ) -> PackedNode {
        init_tracing();
        let n = leaves.len();
        let stats = stwo_run_and_prove_recursive_tree(
            leaves.to_vec(),
            &circuit_registry(),
            &dir.join("root.proof"),
            &dir.join("root_outputs.json"),
            &dir.join("root_packed.json"),
            NonZeroUsize::new(1).unwrap(),
            root_proof_format,
        )
        .unwrap();

        // A balanced two-to-one tree with odd-carry: depth = ceil(log2 n), reductions = n - 1 —
        // except a single-leaf tree, which still runs one reduction (the root self-fold).
        let (expected_layers, expected_reductions) =
            if n == 1 { (1, 1) } else { (n.next_power_of_two().ilog2() as usize, n - 1) };
        assert_eq!(stats.n_leaves, n, "leaf count mismatch");
        assert_eq!(stats.n_layers, expected_layers, "layer count mismatch");
        assert_eq!(stats.n_pair_reductions, expected_reductions, "reduction count mismatch");

        // Independently recompute the whole tree (topology + hashed output values) from the leaves.
        let multiverifier_circuit_hash = multiverifier_circuit_hash(canonical);
        let expected = expected_root(leaves, multiverifier_circuit_hash);

        let actual_outputs: Vec<u32> =
            serde_json::from_str(&std::fs::read_to_string(dir.join("root_outputs.json")).unwrap())
                .unwrap();
        assert_eq!(actual_outputs, expected.output_digest.to_vec(), "root output values mismatch");

        let actual_packed: PackedNode =
            serde_json::from_str(&std::fs::read_to_string(dir.join("root_packed.json")).unwrap())
                .unwrap();
        assert_eq!(
            actual_packed, expected.packed,
            "packed output tree (topology + values) mismatch"
        );
        actual_packed
    }

    /// Folds `n` identical copies of `leaf` (see [`fold_and_check`]).
    fn dupe_and_fold(leaf: &LeafInput, n: usize, dir: &Path, root_proof_format: ProofFormat) {
        let canonical = CanonicalCircuit::build(&circuit_registry()).unwrap();
        fold_and_check(&vec![leaf.clone(); n], dir, &canonical, root_proof_format);
    }

    /// The leaf simple bootloader's input for the aggregator wrapper task. The wrapper runs one
    /// concat-aggregator task over a bootloader output of `n` copies of `leaf`'s task output, so it
    /// aggregates the same tasks the tree's other leaves prove. The bootloader dumps the preimage
    /// of its hashed output after the execution of the wrapper task.
    fn wrapper_leaf_bl_input_json(
        leaf: &LeafInput,
        n: usize,
        poseidon_dump_path: &Path,
        output_preimage_dump_path: &Path,
    ) -> String {
        let wrapper_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data/aggregator_wrapper_compiled.json");
        let task_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data/concat_aggregator_compiled.json");
        // A bootloader output: [config (one felt, skipped), n_tasks, records...], one
        // [record_size, program_hash, output...] record per task. Felts go in as quoted `0x`-hex
        // strings.
        let hex_token = |value: usize| format!("{:?}", format!("{value:#x}"));
        let preimage = &leaf.output.preimage;
        let record: Vec<String> = std::iter::once(hex_token(preimage.len() + 1))
            .chain(preimage.iter().map(|felt| {
                format!(
                    "{:?}",
                    format!("{:#x}", starknet_crypto::Felt::from_dec_str(felt).unwrap())
                )
            }))
            .collect();
        let bootloader_output = [hex_token(0), hex_token(n)]
            .into_iter()
            .chain(std::iter::repeat_n(record, n).flatten())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"{{
    "tasks": [{{
        "type": "RunProgramTask",
        "path": {wrapper_path:?},
        "program_input": {{
            "task": {{
                "type": "RunProgramTask",
                "path": {task_path:?},
                "program_input": {{"bootloader_output": [{bootloader_output}]}},
                "program_hash_function": "blake"
            }},
            "poseidon_preimage_dump_path": {poseidon_dump_path:?},
            "fact_topologies_path": null
        }},
        "program_hash_function": "blake"
    }}],
    "fact_topologies_path": null,
    "single_page": true,
    "output_preimage_dump_path": {output_preimage_dump_path:?}
}}"#
        )
    }

    /// Maps the fold's packed output onto the unpacker's witness tree.
    ///
    /// Leaves are encoded as `Composite { subtasks: [Plain { .. }], .. }`; internal nodes as
    /// `Composite { circuit_hash, subtasks: [Composite { .. }, ..] }`.
    fn packed_to_witness_tree(
        node: &PackedNode,
    ) -> circuits_unpacker::tree::BinaryTree<
        circuits_unpacker::unpacker::Node<QM31>,
        circuits::blake::HashValue<QM31>,
    > {
        use circuits::blake::HashValue;
        use circuits_unpacker::tree::BinaryTree;
        use circuits_unpacker::unpacker::Node;
        use starknet_types_core::hash::Blake2Felt252;
        match node {
            PackedNode::Composite { circuit_hash, subtasks } => match subtasks.as_slice() {
                // A leaf node: the leaf circuit outputs the Blake2s hash of its output preimage.
                [PackedNode::Plain { output_preimage }] => {
                    let felts: Vec<starknet_crypto::Felt> = output_preimage
                        .iter()
                        .map(|felt| starknet_crypto::Felt::from_dec_str(felt).unwrap())
                        .collect();
                    let digest = blake2s_u32s_host(&Blake2Felt252::encode_felts_to_u32s(&felts));
                    BinaryTree::Leaf(Node {
                        circuit_hash: HashValue::from(*circuit_hash),
                        subtree_hash: HashValue::from(digest),
                    })
                }
                [left, right] => BinaryTree::Internal(
                    HashValue::from(*circuit_hash),
                    Box::new([packed_to_witness_tree(left), packed_to_witness_tree(right)]),
                ),
                _ => panic!("unexpected packed-node arity"),
            },
            PackedNode::Plain { .. } => panic!("a plain node outside a composite leaf"),
        }
    }

    /// The applicative flow end to end:
    /// a. Prove the aggregator wrapper's leaf.
    /// b. Fold it as the tree's first leaf, next to the `n` task leaves.
    /// c. Prove the unpacking verifier circuit over the root proof.
    /// d. Prove the multiverifier circuit with the proof from c as its witness.
    fn applicative_fold_and_unpack(leaf: &LeafInput, n: usize, dir: &Path) {
        use applicative_unpacking_verifier::unpacking_verifier::{
            UnpackingCircuitParams, build_applicative_unpacking_verifier_circuit,
        };
        use circuit_common::finalize::pad_to_targets;
        use circuit_common::preprocessed::PreprocessedCircuit;
        use circuit_multiverifier::verify::CircuitVerifierInput;
        use circuit_prover::prover::prove_circuit_assignment;
        use circuit_serialize::deserialize::deserialize_proof_with_config;
        use circuit_serialize::serialize::CircuitSerialize;
        use circuits::blake::HashValue;
        use leaf_proof_format::DigestHex;
        use leaf_prover::prove_leaf::prove_leaf_from_files;
        use starknet_crypto::poseidon_hash_many;

        use crate::fold::{LayerEntry, extract_root_and_outputs, reduce_root_single};

        let registry = circuit_registry();
        let canonical = &CanonicalCircuit::build(&registry).unwrap();
        let proof_config = &canonical.shared_config.proof_config;
        let u32_from_hex =
            |hex: &str| u32::from_str_radix(hex.trim_start_matches("0x"), 16).unwrap();

        // 1. Prove the leaf simple bootloader running the aggregator wrapper over the concat
        //    aggregator, with the same `n` task records the tree's leaves hashed.
        let poseidon_dump_path = dir.join("wrapper_poseidon_preimage.json");
        let preimage_dump_path = dir.join("wrapper_output_preimage.json");
        let input_path = dir.join("wrapper_leaf_bl_input.json");
        std::fs::write(
            &input_path,
            wrapper_leaf_bl_input_json(leaf, n, &poseidon_dump_path, &preimage_dump_path),
        )
        .unwrap();
        let wrapper = prove_leaf_from_files(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../cairo-program-runner-lib/resources/compiled_programs/bootloaders/\
                 leaf_simple_bootloader_compiled.json",
            ),
            &Some(input_path),
            &circuit_registry_path(),
        );

        // 2. The bootloader dumped the preimage it hashed: the wrapper's program hash (asserted
        //    below), then its output felts — one digest per task, zero padding, and the
        //    poseidon-encoded aggregated output.
        let preimage_felts: Vec<String> =
            serde_json::from_str(&std::fs::read_to_string(&preimage_dump_path).unwrap()).unwrap();
        let wrapper_program_hash = starknet_crypto::Felt::from_hex(&preimage_felts[0]).unwrap();
        assert_eq!(
            wrapper_program_hash,
            starknet_crypto::Felt::from_dec_str(&super::program_hash(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("test_data/aggregator_wrapper_compiled.json"),
            ))
            .unwrap(),
            "the leaf hashed a different wrapper program"
        );
        // The wrapper's output words: the other tree slots' digests, then the final digest.
        let output_words: Vec<u32> = preimage_felts[1..].iter().map(|f| u32_from_hex(f)).collect();
        let capacity = applicative_unpacking_verifier::aggregator::CAPACITY;
        assert_eq!(output_words.len(), capacity * circuit_common::N_RESERVED);
        let leaf_digest = leaf.output_digest().unwrap();
        for digest in
            output_words[..n * circuit_common::N_RESERVED].chunks(circuit_common::N_RESERVED)
        {
            assert_eq!(digest, leaf_digest, "all the leaves should have the same digest");
        }
        assert!(
            output_words
                [n * circuit_common::N_RESERVED..(capacity - 1) * circuit_common::N_RESERVED]
                .iter()
                .all(|w| *w == 0),
            "padding digests must be zero"
        );
        let dumped_poseidon: Vec<String> =
            serde_json::from_str(&std::fs::read_to_string(&poseidon_dump_path).unwrap()).unwrap();
        let poseidon_felts: Vec<starknet_crypto::Felt> =
            dumped_poseidon.iter().map(|f| starknet_crypto::Felt::from_hex(f).unwrap()).collect();
        // It opens with the concat aggregator's modified program hash, blake("AGGREGATOR",
        // program hash), as its task is blake-hashed.
        let aggregator_program_hash = starknet_crypto::Felt::from_dec_str(&super::program_hash(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("test_data/concat_aggregator_compiled.json"),
        ))
        .unwrap();
        assert_eq!(
            poseidon_felts[0],
            starknet_types_core::hash::Blake2Felt252::encode_felt252_data_and_calc_blake_hash(&[
                starknet_crypto::Felt::from_bytes_be_slice(b"AGGREGATOR"),
                aggregator_program_hash,
            ]),
            "aggregated output opens with another aggregator's program hash"
        );
        let expected_records: Vec<starknet_crypto::Felt> =
            std::iter::repeat_n(&leaf.output.preimage, n)
                .flatten()
                .map(|felt| starknet_crypto::Felt::from_dec_str(felt).unwrap())
                .collect();
        assert_eq!(
            poseidon_felts[1..],
            expected_records,
            "aggregated output differs from the records"
        );
        let poseidon_hash = poseidon_hash_many(&poseidon_felts);
        let poseidon_bytes = poseidon_hash.to_bytes_le();
        let expected_final_digest: Vec<u32> =
            poseidon_bytes.chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
        assert_eq!(
            output_words[(capacity - 1) * circuit_common::N_RESERVED..],
            expected_final_digest,
            "final digest differs from the recomputed poseidon hash"
        );

        // 3. Fold the wrapper's proof — an ordinary leaf whose preimage is the dumped
        //    `[wrapper_program_hash, output...]` — as the tree's first leaf, next to the task
        //    leaves it attests to.
        let wrapper_leaf = LeafInput {
            proof: wrapper,
            output: LeafOutputPreimage {
                hash_type: LeafOutputHashType::Blake2s,
                preimage: preimage_felts
                    .iter()
                    .map(|f| starknet_crypto::Felt::from_hex(f).unwrap().to_biguint().to_string())
                    .collect(),
            },
        };
        let mut leaves = vec![wrapper_leaf];
        leaves.extend(std::iter::repeat_n(leaf.clone(), n));
        let packed = fold_and_check(&leaves, dir, canonical, ProofFormat::Circuit);

        // 4. Assemble and check the applicative unpacking verifier over the root proof.
        let root_bytes = std::fs::read(dir.join("root.proof")).unwrap();
        let root_outputs: Vec<u32> =
            serde_json::from_str(&std::fs::read_to_string(dir.join("root_outputs.json")).unwrap())
                .unwrap();
        let root_input = CircuitVerifierInput::<QM31> {
            proof: deserialize_proof_with_config(&mut root_bytes.as_slice(), proof_config).unwrap(),
            preprocessed_root: canonical
                .preprocessed_multiverifier
                .preprocessed_root(proof_config.fri.log_blowup_factor)
                .into(),
            output_digest: <[u32; 8]>::try_from(root_outputs.clone()).unwrap().into(),
        };
        let tree = packed_to_witness_tree(&packed);
        let to_hashes = |verifiers: &[circuit_registry::LeafVerifier]| -> Vec<HashValue<QM31>> {
            verifiers.iter().map(|v| HashValue::from(v.circuit_hash.0)).collect()
        };
        let leaf_set = to_hashes(&registry.leaf_verifiers);
        let node_set = vec![HashValue::from(registry.multiverifiers[0].circuit_hash.0)];
        let params = UnpackingCircuitParams {
            leaf_circuit_whitelist: &leaf_set,
            internal_node_circuit_whitelist: &node_set,
            aggregator_wrapper_program_hash: wrapper_program_hash,
        };

        let aggregated_output_digest: [u32; 8] =
            output_words[(capacity - 1) * circuit_common::N_RESERVED..].try_into().unwrap();
        let mut context = build_applicative_unpacking_verifier_circuit(
            root_input,
            &canonical.shared_config,
            &tree,
            aggregated_output_digest,
            &params,
        );
        assert!(context.is_circuit_valid(), "the unpacking verifier rejected the flow");

        // 5. Prove the unpacking verifier's run at the registry's shared target; the proof's
        //    circuit hash must be the registry's unpacking-verifier entry.
        pad_to_targets(&mut context, &canonical.target_sizes);
        let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut context);
        let unpacking_proof = prove_circuit_assignment(
            context.values(),
            &preprocessed,
            &canonical.base_column_pool,
            canonical.shared_config.pcs_config,
        )
        .unwrap();
        let extracted = extract_root_and_outputs(&unpacking_proof).unwrap();
        assert_eq!(
            DigestHex::from(unpacking_proof.circuit_hash.0),
            registry.unpacking_verifiers[0].circuit_hash,
            "the proven unpacking verifier differs from the registry entry"
        );
        assert_eq!(
            extracted.output_digest.to_vec(),
            expected_final_digest,
            "the unpacking verifier's outputs differ from the aggregated-output digest"
        );

        // 6. Prove the multiverifier circuit with `unpacking_proof` as both of its inputs.
        let mut unpacking_bytes = Vec::new();
        let (unpacking_serialized, _public_data) =
            circuit_prover::prover::prepare_circuit_proof_for_circuit_verifier(unpacking_proof);
        unpacking_serialized.serialize(&mut unpacking_bytes);
        let unpacking_entry = LayerEntry {
            proof_bytes: unpacking_bytes,
            preprocessed_root: extracted.preprocessed_root.clone(),
            output_digest: extracted.output_digest,
            packed_node: PackedNode::leaf(extracted.circuit_hash_words, vec![]),
        };
        reduce_root_single(unpacking_entry, canonical, ProofFormat::Circuit)
            .expect("the multiverifier rejected the unpacking verifier's proof");
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

    /// The single-leaf edge case: the root self-fold.
    #[test]
    fn test_fold_one_leaf() {
        let tmp = tempfile::tempdir().unwrap();
        dupe_and_fold(&golden_leaf(), 1, tmp.path(), ProofFormat::CairoVerifier);
    }

    #[test]
    fn test_fold_two_leaves() {
        let tmp = tempfile::tempdir().unwrap();
        dupe_and_fold(&golden_leaf(), 2, tmp.path(), ProofFormat::CairoVerifier);
    }

    #[test]
    fn test_fold_three_leaves_with_carry() {
        let tmp = tempfile::tempdir().unwrap();
        dupe_and_fold(&golden_leaf(), 3, tmp.path(), ProofFormat::CairoVerifier);
    }

    /// The applicative flow over the carry shape (see [`applicative_fold_and_unpack`]).
    #[test]
    fn test_fold_three_leaves_applicative() {
        let tmp = tempfile::tempdir().unwrap();
        applicative_fold_and_unpack(&golden_leaf(), 3, tmp.path());
    }

    /// True end-to-end: `leaf_prover` over the leaf simple bootloader (running the simple-output
    /// task), backend-style preimage injection, 4-leaf fold, comparison against the committed
    /// goldens at `test_data/goldens/four_leaves/`, and finally `scarb execute` of the Cairo
    /// circuit verifier on the fresh root proof (needs `scarb` on PATH, see `.tool-versions`) —
    /// so a circuit change whose proofs the committed verifier cannot verify fails here, whatever
    /// paths the change touched. When run with the `FIX` env var set, it regenerates the goldens
    /// instead of asserting.
    ///
    /// Note the circuit hashes have no dedicated trust-list golden: the trust anchors are the
    /// `circuit_params --registry` artifacts published to the artifacts bucket per commit, and
    /// hash drift is still caught here via the `root_packed.json` golden (every `Composite` node
    /// carries its `circuit_hash`).
    #[test]
    fn test_golden_four_leaves_e2e() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let leaf = generate_leaf(dir, &LEAF_TASK_OUTPUT);
        dupe_and_fold(&leaf, 4, dir, ProofFormat::CairoVerifier);

        // The root proof is the Cairo circuit verifier's input: execute the verifier on it.
        let status = std::process::Command::new("scarb")
            .args(["--profile", "proving", "execute", "-p", "stwo_circuit_verifier"])
            .args(["--features", "qm31_opcode", "--output", "none", "--arguments-file"])
            .arg(dir.join("root.proof"))
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../../stwo_cairo_verifier"))
            .status()
            .unwrap();
        assert!(status.success(), "the Cairo circuit verifier rejected the root proof");

        let goldens = goldens_dir();
        if std::env::var("FIX").is_ok() {
            std::fs::write(goldens.join("leaf.json"), serde_json::to_string_pretty(&leaf).unwrap())
                .unwrap();
            for file in
                ["leaf_preimage.json", "root.proof", "root_outputs.json", "root_packed.json"]
            {
                std::fs::copy(dir.join(file), goldens.join(file)).unwrap();
            }
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

    /// Structural fence for the fold's child-order preservation. With distinct leaves `A` and `B`
    /// laid out `[A, B, B]`, the produced packed tree is `Composite(Composite(A, B), B)`.
    #[test]
    fn test_fold_preserves_child_order_with_distinct_leaves() {
        let tmp = tempfile::tempdir().unwrap();
        let tmp_dir = tmp.path();

        // Two distinct real leaves: same leaf circuit, different task outputs → distinct output
        // preimages. Distinctness is a test invariant: with identical leaves the packed tree
        // check could not distinguish reorderings.
        let leaf_a = golden_leaf();
        let leaf_b = generate_leaf(tmp_dir, &[23, 29, 31]);
        assert_ne!(
            leaf_a.output.preimage, leaf_b.output.preimage,
            "test invariant: A and B must yield distinct preimages"
        );

        let canonical = CanonicalCircuit::build(&circuit_registry()).unwrap();
        fold_and_check(
            &[leaf_a, leaf_b.clone(), leaf_b],
            tmp_dir,
            &canonical,
            ProofFormat::CairoVerifier,
        );
    }
}
