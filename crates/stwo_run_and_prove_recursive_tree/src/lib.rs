//! In-binary recursive **circuit**-proof tree builder.
//!
//! Given an ordered list of `N` leaf circuit proofs (produced by the `leaf_prover` crate — each one
//! is a circuit proof that a Cairo proof was verified by the cairo-verifier circuit), this folds
//! the entire recursive proof tree above the leaves in a single binary invocation by repeatedly
//! pairing adjacent children with the multiverifier : for each pair it builds a
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
//! another fold, so it is proven and serialized in the requested [`ProofFormat`]. That is the
//! Cairo circuit verifier's by default, as the applicative root task, or its
//! `poseidon252_verifier` build's, as the outer bootloader's verifier task. Internal folds use the
//! Blake2s channel the multiverifier circuit verifies. A single-leaf tree folds its leaf with
//! itself (the multiverifier verifies the same proof in both slots), so the root is always a
//! multiverifier proof in the requested format. A root for either Cairo verifier build is proven
//! under the registry's `cairo_verifier_root` config, whose FRI params can differ from the folds'.
//! Every fold proof's circuit hash is compared to the registry's multiverifier entry: an internal
//! node's to its Blake2s identity, such a root's to the root's own, Blake2s or the Poseidon252
//! felt of the outer root.
//!
//! Outputs written for the root layer: the root proof (see above), a flat JSON file with the
//! root node's circuit output values, and a nested packed JSON tree mirroring the fold, with
//! `Composite` verifier nodes down to each leaf's `Plain` node carrying the raw hashed-output
//! preimage, from which the unpacker bootloader verifying the root recomputes every digest in the
//! tree.
//!
//! In the `ApplicativeCircuit` format the fold continues into the applicative unpacking verifier.
//! The proof and output values written are that verifier's, the output being the aggregated
//! output's digest, while the packed tree is still the fold's, rooted at the multiverifier.
//!
//! Within each layer, the independent pair reductions are folded in parallel on `max_concurrency`
//! dedicated rayon sub-pools. `max_concurrency == 1` skips threading and sub-pools entirely.

use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use circuit_registry::{CircuitRegistry, DigestHex, LogSizes, RegistryError};
use starknet_types_core::felt::Felt;
use thiserror::Error;
use tracing::{Level, info, span};

pub mod canonical;
pub mod fold;
pub mod leaf_io;
pub mod output;
pub mod test_data;
#[cfg(feature = "slow-tests")]
pub mod test_support;
pub mod unpacking;

use canonical::CanonicalCircuit;
pub use fold::ProofFormat;
use fold::{LayerEntry, reduce_pair, reduce_root_single};
use itertools::Itertools;
use leaf_io::LeafInput;
pub use leaf_io::{LeafOutputHashType, LeafOutputPreimage, load_leaves};

