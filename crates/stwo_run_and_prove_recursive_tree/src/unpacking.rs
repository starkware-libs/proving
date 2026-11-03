//! The applicative flow's step past the root. It proves the applicative unpacking verifier over
//! the folded tree, the circuit checking that the tree's first leaf, the aggregator wrapper,
//! aggregated exactly the tasks the other leaves prove, and emits that proof as the leaf of the
//! outer tree.

use std::iter::repeat;

use applicative_unpacking_verifier::aggregator::CAPACITY;
use applicative_unpacking_verifier::unpacking_verifier::{
    UnpackingCircuitParams, build_applicative_unpacking_verifier_circuit,
};
use circuit_common::N_RESERVED;
use circuit_common::finalize::pad_to_targets;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    prepare_circuit_proof_for_circuit_verifier, prove_circuit_assignment,
};
use circuit_registry::{CircuitRegistry, UnpackingVerifier};
use circuit_serialize::serialize::CircuitSerialize;
use circuits::blake::HashValue;
use circuits_unpacker::tree::BinaryTree;
use circuits_unpacker::unpacker::Node;
use itertools::chain;
use leaf_proof_format::{DigestHex, PackedNode, SerializedLeafProof};
use starknet_types_core::felt::Felt;
use stwo::core::fields::qm31::QM31;
use tracing::{Level, info, span};

use crate::RecursiveTreeError;
use crate::canonical::CanonicalCircuit;
use crate::fold::{NodeProofData, build_circuit_verifier_input, extract_root_and_outputs};
use crate::leaf_io::{LeafInput, LeafOutputHashType, LeafOutputPreimage};

/// What the applicative flow takes from the aggregator wrapper's leaf. The wrapper's
/// hashed-output preimage is `[program_hash, output...]`, its output one `u32` word per felt:
/// `CAPACITY - 1` task-slot digests, then the aggregated output's poseidon digest.
#[derive(Debug)]
pub(crate) struct WrapperLeaf {
    program_hash: Felt,
    task_slot_digests: Vec<[u32; N_RESERVED]>,
    aggregated_output_digest: [u32; N_RESERVED],
}

impl WrapperLeaf {
    /// Reads the wrapper's fields off its leaf's hashed-output preimage.
    pub(crate) fn parse(output: &LeafOutputPreimage) -> Result<Self, RecursiveTreeError> {
        if output.hash_type != LeafOutputHashType::Blake2s {
            return Err(RecursiveTreeError::BadWrapperLeaf {
                reason: format!("expected a blake2s preimage, got {:?}", output.hash_type),
            });
        }
        let preimage = &output.preimage;
        let expected_len = 1 + CAPACITY * N_RESERVED;
        if preimage.len() != expected_len {
            return Err(RecursiveTreeError::BadWrapperLeaf {
                reason: format!(
                    "expected a preimage of {expected_len} felts (the program hash and {CAPACITY} \
                     digests), got {}",
                    preimage.len()
                ),
            });
        }
        let program_hash = preimage[0];
        let words = preimage[1..]
            .iter()
            .map(|felt| {
                u32::try_from(*felt).map_err(|e| RecursiveTreeError::BadWrapperLeaf {
                    reason: format!("output word {felt:#x} is not a u32: {e}"),
                })
            })
            .collect::<Result<Vec<u32>, _>>()?;
        let (task_slot_words, aggregated_words) = words.split_at((CAPACITY - 1) * N_RESERVED);
        let task_slot_digests = task_slot_words
            .chunks_exact(N_RESERVED)
            .map(|digest| digest.try_into().expect("a chunk of N_RESERVED words"))
            .collect();
        let aggregated_output_digest =
            aggregated_words.try_into().expect("exactly N_RESERVED words");
        Ok(Self { program_hash, task_slot_digests, aggregated_output_digest })
    }
}

/// What the applicative flow needs beyond the fold: the registry's unpacking verifier entry and
/// the wrapper leaf's fields, both checked before the fold.
#[derive(Debug)]
pub(crate) struct ApplicativeInputs<'a> {
    pub(crate) unpacking_entry: &'a UnpackingVerifier,
    pub(crate) wrapper: WrapperLeaf,
}

