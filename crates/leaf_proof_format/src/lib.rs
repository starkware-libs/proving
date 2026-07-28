use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::base64::Base64;
use serde_with::serde_as;

/// A Blake2s digest: eight little-endian u32 words, serialized as an array of `0x`-prefixed hex
/// strings.
///
/// This is the wire format of `circuit_registry::schema::RootHex`. Leaf-prover output is compared
/// against registry entries, so both use the same representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestHex(pub [u32; 8]);

impl From<[u8; 32]> for DigestHex {
    fn from(bytes: [u8; 32]) -> Self {
        let mut words = [0u32; 8];
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
        let words: [String; 8] = Deserialize::deserialize(deserializer)?;
        let mut digest = [0u32; 8];
        for (out, word) in digest.iter_mut().zip(words) {
            let hex = word.strip_prefix("0x").unwrap_or(&word);
            *out = u32::from_str_radix(hex, 16).map_err(serde::de::Error::custom)?;
        }
        Ok(DigestHex(digest))
    }
}

/// Describes the structure of the output JSON file of the leaf prover.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A digest is serialized as eight `0x`-prefixed little-endian words, and parses back.
    #[test]
    fn digest_hex_round_trip() {
        // The first word is the first four bytes, little-endian.
        let digest = DigestHex::from(std::array::from_fn::<u8, 32, _>(|i| i as u8));
        assert_eq!(digest.0[0], 0x03020100);

        let json = serde_json::to_string(&digest).unwrap();
        assert!(json.starts_with(r#"["0x03020100","#), "unexpected encoding: {json}");
        assert_eq!(serde_json::from_str::<DigestHex>(&json).unwrap(), digest);
    }
}