#[derive(Debug, Error)]
pub enum RecursiveTreeError {
    #[error("Empty leaves list; expected at least one leaf entry.")]
    EmptyLeaves,
    #[error("IO error on file '{1:?}': {0}")]
    PathIO(std::io::Error, PathBuf),
    #[error("Failed to parse the leaf input file '{1:?}': {0}")]
    LeafParse(serde_json::Error, PathBuf),
    #[error("Failed to (de)serialize JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    SonicSerialize(#[from] sonic_rs::error::Error),
    #[error("Failed to deserialize a circuit proof: {0}")]
    Deserialize(String),
    #[error("Circuit proving failed at layer {layer_idx} pair {pair_idx}: {reason}")]
    Proving { layer_idx: usize, pair_idx: usize, reason: String },
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
    #[error(
        "The root fold proven here hashes to {got:?}, but the circuit registry's multiverifier \
         `cairo_verifier_root.blake_circuit_hash` is {expected:?}; the registry does not match \
         the circuits this binary builds."
    )]
    RootCircuitHash { expected: DigestHex, got: DigestHex },
    #[error(
        "The root fold proven here hashes to {got:#x} under Poseidon252, but the circuit \
         registry's multiverifier `cairo_verifier_root.poseidon_circuit_hash` is {expected:#x}; \
         the registry does not match the circuits this binary builds."
    )]
    PoseidonRootCircuitHash { expected: Felt, got: Felt },
    #[error(
        "The circuit registry's multiverifier has no `cairo_verifier_root`, so the tree cannot be \
         folded into a root for the Cairo circuit verifier."
    )]
    NoCairoVerifierRoot,
    #[error(
        "The circuit registry's `cairo_verifier_root` config has component log sizes {got:?}, but \
         the multiverifier's are {expected:?}; the root must prove the same padded circuit."
    )]
    CairoVerifierRootSizes { expected: LogSizes, got: LogSizes },
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error("The applicative flow folds two to {capacity} leaves, the wrapper included; got {n}.")]
    BadLeafCount { n: usize, capacity: usize },
    #[error(
        "Leaf {index}'s circuit {circuit_hash:?} with preprocessed root {preprocessed_root:?} is \
         not one of the registry's leaf verifiers, which the unpacking verifier holds the tree to."
    )]
    LeafCircuitNotWhitelisted {
        index: usize,
        circuit_hash: DigestHex,
        preprocessed_root: DigestHex,
    },
    #[error("Invalid aggregator wrapper leaf: {reason}")]
    BadWrapperLeaf { reason: String },
    #[error(
        "Task slot {slot} of the wrapper's output holds {slot_digest:?}, but the task leaves put \
         {leaf_digest:?} there; they must follow the wrapper's task order, with zero digests in \
         the slots past the last one."
    )]
    WrapperTaskSlotMismatch { slot: usize, slot_digest: DigestHex, leaf_digest: DigestHex },
    #[error("The packed tree does not fit the unpacking verifier: {reason}")]
    BadPackedTree { reason: String },
    #[error(
        "The applicative unpacking verifier rejected the tree: the root proof does not verify, \
         the packed tree does not match it, a node's circuit is outside the registry's \
         whitelists, or the wrapper's output does not match the task leaves."
    )]
    UnpackingVerifierRejected,
    #[error("Proving the applicative unpacking verifier failed: {0}")]
    UnpackingVerifierProving(String),
    #[error(
        "The unpacking verifier built here hashes to {got:?}, but the circuit registry describes \
         {expected:?}; the registry does not match the circuits this binary builds, for example \
         because the wrapper leaf's program hash {wrapper_program_hash:#x} is not the aggregator \
         wrapper's the registry was built for."
    )]
    UnpackingVerifierCircuitHash { expected: DigestHex, got: DigestHex, wrapper_program_hash: Felt },
}

/// Aggregate statistics for the completed reduction, returned for logging.
#[derive(Debug, Clone, Default)]
pub struct RecursiveTreeStats {
    pub n_leaves: usize,
    pub n_layers: usize,
    pub n_pair_reductions: usize,
}

/// Where the root layer's files go. In the `ApplicativeCircuit` format the proof and output values
/// are the unpacking verifier's, not the root node's, while the packed tree is still the fold's.
pub struct RootOutputPaths<'a> {
    /// The root node's proof in the requested [`ProofFormat`], or, in the `ApplicativeCircuit`
    /// format, the unpacking verifier's as a `SerializedLeafProof` JSON.
    pub proof: &'a Path,
    /// The proven circuit's output values, the root node's or the unpacking verifier's, as a flat
    /// JSON array of raw `u32` digest words. The unpacking verifier's is the aggregated output's
    /// digest.
    pub program_output: &'a Path,
    /// The nested `PackedNode` JSON tree of the fold, rooted at the multiverifier in every format.
    // TODO(yairv): Consider not writing it in the `ApplicativeCircuit` format, where the unpacking
    // verifier has already consumed it in-process.
    pub packed_node_tree: &'a Path,
}

