use std::sync::LazyLock;

use itertools::{Itertools, zip_eq};
use num_traits::{One, Zero};
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::context::{Context, Var};
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::{Guess, add, eq, mul, pointwise_mul, sub};
use crate::wrappers::M31Wrapper;
use crate::{EXTENSION_DEGREE, eval};

#[cfg(test)]
#[path = "simd_test.rs"]
pub mod test;

static UNIT_VECS: LazyLock<[QM31; 4]> = LazyLock::new(|| {
    [
        qm31_from_u32s(1, 0, 0, 0),
        qm31_from_u32s(0, 1, 0, 0),
        qm31_from_u32s(0, 0, 1, 0),
        qm31_from_u32s(0, 0, 0, 1),
    ]
});
static UNIT_VECS_INV: LazyLock<[QM31; 3]> = LazyLock::new(|| {
    [
        qm31_from_u32s(0, 1, 0, 0).inverse(),
        qm31_from_u32s(0, 0, 1, 0).inverse(),
        qm31_from_u32s(0, 0, 0, 1).inverse(),
    ]
});

/// A vector of `M31` values, represented in packed form.
///
/// Allows to efficiently perform the same operation on multiple `M31` values at once.
#[derive(Clone, Debug)]
pub struct Simd {
    /// Each [Var] is a `QM31` value, which represents 4 `M31` values (the last element may
    /// represent less than 4 values).
    ///
    /// For example, `[a, b, c, d, e, f]` may be represented as:
    /// `[a + b * i + c * u + d * iu, e + f * i]`.
    ///
    /// Note that the last coordinates of the last element are unconstrained,
    /// they are *not* guaranteed to be zero. For example, [Simd::one] returns `1` for the unused
    /// coordinates.
    data: Vec<Var>,
    /// The number of `M31` values represented by [Self::data].
    len: usize,
}
impl Simd {
    /// Constructs a new [Simd] from packed data.
    pub fn from_packed(data: Vec<Var>, len: usize) -> Self {
        // Sanity check: the length of data must be `ceil(len / EXTENSION_DEGREE)`.
        assert_eq!(data.len(), len.div_ceil(EXTENSION_DEGREE));
        Self { data, len }
    }

    /// Returns the number of `M31` values represented by this [Simd].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if there are no `M31` values in this [Simd].
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the data in packed form.
    pub fn get_packed(&self) -> &[Var] {
        &self.data
    }

    /// Returns a [Simd] consisting of `len` copies of the given constant [M31] value.
    pub fn repeat(context: &mut Context<impl IValue>, value: M31, len: usize) -> Simd {
        let v = context.constant(QM31(CM31(value, value), CM31(value, value)));
        Simd { data: vec![v; len.div_ceil(EXTENSION_DEGREE)], len }
    }

    /// Returns a [Simd] consisting of `len` copies of `0`.
    pub fn zero(context: &mut Context<impl IValue>, len: usize) -> Simd {
        Self::repeat(context, M31::zero(), len)
    }

    /// Returns a [Simd] consisting of `len` copies of `1`.
    pub fn one(context: &mut Context<impl IValue>, len: usize) -> Simd {
        Self::repeat(context, M31::one(), len)
    }

    /// Adds gates to the circuit that assert that the two [Simd]s are equal.
    pub fn eq(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) {
        assert_eq!(a.len, b.len);

        let n_chunks = a.len / EXTENSION_DEGREE;
        let n_rem_elements = a.len % EXTENSION_DEGREE;

        for i in 0..n_chunks {
            eq(context, a.data[i], b.data[i]);
        }

        // Handle the last elements.
        if n_rem_elements > 0 {
            let diff = eval!(context, (a.data[n_chunks]) - (b.data[n_chunks]));
            let mask = first_ones(context, n_rem_elements);
            let masked_diff = pointwise_mul(context, diff, mask);
            eq(context, masked_diff, context.zero());
        }
    }

    /// Computes the sum of two [Simd]s (pointwise).
    pub fn add(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) -> Simd {
        assert_eq!(a.len, b.len);
        Simd {
            data: zip_eq(&a.data, &b.data).map(|(x, y)| add(context, *x, *y)).collect(),
            len: a.len,
        }
    }

