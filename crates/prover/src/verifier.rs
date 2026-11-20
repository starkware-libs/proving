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
//!
//! [`verify_cairo`] also takes the FRI parameters (`n_queries`, `pow_bits`, `log_blowup_factor`)
//! from the proof itself, with no floor on the security they give, so a prover could pick
//! parameters that make a forgery likely to pass. The Cairo verifier rejects parameters below
//! `SECURITY_BITS = 96` and below `MIN_POW_BITS = 20`. [`verify_cairo_bound`] enforces the same
//! floor through a [`SecurityPolicy`] before doing any other work.

use core::fmt;

use cairo_air::CairoProofForRustVerifier;
use cairo_air::verifier::{CairoVerificationError, verify_cairo};
use stwo::core::channel::MerkleChannel;
use stwo::core::verifier::PREPROCESSED_TRACE_IDX;
use stwo::prover::backend::BackendForChannel;
use stwo::prover::backend::simd::SimdBackend;

use crate::witness::preprocessed_trace::generate_preprocessed_commitment_root;

/// The minimum security a proof's FRI parameters must give for [`verify_cairo_bound`] to accept
/// it.
///
/// Security is counted as in [`FriParams::security_bits`]:
/// `pow_bits + log_blowup_factor * n_queries`.
///
/// [`FriParams::security_bits`]: stwo::core::fri::FriParams::security_bits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityPolicy {
    /// Lower bound on `pow_bits + log_blowup_factor * n_queries`.
    pub min_security_bits: u32,
    /// Lower bound on the FRI query proof-of-work bits alone.
    pub min_pow_bits: u32,
}

impl SecurityPolicy {
    /// The floor the Cairo verifier enforces: `SECURITY_BITS = 96` and `MIN_POW_BITS = 20`. The
    /// production prover parameters (`pow_bits = 26`, `log_blowup_factor = 1`, `n_queries = 70`)
    /// meet it exactly.
    pub const PRODUCTION: Self = Self { min_security_bits: 96, min_pow_bits: 20 };

    /// Accepts any parameters. Only for tests and benchmarks that prove with
    /// [`FriParams::test_params`](stwo::core::fri::FriParams::test_params).
    pub const NONE: Self = Self { min_security_bits: 0, min_pow_bits: 0 };
}

