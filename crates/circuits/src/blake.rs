use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo_cairo_common::preprocessed_columns::blake::BLAKE_SIGMA;

use crate::circuit::{BlakeGGate, M31ToU32, TripleXor};
use crate::context::{Context, Var};
use crate::eval;
use crate::ivalue::{IValue, NoValue, qm31_from_u32s};
use crate::ops::{Guess, from_partial_evals};
use crate::simd::Simd;
use crate::utils::le_u32s_from_bytes;
use crate::wrappers::U32Wrapper;

#[cfg(test)]
#[path = "blake_test.rs"]
pub mod test;

/// Number of 32-bit words in a Blake2s-256 digest (256 bits / 32).
pub const BLAKE2S_DIGEST_N_WORDS: usize = 8;

/// A Blake2s digest kept as its eight raw 32-bit output words (the result of [`blake2s_u32s`]),
/// each encoded as a QM31 `(low_u16, high_u16, 0, 0)`.
///
/// This is the primary in-circuit representation of a hash: the eight words are the faithful
/// Blake2s output and are *not* reduced mod `M31::P`. See [`ReducedHashValue`] for the variant
/// whose words are reduced so that they pack into just two QM31s.
///
/// `T` is the per-element representation: `QM31` for concrete witness values, or [`Var`] for
/// variables inside a [`Context`].
#[derive(Clone, Debug, PartialEq)]
pub struct HashValue<T>(pub [U32Wrapper<T>; BLAKE2S_DIGEST_N_WORDS]);

