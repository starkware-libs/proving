use blake2::{Blake2s256, Digest};
use circuits::utils::le_u32s_from_bytes;

/// Out-of-circuit implementation of [`circuits::blake::blake2s_u32s`].
pub fn native_blake_u32s(words: &[u32]) -> [u32; 8] {
    let mut hasher = Blake2s256::new();
    for word in words {
        hasher.update(word.to_le_bytes());
    }
    let hash: [u8; 32] = hasher.finalize().into();
    le_u32s_from_bytes(hash)
}
