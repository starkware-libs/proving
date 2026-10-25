use blake2::{Blake2s256, Digest};
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo_cairo_common::preprocessed_columns::blake::BLAKE_SIGMA;

use crate::circuit::{BlakeGGate, M31ToU32, TripleXor};
use crate::context::{Context, Var};
use crate::ivalue::{IValue, NoValue};
use crate::ops::{Constant, Guess};
use crate::simd::get_coord;
use crate::utils::le_u32s_from_bytes;
use crate::wrappers::{M31Wrapper, U32Wrapper};

#[cfg(test)]
#[path = "blake_test.rs"]
pub mod test;

/// Number of 32-bit words in a Blake2s-256 digest (256 bits / 32).
pub const BLAKE2S_DIGEST_N_WORDS: usize = 8;

/// A Blake2s digest kept as its eight raw 32-bit output words (the result of [`blake2s_u32s`]),
/// each encoded as a QM31 `(low_u16, high_u16, 0, 0)`.
///
/// This is the primary in-circuit representation of a hash: the eight words are the faithful
/// Blake2s output and are *not* reduced mod `M31::P`.
///
/// `T` is the per-element representation: `QM31` for concrete witness values, or [`Var`] for
/// variables inside a [`Context`].
#[derive(Clone, Debug, PartialEq)]
pub struct HashValue<T>(pub [U32Wrapper<T>; BLAKE2S_DIGEST_N_WORDS]);

impl<Value: IValue> Guess<Value> for HashValue<Value> {
    type Target = HashValue<Var>;
    /// Guesses the eight words via [`U32Wrapper`]'s guess, so each is range-constrained to a valid
    /// `u32` (two 16-bit limbs each checked against `[0, 2^16)`) rather than an arbitrary field
    /// element.
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        HashValue(self.0.guess(context))
    }
}

impl<Value: IValue> Constant<Value> for HashValue<QM31> {
    type Target = HashValue<Var>;

    fn constant(&self, context: &mut Context<Value>) -> HashValue<Var> {
        HashValue(self.0.constant(context))
    }
}

impl HashValue<NoValue> {
    /// Creates a [`HashValue`] with [`NoValue`] elements, used when building the circuit topology
    /// without concrete witness values.
    pub fn no_value() -> Self {
        Self(std::array::from_fn(|_| U32Wrapper::no_value()))
    }
}

impl<Value: Copy> HashValue<Value> {
    /// Returns an iterator over the 8 u32 elements of the hash value.
    pub fn iter(&self) -> impl Iterator<Item = &U32Wrapper<Value>> {
        self.0.iter()
    }

    /// Returns the 8 u32 elements of the hash value as an array.
    pub fn as_array(&self) -> &[U32Wrapper<Value>; BLAKE2S_DIGEST_N_WORDS] {
        &self.0
    }

    /// Returns the 8 u32 elements of the hash value as a vector.
    pub fn to_vec(&self) -> Vec<U32Wrapper<Value>> {
        self.0.to_vec()
    }
}

