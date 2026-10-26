//! In-binary recursive **circuit**-proof tree builder.
//!
//! Given an ordered list of `N` leaf circuit proofs (produced by the `leaf_prover` crate — each one
//! is a circuit proof that a Cairo proof was verified by the cairo-verifier circuit), this folds
//! the entire recursive proof tree above the leaves in a single binary invocation by repeatedly
//! pairing adjacent children with the `circuit_multiverifier`: for each pair it builds a
//! multiverifier circuit that verifies both children, proves that circuit, and the resulting
//! circuit proof becomes the parent node for the next layer. There is no Cairo bootloader and no
//! wrapping step anymore.
//!
//! The constructed tree is balanced: every layer pairs adjacent entries two-to-one, so the tree has
//! depth `ceil(log2(N))`. An odd entry at any layer is carried through unchanged to the next layer
//! (attaching one level higher). Carrying works because, thanks to the common padding target, every
//! proof in the tree — leaf or internal — has the exact same circuit shape and is therefore
//! verifiable by the same multiverifier circuit (see [`canonical`]).
//!
//! The tree's final reduction is special: its proof is the one the fold's consumer verifies, not
//! another fold, so it is proven and serialized in the requested [`ProofFormat`] — by default the
//! Cairo circuit verifier's, as the applicative root task. Internal folds use the M31 channel the
//! multiverifier circuit verifies. A single-leaf tree folds its leaf with itself (the
//! multiverifier verifies the same proof in both slots), so the root is always a multiverifier
//! proof in the requested format.
//!
//! Outputs written for the root layer: the root proof (see above), a flat JSON file with the
//! root node's circuit output values, and a nested packed JSON tree mirroring the fold —
//! `Composite` verifier nodes down to each leaf's `Plain` node carrying the raw hashed-output
//! preimage, from which a future unpacker recomputes every digest in the tree.

use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use circuit_registry::{CircuitRegistry, DigestHex, RegistryError};
use thiserror::Error;
use tracing::{Level, info, span};

pub mod canonical;
pub mod fold;
pub mod leaf_io;
pub mod output;
#[cfg(feature = "slow-tests")]
pub mod test_support;

use canonical::CanonicalCircuit;
pub use fold::ProofFormat;
use fold::{LayerEntry, reduce_pair, reduce_root_single};
pub use leaf_io::{LeafInput, LeafOutputHashType, LeafOutputPreimage, load_leaves};
pub use leaf_proof_format::{PackedNode, SerializedLeafProof};

#[derive(Debug, Error)]
pub enum RecursiveTreeError {
    #[error("Empty leaves list; expected at least one leaf entry.")]
    EmptyLeaves,
    #[error("IO error on file '{1:?}': {0}")]
    PathIO(std::io::Error, PathBuf),
    #[error("Failed to (de)serialize JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    SonicSerialize(#[from] sonic_rs::error::Error),
    #[error("Failed to deserialize a circuit proof: {0}")]
    Deserialize(String),
    #[error("Circuit proving failed: {0}")]
    Proving(String),
    #[error("Expected {expected} output values from a circuit proof, got {got}.")]
    BadOutputArity { expected: usize, got: usize },
    #[error(
        "Padding parity broken: the leaf and multiverifier circuits do not share the same \
         preprocessed-trace layout (column log sizes / trace_log_size). The registry's target \
         component sizes are likely inconsistent with the circuit configuration."
    )]
    PaddingParity,
    #[error(
        "The multiverifier circuit built here hashes to {got:?}, but the circuit registry \
         describes {expected:?}; the registry does not match the circuits this binary builds."
    )]
    MultiverifierCircuitHash { expected: DigestHex, got: DigestHex },
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error("Could not parse leaf circuit output values: {reason}")]
    BadLeafOutputs { reason: String },
}

/// Aggregate statistics for the completed reduction, returned for logging.
#[derive(Debug, Clone, Default)]
pub struct RecursiveTreeStats {
    pub n_leaves: usize,
    pub n_layers: usize,
    pub n_pair_reductions: usize,
}

