use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue, blake2s_u32s, unpack_qm31s_to_u32_words};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::{IValue, qm31_from_u32s};
<<<<<<< HEAD
use circuits::ops::{eq, inv, pointwise_mul};
use circuits::simd::{Simd, get_coord};
||||||| 167116f40
use circuits::ops::{eq, inv, pointwise_mul};
use circuits::simd::Simd;
=======
use circuits::ops::{eq, from_partial_evals, inv, pointwise_mul};
use circuits::simd::Simd;
>>>>>>> origin/sharp-8.0
use circuits::wrappers::U32Wrapper;
use stwo::core::circle::CirclePoint;
#[cfg(test)]
#[path = "channel_test.rs"]
pub mod test;

pub struct Channel {
    /// The current digest of the channel.
    digest: HashValue<Var>,
    /// The number of times values were taken from the channel.
    n_draws: usize,
}

impl Channel {
    const POW_PREFIX: u32 = 0x12345678;

    /// Constructs a new channel, with a zero digest.
    pub fn new(context: &mut Context<impl IValue>) -> Self {
        Self {
            digest: HashValue([U32Wrapper::const_u32(context, 0); BLAKE2S_DIGEST_N_WORDS]),
            n_draws: 0,
        }
    }

    /// Updates the digest with the given new digest, and resets the number of draws to zero.
    fn update_digest(&mut self, new_digest: HashValue<Var>) {
        self.digest = new_digest;
        self.n_draws = 0;
    }

    #[cfg(test)]
    pub fn digest(&self) -> HashValue<Var> {
        self.digest.clone()
    }

    #[cfg(test)]
    pub fn from_digest(
        context: &mut circuits::context::TraceContext,
        init_digest: [u32; 8],
    ) -> Self {
        Self {
            digest: HashValue(init_digest.map(|x| U32Wrapper::const_u32(context, x))),
            n_draws: 0,
        }
    }

<<<<<<< HEAD
    /// Mixes the given (unreduced) Merkle root into the channel's digest.
    ///
    /// Mirrors the out-of-circuit `Blake2sM31MerkleChannel::mix_hash`: the current
    /// (`M31`-reduced) digest and the full, *unreduced* 32-byte root are concatenated as sixteen
    /// 32-bit words and hashed with the standard Blake2s, then the eight output words are reduced
    /// mod `M31::P` so the channel digest stays in the `M31` field for challenge/query derivation.
||||||| 167116f40
    /// Mixes the given (unreduced) Merkle root into the channel's digest.
    ///
    /// Mirrors the out-of-circuit `Blake2sM31MerkleChannel::mix_root`: the current
    /// (`M31`-reduced) digest and the full, *unreduced* 32-byte root are concatenated as sixteen
    /// 32-bit words and hashed with the standard Blake2s, then the eight output words are reduced
    /// mod `M31::P` so the channel digest stays in the `M31` field for challenge/query derivation.
=======
    /// Mixes the given Merkle root into the channel's digest.
>>>>>>> origin/sharp-8.0
    pub fn mix_commitment(&mut self, context: &mut Context<impl IValue>, root: &HashValue<Var>) {
        self.mix_u32s(context, root.0.iter().cloned());
    }

    /// Mixes the given list of `QM31` values into the channel.
    pub fn mix_qm31s(
        &mut self,
        context: &mut Context<impl IValue>,
        values: impl IntoIterator<Item = Var>,
    ) {
        let mut message = self.digest.to_vec();
        message.extend(unpack_qm31s_to_u32_words(context, values));
        let n_words = message.len();
        let hash = blake2s_u32s(context, message, 4 * n_words);
        self.update_digest(hash);
    }

    /// Mixes the given list of `u32` values into the channel.
    pub fn mix_u32s(
        &mut self,
        context: &mut Context<impl IValue>,
        values: impl ExactSizeIterator<Item = U32Wrapper<Var>>,
    ) {
        let n_words = 8 + values.len();
        let mut message = Vec::with_capacity(n_words);
        message.extend(self.digest.0);
        message.extend(values);
        let hash = blake2s_u32s(context, message, 4 * n_words);
        self.update_digest(hash);
    }

    /// Draw eight `u32` random values from the channel.
    pub fn draw_u32s(&mut self, context: &mut Context<impl IValue>) -> HashValue<Var> {
        let n_draw_u32_var = U32Wrapper::const_u32(context, self.n_draws.try_into().unwrap());
        self.n_draws += 1;
        // Note that we add a zero byte for domain separation between generating randomness and
        // mixing a single u32.
        let mut message = self.digest.to_vec();
        message.push(n_draw_u32_var);
        blake2s_u32s(context, message, (8 + 1) * 4 + 1)
    }

    /// Draws one `QM31` random value from the channel.
    ///
    /// The drawn QM31 value is negligibly close to uniform: each M31 limb is obtained from reducing
    /// a uniformly random u64 word mod M31::P, thus every M31 value is attained with probability
    /// 1/|M31| up to relative error less than 2^-33. It follows that each QM31 value is attained
    /// with probability 1/|QM31| up to relative error less than 2^-31, which is indeed negligible.
    pub fn draw_qm31(&mut self, context: &mut Context<impl IValue>) -> Var {
        let hash = self.draw_u32s(context);
        random_qm31_from_hash(context, hash)
    }

    /// Draws a random point on the (`QM31`) circle from the channel.
    pub fn draw_point(&mut self, context: &mut Context<impl IValue>) -> CirclePoint<Var> {
        let t = self.draw_qm31(context);
        let t2 = eval!(context, (t) * (t));

        let denom = eval!(context, (t2) + (1));
        // denom = t^2 + 1; zero only for t = ±i in QM31.
        let denom_inv = inv(context, denom);
        let x = eval!(context, ((1) - (t2)) * (denom_inv));
        let y = eval!(context, ((2) * (t)) * (denom_inv));
        CirclePoint { x, y }
    }

