use std::cell::Ref;

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirFnStat, LeanCompare, NonComponentStat, PaddingType, TraceType,
    UseOrYield,
};
use compiled_casm_air::public_params::PublicParam;
use compiled_casm_air::utils::{
    JSONS_BUILTINS_DIR, JSONS_INLINE_DIR, JSONS_LOOKUPS_DIR, JSONS_OPCODES_DIR,
    REGISTRY_PROPERTIES_FILE_NAME, SAMPLE_EVALUATIONS_FILE_NAME,
};
use eval_air_fn_constraints::create_sample_evaluation;
use indexmap::IndexMap;
use stwo_cairo_common::prover_types::cpu::PRIME;

// Builtins
use super::builtins::bitwise::*;
use super::builtins::modulo::add_mod::*;
use super::builtins::modulo::mul_mod::*;
use super::builtins::pedersen::pedersen_builtin::*;
use super::builtins::poseidon::poseidon_builtin::*;
use super::builtins::range_check::*;
// Opcodes
use super::opcodes::add_ap_opcode::*;
use super::opcodes::add_opcode::*;
use super::opcodes::assert_eq_opcode::*;
use super::opcodes::blake::blake_compress_opcode::*;
use super::opcodes::call_opcode::*;
use super::opcodes::generic_opcode::generic_opcode::*;
use super::opcodes::jnz_opcode::*;
use super::opcodes::jump_opcode::*;
use super::opcodes::mul_opcode::*;
use super::opcodes::qm31::qm31_add_mul_opcode::*;
use super::opcodes::ret_opcode::*;
use crate::core::air_fn_registry::*;
use crate::core::felt252_id_memory::memory::*;
use crate::utils::test_utils::*;

const TRACE_COLUMNS_PER_LOGUP: usize = 2;
// Assumes blowup factor at most 4 and that tables are not splitted (as in the memory address to
// id).
const MAX_ROWS_PER_COMPONENT: usize = 2_usize.pow(28);

// The casm registry should contain all the builtins and opcodes
// used by Stwo for the casm vm.
// Note that other components used by an opcode or a builtin will be added to the registry
// automatically.

