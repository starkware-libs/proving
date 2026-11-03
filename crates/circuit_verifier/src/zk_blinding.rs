use circuits::blake::{blake_g_gate, m31_to_u32, triple_xor};
use circuits::context::{Context, FinalizedContext, Var};
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{add_into, eq};
use circuits::wrappers::{M31Wrapper, U32Wrapper};
use rand_chacha::rand_core::{RngCore, SeedableRng};
use stwo::core::fields::m31::MODULUS_BITS;

use crate::components;

#[cfg(test)]
#[path = "zk_blinding_test.rs"]
mod test;

/// Adds random values to the qm31_ops component via trivial.
/// the padding is
/// x + 0 = x
/// 0 + y = y
/// 0 + z = z
fn qm31_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let zero = context.zero();
    let value_x = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let x = context.new_var(IValue::from_qm31(value_x));
    add_into(context, x, zero, x);
    let value_y = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let y = context.new_var(IValue::from_qm31(value_y));
    add_into(context, zero, y, y);
    let value_z = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let z = context.new_var(IValue::from_qm31(value_z));
    add_into(context, z, zero, z);
}

/// Adds a random value to the eq component via a trivial `x == x` equality, blinding the eq trace.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
fn eq_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let zero = context.zero();
    let value = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let x = context.new_var(IValue::from_qm31(value));
    // yield x.
    add_into(context, x, zero, x);
    eq(context, x, x);
}

/// Creates a fresh variable holding a random `u32` value, encoded as `(low_u16, high_u16, 0, 0)`,
/// and yields it via a trivial `x + 0 = x` add gate so it can be used as a gate input.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
///
/// Safety: this function is unsafe as it creates a new unconstrained variable (in particular, it
/// is not guaranteed to hold a value of the form (low_u16, high_u16, 0, 0)).
fn random_u32_var(context: &mut Context<impl IValue>, rng: &mut impl RngCore) -> Var {
    let zero = context.zero();
    let x = context.new_var(*IValue::pack_u32(rng.next_u32()).raw());
    add_into(context, x, zero, x);
    x
}

/// Creates a fresh variable holding a random `M31` value, encoded as `(x, 0, 0, 0)`, and yields it
/// via a trivial `x + 0 = x` add gate so it can be used as a gate input.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
fn random_m31_var(context: &mut Context<impl IValue>, rng: &mut impl RngCore) -> M31Wrapper<Var> {
    let zero = context.zero();
    let x = context.new_var(IValue::from_qm31(qm31_from_u32s(rng.next_u32(), 0, 0, 0)));
    add_into(context, x, zero, x);
    M31Wrapper::new_unsafe(x)
}

/// Adds a random row to the triple_xor component, blinding its trace.
fn triple_xor_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    // Safety:
    // - none of the inputs to triple_xor is used anywhere else other than the gates x + 0 = x that
    //   yield them.
    // - the output of triple_xor is unused.
    // Note that the constraints of triple_xor actually enforce that its inputs are of the form
    // (low, high, 0, 0) with low, high in `[0, 2^16)` so the use of [`U32Wrapper::new_unsafe`] is
    // fine.
    let a = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    let b = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    let c = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    triple_xor(context, a, b, c);
}

/// Adds a random row to the m31_to_u32 component, blinding its trace.
fn m31_to_u32_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let input = random_m31_var(context, rng);
    m31_to_u32(context, input);
}

/// Adds a random row to the blake_g_gate component, blinding its trace.
fn blake_g_gate_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    // Safety: the variables returned by [`random_u32_var`] are not guaranteed to be of the form
    // (low, high, 0, 0) with low, high in `[0, 2^16)`. The blake_g_gate constraints do not enforce
    // all of its 6 inputs to have this form either. However, the usage in this function is fine
    // because:
    // - none of the inputs to blake_g_gate is used anywhere else other than the gates x + 0 = x
    //   that yield them.
    // - the output of blake_g_gate is unused.
    let a = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    let b = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    let c = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    let d = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    let f0 = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    let f1 = U32Wrapper::new_unsafe(random_u32_var(context, rng));
    blake_g_gate(context, a, b, c, d, f0, f1);
}

