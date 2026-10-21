use circuits::blake::{blake_g_gate, m31_to_u32, triple_xor};
use circuits::context::{Context, FinalizedContext, Var};
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{add_into, eq};
use circuits::wrappers::{M31Wrapper, U32Wrapper};
use rand_chacha::rand_core::{RngCore, SeedableRng};

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

// Blinding rows per unit of `n_padding`.
//
// Every value the proof reveals about a column is an M31 (~31 bits), so for each revealed value
// the blinding rows must add ~31 random bits to every column the component commits to: its trace
// columns and its interaction columns (each secure interaction column is 4 M31 columns). All the
// columns of a row are computed from the row's random inputs, so a row adds only as many random
// bits as its inputs have: a component needs ~`n_columns * 31 / input_bits` rows. This is a rough
// estimate.

/// 20 trace + 24 interaction columns; 3 random u32 inputs (96 bits): 44 * 31 / 96 ≈ 15.
pub const TRIPLE_XOR_BLINDING_FACTOR: usize = 15;
/// 4 trace + 12 interaction columns; 1 random M31 input (31 bits): 16 * 31 / 31 = 16.
pub const M31_TO_U32_BLINDING_FACTOR: usize = 16;
/// 52 trace + 52 interaction columns; 6 random u32 inputs (192 bits): 104 * 31 / 192 ≈ 17.
pub const BLAKE_G_GATE_BLINDING_FACTOR: usize = 17;

/// Adds ZK blinding to the circuit by adding random rows to all the witness components: qm31_ops,
/// eq, triple_xor, m31_to_u32, and blake_g_gate. The fresh input variables are yielded through
/// trivial `x + 0 = x` add gates in the qm31_ops component.
///
/// Each unit of `n_padding` blinds the values exposed by one query in the proof
/// (amount of added randomness = amount of data exposed by query = size of one trace row).
pub fn add_zk_blinding(
    context: &mut FinalizedContext<impl IValue>,
    seed_bytes: [u8; 32],
    n_padding: usize,
) {
    let context = &mut context.context;
    let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed_bytes);
    for _ in 0..n_padding {
        qm31_zk_blinding(context, &mut rng);
        eq_zk_blinding(context, &mut rng);
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