#[test]
fn test_casm_registry() {
    let mut reg = AirFnRegistry::new_empty();

    // Add builtins

    reg.add_entry(&BitwiseBuiltin::default());
    reg.add_entry(&RangeCheckBuiltin {
        bits: 128,
        memory: Felt252IdMemory::default(),
        segment_start: PublicParam::RangeCheckBuiltinSegmentStart,
    });
    reg.add_entry(&RangeCheckBuiltin {
        bits: 96,
        memory: Felt252IdMemory::default(),
        segment_start: PublicParam::RangeCheck96BuiltinSegmentStart,
    });
    reg.add_entry(&AddModBuiltin::default());
    reg.add_entry(&MulModBuiltin::default());
    reg.add_entry(&PoseidonBuiltin::default());
    reg.add_entry(&PedersenBuiltin::default());

    // Add opcodes

    // Generic opcode
    reg.add_entry(&GenericOpcode::default());
    // AddAp opcode
    reg.add_entry(&AddApOpcode {
        memory: Felt252IdMemory::default(),
    });
    // Add opcode
    reg.add_entry(&AddOpcode {
        small: true,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&AddOpcode {
        small: false,
        memory: Felt252IdMemory::default(),
    });
    // AssertEq opcode
    reg.add_entry(&AssertEqOpcode {
        double_deref: false,
        imm: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&AssertEqOpcode {
        double_deref: true,
        imm: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&AssertEqOpcode {
        double_deref: false,
        imm: true,
        memory: Felt252IdMemory::default(),
    });
    //  Call opcode
    reg.add_entry(&CallOpcode {
        rel_imm: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&CallOpcode {
        rel_imm: true,
        memory: Felt252IdMemory::default(),
    });
    // Jnz opcode
    reg.add_entry(&JnzOpcode {
        taken: true,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&JnzOpcode {
        taken: false,
        memory: Felt252IdMemory::default(),
    });
    // Jump opcode
    reg.add_entry(&JumpOpcode {
        rel: true,
        imm: true,
        double_deref: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&JumpOpcode {
        rel: true,
        imm: false,
        double_deref: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&JumpOpcode {
        rel: false,
        imm: false,
        double_deref: true,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&JumpOpcode {
        rel: false,
        imm: false,
        double_deref: false,
        memory: Felt252IdMemory::default(),
    });
    // Mul opcode
    reg.add_entry(&MulOpcode {
        small: true,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&MulOpcode {
        small: false,
        memory: Felt252IdMemory::default(),
    });
    // Ret opcode
    reg.add_entry(&RetOpcode::default());
    // QM31AddMul opcode
    reg.add_entry(&QM31AddMulOpcode {
        memory: Felt252IdMemory::default(),
    });
    // Blake opcode
    reg.add_entry(&BlakeCompressOpcode::default());

    let mut constraints = IndexMap::new();
    for (name, entry) in reg.air_fns.borrow().iter() {
        let air_body_constraints = entry.air_body.get_constraints();
        constraints.insert(
            name.clone(),
            LeanCompare {
                state_names: entry.state.get_state_names(),
                intermediates: air_body_constraints.intermediates,
                constraints: air_body_constraints.constraints,
                lookups: air_body_constraints.lookups,
            },
        );
    }
    compare_json(
        &constraints,
        &"../compiled_casm_air/src/constraints.json".to_string(),
    );

    // Compile the registry, check the compiled entries jsons and collect the statistics.
    let compiled_reg = reg.compile();
    let mut stat = IndexMap::new();
    let mut const_tables = IndexMap::new();
    let mut sample_evaluations = IndexMap::new();

    for (name, compiled_entry) in compiled_reg.iter() {
        let fns = reg.air_fns.borrow();
        let entry = fns.get(name).unwrap();
        let dir = match entry.trace_type {
            TraceType::Opcode => JSONS_OPCODES_DIR,
            TraceType::Component | TraceType::Memory | TraceType::ChainRound => JSONS_LOOKUPS_DIR,
            TraceType::Builtin => JSONS_BUILTINS_DIR,
            TraceType::Inline => JSONS_INLINE_DIR,
            TraceType::Const => "",
        };

        // Collect preprocessed columns.
        if !compiled_entry.external_states.is_empty() {
            const_tables.insert(name, compiled_entry.external_states.clone());
        }

        // Const functions are not compiled.
        if entry.trace_type == TraceType::Const {
            continue;
        }

        // Check the compiled entry json.
        compare_json(compiled_entry, &format!("{}{}.json", dir, name));

        if entry.trace_type != TraceType::Inline {
            // We don't support sampling inline AirFns because they don't have a trace
            // of their own. This is OK because they are called by AirFns that we do support,
            // so their polynomial is tested as part of their caller.
            sample_evaluations.insert(
                name.to_string(),
                create_sample_evaluation(&compiled_reg, name),
            );

            add_entry_statistics(&fns, compiled_entry, &mut stat);
        }
    }

    compare_json(
        &const_tables,
        &"../compiled_casm_air/src/const_tables.json".to_string(),
    );

    compare_json(
        &stat,
        &format!("../compiled_casm_air/src/{REGISTRY_PROPERTIES_FILE_NAME}"),
    );

    compare_json(
        &IndexMap::from([
            ("keccak".to_string(), get_keccak_stat(&stat)),
            ("ec_op".to_string(), get_ec_op_stat(&stat)),
            ("ecdsa".to_string(), get_ecdsa_stat(&stat)),
        ]),
        &"../compiled_casm_air/src/non_components.json".to_string(),
    );

    compare_json(
        &sample_evaluations,
        &format!("../compiled_casm_air/src/{SAMPLE_EVALUATIONS_FILE_NAME}"),
    );
}

// Collects statistics on a component.
fn add_entry_statistics(
    reg: &Ref<'_, IndexMap<String, AirFnEntry>>,
    compiled_entry: &CompiledAirFn,
    stat: &mut IndexMap<String, CompiledAirFnStat>,
) {
    assert!(
        compiled_entry.r#type != TraceType::Const && compiled_entry.r#type != TraceType::Inline
    );

    let padding = compiled_entry.padding_type == PaddingType::Multiplicity
        || compiled_entry.padding_type == PaddingType::Enabler;
    let num_state_cols = compiled_entry.state_names.len();
    let num_lookup_cols: usize = compiled_entry.constraint_lookups.len();

    let total_num_trace_cols = num_state_cols
        + (TRACE_COLUMNS_PER_LOGUP * num_lookup_cols)
        + ((num_lookup_cols % 2) * TRACE_COLUMNS_PER_LOGUP)
        + (padding as usize);

    let mut use_lookup_cols: IndexMap<String, usize> = IndexMap::new();
    for (name, _) in compiled_entry
        .constraint_lookups
        .iter()
        .filter(|(_, use_or_yield)| *use_or_yield == UseOrYield::Use)
    {
        *use_lookup_cols.entry(name.clone()).or_default() += 1;
    }
    let mut yield_lookup_cols: IndexMap<String, usize> = IndexMap::new();
    for (name, _) in compiled_entry
        .constraint_lookups
        .iter()
        .filter(|(_, use_or_yield)| *use_or_yield == UseOrYield::Yield)
    {
        *yield_lookup_cols.entry(name.clone()).or_default() += 1;
    }

    // An upper bound on the number of cells added to the trace for each `AddInput`
    // to this component. Includes rows added to other lookup components called by
    // this component. Doesn't include cells from components that are always filled
    // by the prover, regardless of whether they're called or not (e.g. const tables, the memory,
    // verify instruction).
    let mut trace_cells_upper_bound = total_num_trace_cols;
    // We compute rows and uses upper bounds for a (possibly indirect) callee C by assuming that all
    // intermediate components are padded to a power of two, but ignoring the padding of C itself.
    // For example, if each row in component A calls component B 3 times, and each row in B calls C
    // 5 times, A.rows_upper_bound.get("C") will be 20, because we'll pad 3 to 4, but won't pad 5.
    // For uses_upper_bound, no need to pad the last component. For rows_upper_bound, the padding of
    // the last component is accounted for later, when computing `max_instances_rows_limit`.
    // We do that instead of using the padded count for all calls from the start because it gives
    // tighter upper bounds.
    let mut uses_upper_bound = IndexMap::new();
    let mut rows_upper_bound = IndexMap::new();

    let entry = reg.get(&compiled_entry.name).unwrap();
    let lookup_rows = entry.air_body.get_lookup_n_rows();

    // The registry is ordered such that each component appears after its callees, so all callees
    // already have their statistics computed.
    for (name, cnt) in lookup_rows.iter() {
        // We round the number of times a lookup is called up to the next power of two, since the
        // statistics apply also to the padded rows.
        let rounded_cnt = if *cnt == 0 {
            0
        } else {
            cnt.next_power_of_two()
        };
        let called_entry = reg.get(name).expect("Called entry not found in registry");
        let compiled_called_entry = called_entry.clone().compile(reg);
        let entry_stats = stat.get(name).expect("Called entry not found in registry");

        if compiled_called_entry.padding_type != PaddingType::Multiplicity {
            // Update trace cells upper bound for the current component.
            trace_cells_upper_bound += rounded_cnt * entry_stats.trace_cells_upper_bound;

            // Update rows upper bound for all lookup components.
            // The number of rows of each component is padded at the end, after adding all inputs
            // from all calls to it.
            *rows_upper_bound.entry(name.clone()).or_default() += cnt;
            entry_stats
                .rows_upper_bound
                .iter()
                .for_each(|(row_name, row_bound)| {
                    *rows_upper_bound.entry(row_name.clone()).or_default() +=
                        rounded_cnt * row_bound;
                });
        }

        // Update uses upper bound for the lowest level lookup components (that have no callees).
        if entry_stats.uses_upper_bound.is_empty() {
            assert!(
                compiled_called_entry.padding_type == PaddingType::Multiplicity,
                "If the entry doesn't use any other components, it must be padded with multiplicity."
            );
            *uses_upper_bound.entry(name.clone()).or_default() += cnt;
        } else {
            entry_stats
                .uses_upper_bound
                .iter()
                .for_each(|(use_name, use_bound)| {
                    *uses_upper_bound.entry(use_name.clone()).or_default() +=
                        rounded_cnt * *use_bound;
                });
        }
    }

    let max_instances_uses_limit = PRIME as usize / *uses_upper_bound.values().max().unwrap_or(&1);
    let max_instances_rows_limit = MAX_ROWS_PER_COMPONENT
        / rows_upper_bound
            .values()
            .max()
            .unwrap_or(&1)
            .next_power_of_two();

    stat.insert(
        compiled_entry.name.clone(),
        CompiledAirFnStat {
            trace_type: compiled_entry.r#type,
            num_state_cols,
            use_lookup_cols,
            yield_lookup_cols,
            lookup_rows,
            padding_type: compiled_entry.padding_type,
            total_num_trace_cols,
            trace_cells_upper_bound,
            uses_upper_bound,
            rows_upper_bound,
            max_instances_uses_limit,
            max_instances_rows_limit,
        },
    );
}

// Given a map of values with top level component names and their counts, for 3000 instances of a
// non-component, like keccak, and the statistics computed on all components, returns a
// `NonComponentStat` that contains the upper bounds on trace cells, uses, steps, and max number of
// instances for the given values.
fn get_non_component_stat(
    stat: &IndexMap<String, CompiledAirFnStat>,
    values: &IndexMap<String, usize>,
) -> NonComponentStat {
    let mut uses_upper_bound = IndexMap::new();
    let mut trace_cells_upper_bound = 0;
    for (name, cnt) in values.iter() {
        let cells = stat.get(name).unwrap().trace_cells_upper_bound;
        trace_cells_upper_bound += cells * cnt;

        let uses = stat.get(name).unwrap().uses_upper_bound.clone();
        for (use_name, use_bound) in uses.iter() {
            *uses_upper_bound.entry(use_name.clone()).or_default() += *use_bound * cnt;
        }
    }
    for (_, cnt) in uses_upper_bound.iter_mut() {
        *cnt /= 3000;
    }
    trace_cells_upper_bound /= 3000;
    let max_num_instances_uses = PRIME as usize / *uses_upper_bound.values().max().unwrap_or(&1);
    let steps = values
        .iter()
        .filter_map(|(k, v)| {
            if !k.contains("builtin") {
                Some(v)
            } else {
                None
            }
        })
        .sum::<usize>()
        / 3000;
    let max_num_instances_steps = 2_usize.pow(26) / steps;

    NonComponentStat {
        trace_cells_upper_bound,
        uses_upper_bound,
        steps,
        max_num_instances_uses,
        max_num_instances_steps,
    }
}

fn get_keccak_stat(stat: &IndexMap<String, CompiledAirFnStat>) -> NonComponentStat {
    let keccak_3000 = IndexMap::from([
        ("bitwise_builtin".to_string(), 4194304),
        ("mul_mod_builtin".to_string(), 0),
        ("pedersen_builtin".to_string(), 512),
        ("poseidon_builtin".to_string(), 0),
        ("range_check_builtin_bits_128".to_string(), 1048576),
        ("range_check_builtin_bits_96".to_string(), 0),
        ("add_mod_builtin".to_string(), 0),
        ("mul_opcode".to_string(), 1651001),
        ("assert_eq_opcode".to_string(), 70300),
        ("jump_opcode".to_string(), 0),
        ("jnz_opcode".to_string(), 2029),
        ("ret_opcode".to_string(), 63158),
        ("call_opcode".to_string(), 1),
        ("generic_opcode".to_string(), 0),
        ("assert_eq_opcode_double_deref".to_string(), 12676053),
        ("add_opcode_small".to_string(), 311243),
        ("jump_opcode_rel".to_string(), 0),
        ("assert_eq_opcode_imm".to_string(), 2767034),
        ("jump_opcode_double_deref".to_string(), 0),
        ("add_ap_opcode".to_string(), 50101),
        ("call_opcode_rel_imm".to_string(), 63157),
        ("mul_opcode_small".to_string(), 27099),
        ("add_opcode".to_string(), 1864576),
        ("jnz_opcode_taken".to_string(), 55592),
        ("jump_opcode_rel_imm".to_string(), 7),
        ("qm_31_add_mul_opcode".to_string(), 0),
        ("blake_compress_opcode".to_string(), 0),
    ]);
    get_non_component_stat(stat, &keccak_3000)
}

fn get_ec_op_stat(stat: &IndexMap<String, CompiledAirFnStat>) -> NonComponentStat {
    let ec_op_3001 = IndexMap::from([
        ("range_check_builtin_bits_128".to_string(), 16),
        ("mul_mod_builtin".to_string(), 0),
        ("poseidon_builtin".to_string(), 0),
        ("range_check_builtin_bits_96".to_string(), 0),
        ("pedersen_builtin".to_string(), 512),
        ("add_mod_builtin".to_string(), 0),
        ("bitwise_builtin".to_string(), 0),
        ("jump_opcode".to_string(), 0),
        ("jump_opcode_double_deref".to_string(), 0),
        ("call_opcode".to_string(), 1),
        ("ret_opcode".to_string(), 402288),
        ("qm_31_add_mul_opcode".to_string(), 0),
        ("add_ap_opcode".to_string(), 387230),
        ("blake_compress_opcode".to_string(), 0),
        ("add_opcode_small".to_string(), 889522),
        ("assert_eq_opcode_imm".to_string(), 780293),
        ("jnz_opcode_taken".to_string(), 1146967),
        ("assert_eq_opcode_double_deref".to_string(), 1934667),
        ("call_opcode_rel_imm".to_string(), 402287),
        ("jump_opcode_rel_imm".to_string(), 753258),
        ("add_opcode".to_string(), 11983562),
        ("jnz_opcode".to_string(), 759282),
        ("generic_opcode".to_string(), 0),
        ("jump_opcode_rel".to_string(), 0),
        ("mul_opcode_small".to_string(), 98),
        ("mul_opcode".to_string(), 6446149),
        ("assert_eq_opcode".to_string(), 2347074),
    ]);

    get_non_component_stat(stat, &ec_op_3001)
}

fn get_ecdsa_stat(stat: &IndexMap<String, CompiledAirFnStat>) -> NonComponentStat {
    let ecdsa_3000 = IndexMap::from([
        ("range_check_builtin_bits_128".to_string(), 16),
        ("range_check_builtin_bits_96".to_string(), 262144),
        ("poseidon_builtin".to_string(), 0),
        ("mul_mod_builtin".to_string(), 16384),
        ("bitwise_builtin".to_string(), 0),
        ("pedersen_builtin".to_string(), 512),
        ("add_mod_builtin".to_string(), 0),
        ("generic_opcode".to_string(), 0),
        ("jump_opcode_rel".to_string(), 0),
        ("jump_opcode".to_string(), 0),
        ("mul_opcode_small".to_string(), 98),
        ("qm_31_add_mul_opcode".to_string(), 0),
        ("jnz_opcode".to_string(), 768029),
        ("call_opcode".to_string(), 1),
        ("mul_opcode".to_string(), 10737001),
        ("jnz_opcode_taken".to_string(), 3147591),
        ("blake_compress_opcode".to_string(), 0),
        ("jump_opcode_rel_imm".to_string(), 741007),
        ("ret_opcode".to_string(), 933154),
        ("add_ap_opcode".to_string(), 867101),
        ("add_opcode_small".to_string(), 1837238),
        ("assert_eq_opcode".to_string(), 5808292),
        ("jump_opcode_double_deref".to_string(), 0),
        ("assert_eq_opcode_double_deref".to_string(), 4985047),
        ("call_opcode_rel_imm".to_string(), 933153),
        ("add_opcode".to_string(), 19599575),
        ("assert_eq_opcode_imm".to_string(), 2415033),
    ]);

    get_non_component_stat(stat, &ecdsa_3000)
}
