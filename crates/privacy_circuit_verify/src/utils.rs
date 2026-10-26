use circuits::blake::BLAKE2S_DIGEST_N_WORDS;
use itertools::Itertools;

/// Number of bytes a serialized [`Version`] occupies (major, minor, patch).
pub const VERSION_BYTES: usize = 3;
pub const BLAKE2S_DIGEST_N_BYTES: usize = 4 * BLAKE2S_DIGEST_N_WORDS;

pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

pub struct ProofHeader {
    pub version: Version,
    pub preprocessed_root: [u8; BLAKE2S_DIGEST_N_BYTES],
}

impl Version {
    /// Returns the version of the privacy crates.
    pub fn current() -> Self {
        Self {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().expect("major version fits in u8"),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().expect("minor version fits in u8"),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().expect("patch version fits in u8"),
        }
    }

    /// Serializes the version into its byte representation.
    pub fn serialize(&self) -> [u8; VERSION_BYTES] {
        [self.major, self.minor, self.patch]
    }

    /// Deserializes a version from its byte representation.
    pub fn deserialize(bytes: &[u8; VERSION_BYTES]) -> Self {
        Self { major: bytes[0], minor: bytes[1], patch: bytes[2] }
    }
}

impl ProofHeader {
    pub const SIZE: usize = BLAKE2S_DIGEST_N_BYTES + VERSION_BYTES;

    pub fn serialize(&self) -> [u8; Self::SIZE] {
        self.version
            .serialize()
            .into_iter()
            .chain(self.preprocessed_root)
            .collect_vec()
            .try_into()
            .unwrap()
    }

    pub fn deserialize(bytes: &[u8; Self::SIZE]) -> Self {
        let version_bytes: [u8; VERSION_BYTES] = bytes[..VERSION_BYTES].try_into().unwrap();
        let pp_bytes: [u8; BLAKE2S_DIGEST_N_BYTES] = bytes[VERSION_BYTES..].try_into().unwrap();
        ProofHeader { version: Version::deserialize(&version_bytes), preprocessed_root: pp_bytes }
    }
}