    /// Computes the subtraction of two [Simd]s (pointwise).
    pub fn sub(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) -> Simd {
        assert_eq!(a.len, b.len);
        Simd {
            data: zip_eq(&a.data, &b.data).map(|(x, y)| sub(context, *x, *y)).collect(),
            len: a.len,
        }
    }

    /// Computes the product of two [Simd]s (pointwise).
    pub fn mul(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) -> Simd {
        assert_eq!(a.len, b.len);
        Simd {
            data: zip_eq(&a.data, &b.data).map(|(x, y)| pointwise_mul(context, *x, *y)).collect(),
            len: a.len,
        }
    }

    /// Multiplies all the elements of `a` by the scalar `b`.
    pub fn scalar_mul(context: &mut Context<impl IValue>, a: &Simd, b: &M31Wrapper<Var>) -> Simd {
        Simd { data: a.data.iter().map(|x| mul(context, *x, *b.get())).collect(), len: a.len }
    }

    /// Returns an *unconstrained* [Simd] initialized (hint) with `1/x` if `x != 0`,
    /// and `0` if`x = 0`.
    /// NOTE: The caller MUST verify that the result is correct.
    pub fn guess_inv_or_zero(&self, context: &mut Context<impl IValue>) -> Simd {
        let values =
            self.data.iter().map(|x| context.get(*x).pointwise_inv_or_zero()).collect_vec();
        Simd { data: values.guess(context), len: self.len }
    }

    /// Verifies that the inputs are bits (0 or 1).
    pub fn assert_bits(&self, context: &mut Context<impl IValue>) {
        // TODO(lior): Consider doing it more efficiently, by adding a constraint of the form:
        //   `input * input = input`.
        let input_sqr = Simd::mul(context, self, self);
        Simd::eq(context, self, &input_sqr);
    }

    /// Computes the inverse of each `M31` in the current [Simd].
    ///
    /// In particular, guarantees that all values are non-zero.
    pub fn inv(&self, context: &mut Context<impl IValue>) -> Simd {
        let res = self.guess_inv_or_zero(context);
        let prod = Simd::mul(context, &res, self);
        let one = Simd::one(context, self.len);

        // Note that `Simd::eq` applies only to the first `self.len` values.
        // The rest will remain unconstrained.
        Simd::eq(context, &prod, &one);

        res
    }

    /// Returns an *unconstrained* [Simd] initialized (hint) with the LSB of each `M31` in the
    /// current [Simd].
    ///
    /// NOTE: The result is guaranteed to be either 0 or 1, but the caller MUST verify that it is
    /// indeed the LSB of the input.
    pub fn guess_lsb(&self, context: &mut Context<impl IValue>) -> Simd {
        let values = self.data.iter().map(|x| context.get(*x).pointwise_lsb()).collect_vec();
        let out = Simd { data: values.guess(context), len: self.len };
        out.assert_bits(context);
        out
    }

    /// Returns `if_zero` if `selector` is 0, and `if_one` if `selector` is 1.
    /// Assumption: `selector` is either 0 or 1.
    pub fn select(
        context: &mut Context<impl IValue>,
        selector: &Simd,
        if_zero: &Simd,
        if_one: &Simd,
    ) -> Simd {
        // Compute: `if_one - if_zero`.
        let x = Simd::sub(context, if_one, if_zero);
        // Compute: `selector * (if_one - if_zero)`.
        let y = Simd::mul(context, selector, &x);
        // Compute `if_zero + selector * (if_one - if_zero)`.
        Simd::add(context, if_zero, &y)
    }

    /// Unpacks a [Simd] into a vector of [Var]s, where each [Var] represents a single [M31] value.
    pub fn unpack(context: &mut Context<impl IValue>, input: &Simd) -> Vec<Var> {
        (0..input.len).map(|i| Self::unpack_idx(context, input, i)).collect_vec()
    }

