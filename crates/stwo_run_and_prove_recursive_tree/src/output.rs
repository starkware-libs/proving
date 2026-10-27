//! Writing the three root-layer output files.

use std::path::Path;

use circuit_serialize::serialize::CircuitSerialize;

use crate::RecursiveTreeError;
use crate::fold::{LayerEntry, NodeProof, ProofFormat};

/// Writes the root-layer outputs to the caller-configured paths:
/// - `proof_path`: the root node's proof, in `root_proof_format`.
/// - `program_output`: the root node's output digest, as a JSON array of raw `u32` words.
/// - `packed_node_tree_path`: the nested `PackedNode` JSON tree mirroring the whole fold.
///
/// Panics if the root's proof is not in `root_proof_format`, the format the fold was asked to
/// prove its root in.
pub fn write_root_outputs(
    root: &LayerEntry,
    root_proof_format: ProofFormat,
    proof_path: &Path,
    program_output: &Path,
    packed_node_tree_path: &Path,
) -> Result<(), RecursiveTreeError> {
    assert_eq!(
        root.proof_data.proof.format(),
        root_proof_format,
        "the fold must prove its root in the requested format"
    );
    match &root.proof_data.proof {
        NodeProof::Circuit(proof) => {
            let mut bytes = Vec::new();
            proof.serialize(&mut bytes);
            std::fs::write(proof_path, bytes)
        }
        NodeProof::CairoVerifierJson(json) => std::fs::write(proof_path, json),
    }
    .map_err(|e| RecursiveTreeError::PathIO(e, proof_path.to_path_buf()))?;

    std::fs::write(program_output, sonic_rs::to_string(&root.proof_data.output_digest.0)?)
        .map_err(|e| RecursiveTreeError::PathIO(e, program_output.to_path_buf()))?;

    std::fs::write(packed_node_tree_path, serde_json::to_string(&root.packed_node)?)
        .map_err(|e| RecursiveTreeError::PathIO(e, packed_node_tree_path.to_path_buf()))?;
    Ok(())
}
