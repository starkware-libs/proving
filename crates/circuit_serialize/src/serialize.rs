use circuits::blake::{HashValue, ReducedHashValue};
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::fri_proof::{FriCommitProof, FriProof, FriWitness};
use circuits_stark_verifier::merkle::{AuthPath, AuthPaths};
use circuits_stark_verifier::oods::EvalDomainSamples;
use circuits_stark_verifier::proof::{InteractionAtOods, Proof};
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

pub trait CircuitSerialize {
    fn serialize(&self, output: &mut Vec<u8>);
}

impl CircuitSerialize for Proof<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self {
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
        } = self;

        channel_salt.serialize(output);
        trace_root.serialize(output);
        interaction_root.serialize(output);
        composition_polynomial_root.serialize(output);
        claimed_sums.serialize(output);
        preprocessed_columns_at_oods.as_slice().serialize(output);
        trace_at_oods.as_slice().serialize(output);
        interaction_at_oods.as_slice().serialize(output);
        composition_eval_at_oods.serialize(output);
        eval_domain_samples.serialize(output);
        eval_domain_auth_paths.serialize(output);
        pow_nonce.serialize(output);
        interaction_pow_nonce.serialize(output);
        fri.serialize(output);
    }
}

impl CircuitSerialize for M31 {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.0.to_le_bytes());
    }
}

impl CircuitSerialize for QM31 {
    fn serialize(&self, output: &mut Vec<u8>) {
        for c in self.to_m31_array() {
            c.serialize(output);
        }
    }
}

impl<T: CircuitSerialize> CircuitSerialize for [T] {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.iter().for_each(|v| v.serialize(output));
    }
}

impl<T: CircuitSerialize, const N: usize> CircuitSerialize for [T; N] {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.as_slice().serialize(output);
    }
}

impl<T: CircuitSerialize> CircuitSerialize for Vec<T> {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.as_slice().serialize(output);
    }
}

impl CircuitSerialize for ReducedHashValue<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self(a, b) = self;
        a.serialize(output);
        b.serialize(output);
    }
}

impl CircuitSerialize for HashValue<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        // Each word is stored as `(low_u16, high_u16, 0, 0)`; recombine it into its 32-bit value
        // and write the four little-endian bytes, so the whole hash occupies exactly 32 bytes.
        let Self(words) = self;
        for word in words {
            let (low, high) = (word.get().0.0.0, word.get().0.1.0);
            output.extend_from_slice(&((high << 16) | low).to_le_bytes());
        }
    }
}

impl CircuitSerialize for InteractionAtOods<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { at_oods, at_prev } = self;
        at_oods.serialize(output);
        if let Some(at_prev) = at_prev {
            at_prev.serialize(output);
        }
    }
}

impl CircuitSerialize for AuthPath<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self(path) = self;
        path.serialize(output);
    }
}

impl CircuitSerialize for AuthPaths<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { data } = self;
        data.serialize(output);
    }
}

impl CircuitSerialize for M31Wrapper<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        // M31Wrapper wraps a value known to be in the base field M31.
        let m31: M31 = self.get().0.0;
        m31.serialize(output);
    }
}

impl CircuitSerialize for EvalDomainSamples<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let n_traces = self.n_traces();
        for trace_idx in 0..n_traces {
            let trace_data = self.data_for_trace(trace_idx);
            for column in trace_data {
                for cell in column {
                    cell.serialize(output);
                }
            }
        }
    }
}

impl CircuitSerialize for FriCommitProof<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { layer_commitments, last_layer_coefs } = self;
        layer_commitments.serialize(output);
        last_layer_coefs.serialize(output);
    }
}

impl CircuitSerialize for FriWitness<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self(witness_per_query_per_tree) = self;
        witness_per_query_per_tree.serialize(output);
    }
}

impl CircuitSerialize for FriProof<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { commit, auth_paths, witness } = self;
        commit.serialize(output);
        auth_paths.serialize(output);
        witness.serialize(output);
    }
}
