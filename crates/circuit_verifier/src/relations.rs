#![allow(non_camel_case_types)]
use stwo_constraint_framework::relation;

// The number of lookup elements computed. These are used by all relations, so the size
// should be at least the size of the largest relation.
const COMMON_LOOKUP_ELEMENTS_SIZE: usize = 128;
relation!(CommonLookupElements, COMMON_LOOKUP_ELEMENTS_SIZE);

// Relation IDs
pub const GATE_RELATION_ID: u32 = 378353459;
pub const RANGE_CHECK_16_RELATION_ID: u32 = 1008385708;
pub const VERIFY_BITWISE_XOR_4_RELATION_ID: u32 = 45448144;
pub const VERIFY_BITWISE_XOR_7_RELATION_ID: u32 = 62225763;
pub const VERIFY_BITWISE_XOR_8_RELATION_ID: u32 = 112558620;
pub const VERIFY_BITWISE_XOR_8_B_RELATION_ID: u32 = 521092554;
pub const VERIFY_BITWISE_XOR_9_RELATION_ID: u32 = 95781001;
pub const VERIFY_BITWISE_XOR_12_RELATION_ID: u32 = 648362599;
