//! Loading the leaves for a fold.
//!
//! The binary is given a manifest (`{"leaves": ["<path>", ...]}`) listing one leaf input file per
//! leaf, each a [`LeafInput`].

use std::path::PathBuf;

use blake2::{Blake2s256, Digest};
use circuit_common::N_RESERVED;
use leaf_proof_format::SerializedLeafProof;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::{Blake2Felt252, Poseidon, StarkHash};

use crate::RecursiveTreeError;

/// One leaf input to the recursive tree: the raw `leaf_prover` output plus the preimage of the
/// output digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeafInput {
    /// The `leaf_prover` output file's contents, verbatim.
    #[serde(flatten)]
    pub proof: SerializedLeafProof,
    /// The leaf circuit's output preimage and its hash type.
    pub output: LeafOutputPreimage,
}

/// The preimage of a leaf circuit's output digest, and the hash that maps it to that digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeafOutputPreimage {
    pub hash_type: LeafOutputHashType,
    /// The preimage's felts as decimal strings, as the leaf's run dumped them.
    pub preimage: Vec<String>,
}

/// The hash a leaf's output preimage opens with; see [`LeafInput::output_digest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeafOutputHashType {
    Blake2s,
    Poseidon,
}

impl LeafInput {
    /// This leaf circuit's `N_RESERVED`-word output digest: the digest of its output preimage, as
    /// eight little-endian u32 words.
    ///
    /// A blake leaf's is `blake2s(cairo0-encode(preimage))`, which the leaf bootloader writes to
    /// its output segment as a Uint256 (low, high) and the leaf circuit emits verbatim (see
    /// `CairoStatement::new`'s `set_outputs(output_hash)`). A poseidon leaf's is
    /// `poseidon_hash_many(preimage)` in the same layout.
    pub fn output_digest(&self) -> Result<[u32; N_RESERVED], RecursiveTreeError> {
        let preimage: Vec<Felt> = self
            .output
            .preimage
            .iter()
            .map(|felt| {
                Felt::from_dec_str(felt).map_err(|e| RecursiveTreeError::BadLeafOutputs {
                    reason: format!("invalid decimal felt {felt:?} in the preimage: {e}"),
                })
            })
            .collect::<Result<_, _>>()?;

        let digest_bytes: [u8; 32] = match self.output.hash_type {
            // The u32-word encoding Cairo's `encode_felt252_data_and_calc_blake2s` hashes.
            LeafOutputHashType::Blake2s => {
                let encoded_bytes: Vec<u8> = Blake2Felt252::encode_felts_to_u32s(&preimage)
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect();
                Blake2s256::digest(&encoded_bytes).into()
            }
            LeafOutputHashType::Poseidon => Poseidon::hash_array(&preimage).to_bytes_le(),
        };
        // The digest's 32 bytes read as eight little-endian words.
        Ok(std::array::from_fn(|i| {
            u32::from_le_bytes(digest_bytes[i * 4..i * 4 + 4].try_into().unwrap())
        }))
    }
}

/// Reads the leaves manifest (`{"leaves": ["<path>", ...]}`) and loads each referenced leaf input
/// file into a [`LeafInput`], preserving order.
pub fn load_leaves(path: &PathBuf) -> Result<Vec<LeafInput>, RecursiveTreeError> {
    #[derive(Deserialize)]
    struct LeavesManifest {
        leaves: Vec<PathBuf>,
    }
    let content =
        std::fs::read_to_string(path).map_err(|e| RecursiveTreeError::PathIO(e, path.clone()))?;
    let manifest: LeavesManifest = serde_json::from_str(&content)?;
    manifest
        .leaves
        .iter()
        .map(|leaf_path| {
            let leaf_json = std::fs::read_to_string(leaf_path)
                .map_err(|e| RecursiveTreeError::PathIO(e, leaf_path.clone()))?;
            Ok(serde_json::from_str(&leaf_json)?)
        })
        .collect()
}
