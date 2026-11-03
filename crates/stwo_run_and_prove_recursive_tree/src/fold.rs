//! The per-node tree state and the core pair reduction (the circuit-world replacement for the old
//! Cairo simple-bootloader pair run).

use circuit_cairo_serialize::prepare_circuit_proof_for_cairo_verifier;
use circuit_common::finalize::pad_to_targets;
use circuit_prover::prover::{
    prepare_circuit_proof_for_circuit_verifier, prove_circuit_assignment,
    prove_circuit_assignment_with_channel,
};
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_verifier::circuit_proof::CircuitProof;
use circuit_verifier::multiverifier::{CircuitVerifierInput, build_multiverifier_circuit};
use circuit_verifier::statement::{all_circuit_components, circuit_component_log_sizes};
use circuits::blake::HashValue;
use circuits::ivalue::NoValue;
use circuits_stark_verifier::proof::{Proof, ProofConfig};
use leaf_proof_format::{DigestHex, PackedNode};
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo::core::vcs_lifted::merkle_hasher::MerkleHasherLifted;
use stwo::core::verifier::PREPROCESSED_TRACE_IDX;
use tracing::{Level, span};

use crate::RecursiveTreeError;
use crate::canonical::CanonicalCircuit;
use crate::leaf_io::LeafInput;

/// A tree node's proof, in the form its consumer reads. Leaves, internal and carried nodes, and a
/// [`ProofFormat::Circuit`] root hold the typed proof a circuit verifier checks. A
/// [`ProofFormat::CairoVerifier`] root holds the serialized JSON of that format.
pub enum NodeProof {
    Circuit(Box<Proof<QM31>>),
    CairoVerifierJson(String),
}

impl NodeProof {
    /// The [`ProofFormat`] this proof is in.
    pub fn format(&self) -> ProofFormat {
        match self {
            NodeProof::Circuit(_) => ProofFormat::Circuit,
            NodeProof::CairoVerifierJson(_) => ProofFormat::CairoVerifier,
        }
    }
}

/// In-memory representation of a single tree node during reduction. At layer 0 these wrap the leaf
/// proofs; at higher layers each entry is the result of folding two children.
pub struct LayerEntry {
    /// This node's proof and public values.
    pub proof_data: NodeProofData,
    /// The packed subtree rooted at this node.
    pub packed_node: PackedNode,
}

/// A tree node's proof and the public values a multiverifier checks it against.
pub struct NodeProofData {
    /// This node's proof.
    pub proof: NodeProof,
    /// Preprocessed root of this node's proof: for a layer-0 leaf it's whatever root the manifest
    /// declared; for an internal/carried node it's the multiverifier root read from the freshly
    /// proved proof.
    pub preprocessed_root: DigestHex,
    /// This node's circuit output digest, a public input to the multiverifier that verifies this
    /// node.
    pub output_digest: DigestHex,
}

impl LayerEntry {
    /// Builds the layer-0 entry for a leaf from its [`LeafInput`], deserializing the inline proof
    /// with `proof_config`, the config the leaf was proven against. The preprocessed root travels
    /// inline in the flattened [`leaf_proof_format::SerializedLeafProof`], and the output digest is
    /// recomputed from the hashed-output preimage next to it.
    pub fn from_leaf(
        leaf: LeafInput,
        proof_config: &ProofConfig,
    ) -> Result<Self, RecursiveTreeError> {
        let proof = deserialize_proof_with_config(&mut leaf.proof.proof.as_slice(), proof_config)
            .map_err(|e| RecursiveTreeError::Deserialize(format!("{e:?}")))?;
        Ok(Self {
            proof_data: NodeProofData {
                proof: NodeProof::Circuit(Box::new(proof)),
                preprocessed_root: leaf.proof.circuit_preprocessed_root,
                output_digest: leaf.output_digest(),
            },
            // The preimage whose (blake or poseidon) digest is the leaf's output.
            packed_node: PackedNode::leaf(leaf.proof.circuit_hash, leaf.output.preimage),
        })
    }
}

/// A fold proof's channel and wire format. Internal folds always use `Circuit`, since another
/// multiverifier re-verifies them; the final reduction uses what the root's consumer expects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum ProofFormat {
    /// For the Cairo circuit verifier program: the Blake2s Merkle channel, serialized as the
    /// felt252 hex-string JSON stream `scarb execute --arguments-file` expects.
    CairoVerifier,
    /// For an enclosing circuit (the applicative unpacking verifier): proven like the internal
    /// folds and written as the serialized `Proof<QM31>`, consumable as a `CircuitVerifierInput`.
    Circuit,
    /// The applicative flow: the tree is folded as `Circuit`, the applicative unpacking verifier
    /// is proven over it, and that proof is the one written, in the `leaf_prover` output format,
    /// as the outer tree's leaf (see [`crate::unpacking`]).
    ApplicativeCircuit,
}

/// Folds two children into a new parent entry: builds a multiverifier circuit that verifies both
/// children, proves it against the canonical preprocessed circuit, and stores the resulting proof
/// in the parent entry for the next layer.
///
/// `proof_format` is `Circuit` for an internal fold, and the tree's requested root format for the
/// final reduction (see [`ProofFormat`]).
pub(crate) fn reduce_pair(
    left: LayerEntry,
    right: LayerEntry,
    layer_idx: usize,
    pair_idx: usize,
    canonical: &CanonicalCircuit,
    proof_format: ProofFormat,
) -> Result<LayerEntry, RecursiveTreeError> {
    let input0 = build_circuit_verifier_input(left.proof_data);
    let input1 = build_circuit_verifier_input(right.proof_data);
    reduce(
        input0,
        input1,
        vec![left.packed_node, right.packed_node],
        layer_idx,
        pair_idx,
        canonical,
        proof_format,
    )
}

