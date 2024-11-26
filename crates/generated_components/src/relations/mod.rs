#![allow(non_camel_case_types)]
use stwo_prover::relation;

// TODO(Ohad): generate from json and rename these.
relation!(MemoryAddressToId, 2);
relation!(MemoryIdToBig, 29);
relation!(opcodes, 3);
relation!(RangeCheck_N_2_bits_4_3, 2);
relation!(RangeCheck_N_3_bits_7_2_5, 3);
relation!(VerifyInstruction, 29);
relation!(NarrowFib_num_steps_20, 4);
