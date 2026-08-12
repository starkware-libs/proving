use std::collections::{BTreeMap, HashMap, HashSet};

use cairo_vm::air_public_input::MemorySegmentAddresses as VMMemorySegmentAddresses;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::types::relocatable::MaybeRelocatable;
use serde::{Deserialize, Serialize};
use stwo_cairo_common::builtins::{
    ADD_MOD_BUILTIN_MEMORY_CELLS, BITWISE_BUILTIN_MEMORY_CELLS, EC_OP_BUILTIN_MEMORY_CELLS,
    ECDSA_MEMORY_CELLS, KECCAK_MEMORY_CELLS, MUL_MOD_BUILTIN_MEMORY_CELLS, OUTPUT_MEMORY_CELLS,
    PEDERSEN_BUILTIN_MEMORY_CELLS, POSEIDON_BUILTIN_MEMORY_CELLS,
    RANGE_CHECK_96_BUILTIN_MEMORY_CELLS, RANGE_CHECK_BUILTIN_MEMORY_CELLS,
};
use tracing::{Level, info, span};

use crate::memory::Memory;

// Minimal builtins instances per segment, chosen to fit SIMD requirements.
pub const MIN_SEGMENT_SIZE: usize = 16;

/// This is a copy of [`cairo_vm::air_public_input::MemorySegmentAddresses`] struct from cairo_vm.
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct MemorySegmentAddresses {
    pub begin_addr: usize,
    pub stop_ptr: usize,
}

impl From<VMMemorySegmentAddresses> for MemorySegmentAddresses {
    fn from(addresses: VMMemorySegmentAddresses) -> Self {
        MemorySegmentAddresses { begin_addr: addresses.begin_addr, stop_ptr: addresses.stop_ptr }
    }
}

// TODO(ohadn): change field types in MemorySegmentAddresses to match address type.
/// This struct holds the builtins used in a Cairo program.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BuiltinSegments {
    pub add_mod_builtin: Option<MemorySegmentAddresses>,
    pub bitwise_builtin: Option<MemorySegmentAddresses>,
    pub output: Option<MemorySegmentAddresses>,
    pub mul_mod_builtin: Option<MemorySegmentAddresses>,
    pub pedersen_builtin: Option<MemorySegmentAddresses>,
    pub poseidon_builtin: Option<MemorySegmentAddresses>,
    pub range_check96_builtin: Option<MemorySegmentAddresses>,
    pub range_check_builtin: Option<MemorySegmentAddresses>,
    pub ec_op_builtin: Option<MemorySegmentAddresses>,
}

impl BuiltinSegments {
    /// Returns the number of instances for each builtin.
    pub fn get_counts(&self) -> HashMap<BuiltinName, usize> {
        let mut counts = HashMap::new();
        let mut insert_builtin = |builtin_name, segment: &Option<_>, n_cells_per_instance| {
            counts.insert(builtin_name, count_instances(segment, n_cells_per_instance));
        };

        insert_builtin(BuiltinName::add_mod, &self.add_mod_builtin, ADD_MOD_BUILTIN_MEMORY_CELLS);
        insert_builtin(BuiltinName::bitwise, &self.bitwise_builtin, BITWISE_BUILTIN_MEMORY_CELLS);
        insert_builtin(BuiltinName::mul_mod, &self.mul_mod_builtin, MUL_MOD_BUILTIN_MEMORY_CELLS);
        insert_builtin(
            BuiltinName::pedersen,
            &self.pedersen_builtin,
            PEDERSEN_BUILTIN_MEMORY_CELLS,
        );
        insert_builtin(
            BuiltinName::poseidon,
            &self.poseidon_builtin,
            POSEIDON_BUILTIN_MEMORY_CELLS,
        );
        insert_builtin(
            BuiltinName::range_check,
            &self.range_check_builtin,
            RANGE_CHECK_BUILTIN_MEMORY_CELLS,
        );
        insert_builtin(
            BuiltinName::range_check96,
            &self.range_check96_builtin,
            RANGE_CHECK_96_BUILTIN_MEMORY_CELLS,
        );
        insert_builtin(BuiltinName::ec_op, &self.ec_op_builtin, EC_OP_BUILTIN_MEMORY_CELLS);
        insert_builtin(BuiltinName::output, &self.output, OUTPUT_MEMORY_CELLS);
        counts
    }