/// Entry point: folds the entire recursive tree above `leaves` (consumed in order by the layer-0
/// entry list) and writes the three root-layer output files:
/// - `proof_path`: the root proof, in `root_proof_format` (see [`ProofFormat`]).
/// - `program_output`: the root node's output values (flat JSON array of raw `u32` digest words).
/// - `packed_node_tree_path`: the nested `PackedNode` JSON tree.
/// - `registry`: the circuit registry, which MUST be the one the leaves were proven against.
/// - `max_concurrency`: upper bound on the number of pair reductions to run in parallel within a
///   single layer.
pub fn stwo_run_and_prove_recursive_tree(
    leaves: Vec<LeafInput>,
    registry: &CircuitRegistry,
    proof_path: &Path,
    program_output: &Path,
    packed_node_tree_path: &Path,
    max_concurrency: NonZeroUsize,
    root_proof_format: ProofFormat,
) -> Result<RecursiveTreeStats, RecursiveTreeError> {
    let _span = span!(Level::INFO, "stwo_run_and_prove_recursive_tree").entered();
    // Note: max_concurrency is currently no-op (the fold below is strictly sequential). Parameter
    // will drive intra-layer parallelism in a follow-up PR.
    let _ = max_concurrency;

    if leaves.is_empty() {
        return Err(RecursiveTreeError::EmptyLeaves);
    }
    let n_leaves = leaves.len();
    info!(n_leaves, "Folding leaf circuit proofs into a recursive tree.");

    let canonical = CanonicalCircuit::build(registry)?;

    // Intermediate proofs live in memory (on each `LayerEntry`) for the duration of the fold; the
    // only files this invocation writes are the root outputs.
    let current_layer: Vec<LayerEntry> =
        leaves.iter().map(LayerEntry::from_leaf).collect::<Result<Vec<_>, _>>()?;

    let (root, stats) = fold_entries(current_layer, &canonical, root_proof_format)?;
    output::write_root_outputs(&root, proof_path, program_output, packed_node_tree_path)?;
    info!(
        n_layers = stats.n_layers,
        n_pair_reductions = stats.n_pair_reductions,
        "Recursive tree reduction complete."
    );
    Ok(stats)
}

/// Folds the layer-0 `entries` (in order) up to the tree's single root entry. Split out of
/// [`stwo_run_and_prove_recursive_tree`] so the e2e test can drive the fold from hand-built
/// entries (its fixture proof's output values are not derivable from a leaf output preimage).
pub(crate) fn fold_entries(
    entries: Vec<LayerEntry>,
    canonical: &CanonicalCircuit,
    root_proof_format: ProofFormat,
) -> Result<(LayerEntry, RecursiveTreeStats), RecursiveTreeError> {
    let mut current_layer = entries;
    let mut stats =
        RecursiveTreeStats { n_leaves: current_layer.len(), n_layers: 0, n_pair_reductions: 0 };

    if current_layer.len() == 1 {
        info!("Single-leaf tree: folding the leaf with itself for the root pass.");
        let root = reduce_root_single(
            current_layer.pop().expect("exactly one entry"),
            canonical,
            root_proof_format,
        )?;
        stats.n_layers = 1;
        stats.n_pair_reductions = 1;
        return Ok((root, stats));
    }

    let mut layer_idx: usize = 0;
    while current_layer.len() > 1 {
        info!(layer_idx, n_entries = current_layer.len(), "Reducing recursive-tree layer.");
        // A two-entry layer folds into the tree's single root: that final reduction produces the
        // proof the fold's consumer verifies, in `root_proof_format` (see `reduce_pair`).
        let proof_format =
            if current_layer.len() == 2 { root_proof_format } else { ProofFormat::Circuit };
        let mut next_layer: Vec<LayerEntry> = Vec::with_capacity(current_layer.len().div_ceil(2));
        let mut pairs = current_layer.into_iter();
        let mut pair_idx: usize = 0;
        while let Some(left) = pairs.next() {
            match pairs.next() {
                Some(right) => {
                    let entry =
                        reduce_pair(left, right, layer_idx + 1, pair_idx, canonical, proof_format)?;
                    next_layer.push(entry);
                    stats.n_pair_reductions += 1;
                }
                None => {
                    info!(layer_idx, pair_idx, "Carrying unpaired entry to next layer.");
                    next_layer.push(left);
                }
            }
            pair_idx += 1;
        }
        current_layer = next_layer;
        layer_idx += 1;
    }
    stats.n_layers = layer_idx;

    let root = current_layer.pop().expect(
        "reduction loop terminates only when current_layer.len() == 1, so the final layer must \
         contain exactly one root entry",
    );
    Ok((root, stats))
}

#[cfg(test)]
mod tests;