impl<T> std::ops::Deref for HashValue<T> {
    type Target = [U32Wrapper<T>; BLAKE2S_DIGEST_N_WORDS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Value: IValue> Guess<Value> for HashValue<Value> {
    type Target = HashValue<Var>;
    /// Guesses the eight words via [`U32Wrapper`]'s guess, so each is range-constrained to a valid
    /// `u32` (two 16-bit limbs each checked against `[0, 2^16)`) rather than an arbitrary field
    /// element.
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        HashValue(self.0.guess(context))
    }
}

impl HashValue<NoValue> {
    /// Creates a [`HashValue`] with [`NoValue`] elements, used when building the circuit topology
    /// without concrete witness values.
    pub fn no_value() -> Self {
        Self(std::array::from_fn(|_| U32Wrapper::new_unsafe(NoValue)))
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

/// A variant of [`HashValue`] whose eight 32-bit output words have each been reduced modulo
/// M31 (`p = 2^31 - 1`) so that they pack into just two [`QM31`]s (four M31 limbs each). The two
/// fields hold the reduced limbs `0..4` and `4..8` respectively.
///
/// "Reduced" refers to the `reduce_to_m31` step applied to the raw 256-bit Blake2s output: the
/// digest is no longer a faithful 256-bit hash but a field-friendly representation usable directly
/// inside the circuit.
///
/// `T` is the per-element representation: `QM31` for concrete witness values, or [`Var`] for
/// variables inside a [`Context`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReducedHashValue<T>(pub T, pub T);

impl<Value: IValue> Guess<Value> for ReducedHashValue<Value> {
    type Target = ReducedHashValue<Var>;
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        ReducedHashValue(self.0.guess(context), self.1.guess(context))
    }
}

impl From<Blake2sHash> for ReducedHashValue<QM31> {
    fn from(value: Blake2sHash) -> Self {
        ReducedHashValue(
            qm31_from_bytes(&value.0[0..16].try_into().unwrap()),
            qm31_from_bytes(&value.0[16..32].try_into().unwrap()),
        )
    }
}

impl From<[u32; BLAKE2S_DIGEST_N_WORDS]> for ReducedHashValue<QM31> {
    fn from(value: [u32; BLAKE2S_DIGEST_N_WORDS]) -> Self {
        ReducedHashValue(
            qm31_from_u32s(value[0], value[1], value[2], value[3]),
            qm31_from_u32s(value[4], value[5], value[6], value[7]),
        )
    }
}

impl From<[u32; BLAKE2S_DIGEST_N_WORDS]> for HashValue<QM31> {
    /// Encodes the eight raw 32-bit hash words losslessly, each held as a QM31
    /// `(low_u16, high_u16, 0, 0)`. Unlike [`ReducedHashValue`], the words are *not* reduced
    /// mod `M31::P`.
    fn from(value: [u32; BLAKE2S_DIGEST_N_WORDS]) -> Self {
        HashValue(value.map(|word| U32Wrapper::new_unsafe(IValue::pack_u32(word))))
    }
}

/// Create QM31 from 16 bytes (4 u32s)
pub fn qm31_from_bytes(bytes: &[u8; 16]) -> QM31 {
    let a = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let b = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let c = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let d = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    qm31_from_u32s(a, b, c, d)
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

/// Adds gates to compute the Blake2s hash with the 8 u32 limbs of the output reduced modulo M31.
/// The input message is given as a sequence of QM31 values, and the two output variables are
/// returned as [`ReducedHashValue`].
///
/// NOTE: If the number of bytes is not a multiple of 16, the caller must make sure that the
/// remaining bytes are zero.
/// For example, if `n_bytes` is 4, only the first coordinate of the [`QM31`] may be non-zero.
/// If `n_bytes` is 1, that coordinate must be < 256.
pub fn blake2s_m31<Value: IValue>(
    ctx: &mut Context<Value>,
    input: &[Var],
    n_bytes: usize,
) -> ReducedHashValue<Var> {
    let hash = blake2s(ctx, input, n_bytes);
    reduce_hash_value(ctx, hash)
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
        let simd = Simd::from_packed(vec![var], 4);
        for coord in 0..4 {
            let comp = Simd::unpack_idx(ctx, &simd, coord);
            words.push(U32Wrapper::new_unsafe(m31_to_u32(ctx, comp)));
        }
    }
    words
}

/// Reduces the eight raw Blake2s output words of a [`HashValue`] mod `M31::P` and packs them
/// into a [`ReducedHashValue`] (two QM31s of four `M31` words each).
///
/// Each word, stored as `(low_u16, high_u16, 0, 0)`, is recombined as `low + high * 2^16`; the
/// field arithmetic reduces it mod `M31::P`.
pub fn reduce_hash_value<Value: IValue>(
    ctx: &mut Context<Value>,
    hash: HashValue<Var>,
) -> ReducedHashValue<Var> {
    let c_2_pow_16 = ctx.constant(M31::from(1u32 << 16).into());
    let reduced: [Var; BLAKE2S_DIGEST_N_WORDS] = std::array::from_fn(|i| {
        let h_simd = Simd::from_packed(vec![*hash[i].get()], 2);
        let low = Simd::unpack_idx(ctx, &h_simd, 0);
        let high = Simd::unpack_idx(ctx, &h_simd, 1);
        eval!(ctx, (low) + ((high) * (c_2_pow_16)))
    });

    let out0 = from_partial_evals(ctx, [reduced[0], reduced[1], reduced[2], reduced[3]]);
    let out1 = from_partial_evals(ctx, [reduced[4], reduced[5], reduced[6], reduced[7]]);

    ReducedHashValue(out0, out1)
}

/// Adds the Blake2s block-compression gates to the circuit and returns the hash state as the eight
/// `h` words, each encoded as QM31 `(low_u16, high_u16, 0, 0)`. No reduction modulo `M31::P` is
/// performed (the two 16-bit limbs are not combined into a single field element).
///
/// Each entry of `message_u32s` is a single message word encoded as QM31 `(low_u16, high_u16, 0,
/// 0)`. The vector is zero-padded up to a whole number of 64-byte blocks.
///
/// NOTE: If the number of bytes is not a multiple of 4, the caller must make sure that the
/// remaining bytes of the last word are zero.
pub fn blake2s_u32s<Value: IValue>(
    ctx: &mut Context<Value>,
    mut message_u32s: Vec<U32Wrapper<Var>>,
    n_bytes: usize,
) -> HashValue<Var> {
    const BLOCK_BYTES: usize = 64;
    const WORDS_PER_BLOCK: usize = 16;

    let n_blocks = std::cmp::max(1, n_bytes.div_ceil(BLOCK_BYTES));
    let total_words = n_blocks * WORDS_PER_BLOCK;
    let zero_u32 = ctx.constant(QM31::pack_u32(0));
    while message_u32s.len() < total_words {
        message_u32s.push(U32Wrapper::new_unsafe(zero_u32));
    }

    // `h`: IV XORed with the parameter block (depth 1, fanout 1, digest length 32, key length 0).
    let mut h: [Var; 8] = std::array::from_fn(|i| {
        let iv_val = if i == 0 { BLAKE2S_IV[0] ^ 0x01010020 } else { BLAKE2S_IV[i] };
        ctx.constant(QM31::pack_u32(iv_val))
    });

    for block_idx in 0..n_blocks {
        let block: [Var; WORDS_PER_BLOCK] =
            std::array::from_fn(|i| *message_u32s[block_idx * WORDS_PER_BLOCK + i].get());
        let t0 = std::cmp::min(n_bytes, (block_idx + 1) * BLOCK_BYTES) as u32;
        let t1 = 0u32;
        let last = block_idx == n_blocks - 1;

        let prev_h = h;

        let mut v: [Var; 16] = std::array::from_fn(|i| {
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
                ctx.constant(QM31::pack_u32(iv))
            }
        });

        for permutation in BLAKE_SIGMA.iter() {
            for (g_idx, &(ai, bi, ci, di)) in G_STATE_INDICES.iter().enumerate() {
                let (new_a, new_b, new_c, new_d) = blake_g_gate(
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
    HashValue(h.map(U32Wrapper::new_unsafe))
}

/// Adds a TripleXor gate to the circuit: XOR three u32 values encoded as QM31 `(u16, u16, 0, 0)`
/// and return the result in the same encoding.
pub fn triple_xor<Value: IValue>(
    ctx: &mut Context<Value>,
    input_a: Var,
    input_b: Var,
    input_c: Var,
) -> Var {
    let a = ctx.get(input_a).unpack_u32();
    let b = ctx.get(input_b).unpack_u32();
    let c = ctx.get(input_c).unpack_u32();
    let out = ctx.new_var(Value::pack_u32(a ^ b ^ c));
    ctx.stats.triple_xor += 1;
    ctx.circuit.triple_xor.push(TripleXor {
        input_a: input_a.idx,
        input_b: input_b.idx,
        input_c: input_c.idx,
        out: out.idx,
    });
    out
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
pub fn m31_to_u32<Value: IValue>(ctx: &mut Context<Value>, input: Var) -> Var {
    let out = ctx.new_var(ctx.get(input).m31_to_u32());
    m31_to_u32_into(ctx, input, out);
    out
}

pub fn m31_to_u32_into<Value: IValue>(ctx: &mut Context<Value>, input: Var, out: Var) {
    ctx.stats.m31_to_u32 += 1;
    ctx.circuit.m31_to_u32.push(M31ToU32 { input: input.idx, out: out.idx });
}

/// Adds a Blake2s G function gate to the circuit: G(a, b, c, d, f0, f1) = (a', b', c', d').
/// Inputs and outputs are all encoded as `(low_u16, high_u16, 0, 0)` in QM31.
pub fn blake_g_gate<Value: IValue>(
    ctx: &mut Context<Value>,
    input_a: Var,
    input_b: Var,
    input_c: Var,
    input_d: Var,
    input_f0: Var,
    input_f1: Var,
) -> (Var, Var, Var, Var) {
    let a = ctx.get(input_a).unpack_u32();
    let b = ctx.get(input_b).unpack_u32();
    let c = ctx.get(input_c).unpack_u32();
    let d = ctx.get(input_d).unpack_u32();
    let f0 = ctx.get(input_f0).unpack_u32();
    let f1 = ctx.get(input_f1).unpack_u32();

    let (a_out, b_out, c_out, d_out) = blake2s_g(a, b, c, d, f0, f1);

    let out_a = ctx.new_var(Value::pack_u32(a_out));
    let out_b = ctx.new_var(Value::pack_u32(b_out));
    let out_c = ctx.new_var(Value::pack_u32(c_out));
    let out_d = ctx.new_var(Value::pack_u32(d_out));

    ctx.circuit.blake_g_gate.push(BlakeGGate {
        input_a: input_a.idx,
        input_b: input_b.idx,
        input_c: input_c.idx,
        input_d: input_d.idx,
        input_f0: input_f0.idx,
        input_f1: input_f1.idx,
        out_a: out_a.idx,
        out_b: out_b.idx,
        out_c: out_c.idx,
        out_d: out_d.idx,
    });

    (out_a, out_b, out_c, out_d)
}
