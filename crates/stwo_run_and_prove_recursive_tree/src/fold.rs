//! The per-node tree state and the core pair reduction (the circuit-world replacement for the old
//! Cairo simple-bootloader pair run).

use circuit_cairo_serialize::prepare_circuit_proof_for_cairo_verifier;
use circuit_common::finalize::pad_to_targets;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    SimdBackend, prepare_circuit_proof_for_circuit_verifier, prove_circuit_assignment_with_channel,
};
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_verifier::circuit_proof::CircuitProof;
use circuit_verifier::multiverifier::{CircuitVerifierInput, build_multiverifier_circuit};
use circuit_verifier::statement::{all_circuit_components, circuit_component_log_sizes};
use circuits::hash::HashValue;
use circuits::ivalue::NoValue;
use circuits_stark_verifier::proof::{Proof, ProofConfig};
use leaf_proof_format::{DigestHex, PackedNode};
use starknet_types_core::felt::Felt;
use stwo::core::channel::MerkleChannel;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs::poseidon252_hash::Poseidon252Hash;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo::core::vcs_lifted::hasher::Hasher;
use stwo::core::vcs_lifted::merkle_hasher::MerkleHasherLifted;
use stwo::core::vcs_lifted::poseidon252_merkle::Poseidon252MerkleChannel;
use stwo::core::verifier::PREPROCESSED_TRACE_IDX;
use stwo::prover::backend::BackendForChannel;
use stwo_cairo_serialize::{CairoDeserialize, CairoSerialize};
use tracing::{Level, span};

use crate::RecursiveTreeError;
use crate::canonical::CanonicalCircuit;
use crate::leaf_io::LeafInput;

/// A tree node's proof, in the form its consumer reads. Leaves, internal and carried nodes, and a
/// [`ProofFormat::Circuit`] root hold the typed proof a circuit verifier checks. A
/// [`ProofFormat::CairoVerifier`] or [`ProofFormat::PoseidonCairoVerifier`] root holds the
/// serialized JSON of that format.
pub enum NodeProof {
    Circuit(Box<Proof<QM31>>),
    CairoVerifierJson(String),
    PoseidonCairoVerifierJson(String),
}