impl<T> IntoIterator for HashValue<T> {
    type Item = U32Wrapper<T>;
    type IntoIter = std::array::IntoIter<U32Wrapper<T>, BLAKE2S_DIGEST_N_WORDS>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Value: Copy> std::ops::Index<usize> for HashValue<Value> {
    type Output = U32Wrapper<Value>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl From<Blake2sHash> for HashValue<QM31> {
    /// Encodes the [`Blake2sHash`] into eight [`U32Wrapper`]s, one per 32-bit (little-endian)
    /// hash word, each word is held as a QM31 `(low_u16, high_u16, 0, 0)`.
    fn from(value: Blake2sHash) -> Self {
        let words: [u32; BLAKE2S_DIGEST_N_WORDS] = le_u32s_from_bytes(value.0);
        HashValue::from(words)
    }
}

impl From<[u32; BLAKE2S_DIGEST_N_WORDS]> for HashValue<QM31> {
    /// Encodes the eight raw 32-bit hash words losslessly, each held as a QM31
    /// `(low_u16, high_u16, 0, 0)`. The words are not reduced mod `M31::P`.
    fn from(value: [u32; BLAKE2S_DIGEST_N_WORDS]) -> Self {
        HashValue(value.map(IValue::pack_u32))
    }
}

/// Blake2s IV.
pub const BLAKE2S_IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

/// Column indices of the states sent to `G` in each Blake2s round.
const G_STATE_INDICES: [(usize, usize, usize, usize); 8] = [
    (0, 4, 8, 12),
    (1, 5, 9, 13),
    (2, 6, 10, 14),
    (3, 7, 11, 15),
    (0, 5, 10, 15),
    (1, 6, 11, 12),
    (2, 7, 8, 13),
    (3, 4, 9, 14),
];

/// Computes the Blake2s hash of the first `n_bytes` bytes of `input` outside the circuit.
///
/// Each QM31 is serialized as its four M31 coordinates, as little-endian u32 words. The result
/// holds the eight raw output words.
pub fn blake_qm31(input: &[QM31], n_bytes: usize) -> HashValue<QM31> {
    const BYTES_PER_QM31: usize = 16; // 4 coords per QM31, 4 bytes per M31.

    // Sanity check: check the number of bytes is consistent with the number of [QM31] values.
    assert_eq!(input.len(), n_bytes.div_ceil(BYTES_PER_QM31));

    // Serialize each QM31's four M31 coordinates as little-endian u32 words.
    let mut input_bytes: Vec<u8> = Vec::with_capacity(BYTES_PER_QM31 * input.len());
    for x in input {
        for coord in x.to_m31_array() {
            input_bytes.extend_from_slice(&coord.0.to_le_bytes());
        }
    }

    let mut hasher = Blake2s256::new();
    hasher.update(&input_bytes[0..n_bytes]);
    let hash: [u8; 32] = hasher.finalize().into();

    let words: [u32; BLAKE2S_DIGEST_N_WORDS] = le_u32s_from_bytes(hash);
    HashValue::from(words)
}

/// Same as [`blake2s_u32s`], but the input is given as QM31s.
///
/// Each input `Var` is a QM31 packing four message words; they are unpacked into single-word
/// `(low_u16, high_u16, 0, 0)` form (via [`m31_to_u32`]) before being fed to [`blake2s_u32s`].
///
/// NOTE: If the number of bytes is not a multiple of 16, the caller must make sure that the
/// remaining bytes are zero.
/// For example, if `n_bytes` is 4, only the first coordinate of the [`QM31`] may be non-zero.
/// If `n_bytes` is 1, that coordinate must be < 256.
pub fn blake2s<Value: IValue>(
    ctx: &mut Context<Value>,
    input: &[Var],
    n_bytes: usize,
) -> HashValue<Var> {
    // Sanity check: check the number of bytes is consistent with the number of [QM31] values.
    assert_eq!(input.len(), n_bytes.div_ceil(16));

    let message_u32s = unpack_qm31s_to_u32_words(ctx, input.iter().copied());

    blake2s_u32s(ctx, message_u32s, n_bytes)
}

/// Unpacks each QM31 in `input` into its four message words, re-encoding each word as a `u32` in
/// `(low_u16, high_u16, 0, 0)` form via [`m31_to_u32`].
pub fn unpack_qm31s_to_u32_words<Value: IValue>(
    ctx: &mut Context<Value>,
    input: impl IntoIterator<Item = Var>,
) -> Vec<U32Wrapper<Var>> {
    let mut words = Vec::new();
    for var in input {
        for coord in 0..4 {
            let comp = get_coord(ctx, &var, coord);
            words.push(m31_to_u32(ctx, comp));
        }
    }
    words
}

/// Adds the Blake2s block-compression gates to the circuit and returns the hash state as the eight
/// `h` words, each encoded as QM31 `(low_u16, high_u16, 0, 0)`. No reduction modulo `M31::P` is
/// performed (the two 16-bit limbs are not combined into a single field element).
///
/// Each entry of `message_u32s` is a single message word encoded as QM31 `(low_u16, high_u16, 0,
/// 0)`. The vector is zero-padded up to a whole number of 64-byte blocks.
///
/// NOTE: The caller must make sure that every byte past `n_bytes` in the hashed blocks is zero;
/// words past the last hashed block are ignored.
///
/// Panics if `n_bytes` exceeds `4 * message_u32s.len()`.
pub fn blake2s_u32s<Value: IValue>(
    ctx: &mut Context<Value>,
    mut message_u32s: Vec<U32Wrapper<Var>>,
    n_bytes: usize,
) -> HashValue<Var> {
    const BLOCK_BYTES: usize = 64;
    const WORDS_PER_BLOCK: usize = 16;

    assert!(
        n_bytes <= 4 * message_u32s.len(),
        "Cannot hash {n_bytes} bytes from {} message words",
        message_u32s.len()
    );
    let n_blocks = std::cmp::max(1, n_bytes.div_ceil(BLOCK_BYTES));
    let total_words = n_blocks * WORDS_PER_BLOCK;
    let zero_u32 = QM31::pack_u32(0).constant(ctx);
    while message_u32s.len() < total_words {
        message_u32s.push(zero_u32);
    }

    // `h`: IV XORed with the parameter block (depth 1, fanout 1, digest length 32, key length 0).
    let mut h: [U32Wrapper<Var>; 8] = std::array::from_fn(|i| {
        let iv_val = if i == 0 { BLAKE2S_IV[0] ^ 0x01010020 } else { BLAKE2S_IV[i] };
        QM31::pack_u32(iv_val).constant(ctx)
    });

    for block_idx in 0..n_blocks {
        let block: [U32Wrapper<Var>; WORDS_PER_BLOCK] =
            std::array::from_fn(|i| message_u32s[block_idx * WORDS_PER_BLOCK + i]);
        let t0 = std::cmp::min(n_bytes, (block_idx + 1) * BLOCK_BYTES) as u32;
        let t1 = 0u32;
        let last = block_idx == n_blocks - 1;

        let prev_h = h;

        let mut v: [U32Wrapper<Var>; 16] = std::array::from_fn(|i| {
            if i < 8 {
                h[i]
            } else {
                let mut iv = BLAKE2S_IV[i - 8];
                if i == 12 {
                    iv ^= t0;
                }
                if i == 13 {
                    iv ^= t1;
                }
                if i == 14 && last {
                    iv ^= 0xFFFF_FFFF;
                }
                QM31::pack_u32(iv).constant(ctx)
            }
        });

        for permutation in BLAKE_SIGMA.iter() {
            for (g_idx, &(ai, bi, ci, di)) in G_STATE_INDICES.iter().enumerate() {
                let [new_a, new_b, new_c, new_d] = blake_g_gate(
                    ctx,
                    v[ai],
                    v[bi],
                    v[ci],
                    v[di],
                    block[permutation[g_idx * 2] as usize],
                    block[permutation[g_idx * 2 + 1] as usize],
                );
                v[ai] = new_a;
                v[bi] = new_b;
                v[ci] = new_c;
                v[di] = new_d;
            }
        }

        for i in 0..8 {
            h[i] = triple_xor(ctx, prev_h[i], v[i], v[i + 8]);
        }
    }

    ctx.stats.blake_updates += n_blocks;
    HashValue(h)
}

/// Adds a TripleXor gate to the circuit: XOR three u32 values encoded as QM31 `(u16, u16, 0, 0)`
/// and return the result in the same encoding.
pub fn triple_xor<Value: IValue>(
    ctx: &mut Context<Value>,
    input_a: U32Wrapper<Var>,
    input_b: U32Wrapper<Var>,
    input_c: U32Wrapper<Var>,
) -> U32Wrapper<Var> {
    let a = ctx.get(*input_a.raw()).unpack_u32();
    let b = ctx.get(*input_b.raw()).unpack_u32();
    let c = ctx.get(*input_c.raw()).unpack_u32();
    let out = ctx.new_var(*Value::pack_u32(a ^ b ^ c).raw());
    ctx.stats.triple_xor += 1;
    ctx.circuit.triple_xor.push(TripleXor {
        input_a: input_a.raw().idx,
        input_b: input_b.raw().idx,
        input_c: input_c.raw().idx,
        out: out.idx,
    });
    U32Wrapper::new_unsafe(out)
}

/// Blake2s mixing function *G* on four state words `(a, b, c, d)` with message words `f0`, `f1`.
#[must_use]
pub fn blake2s_g(a: u32, b: u32, c: u32, d: u32, f0: u32, f1: u32) -> (u32, u32, u32, u32) {
    let a = a.wrapping_add(b).wrapping_add(f0);
    let d = (d ^ a).rotate_right(16);
    let c = c.wrapping_add(d);
    let b = (b ^ c).rotate_right(12);
    let a = a.wrapping_add(b).wrapping_add(f1);
    let d = (d ^ a).rotate_right(8);
    let c = c.wrapping_add(d);
    let b = (b ^ c).rotate_right(7);
    (a, b, c, d)
}

/// Adds an M31ToU32 gate to the circuit: convert an `M31` value into its `u32` representation, i.e
/// `(x, 0, 0, 0)` into `(x & 0xFFFF, x >> 16, 0, 0)`.
pub fn m31_to_u32<Value: IValue>(
    ctx: &mut Context<Value>,
    input: M31Wrapper<Var>,
) -> U32Wrapper<Var> {
    let out = ctx.new_var(ctx.get(*input.raw()).m31_to_u32());
    m31_to_u32_into(ctx, *input.raw(), out);
    U32Wrapper::new_unsafe(out)
}

pub fn m31_to_u32_into<Value: IValue>(ctx: &mut Context<Value>, input: Var, out: Var) {
    ctx.stats.m31_to_u32 += 1;
    ctx.circuit.m31_to_u32.push(M31ToU32 { input: input.idx, out: out.idx });
}

/// Adds a Blake2s G function gate to the circuit: G(a, b, c, d, f0, f1) = (a', b', c', d').
/// Inputs and outputs are all encoded as `(low_u16, high_u16, 0, 0)` in QM31.
pub fn blake_g_gate<Value: IValue>(
    ctx: &mut Context<Value>,
    input_a: U32Wrapper<Var>,
    input_b: U32Wrapper<Var>,
    input_c: U32Wrapper<Var>,
    input_d: U32Wrapper<Var>,
    input_f0: U32Wrapper<Var>,
    input_f1: U32Wrapper<Var>,
) -> [U32Wrapper<Var>; 4] {
    let a = ctx.get(*input_a.raw()).unpack_u32();
    let b = ctx.get(*input_b.raw()).unpack_u32();
    let c = ctx.get(*input_c.raw()).unpack_u32();
    let d = ctx.get(*input_d.raw()).unpack_u32();
    let f0 = ctx.get(*input_f0.raw()).unpack_u32();
    let f1 = ctx.get(*input_f1.raw()).unpack_u32();

    let (a_out, b_out, c_out, d_out) = blake2s_g(a, b, c, d, f0, f1);

    let out_a = ctx.new_var(*Value::pack_u32(a_out).raw());
    let out_b = ctx.new_var(*Value::pack_u32(b_out).raw());
    let out_c = ctx.new_var(*Value::pack_u32(c_out).raw());
    let out_d = ctx.new_var(*Value::pack_u32(d_out).raw());

    ctx.circuit.blake_g_gate.push(BlakeGGate {
        input_a: input_a.raw().idx,
        input_b: input_b.raw().idx,
        input_c: input_c.raw().idx,
        input_d: input_d.raw().idx,
        input_f0: input_f0.raw().idx,
        input_f1: input_f1.raw().idx,
        out_a: out_a.idx,
        out_b: out_b.idx,
        out_c: out_c.idx,
        out_d: out_d.idx,
    });

    [out_a, out_b, out_c, out_d].map(U32Wrapper::new_unsafe)
}
