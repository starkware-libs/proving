//! Writing the three root-layer output files.

use std::borrow::Cow;

use circuit_common::N_RESERVED;
use circuit_serialize::serialize::CircuitSerialize;
use leaf_proof_format::PackedNode;

use crate::fold::{LayerEntry, NodeProof, ProofFormat};
use crate::{RecursiveTreeError, RootOutputPaths};

/// Writes the root node's proof, output digest and packed tree to `paths` (see
/// [`RootOutputPaths`]).
///
/// Panics if the root's proof is not in `root_proof_format`, the format the fold was asked to
/// prove its root in.
pub fn write_root_outputs(
    root: &LayerEntry,
    root_proof_format: ProofFormat,
    paths: &RootOutputPaths<'_>,
) -> Result<(), RecursiveTreeError> {
    assert_eq!(
        root.proof_data.proof.format(),
        root_proof_format,
        "the fold must prove its root in the requested format"
    );
    let proof_bytes: Cow<'_, [u8]> = match &root.proof_data.proof {
        NodeProof::Circuit(proof) => {
            let mut bytes = Vec::new();
            proof.serialize(&mut bytes);
            Cow::Owned(bytes)
        }
        NodeProof::CairoVerifierJson(json) => Cow::Borrowed(json.as_bytes()),
    };
    write_outputs(&proof_bytes, &root.proof_data.output_digest.0, &root.packed_node, paths)
}

/// Writes a proof's bytes, its circuit's output digest and the fold's packed tree to `paths` (see
/// [`RootOutputPaths`]).
pub fn write_outputs(
    proof_bytes: &[u8],
    output_digest: &[u32; N_RESERVED],
    packed_node: &PackedNode,
    paths: &RootOutputPaths<'_>,
) -> Result<(), RecursiveTreeError> {
    std::fs::write(paths.proof, proof_bytes)
        .map_err(|e| RecursiveTreeError::PathIO(e, paths.proof.to_path_buf()))?;

    std::fs::write(paths.program_output, sonic_rs::to_string(output_digest)?)
        .map_err(|e| RecursiveTreeError::PathIO(e, paths.program_output.to_path_buf()))?;

    std::fs::write(paths.packed_node_tree, serde_json::to_string(packed_node)?)
        .map_err(|e| RecursiveTreeError::PathIO(e, paths.packed_node_tree.to_path_buf()))?;
    Ok(())
}