impl NodeProof {
    /// The [`ProofFormat`] this proof is in.
    pub fn format(&self) -> ProofFormat {
        match self {
            NodeProof::Circuit(_) => ProofFormat::Circuit,
            NodeProof::CairoVerifierJson(_) => ProofFormat::CairoVerifier,
            NodeProof::PoseidonCairoVerifierJson(_) => ProofFormat::PoseidonCairoVerifier,
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
        let mut proof_bytes = leaf.proof.proof.as_slice();
        let proof = deserialize_proof_with_config(&mut proof_bytes, proof_config)
            .map_err(|e| RecursiveTreeError::Deserialize(format!("{e:?}")))?;
        if !proof_bytes.is_empty() {
            return Err(RecursiveTreeError::Deserialize(format!(
                "{} trailing bytes after the proof",
                proof_bytes.len()
            )));
        }
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
    /// For the Cairo circuit verifier program: the Blake2s Merkle channel under the registry's
    /// `cairo_verifier_root` config, serialized as the felt252 hex-string JSON stream
    /// `scarb execute --arguments-file` expects. The root's identity is the registry's
    /// `cairo_verifier_root.blake_circuit_hash`.
    CairoVerifier,
    /// For the Cairo circuit verifier built with `poseidon252_verifier`, the outer bootloader's
    /// verifier task: the Poseidon252 Merkle channel under the same config, in the same stream.
    /// The root's identity is then a felt, the registry's
    /// `cairo_verifier_root.poseidon_circuit_hash`.
    PoseidonCairoVerifier,
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
/// Panics on a proof in a Cairo verifier format, [`ProofFormat::CairoVerifier`] or
/// [`ProofFormat::PoseidonCairoVerifier`]. Only the root is proven in those formats, and the root
/// never re-enters a fold.
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
/// parent entry, with `subtasks` as its packed children. The proof's circuit hash is held to the
/// registry's multiverifier entry, the identity the unpackers hold every fold node to, or for a
/// root proven for a Cairo circuit verifier, to that root's identity.
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

    let values = context.values();
    let (proof, extracted) = match proof_format {
        ProofFormat::CairoVerifier => {
            let root = canonical
                .cairo_verifier_root
                .as_ref()
                .ok_or(RecursiveTreeError::NoCairoVerifierRoot)?;
            // The root's own config, not the one its children were proven with: the multiverifier
            // circuit verifies the children under `shared_config`, whatever config proves it.
            let (circuit_proof, extracted) = prove_fold::<Blake2sMerkleChannel>(
                values,
                canonical,
                root.pcs_config,
                layer_idx,
                pair_idx,
            )?;
            // The circuit hash covers the blowup, so the root's differs from the folds'.
            if extracted.circuit_hash != root.blake_circuit_hash {
                return Err(RecursiveTreeError::RootCircuitHash {
                    expected: root.blake_circuit_hash,
                    got: extracted.circuit_hash,
                });
            }
            let json =
                cairo_verifier_proof_json(circuit_proof, &canonical.preprocessed_multiverifier)?;
            (NodeProof::CairoVerifierJson(json), extracted)
        }
        ProofFormat::PoseidonCairoVerifier => {
            let root = canonical
                .cairo_verifier_root
                .as_ref()
                .ok_or(RecursiveTreeError::NoCairoVerifierRoot)?;
            // Under the root's own config, as in the `CairoVerifier` arm.
            let (circuit_proof, extracted) = prove_fold::<Poseidon252MerkleChannel>(
                values,
                canonical,
                root.pcs_config,
                layer_idx,
                pair_idx,
            )?;
            let circuit_hash = Felt::from_bytes_be(&circuit_proof.circuit_hash.to_bytes_be());
            if circuit_hash != root.poseidon_circuit_hash {
                return Err(RecursiveTreeError::PoseidonRootCircuitHash {
                    expected: root.poseidon_circuit_hash,
                    got: circuit_hash,
                });
            }
            let json =
                cairo_verifier_proof_json(circuit_proof, &canonical.preprocessed_multiverifier)?;
            (NodeProof::PoseidonCairoVerifierJson(json), extracted)
        }
        ProofFormat::Circuit | ProofFormat::ApplicativeCircuit => {
            let (circuit_proof, extracted) = prove_fold::<Blake2sMerkleChannel>(
                values,
                canonical,
                canonical.shared_config.pcs_config,
                layer_idx,
                pair_idx,
            )?;
            // A fold must prove the registry's multiverifier circuit. The prover computes the hash
            // anyway, so the check costs nothing.
            if extracted.circuit_hash != canonical.multiverifier_circuit_hash {
                return Err(RecursiveTreeError::MultiverifierCircuitHash {
                    expected: canonical.multiverifier_circuit_hash,
                    got: extracted.circuit_hash,
                });
            }
            // The public data is the output digest, which `extracted` already holds.
            let (proof, _public_data) = prepare_circuit_proof_for_circuit_verifier(circuit_proof);
            (NodeProof::Circuit(Box::new(proof)), extracted)
        }
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

/// Proves the padded multiverifier circuit's assignment `values` with the channel `MC` under
/// `pcs_config`. Returns the proof and the parent's extracted proof data.
fn prove_fold<MC>(
    values: &[QM31],
    canonical: &CanonicalCircuit,
    pcs_config: PcsConfig,
    layer_idx: usize,
    pair_idx: usize,
) -> Result<(CircuitProof<MC::H>, ExtractedProofData), RecursiveTreeError>
where
    MC: MerkleChannel,
    SimdBackend: BackendForChannel<MC>,
    <MC::H as Hasher>::Hash: HashDigest,
{
    // In a release build `reduce`'s validity check is compiled out, so a bad child first surfaces
    // as a prover error; this is where the failing reduction gets named.
    let proving_error =
        |e| RecursiveTreeError::Proving { layer_idx, pair_idx, reason: format!("{e:?}") };
    let circuit_proof = prove_circuit_assignment_with_channel::<MC>(
        values,
        &canonical.preprocessed_multiverifier,
        &canonical.base_column_pool,
        pcs_config,
    )
    .map_err(proving_error)?;
    let extracted = extract_root_and_outputs(&circuit_proof);
    Ok((circuit_proof, extracted))
}

/// Serializes a fold proof into the Cairo circuit verifier's arguments file; see
/// [`prepare_circuit_proof_for_cairo_verifier`] for the stream's layout.
fn cairo_verifier_proof_json<H: MerkleHasherLifted>(
    circuit_proof: CircuitProof<H>,
    preprocessed_multiverifier: &PreprocessedCircuit,
) -> Result<String, RecursiveTreeError>
where
    H::Hash: CairoSerialize + CairoDeserialize,
{
    let component_log_sizes = circuit_component_log_sizes(
        &all_circuit_components::<NoValue>(),
        &preprocessed_multiverifier.preprocessed_trace.log_sizes(),
    );
    let felts = prepare_circuit_proof_for_cairo_verifier(circuit_proof, &component_log_sizes);
    let proof_hex: Vec<String> = felts.iter().map(|felt| format!("0x{felt:x}")).collect();
    Ok(serde_json::to_string_pretty(&proof_hex)?)
}

/// The parent-entry data extracted from a freshly proven circuit proof: its preprocessed root (for
/// the multiverifier input that re-verifies this node), the `circuit_hash` identifying the circuit
/// (the unpacker's trust anchor, carried in `PackedNode`), and its circuit output digest.
pub(crate) struct ExtractedProofData {
    pub(crate) preprocessed_root: DigestHex,
    pub(crate) circuit_hash: DigestHex,
    pub(crate) output_digest: DigestHex,
}

/// Extracts the parent entry's preprocessed root, circuit hash and output digest from a freshly
/// proven circuit proof.
pub(crate) fn extract_root_and_outputs<H: MerkleHasherLifted>(
    circuit_proof: &CircuitProof<H>,
) -> ExtractedProofData
where
    H::Hash: HashDigest,
{
    let preprocessed_root =
        circuit_proof.stark_proof.proof.commitments[PREPROCESSED_TRACE_IDX].digest_hex();
    // The circuit's full identity, as the multiverifier mixes it and the unpacker recomputes it.
    let circuit_hash = circuit_proof.circuit_hash.digest_hex();
    let output_digest = DigestHex(circuit_proof.claim.output_digest);
    ExtractedProofData { preprocessed_root, circuit_hash, output_digest }
}

/// A channel's hash in the [`DigestHex`] encoding, the form in which preprocessed roots and
/// circuit identities travel outside the circuits.
pub(crate) trait HashDigest {
    fn digest_hex(&self) -> DigestHex;
}

/// Matching `HashValue`'s `From<Blake2sHash>`: the digest's bytes as eight little-endian words.
impl HashDigest for Blake2sHash {
    fn digest_hex(&self) -> DigestHex {
        DigestHex::from(self.0)
    }
}

/// How a Poseidon252 root's felt identity sits in the packed tree, whose consumer holds it to the
/// config's felt rather than looking it up: the felt's little-endian bytes as eight words.
impl HashDigest for Poseidon252Hash {
    fn digest_hex(&self) -> DigestHex {
        let mut bytes_le = self.to_bytes_be();
        bytes_le.reverse();
        DigestHex::from(bytes_le)
    }
}
