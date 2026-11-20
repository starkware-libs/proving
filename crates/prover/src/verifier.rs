//! Root-bound wrapper around the Rust Cairo verifier.
//!
//! [`cairo_air::verifier::verify_cairo`] commits `stark_proof.commitments[0]` (the preprocessed
//! trace root) straight from the proof, without checking it against the root the verifier expects.
//! The preprocessed trace holds the AIR's fixed tables (range checks, bitwise, hash constants), so
//! a prover that commits a modified preprocessed trace picks both the tables and their root. The
//! Cairo (circuit) verifier guards against this by binding the committed root to the expected
//! canonical root before using it; the Rust path did not.
//!
//! This module closes that gap for the Rust verification path. It recomputes the canonical
//! preprocessed root from the repository's own preprocessed-trace definition
//! (via [`generate_preprocessed_commitment_root`]) and rejects any proof whose committed root does
//! not match it, before handing off to [`verify_cairo`].

use core::fmt;

use cairo_air::CairoProofForRustVerifier;
use cairo_air::verifier::{CairoVerificationError, verify_cairo};
use stwo::core::channel::MerkleChannel;
use stwo::core::verifier::PREPROCESSED_TRACE_IDX;
use stwo::prover::backend::BackendForChannel;
use stwo::prover::backend::simd::SimdBackend;

use crate::witness::preprocessed_trace::generate_preprocessed_commitment_root;

/// Error returned by [`verify_cairo_bound`].
#[derive(Debug)]
pub enum BoundVerificationError {
    /// The preprocessed-trace root committed in the proof does not match the canonical root the
    /// verifier expects for the proof's declared `preprocessed_trace_variant` and lifting size.
    ///
    /// A valid proof commits the canonical preprocessed trace; a mismatch means the prover
    /// committed a different (possibly malicious) preprocessed trace.
    PreprocessedRootMismatch,
    /// The underlying STARK verification failed.
    Verification(CairoVerificationError),
}

impl fmt::Display for BoundVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoundVerificationError::PreprocessedRootMismatch => f.write_str(
                "preprocessed trace root in the proof does not match the expected canonical root \
                 for the declared variant",
            ),
            BoundVerificationError::Verification(e) => write!(f, "stark verification error: {e}"),
        }
    }
}

impl std::error::Error for BoundVerificationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BoundVerificationError::PreprocessedRootMismatch => None,
            BoundVerificationError::Verification(e) => Some(e),
        }
    }
}

impl From<CairoVerificationError> for BoundVerificationError {
    fn from(e: CairoVerificationError) -> Self {
        BoundVerificationError::Verification(e)
    }
}

/// Verifies a Cairo proof, binding the committed preprocessed-trace root to the canonical root the
/// verifier derives from the repository's own preprocessed-trace definition.
///
/// This is the root-checking counterpart to [`verify_cairo`]: it rejects a proof whose
/// `commitments[0]` (the preprocessed root) is not the canonical one for the proof's declared
/// `preprocessed_trace_variant` and lifting size, then defers to [`verify_cairo`] for the rest.
///
/// Prefer this over [`verify_cairo`] whenever the proof is untrusted.
pub fn verify_cairo_bound<MC: MerkleChannel>(
    proof: CairoProofForRustVerifier<MC::H>,
) -> Result<(), BoundVerificationError>
where
    SimdBackend: BackendForChannel<MC>,
{
    // All fields read here are `Copy`, so the proof is left intact to hand to `verify_cairo`.
    let log_blowup_factor = proof.stark_proof.config.fri_params.log_blowup_factor;
    let preprocessed_lifting_log_size = proof.stark_proof.config.preprocessed_lifting_log_size;
    // `generate_preprocessed_commitment_root` re-adds the blowup factor, so pass the pre-blowup
    // log trace size.
    let log_trace_size = preprocessed_lifting_log_size - log_blowup_factor;

    let expected_root = generate_preprocessed_commitment_root::<MC>(
        log_blowup_factor,
        proof.preprocessed_trace_variant,
        log_trace_size,
    );
    let committed_root = proof.stark_proof.commitments[PREPROCESSED_TRACE_IDX];
    if committed_root != expected_root {
        return Err(BoundVerificationError::PreprocessedRootMismatch);
    }

    verify_cairo::<MC>(proof).map_err(BoundVerificationError::Verification)
}

#[cfg(test)]
mod tests {
    use cairo_vm::types::layout_name::LayoutName;
    use stwo::core::fri::FriParams;
    use stwo::core::vcs::blake2_hash::Blake2sHash;
    use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
    use stwo::core::verifier::PREPROCESSED_TRACE_IDX;
    use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
    use stwo_cairo_dev_utils::utils::get_compiled_cairo_program_path;
    use stwo_cairo_dev_utils::vm_utils::{ProgramType, run_and_adapt};

    use super::{BoundVerificationError, verify_cairo_bound};
    use crate::prover::{ChannelHash, LiftingSizePolicy, ProverParameters, prove_cairo};

    /// `verify_cairo_bound` accepts a well-formed proof (its committed preprocessed root is the
    /// canonical one) and rejects a proof whose preprocessed root has been swapped for a
    /// non-canonical value, with the dedicated [`BoundVerificationError::PreprocessedRootMismatch`]
    /// rather than a downstream STARK error.
    #[test]
    fn verify_cairo_bound_rejects_non_canonical_preprocessed_root() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_pedersen_builtin");
        let input =
            run_and_adapt(&compiled_program, ProgramType::Json, LayoutName::stwo_no_ecop, None)
                .unwrap();
        let prover_params = ProverParameters {
            channel_hash: ChannelHash::Blake2s,
            fri_params: FriParams::test_params(),
            preprocessed_trace_variant: PreProcessedTraceVariant::CanonicalSmall,
            channel_salt: 0,
            store_polynomials_coefficients: false,
            include_all_preprocessed_columns: false,
            opt_n_id_to_big_components: None,
            lifting_size_policy: LiftingSizePolicy::Auto,
        };
        let cairo_proof = prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap();

        // A genuine proof carries the canonical preprocessed root and verifies.
        let good_proof = cairo_proof.clone().into();
        verify_cairo_bound::<Blake2sMerkleChannel>(good_proof).unwrap();

        // Swap the committed preprocessed root for a non-canonical value, as a prover that
        // committed a modified preprocessed trace would. Flipping one byte of the real root
        // guarantees a different, non-canonical hash.
        let mut tampered = cairo_proof.into();
        let real_root: Blake2sHash = {
            let proof: &CairoProofForRustVerifier<_> = &tampered;
            proof.stark_proof.commitments[PREPROCESSED_TRACE_IDX]
        };
        let mut bad_bytes = real_root.0;
        bad_bytes[0] ^= 1;
        set_preprocessed_root(&mut tampered, Blake2sHash(bad_bytes));

        let err = verify_cairo_bound::<Blake2sMerkleChannel>(tampered).unwrap_err();
        assert!(
            matches!(err, BoundVerificationError::PreprocessedRootMismatch),
            "expected PreprocessedRootMismatch, got: {err:?}",
        );
    }

    use cairo_air::CairoProofForRustVerifier;
    use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;

    /// Overwrites the committed preprocessed-trace root. `StarkProof` is a newtype over
    /// `CommitmentSchemeProof` and `TreeVec` over `Vec`, so reach the entry through the public
    /// tuple fields.
    fn set_preprocessed_root(
        proof: &mut CairoProofForRustVerifier<Blake2sMerkleHasher>,
        root: Blake2sHash,
    ) {
        proof.stark_proof.0.commitments.0[PREPROCESSED_TRACE_IDX] = root;
    }
}