    pub fn pow(&mut self, context: &mut Context<impl IValue>, n_bits: u32, nonce: Var) {
        assert!(n_bits <= 30);

        // Compute `H(POW_PREFIX, [0_u8; 12], digest, n_bits)`.
        let mut input = vec![
            U32Wrapper::const_u32(context, Self::POW_PREFIX),
            U32Wrapper::const_u32(context, 0),
            U32Wrapper::const_u32(context, 0),
            U32Wrapper::const_u32(context, 0),
        ];
        input.extend(self.digest.0);
        input.push(U32Wrapper::const_u32(context, n_bits));
        let prefixed_digest = blake2s_u32s(context, input, 4 * 13);

        // TODO: Update nonce to be [U32Wrapper; 2].
        // Check that `nonce` consists of only the first two M31 elements.
        let nonce_high_mask = context.constant(qm31_from_u32s(0, 0, 1, 1));
        let masked_nonce = pointwise_mul(context, nonce, nonce_high_mask);
        eq(context, masked_nonce, context.zero());

        let unpacked_nonce = unpack_qm31s_to_u32_words(context, [nonce]);

        // Compute `H(prefixed_digest, nonce)`.
        let mut input = prefixed_digest.to_vec();
        input.extend(unpacked_nonce.clone());
        let hash = blake2s_u32s(context, input, (8 + 2) * 4);
        for var in hash.0.iter().skip(1) {
            context.mark_as_unused(*var.get());
        }

<<<<<<< HEAD
        // Take the first word.
        let first_word = get_coord(context, &res0, 0);

        // Check that the n_bits least significant bits are zero. The only deviation from a
        // uniform draw is that M31 value 0 has one extra u32 preimage (2*M31 in addition to
        // 0 and M31), making PoW easier by a relative factor of 1/2^(32-n_bits), i.e. a
        // loss of 0.32 bits of security for n_bits=30 and much less for smaller values.
        // Value 1 also has an extra preimage but is irrelevant here since 1 is never
        // all-zero bits.
        let bits = extract_bits(context, &Simd::pack_single(first_word), MODULUS_BITS);
        for bit in &bits[0..n_bits.try_into().unwrap()] {
||||||| 167116f40
        // Take the first word.
        let first_word = pointwise_mul(context, res0, context.one());

        // Check that the n_bits least significant bits are zero. The only deviation from a
        // uniform draw is that M31 value 0 has one extra u32 preimage (2*M31 in addition to
        // 0 and M31), making PoW easier by a relative factor of 1/2^(32-n_bits), i.e. a
        // loss of 0.32 bits of security for n_bits=30 and much less for smaller values.
        // Value 1 also has an extra preimage but is irrelevant here since 1 is never
        // all-zero bits.
        let bits = extract_bits(context, &Simd::from_packed(vec![first_word], 1), MODULUS_BITS);
        for bit in &bits[0..n_bits.try_into().unwrap()] {
=======
        // Check that the n_bits least significant bits are zero.
        let simd = Simd::from_packed(vec![*hash[0].get()], 2);
        let low = Simd::unpack_idx(context, &simd, 0);
        let (var_to_extract, bits_to_extract) = if n_bits < 16 {
            (low, n_bits)
        } else {
            eq(context, low, context.zero());
            let high = Simd::unpack_idx(context, &simd, 1);
            (high, n_bits - 16)
        };
        let bits = extract_bits(context, &Simd::from_packed(vec![var_to_extract], 1), 16);
        for bit in &bits[0..bits_to_extract.try_into().unwrap()] {
>>>>>>> origin/sharp-8.0
            eq(context, bit.get_packed()[0], context.zero());
        }

        // Mix nonce into the channel.
        let mut input = self.digest.to_vec();
        input.extend(unpacked_nonce);
        let hash = blake2s_u32s(context, input, (8 + 2) * 4);
        self.update_digest(hash);
    }
}

/// Reinterprets the raw Blake2s output [`HashValue`] of eight u32 words as four u64 words,
/// reduces each one mod `M31::P` and packs the result into a QM31.
///
/// Each pair of u32 words is stored as `low32 = (low_u32_low_u16, low_u32_high_u16, 0, 0)`,
/// `high32 = (high_u32_low_u16, high_u32_high_u16, 0, 0)`. They are first recombined as QM31 into
/// `reduced32 = low32 + 2 * high32`, which is equivalent to `low32 + 2^32 * high32` mod M31::P.
/// Then reduced32 is reduced into a single value modulo M31::P by multiplying with the "combiner"
/// `(1, -2^16, 0, 0)`, which puts the desired value `reduced32_low16 + 2^16 * reduced32_high16`
///  in the 0-th limb of the result.
pub fn random_qm31_from_hash<Value: IValue>(ctx: &mut Context<Value>, hash: HashValue<Var>) -> Var {
    // The QM31 constant 1 - 2^16 * i.
    let combiner = ctx.constant(qm31_from_u32s(1, (1 << 31) - (1 << 16) - 1, 0, 0));
    let reduced: [Var; BLAKE2S_DIGEST_N_WORDS / 2] = std::array::from_fn(|i| {
        let low_32 = *hash[2 * i].get();
        let high_32 = *hash[2 * i + 1].get();
        let reduced_32 = eval!(ctx, (low_32) + ((high_32) + (high_32)));
        let combined = eval!(ctx, (reduced_32) * (combiner));
        let simd = Simd::from_packed(vec![combined], 2);

        Simd::unpack_idx(ctx, &simd, 0)
    });

    from_partial_evals(ctx, reduced)
}