    /// Counts the number of unique inputs to the poseidon and pedersen builtins.
    pub fn count_unique_aggregator_inputs(&self, memory: &Memory) -> HashMap<BuiltinName, usize> {
        let _span = span!(Level::INFO, "count_unique_aggregator_inputs").entered();
        let mut counts = HashMap::new();

        let unique =
            count_unique_instances::<PEDERSEN_BUILTIN_MEMORY_CELLS>(memory, &self.pedersen_builtin);
        counts.insert(BuiltinName::pedersen, unique);

        let unique =
            count_unique_instances::<POSEIDON_BUILTIN_MEMORY_CELLS>(memory, &self.poseidon_builtin);
        counts.insert(BuiltinName::poseidon, unique);

        counts
    }

    /// Returns the memory segment addresses for the given builtin name.
    /// These names are not the same as the BuiltinName enum defined by the vm, as they include
    /// "pedersen_builtin_narrow_windows", which is an alternative implementation for the pedersen
    /// builtin.
    pub fn get_segment_by_name(&self, name: &str) -> Option<MemorySegmentAddresses> {
        match name {
            "add_mod_builtin" => self.add_mod_builtin,
            "bitwise_builtin" => self.bitwise_builtin,
            "output" => self.output,
            "mul_mod_builtin" => self.mul_mod_builtin,
            "pedersen_builtin" => self.pedersen_builtin,
            "pedersen_builtin_narrow_windows" => self.pedersen_builtin,
            "poseidon_builtin" => self.poseidon_builtin,
            "range_check96_builtin" => self.range_check96_builtin,
            "range_check_builtin" => self.range_check_builtin,
            "ec_op_builtin" => self.ec_op_builtin,
            _ => panic!("Invalid builtin name: {name}"),
        }
    }

    // Pads the relocatable builtin segments output by the VM to match the size required by Stwo.
    // Assumes and verifies that the segments contain no holes and that their length is divisible by
    // the number of cells per instance.
    pub fn pad_relocatble_builtin_segments(
        relocatable_memory: &mut [Vec<Option<MaybeRelocatable>>],
        builtin_segments: &BTreeMap<usize, BuiltinName>,
    ) {
        let _span = span!(Level::INFO, "pad_relocatble_builtin_segments").entered();
        for (segment_index, builtin_name) in builtin_segments {
            if *builtin_name == BuiltinName::segment_arena || *builtin_name == BuiltinName::output {
                continue;
            };
            let current_builtin_segment = &mut relocatable_memory[*segment_index];

            let original_segment_len = current_builtin_segment.len();

            if original_segment_len == 0 {
                // Do not pad empty segments.
                continue;
            }

            let cells_per_instance = match builtin_name {
                BuiltinName::add_mod => ADD_MOD_BUILTIN_MEMORY_CELLS,
                BuiltinName::bitwise => BITWISE_BUILTIN_MEMORY_CELLS,
                BuiltinName::ec_op => EC_OP_BUILTIN_MEMORY_CELLS,
                BuiltinName::ecdsa => ECDSA_MEMORY_CELLS,
                BuiltinName::keccak => KECCAK_MEMORY_CELLS,
                BuiltinName::mul_mod => MUL_MOD_BUILTIN_MEMORY_CELLS,
                BuiltinName::pedersen => PEDERSEN_BUILTIN_MEMORY_CELLS,
                BuiltinName::poseidon => POSEIDON_BUILTIN_MEMORY_CELLS,
                BuiltinName::range_check96 => RANGE_CHECK_96_BUILTIN_MEMORY_CELLS,
                BuiltinName::range_check => RANGE_CHECK_BUILTIN_MEMORY_CELLS,
                _ => panic!("Invalid builtin name"),
            };
            assert!(
                original_segment_len.is_multiple_of(cells_per_instance),
                "builtin segment: {builtin_name} size is {original_segment_len}, which is not \
                 divisible by {cells_per_instance}"
            );

            if !current_builtin_segment.iter().all(|x| x.is_some()) {
                panic!(
                    "Builtins segments '{builtin_name}' at segment index: {segment_index}, \
                     contains a hole."
                );
            }

            // Builtin segment size is extended to the next power of two instances.
            let new_segment_size = original_segment_len
                .div_ceil(cells_per_instance)
                .next_power_of_two()
                .max(MIN_SEGMENT_SIZE)
                * cells_per_instance;
            current_builtin_segment.resize(new_segment_size, None);

            // Fill the segment extension with copies of the last instance.
            let last_instance_start = original_segment_len - cells_per_instance;
            for i in original_segment_len..new_segment_size {
                current_builtin_segment[i] =
                    current_builtin_segment[last_instance_start + (i % cells_per_instance)].clone();
            }

            info!(
                "Padded builtin segment '{}' from {} to {} instances.",
                builtin_name,
                original_segment_len / cells_per_instance,
                new_segment_size / cells_per_instance
            );
        }
    }
}