/// Error returned by [`verify_cairo_bound`].
#[derive(Debug)]
pub enum BoundVerificationError {
    /// The proof's FRI parameters give less security than the [`SecurityPolicy`] requires.
    SecurityTooLow {
        /// `pow_bits + log_blowup_factor * n_queries`, computed without overflow.
        security_bits: u64,
        pow_bits: u32,
        policy: SecurityPolicy,
    },
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
            BoundVerificationError::SecurityTooLow { security_bits, pow_bits, policy } => write!(
                f,
                "proof parameters give {security_bits} security bits and {pow_bits} pow bits, \
                 below the required {} and {}",
                policy.min_security_bits, policy.min_pow_bits,
            ),
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
            BoundVerificationError::SecurityTooLow { .. } => None,
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
/// verifier derives from the repository's own preprocessed-trace definition, and requiring the
/// proof's FRI parameters to meet `policy`.
///
/// This is the checked counterpart to [`verify_cairo`]. It rejects a proof whose FRI parameters
/// fall below `policy`, or whose `commitments[0]` (the preprocessed root) is not the canonical one
/// for the proof's declared `preprocessed_trace_variant` and lifting size, then defers to
/// [`verify_cairo`] for the rest.
///
/// Prefer this over [`verify_cairo`] whenever the proof is untrusted, with
/// [`SecurityPolicy::PRODUCTION`].
pub fn verify_cairo_bound<MC: MerkleChannel>(
    proof: CairoProofForRustVerifier<MC::H>,
    policy: SecurityPolicy,
) -> Result<(), BoundVerificationError>
where
    SimdBackend: BackendForChannel<MC>,
{
    // Check the parameters first: it is cheap, and a proof below the floor is rejected whatever
    // else it contains. Compute in u64 rather than with `FriParams::security_bits`, which casts
    // `n_queries` to u32 and multiplies in u32, so proof-chosen values cannot truncate or overflow
    // into a passing result.
    let fri_params = proof.stark_proof.config.fri_params;
    let security_bits = u64::from(fri_params.pow_bits)
        + u64::from(fri_params.log_blowup_factor) * fri_params.n_queries as u64;
    if security_bits < u64::from(policy.min_security_bits)
        || fri_params.pow_bits < policy.min_pow_bits
    {
        return Err(BoundVerificationError::SecurityTooLow {
            security_bits,
            pow_bits: fri_params.pow_bits,
            policy,
        });
    }

    // All fields read here are `Copy`, so the proof is left intact to hand to `verify_cairo`.
    let log_blowup_factor = fri_params.log_blowup_factor;
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

    use super::{BoundVerificationError, SecurityPolicy, verify_cairo_bound};
    use crate::prover::{ChannelHash, LiftingSizePolicy, ProverParameters, prove_cairo};

    /// Proves a small program with [`FriParams::test_params`]: 10 pow bits, log blowup 1 and 3
    /// queries, so 13 security bits.
    fn prove_with_test_params() -> CairoProofForRustVerifier<Blake2sMerkleHasher> {
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
        prove_cairo::<Blake2sMerkleChannel>(input, prover_params).unwrap().into()
    }

    /// `verify_cairo_bound` accepts a well-formed proof (its committed preprocessed root is the
    /// canonical one) and rejects a proof whose preprocessed root has been swapped for a
    /// non-canonical value, with the dedicated [`BoundVerificationError::PreprocessedRootMismatch`]
    /// rather than a downstream STARK error.
    #[test]
    fn verify_cairo_bound_rejects_non_canonical_preprocessed_root() {
        let cairo_proof = prove_with_test_params();

        // A genuine proof carries the canonical preprocessed root and verifies.
        let good_proof = cairo_proof.clone();
        verify_cairo_bound::<Blake2sMerkleChannel>(good_proof, SecurityPolicy::NONE).unwrap();

        // Swap the committed preprocessed root for a non-canonical value, as a prover that
        // committed a modified preprocessed trace would. Flipping one byte of the real root
        // guarantees a different, non-canonical hash.
        let mut tampered = cairo_proof;
        let real_root: Blake2sHash = tampered.stark_proof.commitments[PREPROCESSED_TRACE_IDX];
        let mut bad_bytes = real_root.0;
        bad_bytes[0] ^= 1;
        set_preprocessed_root(&mut tampered, Blake2sHash(bad_bytes));

        let err =
            verify_cairo_bound::<Blake2sMerkleChannel>(tampered, SecurityPolicy::NONE).unwrap_err();
        assert!(
            matches!(err, BoundVerificationError::PreprocessedRootMismatch),
            "expected PreprocessedRootMismatch, got: {err:?}",
        );
    }

    /// `verify_cairo_bound` enforces the security floor on the proof's own FRI parameters. The
    /// test proof has 13 security bits and 10 pow bits: it passes a policy asking for exactly
    /// that, and is rejected with [`BoundVerificationError::SecurityTooLow`] when either bound is
    /// one higher, and under [`SecurityPolicy::PRODUCTION`].
    #[test]
    fn verify_cairo_bound_enforces_security_policy() {
        let cairo_proof = prove_with_test_params();
        let fri_params = cairo_proof.stark_proof.config.fri_params;
        assert_eq!(fri_params.security_bits(), 13);
        assert_eq!(fri_params.pow_bits, 10);

        let exact = SecurityPolicy { min_security_bits: 13, min_pow_bits: 10 };
        verify_cairo_bound::<Blake2sMerkleChannel>(cairo_proof.clone(), exact).unwrap();

        for policy in [
            SecurityPolicy { min_security_bits: 14, min_pow_bits: 10 },
            SecurityPolicy { min_security_bits: 13, min_pow_bits: 11 },
            SecurityPolicy::PRODUCTION,
        ] {
            let err = verify_cairo_bound::<Blake2sMerkleChannel>(cairo_proof.clone(), policy)
                .unwrap_err();
            assert!(
                matches!(
                    err,
                    BoundVerificationError::SecurityTooLow { security_bits: 13, pow_bits: 10, .. }
                ),
                "policy {policy:?}: expected SecurityTooLow, got: {err:?}",
            );
        }
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
