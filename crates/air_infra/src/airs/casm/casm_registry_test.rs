use std::cell::Ref;

use compiled_casm_air::compiled_structs::{CompiledAirFn, CompiledAirFnStat, LeanCompare};
use compiled_casm_air::public_params::PublicParam;
use compiled_casm_air::utils::{JSONS_BUILTINS_DIR, JSONS_LOOKUPS_DIR, JSONS_OPCODES_DIR};
use indexmap::IndexMap;

// Builtins
use super::builtins::bitwise::*;
use super::builtins::modulo::add_mod::*;
use super::builtins::modulo::mul_mod::*;
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
use super::opcodes::ret_opcode::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::felt252_id_memory::memory::*;
use crate::utils::test_utils::*;

const TRACE_COLUMNS_PER_LOGUP: usize = 2;

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

    // Add opcodes

    // Generic opcode
    reg.add_entry(&GenericOpcode::default());
    // AddAp opcode
    reg.add_entry(&AddApOpcode {
        imm: false,
        op1_base_fp: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&AddApOpcode {
        imm: true,
        op1_base_fp: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&AddApOpcode {
        imm: false,
        op1_base_fp: true,
        memory: Felt252IdMemory::default(),
    });
    // Add opcode
    reg.add_entry(&AddOpcode {
        small: true,
        imm: true,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&AddOpcode {
        small: false,
        imm: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&AddOpcode {
        small: true,
        imm: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&AddOpcode {
        small: false,
        imm: true,
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
        rel: false,
        op1_base_fp: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&CallOpcode {
        rel: true,
        op1_base_fp: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&CallOpcode {
        rel: false,
        op1_base_fp: true,
        memory: Felt252IdMemory::default(),
    });
    // Jnz opcode
    reg.add_entry(&JnzOpcode {
        taken: true,
        dst_base_fp: true,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&JnzOpcode {
        taken: false,
        dst_base_fp: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&JnzOpcode {
        taken: true,
        dst_base_fp: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&JnzOpcode {
        taken: false,
        dst_base_fp: true,
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
        imm: true,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&MulOpcode {
        small: true,
        imm: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&MulOpcode {
        small: false,
        imm: false,
        memory: Felt252IdMemory::default(),
    });
    reg.add_entry(&MulOpcode {
        small: false,
        imm: true,
        memory: Felt252IdMemory::default(),
    });
    // Ret opcode
    reg.add_entry(&RetOpcode::default());
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
    let mut stat = IndexMap::<String, CompiledAirFnStat>::new();
    let mut const_tables = IndexMap::new();

    for (name, compiled_entry) in compiled_reg.iter() {
        let fns = reg.air_fns.borrow();
        let entry = fns.get(name).unwrap();
        let dir = match entry.trace_type {
            TraceType::Opcode => JSONS_OPCODES_DIR,
            TraceType::Component | TraceType::Memory | TraceType::ChainRound => JSONS_LOOKUPS_DIR,
            TraceType::Builtin => JSONS_BUILTINS_DIR,
            TraceType::Const | TraceType::Inline => "",
        };

        // Collect preprocessed columns.
        if !compiled_entry.external_states.is_empty() {
            const_tables.insert(name, compiled_entry.external_states.clone());
        }

        // Inline and const functions are not compiled.
        if entry.trace_type == TraceType::Const || entry.trace_type == TraceType::Inline {
            continue;
        }

        // Check the compiled entry json.
        compare_json(compiled_entry, &format!("{}{}.json", dir, name));
        // Collect statistics.
        add_entry_statistics(&fns, compiled_entry, &mut stat);
    }

    compare_json(
        &stat,
        &"../compiled_casm_air/src/casm_registry.json".to_string(),
    );
    compare_json(
        &const_tables,
        &"../compiled_casm_air/src/const_tables.json".to_string(),
    );
}

// Collects statistics on a component.
fn add_entry_statistics(
    reg: &Ref<'_, IndexMap<String, AirFnEntry>>,
    compiled_entry: &CompiledAirFn,
    stat: &mut IndexMap<String, CompiledAirFnStat>,
) {
    let entry = reg.get(&compiled_entry.name).unwrap();
    assert!(entry.trace_type != TraceType::Const && entry.trace_type != TraceType::Inline);

    // Bulitins don't have yield columns.
    let lookup_yield = entry.trace_type != TraceType::Builtin;
    let lookup_multiplicity = compiled_entry.multiplicity_col_index.is_some();
    let num_state_cols = compiled_entry.state_names.len();
    let lookup_use_cols = entry.air_body.get_lookup_n_use_cols();
    let num_lookup_cols: usize = lookup_use_cols.iter().map(|(_, count)| count).sum();

    let total_num_trace_cols = num_state_cols
        + (TRACE_COLUMNS_PER_LOGUP * num_lookup_cols)
        + ((num_lookup_cols % 2) * TRACE_COLUMNS_PER_LOGUP)
        + (lookup_multiplicity as usize)
        + (TRACE_COLUMNS_PER_LOGUP * (lookup_yield as usize));

    // An upper bound on the number of cells added to the trace for each `AddInput`
    // to this component. Includes rows added to other lookup components called by
    // this component. Doesn't include cells from components that are always filled
    // by the prover, regardless of whether they're called or not (e.g. const tables,
    // the memory).
    // This is still an upper bound and not an exact number because some components
    // may or may not have rows added to them when called (for example VerifyInstruction,
    // where the same instruction might be verified multiple times in a single proof,
    // reusing the same row). This statistic pessimistically assumes that calls to
    // such components always add new rows.
    let mut trace_cells_upper_bound = total_num_trace_cols;

    let lookup_rows = entry.air_body.get_lookup_n_rows();
    for (name, cnt) in lookup_rows.iter() {
        let called_entry = reg.get(name).unwrap();
        let entry_stats = stat.get(name).unwrap();

        // For now, the only components with external inputs are lookups into const tables (like
        // range check) and memory tables. If we had a component with Seq of unfixed length
        // in its external input that we would like to include in the tighter upper bound, we would
        // need to update this condition.
        if called_entry.ext_input.is_none() && name != "verify_instruction" {
            trace_cells_upper_bound += cnt * entry_stats.trace_cells_upper_bound;
        }
    }

    stat.insert(
        compiled_entry.name.clone(),
        CompiledAirFnStat {
            trace_type: format!("{:?}", entry.trace_type),
            num_state_cols,
            lookup_use_cols,
            lookup_rows,
            lookup_yield,
            lookup_multiplicity,
            total_num_trace_cols,
            trace_cells_upper_bound,
        },
    );
}