/// Entry point: folds the entire recursive tree above `leaves` (consumed in order by the layer-0
/// entry list) and writes the root layer's files to `outputs`.
/// - `registry`: the circuit registry, which MUST be the one the leaves were proven against.
/// - `max_concurrency`: upper bound on the number of pair reductions to run in parallel within a
///   single layer.
/// - `root_proof_format`: who consumes the proof written (see [`ProofFormat`]). In the
///   `ApplicativeCircuit` format `leaves[0]` is the aggregator wrapper's leaf and the others are
///   its tasks' leaves, in the wrapper's task order, and the proof written is the applicative
///   unpacking verifier's over the folded tree (see [`unpacking`]).
pub fn stwo_run_and_prove_recursive_tree(
    leaves: Vec<LeafInput>,
    registry: &CircuitRegistry,
    outputs: &RootOutputPaths<'_>,
    max_concurrency: NonZeroUsize,
    root_proof_format: ProofFormat,
) -> Result<RecursiveTreeStats, RecursiveTreeError> {
    let _span = span!(Level::INFO, "stwo_run_and_prove_recursive_tree").entered();

    if leaves.is_empty() {
        return Err(RecursiveTreeError::EmptyLeaves);
    }
    let n_leaves = leaves.len();
    if root_proof_format == ProofFormat::ApplicativeCircuit {
        // The first leaf is the aggregator wrapper's, the others are the tasks'.
        let n_task_leaves = n_leaves - 1;
        info!(
            "Folding the wrapper leaf and the {n_task_leaves} task leaves into a recursive tree."
        );
    } else {
        info!(n_leaves, "Folding leaf circuit proofs into a recursive tree.");
    }

    // The applicative flow's inputs are checked first, so bad input fails before the canonical
    // circuit is built and before any proving.
    let applicative_inputs = (root_proof_format == ProofFormat::ApplicativeCircuit)
        .then(|| unpacking::validate_applicative_inputs(&leaves, registry))
        .transpose()?;
    // Likewise a root for a Cairo circuit verifier that the registry does not describe.
    if matches!(root_proof_format, ProofFormat::CairoVerifier | ProofFormat::PoseidonCairoVerifier)
        && registry.multiverifier()?.cairo_verifier_root.is_none()
    {
        return Err(RecursiveTreeError::NoCairoVerifierRoot);
    }

    let canonical = CanonicalCircuit::build(registry)?;
    let proof_config = &canonical.shared_config.proof_config;
    let current_layer: Vec<LayerEntry> = leaves
        .into_iter()
        .map(|leaf| LayerEntry::from_leaf(leaf, proof_config))
        .collect::<Result<Vec<_>, _>>()?;
    // The unpacking verifier's leaf digests, taken before the fold consumes the entries.
    let leaf_digests: Vec<_> =
        current_layer.iter().map(|entry| entry.proof_data.output_digest.0).collect();

    // The applicative flow's root is the unpacking verifier's input, an ordinary circuit proof.
    let fold_format = match root_proof_format {
        ProofFormat::ApplicativeCircuit => ProofFormat::Circuit,
        other => other,
    };
    let (root, stats) = fold_entries(current_layer, &canonical, max_concurrency, fold_format)?;
    info!(
        n_layers = stats.n_layers,
        n_pair_reductions = stats.n_pair_reductions,
        "Recursive tree reduction complete."
    );

    match applicative_inputs {
        Some(inputs) => {
            let LayerEntry { proof_data, packed_node } = root;
            let unpacking = unpacking::prove_unpacking_verifier(
                proof_data,
                &packed_node,
                &leaf_digests,
                &inputs,
                registry,
                &canonical,
            )?;
            let proof_json = serde_json::to_string_pretty(&unpacking.proof)?;
            output::write_outputs(
                proof_json.as_bytes(),
                &unpacking.output_digest.0,
                &packed_node,
                outputs,
            )?;
        }
        None => output::write_root_outputs(&root, root_proof_format, outputs)?,
    }
    Ok(stats)
}