/// Validates the applicative flow's inputs before the fold, so bad input fails before any proving.
pub(crate) fn validate_applicative_inputs<'a>(
    leaves: &[LeafInput],
    registry: &'a CircuitRegistry,
) -> Result<ApplicativeInputs<'a>, RecursiveTreeError> {
    // The registry has one unpacking verifier.
    let unpacking_entry = registry.unpacking_verifier()?;

    // 2 <= number of leaves <= CAPACITY, the wrapper leaf included.
    let n = leaves.len();
    if !(2..=CAPACITY).contains(&n) {
        return Err(RecursiveTreeError::BadLeafCount { n, capacity: CAPACITY });
    }

    // Every leaf's circuit is one of the registry's leaf verifiers, with that verifier's
    // preprocessed root: the fold checks the leaf's proof against the root, and the unpacking
    // verifier holds the leaf to the hash.
    let whitelisted = |leaf: &LeafInput| {
        registry.leaf_verifiers.iter().any(|v| {
            v.circuit_hash == leaf.proof.circuit_hash
                && v.preprocessed_root == leaf.proof.circuit_preprocessed_root
        })
    };
    if let Some(index) = leaves.iter().position(|leaf| !whitelisted(leaf)) {
        return Err(RecursiveTreeError::LeafCircuitNotWhitelisted {
            index,
            circuit_hash: leaves[index].proof.circuit_hash,
            preprocessed_root: leaves[index].proof.circuit_preprocessed_root,
        });
    }

    // The first leaf is a well-formed wrapper leaf.
    // TODO(yairv): consider computing the aggregator wrapper's program hash from the committed
    // `compiled_cairo_programs/aggregator_wrapper_compiled.json`, as the registry generation does
    // (`blake_program_hash`), and comparing it with the wrapper leaf's preimage here, so a wrapper
    // from another program fails before the fold rather than as `UnpackingVerifierCircuitHash`
    // after building the entire tree.
    let wrapper = WrapperLeaf::parse(&leaves[0].output)?;

    // The task leaves follow the wrapper's task order. The unpacking verifier opens the wrapper's
    // output over the task leaves' digests in fold order, then zero digests in the unused slots,
    // so an order mismatch would otherwise fail only after the fold and the verifier's prove.
    let task_digests =
        chain!(leaves[1..].iter().map(|leaf| leaf.output_digest().0), repeat([0; N_RESERVED]));
    let mismatch = wrapper
        .task_slot_digests
        .iter()
        .zip(task_digests)
        .enumerate()
        .find(|(_, (slot_digest, leaf_digest))| *slot_digest != leaf_digest);
    if let Some((slot, (slot_digest, leaf_digest))) = mismatch {
        return Err(RecursiveTreeError::WrapperTaskSlotMismatch {
            slot,
            slot_digest: DigestHex(*slot_digest),
            leaf_digest: DigestHex(leaf_digest),
        });
    }
    Ok(ApplicativeInputs { unpacking_entry, wrapper })
}

/// The applicative unpacking verifier's proof in the `leaf_prover` output format, the outer tree's
/// leaf, and the output digest it proved, the aggregated output's.
pub(crate) struct UnpackingLeaf {
    pub(crate) proof: SerializedLeafProof,
    pub(crate) output_digest: DigestHex,
}

