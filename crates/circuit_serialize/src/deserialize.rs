use std::fmt;

use circuits::blake::{HashValue, ReducedHashValue};
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::fri_proof::{
    FriCommitProof, FriConfig, FriProof, FriWitness, compute_all_fold_steps,
};
use circuits_stark_verifier::merkle::{AuthPath, AuthPaths};
use circuits_stark_verifier::oods::{EvalDomainSamples, N_COMPOSITION_COLUMNS};
use circuits_stark_verifier::proof::{InteractionAtOods, N_TRACES, Proof, ProofConfig};
use stwo::core::fields::m31::{M31, P};
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;

#[derive(Debug)]
pub enum DeserializeError {
    NotEnoughData,
    ValueOutOfRange,
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeserializeError::NotEnoughData => write!(f, "not enough data to deserialize"),
            DeserializeError::ValueOutOfRange => write!(f, "field element value out of range"),
        }
    }
}

impl std::error::Error for DeserializeError {}

pub type DeserializeResult<T> = Result<T, DeserializeError>;

/// Deserializes types from a byte format serialized by the corresponding `CircuitSerialize`
/// implementations. Fixed-size types implement the trait directly; composite types that require
/// length information from the [`ProofConfig`] use dedicated `deserialize_with_config` methods.
pub trait CircuitDeserialize: Sized {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self>;
}

impl CircuitDeserialize for M31 {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        let bytes: [u8; 4] = take_bytes(data, 4)?.try_into().unwrap();
        let val = u32::from_le_bytes(bytes);
        // Assert that the value is in [0, P).
        if val >= P {
            return Err(DeserializeError::ValueOutOfRange);
        }
        Ok(M31::from_u32_unchecked(val))
    }
}

impl CircuitDeserialize for QM31 {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        let m31_values: [M31; 4] = [
            M31::deserialize(data)?,
            M31::deserialize(data)?,
            M31::deserialize(data)?,
            M31::deserialize(data)?,
        ];
        Ok(QM31::from_m31_array(m31_values))
    }
}

impl<T: CircuitDeserialize, const N: usize> CircuitDeserialize for [T; N] {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        deserialize_vec(data, N)?.try_into().map_err(|_| DeserializeError::NotEnoughData)
    }
}

impl CircuitDeserialize for ReducedHashValue<QM31> {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        Ok(ReducedHashValue(QM31::deserialize(data)?, QM31::deserialize(data)?))
    }
}

impl CircuitDeserialize for HashValue<QM31> {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        // A `HashValue` is stored as its 32 raw bytes; split each 32-bit word back into
        // `(low_u16, high_u16, 0, 0)` form, mirroring `HashValue::from(Blake2sHash)`.
        let bytes: [u8; 32] = take_bytes(data, 32)?.try_into().unwrap();
        Ok(HashValue::from(Blake2sHash(bytes)))
    }
}

impl CircuitDeserialize for M31Wrapper<QM31> {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        let m31 = M31::deserialize(data)?;
        Ok(M31Wrapper::from(m31))
    }
}

/// Deserializes a `Vec<T>` of the given length from the byte stream.
fn deserialize_vec<T: CircuitDeserialize>(
    data: &mut &[u8],
    len: usize,
) -> DeserializeResult<Vec<T>> {
    (0..len).map(|_| T::deserialize(data)).collect()
}

/// Takes exactly `n` bytes from the front of `data`, advancing it past them.
fn take_bytes<'a>(data: &mut &'a [u8], n: usize) -> DeserializeResult<&'a [u8]> {
    data.split_off(..n).ok_or(DeserializeError::NotEnoughData)
}

/// Deserializes a proof from a byte stream, using the [`ProofConfig`] for all length
/// information.
pub fn deserialize_proof_with_config(
    data: &mut &[u8],
    config: &ProofConfig,
) -> DeserializeResult<Proof<QM31>> {
    let channel_salt = QM31::deserialize(data)?;
    let trace_root = HashValue::<QM31>::deserialize(data)?;
    let interaction_root = HashValue::<QM31>::deserialize(data)?;
    let composition_polynomial_root = HashValue::<QM31>::deserialize(data)?;
    let claimed_sums = deserialize_vec(data, config.n_components())?;
    let preprocessed_columns_at_oods = deserialize_vec(data, config.n_preprocessed_columns)?;
    let trace_at_oods = deserialize_vec(data, config.n_trace_columns)?;
    let interaction_at_oods = deserialize_interaction_at_oods(data, config)?;
    let composition_eval_at_oods = <[QM31; N_COMPOSITION_COLUMNS]>::deserialize(data)?;
    let eval_domain_samples = deserialize_eval_domain_samples(data, config)?;
    let eval_domain_auth_paths = deserialize_eval_domain_auth_paths(data, config)?;
    let pow_nonce = QM31::deserialize(data)?;
    let interaction_pow_nonce = QM31::deserialize(data)?;
    let fri = deserialize_fri_proof(data, &config.fri)?;

    Ok(Proof {
        channel_salt,
        trace_root,
        interaction_root,
        composition_polynomial_root,
        preprocessed_columns_at_oods,
        trace_at_oods,
        composition_eval_at_oods,
        claimed_sums,
        interaction_at_oods,
        eval_domain_samples,
        eval_domain_auth_paths,
        pow_nonce,
        interaction_pow_nonce,
        fri,
    })
}

