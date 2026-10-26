use cairo_air::air::PubMemoryValue;

use super::output_hash_from_output_cells;
use crate::statement::{N_OUTPUTS, N_WORDS_PER_OUTPUT_CELL};

/// Builds an output cell whose value has `low` as its low `N_WORDS_PER_OUTPUT_CELL` (128-bit)
/// words and `high_word` as the first of the remaining words (used to exercise the 128-bit check).
fn cell(id: u32, low: [u32; N_WORDS_PER_OUTPUT_CELL], high_word: u32) -> PubMemoryValue {
    let mut value = [0u32; 8];
    value[..N_WORDS_PER_OUTPUT_CELL].copy_from_slice(&low);
    value[N_WORDS_PER_OUTPUT_CELL] = high_word;
    (id, value)
}

#[test]
fn output_hash_packs_low_128_bits_of_each_cell() {
    let low0 = [0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444];
    let low1 = [0x5555_5555, 0x6666_6666, 0x7777_7777, 0x8888_8888];
    let hash = output_hash_from_output_cells(&[cell(0, low0, 0), cell(1, low1, 0)]).unwrap();

    // The digest is the two cells' low 128 bits concatenated little-endian, cell 0 first.
    let mut expected = [0u8; 32];
    for (i, word) in low0.iter().chain(low1.iter()).enumerate() {
        expected[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    assert_eq!(hash.0, expected);
}

#[test]
fn output_hash_accepts_full_128_bit_cells() {
    // Every bit of the low 128 bits may be set; only bits >= 128 are rejected.
    let full = [u32::MAX; N_WORDS_PER_OUTPUT_CELL];
    assert!(output_hash_from_output_cells(&[cell(0, full, 0), cell(1, full, 0)]).is_ok());
}

#[test]
fn output_hash_rejects_wrong_output_count() {
    let c = cell(0, [0; N_WORDS_PER_OUTPUT_CELL], 0);
    for count in [0, 1, N_OUTPUTS + 1] {
        let err = output_hash_from_output_cells(&vec![c; count]).unwrap_err();
        assert!(err.contains(&format!("exactly {N_OUTPUTS} output cells")), "{err}");
    }
}

#[test]
fn output_hash_rejects_cell_over_128_bits() {
    // A single bit above the low 128 bits (word index N_WORDS_PER_OUTPUT_CELL) is rejected.
    let output =
        [cell(0, [0; N_WORDS_PER_OUTPUT_CELL], 1), cell(1, [0; N_WORDS_PER_OUTPUT_CELL], 0)];
    let err = output_hash_from_output_cells(&output).unwrap_err();
    assert!(err.contains("128 bits"), "{err}");
}