/// Proves the applicative unpacking verifier over the folded tree:
/// - `root`: the inner fold's root proof data, in the `Circuit` proof format;
/// - `packed_node`: its packed tree;
/// - `leaf_digests`: the layer-0 output digests in fold order, the wrapper leaf's first;
/// - `inputs`: what `validate_applicative_inputs` checked, the registry's unpacking verifier entry
///   and the wrapper leaf;
/// - `registry`: the source of the verifier's leaf and multiverifier whitelists;
/// - `canonical`: supplies the shared circuit config and the target sizes the circuit is padded to.
///
/// Returns the unpacking verifier's proof in the `leaf_prover` output format and the output
/// digest it proved. The proven circuit must be the registry's unpacking verifier.
pub(crate) fn prove_unpacking_verifier(
    root: NodeProofData,
    packed_node: &PackedNode,
    leaf_digests: &[[u32; N_RESERVED]],
    inputs: &ApplicativeInputs<'_>,
    registry: &CircuitRegistry,
    canonical: &CanonicalCircuit,
) -> Result<UnpackingLeaf, RecursiveTreeError> {
    let _span = span!(Level::INFO, "prove_unpacking_verifier").entered();
    let ApplicativeInputs { unpacking_entry, wrapper } = inputs;

    // The unpacker's witness: the fold's packed tree with each leaf's output digest, one digest
    // per layer-0 entry.
    let root_input = build_circuit_verifier_input(root);
    let mut digests = leaf_digests.iter().copied();
    let tree = witness_tree(packed_node, &mut digests)?;
    if digests.next().is_some() {
        return Err(RecursiveTreeError::BadPackedTree {
            reason: "fewer leaves than layer-0 entries".to_string(),
        });
    }

    // The same whitelists, in the same order, as `circuit_params` baked into the registry's entry.
    let leaf_set: Vec<HashValue<QM31>> = registry
        .leaf_verifiers
        .iter()
        .map(|verifier| HashValue::from(verifier.circuit_hash.0))
        .collect();
    let internal_node_set = [HashValue::from(registry.multiverifier()?.circuit_hash.0)];
    let params = UnpackingCircuitParams {
        leaf_circuit_whitelist: &leaf_set,
        internal_node_circuit_whitelist: &internal_node_set,
        aggregator_wrapper_program_hash: wrapper.program_hash,
    };

    // Build the circuit over the witness and check it before proving, so a rejected tree fails
    // before any proving work.
    let mut context = build_applicative_unpacking_verifier_circuit::<QM31>(
        root_input,
        &canonical.shared_config,
        &tree,
        wrapper.aggregated_output_digest,
        &params,
    );
    if !context.is_circuit_valid() {
        return Err(RecursiveTreeError::UnpackingVerifierRejected);
    }

    // Prove the circuit, padded to the canonical component sizes.
    pad_to_targets(&mut context, &canonical.target_sizes);
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut context);
    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &canonical.base_column_pool,
        canonical.shared_config.pcs_config,
    )
    .map_err(|e| RecursiveTreeError::UnpackingVerifierProving(format!("{e:?}")))?;

    // A circuit hash other than the registry's means this binary and the registry disagree on the
    // unpacking verifier. One way is a wrapper leaf from another program than the
    // aggregator_wrapper the registry's entry was built for (its program hash is a constant in the
    // circuit).
    let extracted = extract_root_and_outputs(&circuit_proof);
    if extracted.circuit_hash != unpacking_entry.circuit_hash {
        return Err(RecursiveTreeError::UnpackingVerifierCircuitHash {
            expected: unpacking_entry.circuit_hash,
            got: extracted.circuit_hash,
            wrapper_program_hash: wrapper.program_hash,
        });
    }

    // Serialize the proof in the `leaf_prover` output format.
    let (proof, _public_data) = prepare_circuit_proof_for_circuit_verifier(circuit_proof);
    let mut proof_bytes = Vec::new();
    proof.serialize(&mut proof_bytes);
    info!("Applicative unpacking verifier proven.");
    Ok(UnpackingLeaf {
        proof: SerializedLeafProof {
            circuit_preprocessed_root: extracted.preprocessed_root,
            circuit_hash: extracted.circuit_hash,
            proof: proof_bytes,
        },
        output_digest: extracted.output_digest,
    })
}

/// Maps the fold's packed output onto the unpacker's witness tree: a `Composite` over one `Plain`
/// is a leaf, whose digest is the next of `leaf_digests` (fold order); a `Composite` over two
/// children is an internal node. Both carry their circuit hash.
fn witness_tree(
    node: &PackedNode,
    leaf_digests: &mut impl Iterator<Item = [u32; N_RESERVED]>,
) -> Result<BinaryTree<Node<QM31>, HashValue<QM31>>, RecursiveTreeError> {
    let bad = |reason: &str| RecursiveTreeError::BadPackedTree { reason: reason.to_string() };
    match node {
        PackedNode::Composite { circuit_hash, subtasks } => match subtasks.as_slice() {
            // A leaf: the leaf circuit over its task's preimage, with the next layer-0 digest.
            [PackedNode::Plain { .. }] => {
                let digest = leaf_digests.next().ok_or_else(|| bad("more leaves than entries"))?;
                Ok(BinaryTree::Leaf(Node {
                    circuit_hash: HashValue::from(circuit_hash.0),
                    subtree_hash: HashValue::from(digest),
                }))
            }
            // A fold node over two children.
            [left, right] => Ok(BinaryTree::Internal(
                HashValue::from(circuit_hash.0),
                Box::new([witness_tree(left, leaf_digests)?, witness_tree(right, leaf_digests)?]),
            )),
            // Malformed: a self-fold, which only a one-leaf tree has, or more than two children.
            [_] => Err(bad("a self-folded node; the wrapper's tree has at least two leaves")),
            _ => Err(bad("a node with more than two children")),
        },
        // A `Plain` node only appears under a leaf.
        PackedNode::Plain { .. } => Err(bad("a plain node outside a leaf")),
    }
}