fn deserialize_interaction_at_oods(
    data: &mut &[u8],
    config: &ProofConfig,
) -> DeserializeResult<Vec<InteractionAtOods<QM31>>> {
    config
        .cumulative_sum_columns
        .iter()
        .map(|is_cumulative_sum| {
            let at_oods = QM31::deserialize(data)?;
            let at_prev = if *is_cumulative_sum { Some(QM31::deserialize(data)?) } else { None };
            Ok(InteractionAtOods { at_oods, at_prev })
        })
        .collect()
}

fn deserialize_eval_domain_samples(
    data: &mut &[u8],
    config: &ProofConfig,
) -> DeserializeResult<EvalDomainSamples<QM31>> {
    let n_columns_per_trace = config.n_columns_per_trace();
    let n_queries = config.n_queries();
    let mut data_vec = Vec::with_capacity(n_columns_per_trace.len());
    for &n_columns in &n_columns_per_trace {
        let mut trace_data = Vec::with_capacity(n_columns);
        for _ in 0..n_columns {
            let column = deserialize_vec(data, n_queries)?;
            trace_data.push(column);
        }
        data_vec.push(trace_data);
    }
    Ok(EvalDomainSamples::from_m31s(data_vec))
}

fn deserialize_eval_domain_auth_paths(
    data: &mut &[u8],
    config: &ProofConfig,
) -> DeserializeResult<AuthPaths<QM31>> {
    let n_queries = config.n_queries();
    let path_len = config.log_evaluation_domain_size();
    let mut trees = Vec::with_capacity(N_TRACES);
    for _ in 0..N_TRACES {
        let mut paths = Vec::with_capacity(n_queries);
        for _ in 0..n_queries {
            let hashes: Vec<HashValue<QM31>> = deserialize_vec(data, path_len)?;
            paths.push(AuthPath(hashes));
        }
        trees.push(paths);
    }
    Ok(AuthPaths { data: trees })
}

fn deserialize_fri_commit_proof(
    data: &mut &[u8],
    config: &FriConfig,
    all_fold_steps: &[usize],
) -> DeserializeResult<FriCommitProof<QM31>> {
    let n_layers = all_fold_steps.len();
    let n_last_layer_coefs = 1 << config.log_n_last_layer_coefs;
    Ok(FriCommitProof {
        layer_commitments: deserialize_vec(data, n_layers)?,
        last_layer_coefs: deserialize_vec(data, n_last_layer_coefs)?,
    })
}

fn deserialize_fri_proof(
    data: &mut &[u8],
    fri_config: &FriConfig,
) -> DeserializeResult<FriProof<QM31>> {
    let all_fold_steps = compute_all_fold_steps(
        fri_config.log_trace_size - fri_config.log_n_last_layer_coefs,
        fri_config.fold_step,
    );
    let commit = deserialize_fri_commit_proof(data, fri_config, &all_fold_steps)?;

    let mut path_len = fri_config.log_evaluation_domain_size();
    let mut auth_path_trees = Vec::with_capacity(all_fold_steps.len());
    for step in all_fold_steps.iter() {
        path_len -= step;
        let mut paths = Vec::with_capacity(fri_config.n_queries);
        for _ in 0..fri_config.n_queries {
            let hashes: Vec<HashValue<QM31>> = deserialize_vec(data, path_len)?;
            paths.push(AuthPath(hashes));
        }
        auth_path_trees.push(paths);
    }
    let auth_paths = AuthPaths { data: auth_path_trees };
    let mut witness_per_query_per_tree = vec![];

    for step in all_fold_steps.iter() {
        let mut witness_per_query = vec![];
        for _ in 0..fri_config.n_queries {
            let witness: Vec<QM31> = deserialize_vec(data, 1 << step)?;
            witness_per_query.push(witness);
        }
        witness_per_query_per_tree.push(witness_per_query);
    }
    let witness = FriWitness(witness_per_query_per_tree);
    Ok(FriProof { commit, auth_paths, witness })
}