    /// Unpacks the `idx`-th [M31] value from the [Simd].
    pub fn unpack_idx(context: &mut Context<impl IValue>, input: &Simd, idx: usize) -> Var {
        let qm31_var = input.data[idx / 4];
        let coord = idx % 4;
        // To obtain the `coord`-th coordinate, `c`, start with pointwise multiplication
        // by a unit vector. This results in `c * unit_vecs[coord]`.
        let unit_vec = context.constant(UNIT_VECS[coord]);
        let x = pointwise_mul(context, qm31_var, unit_vec);
        // Then, divide by `unit_vecs[coord]` to get `c`.
        // For coord == 0, UNIT_VECS[0] = (1,0,0,0), so the pointwise multiplication already
        // zeroed out the other coordinates and leaves c unchanged — no division needed.
        if coord == 0 {
            x
        } else {
            eval!(context, (x) * (context.constant(UNIT_VECS_INV[coord - 1])))
        }
    }

    /// Packs a vector of [M31] values into [Simd].
    pub fn pack(context: &mut Context<impl IValue>, values: &[M31Wrapper<Var>]) -> Simd {
        let unit_vecs = UNIT_VECS.map(|v| context.constant(v));

        let n = values.len();
        let data = (0..n.div_ceil(4))
            .map(|i| {
                let mut res = *values[4 * i].get();
                for j in 1..4 {
                    if 4 * i + j == n {
                        break;
                    }
                    res = eval!(context, (res) + ((unit_vecs[j]) * (*values[4 * i + j].get())));
                }
                res
            })
            .collect();
        Simd::from_packed(data, values.len())
    }

    /// Computes `2^n` per lane, where `n` is the integer whose little-endian bit decomposition
    /// is given by `bits`: `n = bits[0] + bits[1]*2 + bits[2]*4 + ...`.
    ///
    /// Each `bits[i]` must contain only 0/1 values; `bits` must be non-empty and have at most
    /// 5 elements, and the encoded value `n` must be at most 30 (so that `2^n` stays within M31).
    pub fn pow2(context: &mut Context<impl IValue>, bits: &[Simd]) -> Simd {
        let len = bits[0].len();
        let mut res = Simd::one(context, len);
        let mut pow2 = M31Wrapper::new_unsafe(context.constant(2.into()));
        for (bit_idx, bit) in bits.iter().enumerate() {
            let res_if_bit_is_one = Simd::scalar_mul(context, &res, &pow2);
            // Select between `res` and `res_if_bit_is_one` based on the value of the bit.
            res = Simd::select(context, bit, &res, &res_if_bit_is_one);
            // Square `pow2` to advance to the next power of two, except on the last iteration.
            if bit_idx < bits.len() - 1 {
                pow2 = M31Wrapper::mul(context, pow2.clone(), pow2.clone());
            }
        }
        res
    }

    /// Packs little-endian bit-slices into a SIMD integer.
    /// This is the inverse of `extract_bits`.
    ///
    /// Assumes each `bits[i]` contains only 0/1 values;
    /// returns Σ `bits[i] * (1 << i)` per lane.
    ///
    /// `bits` must be non-empty and have at most 30 elements (so the result fits in M31 without
    /// wrapping — 31 all-one bits would produce 2^31 - 1 = M31::P ≡ 0).
    pub fn combine_bits(context: &mut Context<impl IValue>, bits: &[Simd]) -> Simd {
        let mut iter = bits.iter().rev();
        let mut res = iter.next().unwrap().clone();
        let two = M31Wrapper::new_unsafe(context.constant(2.into()));
        for bit in iter {
            res = Simd::scalar_mul(context, &res, &two);
            res = Simd::add(context, &res, bit);
        }
        res
    }

    /// Marks the variables in the Simd as "maybe unused". This is intended for cases
    /// where we create a Simd but we will only use some of its elements.
    pub fn mark_partly_used(context: &mut Context<impl IValue>, simd: &Simd) {
        for chunk in &simd.data {
            context.mark_as_maybe_unused(chunk);
        }
    }
}

/// Returns a (constant) [Var] with the first `n` coordinates set to 1, and the rest to 0.
///
/// `n` must be between 1 and 3.
fn first_ones(context: &mut Context<impl IValue>, n: usize) -> Var {
    match n {
        1 => context.constant(qm31_from_u32s(1, 0, 0, 0)),
        2 => context.constant(qm31_from_u32s(1, 1, 0, 0)),
        3 => context.constant(qm31_from_u32s(1, 1, 1, 0)),
        _ => panic!("Unsupported number of ones: {n}"),
    }
}