/// The minimum number of instances in a builtin segment supported by the prover. This must be a
/// power of 2.
const MIN_N_INSTANCES_IN_BUILTIN_SEGMENT: u32 = 16;

// Assert MIN_N_INSTANCES_IN_BUILTIN_SEGMENT is a power of 2.
const _: () = assert!(MIN_N_INSTANCES_IN_BUILTIN_SEGMENT.is_power_of_two());

/// Return the size of a memory segment.
fn get_memory_segment_size(segment: &MemorySegmentAddresses) -> usize {
    segment.stop_ptr - segment.begin_addr
}

/// Counts the number of instances of a builtin inside `segment`, given the number of memory cells
/// per instance.
fn count_instances(segment: &Option<MemorySegmentAddresses>, n_cells_per_instance: usize) -> usize {
    segment.as_ref().map(get_memory_segment_size).unwrap_or(0) / n_cells_per_instance
}

/// Counts the distinct tuples of memory ids over the `N`-cell instances of `segment`.
fn count_unique_instances<const N: usize>(
    memory: &Memory,
    segment: &Option<MemorySegmentAddresses>,
) -> usize {
    let n_instances = count_instances(segment, N);
    let Some(segment) = segment else {
        return 0;
    };
    let begin = segment.begin_addr as u32;
    let unique: HashSet<[u32; N]> = (0..n_instances as u32)
        .map(|i| {
            let addr = begin + i * N as u32;
            std::array::from_fn(|cell| memory.get_raw_id(addr + cell as u32))
        })
        .collect();
    unique.len()
}

#[cfg(test)]
mod test_builtin_segments {
    use super::*;
    use crate::builtins::BITWISE_BUILTIN_MEMORY_CELLS;
    use crate::memory::{MemoryBuilder, MemoryConfig, MemoryEntry, P_MIN_1};

