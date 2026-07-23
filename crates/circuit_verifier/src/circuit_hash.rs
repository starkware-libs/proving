use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue, blake2s_u32s};
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::utils::le_u32s_from_bytes;
use circuits::wrappers::U32Wrapper;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use itertools::{Itertools, chain};
use stwo::core::fields::qm31::QM31;

use crate::circuit_components::{COMPONENT_NAMES, N_COMPONENTS};

#[cfg(test)]
#[path = "circuit_hash_test.rs"]
mod test;

/// Number of bytes in `config_words`: one for `log_blowup_factor` plus one per component log size.
const CONFIG_N_BYTES: usize = 1 + N_COMPONENTS;
// `config_bytes` is packed 4 per u32 with no padding, so its length must be a multiple of 4.
const _: () =
    assert!(CONFIG_N_BYTES.is_multiple_of(4), "CONFIG_N_BYTES must pack into whole u32 words");

/// Packs `log_blowup_factor` (byte 0) then each component's preprocessed log size (in
/// [`COMPONENT_NAMES`] order), one byte each, into little-endian u32 words.
pub fn config_words(
    log_blowup_factor: u32,
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
) -> [u32; CONFIG_N_BYTES / 4] {
    let log_blowup_factor =
        u8::try_from(log_blowup_factor).expect("log_blowup_factor does not fit in a byte");
    let config_bytes: [u8; CONFIG_N_BYTES] = std::array::from_fn(|i| {
        if i == 0 {
            return log_blowup_factor;
        }
        let size = *component_log_sizes.get(COMPONENT_NAMES[i - 1]).unwrap();
        u8::try_from(size).unwrap_or_else(|_| panic!("log_size {size} does not fit in a byte"))
    });
    le_u32s_from_bytes(config_bytes)
}

/// Computes the circuit hash: `blake2s(log_blowup_factor || component_log_sizes ||
/// preprocessed_root)`
pub fn compute_circuit_hash<Value: IValue>(
    context: &mut Context<Value>,
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: &HashValue<Var>,
) -> HashValue<Var> {
    // Materialize the packed config words as circuit constants.
    let config_words: Vec<U32Wrapper<Var>> = config_words(log_blowup_factor, component_log_sizes)
        .into_iter()
        .map(|word| U32Wrapper::new_unsafe(context.constant(QM31::pack_u32(word))))
        .collect();
    let config_and_root =
        chain!(config_words.iter().copied(), preprocessed_root.iter().copied()).collect_vec();
    let n_bytes = 4 * (config_words.len() + BLAKE2S_DIGEST_N_WORDS);
    blake2s_u32s(context, config_and_root, n_bytes)
}