/// The number of random bits in a random QM31.
const QM31_N_RANDOM_BITS: usize = 4 * MODULUS_BITS as usize;
/// `qm31_zk_blinding` draws 3 random QM31s per call.
const QM31_OPS_N_RANDOM_BITS: usize = 3 * QM31_N_RANDOM_BITS;
/// `eq_zk_blinding` draws 1 random QM31 per call.
const EQ_N_RANDOM_BITS: usize = QM31_N_RANDOM_BITS;
/// `triple_xor_zk_blinding` draws 3 random u32s per call.
const TRIPLE_XOR_N_RANDOM_BITS: usize = 3 * u32::BITS as usize;
/// `m31_to_u32_zk_blinding` draws 1 random M31 per call.
const M31_TO_U32_N_RANDOM_BITS: usize = MODULUS_BITS as usize;
/// `blake_g_gate_zk_blinding` draws 6 random u32s per call.
const BLAKE_G_GATE_N_RANDOM_BITS: usize = 6 * u32::BITS as usize;

// For every component (except lookup tables), the number of calls to the component's blinding
// function, per unit of `n_padding`.
//
// Every value the proof reveals about a column is an M31 (~31 bits), so for each revealed value
// the blinding rows must add ~31 random bits to every column the component commits to: its trace
// columns and its interaction columns (each secure interaction column is 4 M31 columns). All the
// columns of a row are computed from the row's random inputs, so a row adds only as many random
// bits as its inputs have: a component needs ~`n_columns * 31 / n_random_bits` calls of its
// `*_zk_blinding` function, where `n_random_bits` is the number of random bits the function
// draws. This is a rough estimate.
//
// The lookup-table components (verify_bitwise_xor_*, range_check_16) are preprocessed with a fixed
// number of rows, so no rows can be added to them. Their only witness columns are multiplicities,
// which are blinded by the random lookups that the blinding rows of the other components make.

/// The number of `*_zk_blinding` calls per unit of `n_padding` for a component with `n_columns`
/// columns whose `*_zk_blinding` function draws `n_random_bits` random bits per call.
const fn blinding_factor(n_columns: usize, n_random_bits: usize) -> usize {
    (n_columns * MODULUS_BITS as usize).div_ceil(n_random_bits)
}

const QM31_OPS_BLINDING_FACTOR: usize = blinding_factor(
    components::qm_31_ops::N_TRACE_COLUMNS + components::qm_31_ops::N_INTERACTION_COLUMNS,
    QM31_OPS_N_RANDOM_BITS,
);
const EQ_BLINDING_FACTOR: usize = blinding_factor(
    components::eq::N_TRACE_COLUMNS + components::eq::N_INTERACTION_COLUMNS,
    EQ_N_RANDOM_BITS,
);
const TRIPLE_XOR_BLINDING_FACTOR: usize = blinding_factor(
    components::triple_xor::N_TRACE_COLUMNS + components::triple_xor::N_INTERACTION_COLUMNS,
    TRIPLE_XOR_N_RANDOM_BITS,
);
const M31_TO_U32_BLINDING_FACTOR: usize = blinding_factor(
    components::m_31_to_u_32::N_TRACE_COLUMNS + components::m_31_to_u_32::N_INTERACTION_COLUMNS,
    M31_TO_U32_N_RANDOM_BITS,
);
const BLAKE_G_GATE_BLINDING_FACTOR: usize = blinding_factor(
    components::blake_g_gate::N_TRACE_COLUMNS + components::blake_g_gate::N_INTERACTION_COLUMNS,
    BLAKE_G_GATE_N_RANDOM_BITS,
);

/// Adds ZK blinding to the circuit by adding random rows to all the witness components: qm31_ops,
/// eq, triple_xor, m31_to_u32, and blake_g_gate. The fresh input variables are yielded through
/// trivial `x + 0 = x` add gates in the qm31_ops component.
///
/// Each unit of `n_padding` calls each component's `*_zk_blinding` function `*_BLINDING_FACTOR`
/// times.
pub fn add_zk_blinding(
    context: &mut FinalizedContext<impl IValue>,
    seed_bytes: [u8; 32],
    n_padding: usize,
) {
    let context = &mut context.context;
    let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed_bytes);
    for _ in 0..n_padding {
        for _ in 0..QM31_OPS_BLINDING_FACTOR {
            qm31_zk_blinding(context, &mut rng);
        }
        for _ in 0..EQ_BLINDING_FACTOR {
            eq_zk_blinding(context, &mut rng);
        }
        for _ in 0..TRIPLE_XOR_BLINDING_FACTOR {
            triple_xor_zk_blinding(context, &mut rng);
        }
        for _ in 0..M31_TO_U32_BLINDING_FACTOR {
            m31_to_u32_zk_blinding(context, &mut rng);
        }
        for _ in 0..BLAKE_G_GATE_BLINDING_FACTOR {
            blake_g_gate_zk_blinding(context, &mut rng);
        }
    }
}