/// The single-leaf tree's root pass: folds the lone leaf with itself, so the root is an ordinary
/// multiverifier proof.
pub(crate) fn reduce_root_single(
    entry: LayerEntry,
    canonical: &CanonicalCircuit,
    root_proof_format: ProofFormat,
) -> Result<LayerEntry, RecursiveTreeError> {
    let input = build_circuit_verifier_input(entry.proof_data);
    reduce(input.clone(), input, vec![entry.packed_node], 1, 0, canonical, root_proof_format)
}

/// Builds the [`CircuitVerifierInput`] the next fold verifies from a current layer's node.
///
/// Panics on a [`ProofFormat::CairoVerifier`] proof. Only the root is proven in that format, and
/// the root never re-enters a fold.
pub(crate) fn build_circuit_verifier_input(
    proof_data: NodeProofData,
) -> CircuitVerifierInput<QM31> {
    let NodeProofData { proof, preprocessed_root, output_digest } = proof_data;
    let NodeProof::Circuit(proof) = proof else {
        panic!("a Cairo-verifier-format root proof never re-enters a fold");
    };
    CircuitVerifierInput {
        proof: *proof,
        preprocessed_root: HashValue::from(preprocessed_root.0),
        output_digest: output_digest.0.into(),
    }
}

/// Shared reduction body: proves a multiverifier over the two inputs and wraps the result in the
/// parent entry, with `subtasks` as its packed children.
fn reduce(
    input0: CircuitVerifierInput<QM31>,
    input1: CircuitVerifierInput<QM31>,
    subtasks: Vec<PackedNode>,
    layer_idx: usize,
    pair_idx: usize,
    canonical: &CanonicalCircuit,
    proof_format: ProofFormat,
) -> Result<LayerEntry, RecursiveTreeError> {
    let _span = span!(Level::INFO, "reduce", layer_idx, pair_idx, ?proof_format).entered();

    let mut context =
        build_multiverifier_circuit::<QM31>(vec![input0, input1], &canonical.shared_config);
    pad_to_targets(&mut context, &canonical.target_sizes);
    debug_assert!(
        context.is_circuit_valid(),
        "multiverifier circuit rejected its inputs at layer {layer_idx} pair {pair_idx}"
    );
    // In a release build the check above is compiled out, so a bad child first surfaces as a
    // prover error; this is where the failing reduction gets named.
    let proving_error =
        |e| RecursiveTreeError::Proving { layer_idx, pair_idx, reason: format!("{e:?}") };

    let (proof, extracted) = if proof_format == ProofFormat::CairoVerifier {
        let circuit_proof = prove_circuit_assignment_with_channel::<Blake2sMerkleChannel>(
            context.values(),
            &canonical.preprocessed_multiverifier,
            &canonical.base_column_pool,
            canonical.shared_config.pcs_config,
        )
        .map_err(proving_error)?;
        let extracted = extract_root_and_outputs(&circuit_proof);

        // Serialize for the Cairo circuit verifier: only the proof goes on the wire; the
        // verifier-config constants are baked into the Cairo binary.
        let component_log_sizes = circuit_component_log_sizes(
            &all_circuit_components::<NoValue>(),
            &canonical.preprocessed_multiverifier.preprocessed_trace.log_sizes(),
        );
        let felts = prepare_circuit_proof_for_cairo_verifier(circuit_proof, &component_log_sizes);
        let proof_hex: Vec<String> = felts.iter().map(|felt| format!("0x{felt:x}")).collect();
        (NodeProof::CairoVerifierJson(serde_json::to_string_pretty(&proof_hex)?), extracted)
    } else {
        let circuit_proof = prove_circuit_assignment(
            context.values(),
            &canonical.preprocessed_multiverifier,
            &canonical.base_column_pool,
            canonical.shared_config.pcs_config,
        )
        .map_err(proving_error)?;
        let extracted = extract_root_and_outputs(&circuit_proof);

        let (proof, _public_data) = prepare_circuit_proof_for_circuit_verifier(circuit_proof);
        (NodeProof::Circuit(Box::new(proof)), extracted)
    };

    Ok(LayerEntry {
        proof_data: NodeProofData {
            proof,
            preprocessed_root: extracted.preprocessed_root,
            output_digest: extracted.output_digest,
        },
        packed_node: PackedNode::Composite { circuit_hash: extracted.circuit_hash, subtasks },
    })
}

/// The parent-entry data extracted from a freshly proven circuit proof: its preprocessed root (for
/// the multiverifier input that re-verifies this node), the `circuit_hash` identifying the circuit
/// (the unpacker's trust anchor, carried in `PackedNode`), and its circuit output digest.
pub(crate) struct ExtractedProofData {
    pub(crate) preprocessed_root: DigestHex,
    pub(crate) circuit_hash: DigestHex,
    pub(crate) output_digest: DigestHex,
}

/// Extracts the parent entry's preprocessed root and output digest from a freshly proven circuit
/// proof.
pub(crate) fn extract_root_and_outputs<H: MerkleHasherLifted<Hash = Blake2sHash>>(
    circuit_proof: &CircuitProof<H>,
) -> ExtractedProofData {
    let root_hash = circuit_proof.stark_proof.proof.commitments[PREPROCESSED_TRACE_IDX];
    let preprocessed_root = DigestHex::from(root_hash.0);
    // The circuit's full identity, as the multiverifier mixes it and the unpacker recomputes it.
    let circuit_hash = DigestHex::from(circuit_proof.circuit_hash.0);
    let output_digest = DigestHex(circuit_proof.claim.output_digest);
    ExtractedProofData { preprocessed_root, circuit_hash, output_digest }
}