/// Folds the layer-0 `entries` (in order) up to the tree's single root entry. Split out of
/// [`stwo_run_and_prove_recursive_tree`] so the e2e test can drive the fold from hand-built
/// entries (its fixture proof's output values are not derivable from a leaf output preimage).
pub(crate) fn fold_entries(
    entries: Vec<LayerEntry>,
    canonical: &CanonicalCircuit,
    max_concurrency: NonZeroUsize,
    root_proof_format: ProofFormat,
) -> Result<(LayerEntry, RecursiveTreeStats), RecursiveTreeError> {
    let mut current_layer = entries;
    // Cap to the available rayon workers; per-chunk sizing in `reduce_layer_pairs` shrinks
    // further when a chunk has fewer pairs than `max_concurrency`.
    let max_concurrency = max_concurrency.get().min(rayon::current_num_threads());
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

    let mut pools: Vec<rayon::ThreadPool> = Vec::new();
    let mut layer_idx: usize = 0;
    while current_layer.len() > 1 {
        info!(layer_idx, n_entries = current_layer.len(), "Reducing recursive-tree layer.");
        // A two-entry layer folds into the tree's single root: that final reduction produces the
        // proof the fold's consumer verifies, in `root_proof_format` (see `reduce_pair`).
        let proof_format =
            if current_layer.len() == 2 { root_proof_format } else { ProofFormat::Circuit };

        let mut tuples = current_layer.into_iter().tuples();
        let pairs: Vec<(LayerEntry, LayerEntry)> = tuples.by_ref().collect();
        let carry = tuples.into_buffer().next();
        stats.n_pair_reductions += pairs.len();

        let mut next_layer = reduce_layer_pairs(
            pairs,
            layer_idx,
            proof_format,
            canonical,
            max_concurrency,
            &mut pools,
        )?;
        if let Some(entry) = carry {
            info!(layer_idx, pair_idx = next_layer.len(), "Carrying unpaired entry to next layer.");
            next_layer.push(entry);
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

/// Builds `n_pools` rayon sub-pools, splitting the process's total rayon workers evenly. Each pool
/// gets `max(1, rayon::current_num_threads() / n_pools)` worker threads, so `n_pools` concurrent
/// pair reductions can run each on its own dedicated slice of cores. Reads
/// [`rayon::current_num_threads`] so the split honors `RAYON_NUM_THREADS` when set.
fn build_worker_pools(n_pools: usize) -> Vec<rayon::ThreadPool> {
    let total = rayon::current_num_threads();
    let per_pool = std::cmp::max(1, total / n_pools);
    info!(
        n_pools,
        threads_per_pool = per_pool,
        total_rayon_threads = total,
        "Building per-worker rayon pools for concurrent pair reductions."
    );
    (0..n_pools)
        .map(|pool_idx| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(per_pool)
                .thread_name(move |idx| format!("recursive-tree-worker-{pool_idx}-{idx}"))
                .build()
                .expect("failed to build rayon worker pool")
        })
        .collect()
}

/// Runs the pair reductions of a single layer, preserving input order. Dispatches pairs across
/// chunks of at most `max_concurrency` pairs. Each chunk runs on `chunk.len()` rayon sub-pools (so
/// a short trailing chunk gets threads-per-pool scaled up to its size), each pinned via
/// [`rayon::ThreadPool::install`] so its `par_iter` calls stay on a dedicated slice of cores.
/// A single-pair chunk (including the whole `max_concurrency == 1` case) runs serially on the
/// main pool.
fn reduce_layer_pairs(
    pairs: Vec<(LayerEntry, LayerEntry)>,
    layer_idx: usize,
    proof_format: ProofFormat,
    canonical: &CanonicalCircuit,
    max_concurrency: usize,
    pools: &mut Vec<rayon::ThreadPool>,
) -> Result<Vec<LayerEntry>, RecursiveTreeError> {
    let reduce = |(pair_idx, (left, right)): (usize, (LayerEntry, LayerEntry))| {
        reduce_pair(left, right, layer_idx + 1, pair_idx, canonical, proof_format)
    };

    let mut next_layer = Vec::with_capacity(pairs.len());
    let mut pairs_iter = pairs.into_iter().enumerate();
    loop {
        let mut chunk: Vec<_> = pairs_iter.by_ref().take(max_concurrency).collect();
        match chunk.len() {
            0 => break,
            1 => {
                next_layer.push(reduce(chunk.pop().unwrap())?);
                continue;
            }
            _ => {}
        }
        if pools.len() != chunk.len() {
            // Drop the old pools first, so their threads exit before the new ones are spawned.
            pools.clear();
            *pools = build_worker_pools(chunk.len());
        }
        let chunk_results: Vec<Result<LayerEntry, RecursiveTreeError>> = std::thread::scope(|s| {
            let handles: Vec<_> = chunk
                .into_iter()
                .zip(pools.iter())
                .map(|(pair, pool)| s.spawn(move || pool.install(|| reduce(pair))))
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap_or_else(|payload| std::panic::resume_unwind(payload)))
                .collect()
        });
        for entry in chunk_results {
            next_layer.push(entry?);
        }
    }
    Ok(next_layer)
}

#[cfg(test)]
mod tests;
