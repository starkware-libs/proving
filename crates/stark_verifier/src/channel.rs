use circuits::blake::{
    HashValue, ReducedHashValue, blake2s_m31, blake2s_u32s, reduce_hash_value,
    unpack_qm31s_to_u32_words,
};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{eq, inv, pointwise_mul};
use circuits::simd::Simd;
use circuits::wrappers::U32Wrapper;
use stwo::core::circle::CirclePoint;
use stwo::core::fields::m31::MODULUS_BITS;

#[cfg(test)]
#[path = "channel_test.rs"]
pub mod test;

pub struct Channel {
    /// The current digest of the channel.
    digest: ReducedHashValue<Var>,
    /// The number of times values were taken from the channel.
    n_draws: usize,
}

impl Channel {
    const POW_PREFIX: u32 = 0x12345678;

    /// Constructs a new channel, with a zero digest.
    pub fn new(context: &mut Context<impl IValue>) -> Self {
        let zero = context.zero();
        Self { digest: ReducedHashValue(zero, zero), n_draws: 0 }
    }

    /// Updates the digest with the given new digest, and resets the number of draws to zero.
    fn update_digest(&mut self, new_digest: ReducedHashValue<Var>) {
        self.digest = new_digest;
        self.n_draws = 0;
    }

    #[cfg(test)]
    pub fn digest(&self) -> ReducedHashValue<Var> {
        self.digest
    }

    #[cfg(test)]
    pub fn from_digest(
        context: &mut circuits::context::TraceContext,
        init_digest: [stwo::core::fields::qm31::QM31; 2],
    ) -> Self {
        Self {
            digest: ReducedHashValue(
                context.constant(init_digest[0]),
                context.constant(init_digest[1]),
            ),
            n_draws: 0,
        }
    }

    /// Mixes the given (unreduced) Merkle root into the channel's digest.
    ///
    /// Mirrors the out-of-circuit `Blake2sM31MerkleChannel::mix_root`: the current
    /// (`M31`-reduced) digest and the full, *unreduced* 32-byte root are concatenated as sixteen
    /// 32-bit words and hashed with the standard Blake2s, then the eight output words are reduced
    /// mod `M31::P` so the channel digest stays in the `M31` field for challenge/query derivation.
    pub fn mix_commitment(&mut self, context: &mut Context<impl IValue>, root: &HashValue<Var>) {
        // Eight message words from the current (reduced) digest, unpacked from its two QM31s into
        // single-word `(low_u16, high_u16, 0, 0)` form.
        let mut message = unpack_qm31s_to_u32_words(context, [self.digest.0, self.digest.1]);
        message.extend(root.0);

        let hash = blake2s_u32s(context, message, 16 * 4);
        self.update_digest(reduce_hash_value(context, hash));
    }

    /// Mixes the given list of `QM31` values into the channel.
    pub fn mix_qm31s(
        &mut self,
        context: &mut Context<impl IValue>,
        values: impl IntoIterator<Item = Var>,
    ) {
        let mut blake_input = vec![self.digest.0, self.digest.1];
        blake_input.extend(values);
        self.update_digest(blake2s_m31(context, &blake_input, 16 * blake_input.len()));
    }

    /// Mixes the given list of `u32` values into the channel.
    ///
    /// Mirrors stwo's `Blake2sM31Channel::mix_u32s`: the new digest is the `M31`-reduced
    /// `Blake2s(digest_words || values)`, where `digest_words` is the current digest encoded as
    /// u32s.
    pub fn mix_u32s(
        &mut self,
        context: &mut Context<impl IValue>,
        values: impl ExactSizeIterator<Item = U32Wrapper<Var>>,
    ) {
        let digest_words = unpack_qm31s_to_u32_words(context, [self.digest.0, self.digest.1]);
        let mut message = Vec::with_capacity(digest_words.len() + values.len());
        message.extend(digest_words);
        message.extend(values);
        // Each word is four bytes: the eight digest words followed by the mixed values.
        let n_bytes = 4 * message.len();
        let hash = blake2s_u32s(context, message, n_bytes);
        self.update_digest(reduce_hash_value(context, hash));
    }

    /// Draws one `QM31` random value from the channel.
    pub fn draw_qm31(&mut self, context: &mut Context<impl IValue>) -> Var {
        let [first, second] = self.draw_two_qm31s(context);
        context.mark_as_unused(second);
        first
    }

    /// Draws two `QM31` random values from the channel.
    ///
    /// The two returned QM31 values are negligibly close to uniform: the per-bit bias from
    /// a perfectly uniform draw is at most 2^{-31}, which is negligible for Fiat-Shamir
    /// security.
    ///
    /// Detailed breakdown: each QM31 consists of 4 M31 limbs, each a 32-bit Blake word
    /// reduced mod M31 (p = 2^31 - 1). Since 2^32 = 2*M31 + 2, values 0 and 1 each have
    /// 3 preimages in the u32 range (probability 3/2^32 each) while every v in
    /// {2, ..., M31-1} has exactly 2 preimages (probability 2/2^32). Propagating this
    /// through the bit representation of the M31 value, we have that P(k-th bit = 1) is
    ///   = (2^31 - 1) / 2^32 if k = 0  (bias 2^{-32} from uniform)
    ///   = (2^31 - 2) / 2^32 if k >= 1 (bias 2^{-31} from uniform)
    /// (bits k >= 1 are slightly more biased because values 0 and 1 both have bit k = 0)
    pub fn draw_two_qm31s(&mut self, context: &mut Context<impl IValue>) -> [Var; 2] {
        let n_draws_var =
            context.constant(qm31_from_u32s(self.n_draws.try_into().unwrap(), 0, 0, 0));
        // Note that we add a zero byte for domain separation between generating randomness and
        // mixing a single u32.
        let res =
            blake2s_m31(context, &[self.digest.0, self.digest.1, n_draws_var], 16 + 16 + 4 + 1);
        self.n_draws += 1;
        [res.0, res.1]
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
        let input = [
            context.constant(qm31_from_u32s(Self::POW_PREFIX, 0, 0, 0)),
            self.digest.0,
            self.digest.1,
            context.constant(qm31_from_u32s(n_bits, 0, 0, 0)),
        ];
        let prefixed_digest = blake2s_m31(context, &input, 52);

        // Check that `nonce` consists of only the first two M31 elements.
        let nonce_high_mask = context.constant(qm31_from_u32s(0, 0, 1, 1));
        let masked_nonce = pointwise_mul(context, nonce, nonce_high_mask);
        eq(context, masked_nonce, context.zero());

        // Compute `H(prefixed_digest, nonce)`.
        let input = [prefixed_digest.0, prefixed_digest.1, nonce];
        let ReducedHashValue(res0, res1) = blake2s_m31(context, &input, 40);
        context.mark_as_unused(res1);

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
            eq(context, bit.get_packed()[0], context.zero());
        }

        // Mix nonce into the channel.
        self.update_digest(blake2s_m31(context, &[self.digest.0, self.digest.1, nonce], 40));
    }
}