    const A: [u32; 8] = [1, 0, 0, 0, 0, 0, 0, 0];
    const B: [u32; 8] = [2, 0, 0, 0, 0, 0, 0, 0];
    const C: [u32; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
    const BIG: [u32; 8] = P_MIN_1;

    /// 4 pedersen instances of 3 cells, of which #0 and #1 are equal: 3 unique inputs.
    const PEDERSEN_CELLS: [[u32; 8]; 12] = [A, B, C, A, B, C, A, B, BIG, C, A, B];
    /// 2 poseidon instances of 6 cells, both equal: 1 unique input.
    const POSEIDON_CELLS: [[u32; 8]; 12] = [A, B, C, BIG, B, A, A, B, C, BIG, B, A];

    const FIRST_ADDR: usize = 1;

    /// Builds a memory holding `values` at consecutive addresses starting from `begin_addr`.
    fn memory_from_values(begin_addr: u32, values: &[[u32; 8]]) -> Memory {
        let entries = values
            .iter()
            .enumerate()
            .map(|(i, &value)| MemoryEntry { address: (begin_addr + i as u32) as u64, value });
        MemoryBuilder::from_iter(MemoryConfig::default(), entries).build().0
    }

    fn segment_of(begin_addr: usize, n_cells: usize) -> Option<MemorySegmentAddresses> {
        Some(MemorySegmentAddresses { begin_addr, stop_ptr: begin_addr + n_cells })
    }

    #[test]
    fn test_count_unique_aggregator_inputs() {
        let poseidon_begin = FIRST_ADDR + PEDERSEN_CELLS.len();
        let memory = memory_from_values(
            FIRST_ADDR as u32,
            &[PEDERSEN_CELLS.as_slice(), POSEIDON_CELLS.as_slice()].concat(),
        );
        let segments = BuiltinSegments {
            pedersen_builtin: segment_of(FIRST_ADDR, PEDERSEN_CELLS.len()),
            poseidon_builtin: segment_of(poseidon_begin, POSEIDON_CELLS.len()),
            ..Default::default()
        };

        let unique_inputs = segments.count_unique_aggregator_inputs(&memory);

        assert_eq!(
            unique_inputs,
            HashMap::from([(BuiltinName::pedersen, 3), (BuiltinName::poseidon, 1)])
        );
    }

    #[test]
    fn test_count_unique_aggregator_inputs_of_absent_builtins() {
        let pedersen_memory = memory_from_values(FIRST_ADDR as u32, &PEDERSEN_CELLS);
        let poseidon_memory = memory_from_values(FIRST_ADDR as u32, &POSEIDON_CELLS);
        let runs = [
            (
                &pedersen_memory,
                BuiltinSegments {
                    pedersen_builtin: segment_of(FIRST_ADDR, PEDERSEN_CELLS.len()),
                    ..Default::default()
                },
                [(BuiltinName::pedersen, 3), (BuiltinName::poseidon, 0)],
            ),
            (
                &poseidon_memory,
                BuiltinSegments {
                    poseidon_builtin: segment_of(FIRST_ADDR, POSEIDON_CELLS.len()),
                    ..Default::default()
                },
                [(BuiltinName::pedersen, 0), (BuiltinName::poseidon, 1)],
            ),
            (
                &pedersen_memory,
                BuiltinSegments {
                    bitwise_builtin: segment_of(FIRST_ADDR, BITWISE_BUILTIN_MEMORY_CELLS),
                    ..Default::default()
                },
                [(BuiltinName::pedersen, 0), (BuiltinName::poseidon, 0)],
            ),
        ];

        for (memory, segments, expected_counts) in runs {
            assert_eq!(
                segments.count_unique_aggregator_inputs(memory),
                HashMap::from(expected_counts)
            );
        }
    }

    #[test]
    fn test_pad_relocatble_builtin_segments() {
        let bitwise_instance = [787, 365, 257, 895, 638, 55, 102, 3, 4, 7];
        let segment0 = bitwise_instance
            .iter()
            .map(|&x| Some(MaybeRelocatable::Int(x.into())))
            .collect::<Vec<_>>();
        let segment0_extended =
            segment0.clone().into_iter().cycle().take(18 * 5).collect::<Vec<_>>();

        let ec_op_instance = [1, 2, 3, 4, 5, 6, 7];
        let segment1 = ec_op_instance
            .iter()
            .map(|&x| Some(MaybeRelocatable::Int(x.into())))
            .collect::<Vec<_>>();

        let non_builtin_segment = vec![Some(MaybeRelocatable::Int(5.into())); 10];

        let mut relocatable_memory = [segment0_extended, segment1.clone(), non_builtin_segment];

        let builtin_segments = BTreeMap::from([(0, BuiltinName::bitwise), (1, BuiltinName::ec_op)]);

        BuiltinSegments::pad_relocatble_builtin_segments(
            &mut relocatable_memory,
            &builtin_segments,
        );

        assert!(relocatable_memory[0].len() == 32 * BITWISE_BUILTIN_MEMORY_CELLS);
        assert!(relocatable_memory[1].len() == 16 * EC_OP_BUILTIN_MEMORY_CELLS);
        assert!(relocatable_memory[2].len() == 10);

        // Check that the values in the extended segment are identical to the original segment.
        let last_bitwise_instance = &segment0[5..10];
        for (value, expected_value) in
            relocatable_memory[0].iter().skip(17 * 5).zip(last_bitwise_instance.iter().cycle())
        {
            assert_eq!(value, expected_value);
        }
        for (value, expected_value) in relocatable_memory[1].iter().zip(segment1.iter().cycle()) {
            assert_eq!(value, expected_value);
        }
    }

    #[test]
    #[should_panic(
        expected = "builtin segment: mul_mod_builtin size is 864, which is not divisible by 7"
    )]
    fn test_builtin_segment_invalid_size() {
        let mul_mod_segment =
            vec![Some(MaybeRelocatable::Int(7.into())); MUL_MOD_BUILTIN_MEMORY_CELLS * 123 + 3];
        let mut relocatable_memory = [mul_mod_segment];
        let builtin_segments = BTreeMap::from([(0, BuiltinName::mul_mod)]);

        BuiltinSegments::pad_relocatble_builtin_segments(
            &mut relocatable_memory,
            &builtin_segments,
        );
    }

    #[test]
    #[should_panic(
        expected = "Builtins segments 'bitwise_builtin' at segment index: 0, contains a hole."
    )]
    fn test_builtin_segment_contains_hole() {
        let mut segment0 = vec![Some(MaybeRelocatable::Int(1.into())); 80];
        segment0[62] = None;

        let mut relocatable_memory = vec![segment0];
        let builtin_segments = BTreeMap::from([(0, BuiltinName::bitwise)]);

        BuiltinSegments::pad_relocatble_builtin_segments(
            &mut relocatable_memory,
            &builtin_segments,
        );
    }
}
