//! The JSON wire formats of the proving pipeline's binaries: the leaf prover's proof output
//! ([`SerializedLeafProof`]) and the recursive tree prover's packed-output tree ([`PackedNode`]).

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::base64::Base64;
use serde_with::serde_as;

/// The number of little-endian u32 words in a Blake2s digest.
pub const N_DIGEST_WORDS: usize = 8;

/// A Blake2s digest: eight little-endian u32 words, serialized as an array of `0x`-prefixed hex
/// strings.
///
/// This is also the wire format of the circuit registry's digests: leaf-prover output is compared
/// against registry entries, so both use the same representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigestHex(pub [u32; N_DIGEST_WORDS]);

impl From<[u8; 32]> for DigestHex {
    fn from(bytes: [u8; 32]) -> Self {
        let mut words = [0u32; N_DIGEST_WORDS];
        for (word, src) in words.iter_mut().zip(bytes.chunks_exact(4)) {
            *word = u32::from_le_bytes(src.try_into().unwrap());
        }
        DigestHex(words)
    }
}

impl Serialize for DigestHex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.map(|word| format!("{word:#010x}")).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DigestHex {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let words: [String; N_DIGEST_WORDS] = Deserialize::deserialize(deserializer)?;
        let mut digest = [0u32; N_DIGEST_WORDS];
        for (out, word) in digest.iter_mut().zip(words) {
            let hex = word.strip_prefix("0x").unwrap_or(&word);
            *out = u32::from_str_radix(hex, 16).map_err(serde::de::Error::custom)?;
        }
        Ok(DigestHex(digest))
    }
}

/// Describes the structure of the output JSON file of the leaf prover.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SerializedLeafProof {
    /// The preprocessed root of the proof of the verifier circuit.
    pub circuit_preprocessed_root: DigestHex,
    /// `blake2s(log_blowup_factor || component_log_sizes || preprocessed_root)`: the identity of
    /// the verifier circuit, as mixed into the channel by the prover.
    pub circuit_hash: DigestHex,
    /// The serialized proof of the verifier circuit execution.
    #[serde_as(as = "Base64")]
    pub proof: Vec<u8>,
}

/// The recursive tree prover's nested packed-output tree, one `Composite` per verifier node:
///
/// ```text
/// Composite { subtasks: [left, right] }   // a fold: the multiverifier over two children
///   ...
///     Composite { subtasks: [plain] }     // a leaf: the cairo-verifier circuit over one task run
///       └─ Plain { output_preimage }      // terminal: the leaf task's revealed output
/// ```
///
/// The tree carries only data that cannot be recomputed: its consumer (the circuit-unpacking
/// applicative bootloader) rederives every digest — the leaf's hashed output and each fold's
/// circuit output — bottom-up from the preimages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PackedNode {
    /// A leaf's hashed-output preimage — the task's program hash followed by the task's raw output
    /// (each felt a decimal string; see the recursive tree's `LeafInput::output_preimage`).
    Plain { output_preimage: Vec<String> },
    /// A verifier node — a fold over two children, a self-fold over one `Composite` child (the
    /// single-leaf tree's root pass, where the multiverifier verifies the same proof in both
    /// slots), or the leaf circuit over its single `Plain` child. Carries the `circuit_hash` of
    /// this node's proof (eight little-endian u32 words) — the full circuit identity
    /// (`blake2s(log_blowup ‖ component_log_sizes ‖ preprocessed_root)`) that the unpacker mixes
    /// into this node's fold contribution and looks up in its supported trust list.
    Composite { circuit_hash: [u32; N_DIGEST_WORDS], subtasks: Vec<PackedNode> },
}

impl PackedNode {
    /// A leaf entry: the leaf circuit node over the `Plain` preimage reveal.
    pub fn leaf(circuit_hash: [u32; N_DIGEST_WORDS], output_preimage: Vec<String>) -> Self {
        PackedNode::Composite {
            circuit_hash,
            subtasks: vec![PackedNode::Plain { output_preimage }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A digest is serialized as eight `0x`-prefixed little-endian words, and parses back.
    #[test]
    fn test_digest_hex_round_trip() {
        // The first word is the first four bytes, little-endian.
        let digest = DigestHex::from(std::array::from_fn::<u8, 32, _>(|i| i as u8));
        assert_eq!(digest.0[0], 0x03020100);

        let json = serde_json::to_string(&digest).unwrap();
        assert!(json.starts_with(r#"["0x03020100","#), "unexpected encoding: {json}");
        assert_eq!(serde_json::from_str::<DigestHex>(&json).unwrap(), digest);
    }

    /// The packed tree round-trips through its externally tagged JSON, with circuit hashes as
    /// plain word arrays (the encoding the tree prover writes and the bootloader hints read).
    #[test]
    fn test_packed_node_round_trips() {
        let leaf = PackedNode::leaf([1, 2, 3, 4, 5, 6, 7, 8], vec!["11".to_string()]);
        let tree = PackedNode::Composite {
            circuit_hash: [9; N_DIGEST_WORDS],
            subtasks: vec![leaf.clone(), leaf],
        };

        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&tree).unwrap()).unwrap();
        assert_eq!(json["Composite"]["circuit_hash"][0], 9);
        assert_eq!(
            json["Composite"]["subtasks"][0]["Composite"]["subtasks"][0]["Plain"]
                ["output_preimage"][0],
            "11"
        );

        assert_eq!(serde_json::from_value::<PackedNode>(json).unwrap(), tree);
    }

    /// A circuit hash of the wrong length is rejected at parse time — the fixed-size array is what
    /// lets consumers use it without re-validating its length.
    #[test]
    fn test_packed_node_rejects_wrong_length_circuit_hash() {
        let json = r#"{"Composite":{"circuit_hash":[1,2,3],"subtasks":[]}}"#;
        assert!(serde_json::from_str::<PackedNode>(json).is_err());
    }
}
