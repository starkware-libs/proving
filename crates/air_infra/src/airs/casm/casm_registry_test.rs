use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirFnStat, LookupTerm, TraceGenStep, UseOrYield,
};
use compiled_casm_air::public_params::PublicParam;
use compiled_casm_air::relations::OPCODES_RELATION_NAME;
use compiled_casm_air::utils::{JSONS_BUILTINS_DIR, JSONS_LOOKUPS_DIR, JSONS_OPCODES_DIR};
use indexmap::IndexMap;

// Builtins
use super::builtins::bitwise::*;
use super::builtins::modulo::add_mod::*;
use super::builtins::modulo::mul_mod::*;
use super::builtins::range_check::*;
// Opcodes
use super::opcodes::add_ap_opcode::*;
use super::opcodes::add_opcode::*;
use super::opcodes::assert_eq_opcode::*;
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

    //
    let mut constraints = IndexMap::new();
    for (name, entry) in reg.air_fns.borrow().iter() {
        constraints.insert(name.clone(), entry.air_body.get_constraints());
    }
    compare_json(
        &constraints,
        &"../compiled_casm_air/src/constraints.json".to_string(),
    );

    // Compile the registry, check the compiled entries jsons and collect the statistics.
    let compiled_reg = reg.compile();
    let mut stat = IndexMap::<String, CompiledAirFnStat>::new();
    for (name, (trace_type, compiled_entry)) in compiled_reg.iter() {
        let dir = match trace_type {
            TraceType::Opcode => JSONS_OPCODES_DIR,
            TraceType::Component | TraceType::Memory | TraceType::ChainRound => JSONS_LOOKUPS_DIR,
            TraceType::Builtin => JSONS_BUILTINS_DIR,
            TraceType::Const | TraceType::Inline => "",
        };

        if trace_type != &TraceType::Const && trace_type != &TraceType::Inline {
            // Check the compiled entry json.
            compare_json(&compiled_entry, &format!("{}{}.json", dir, name));
        }

        // Collect statistics.
        get_compiled_entry_statistics(compiled_entry, trace_type, &mut stat);
    }

    compare_json(
        &stat,
        &"../compiled_casm_air/src/casm_registry.json".to_string(),
    );
}

// Collects statistics on a compiled entry.
fn get_compiled_entry_statistics(
    compiled_entry: &CompiledAirFn,
    trace_type: &TraceType,
    stat: &mut IndexMap<String, CompiledAirFnStat>,
) {
    // Opcodes, components and memory have a lookup yield column, bulitins do not.
    let lookup_yield = trace_type != &TraceType::Builtin
        && trace_type != &TraceType::Const
        && trace_type != &TraceType::Inline;
    let lookup_multiplicity = compiled_entry.multiplicity_col_index.is_some();
    let num_state_cols = compiled_entry.state_names.len();
    let lookup_uses = get_lookup_uses_count(compiled_entry.deductions.clone());
    let num_lookup_uses = lookup_uses.iter().map(|(_, count)| count).sum();

    let total_num_trace_cols = num_state_cols
        + (TRACE_COLUMNS_PER_LOGUP * num_lookup_uses)
        + (lookup_multiplicity as usize)
        + (TRACE_COLUMNS_PER_LOGUP * (lookup_yield as usize));
    let mut trace_cells_upper_bound = total_num_trace_cols;
    let mut lookup_uses_upper_bound = num_lookup_uses;
    for (used_entry, num_uses) in lookup_uses.iter() {
        if stat.contains_key(used_entry) {
            trace_cells_upper_bound +=
                *num_uses * stat.get(used_entry).unwrap().trace_cells_upper_bound;
            lookup_uses_upper_bound +=
                *num_uses * stat.get(used_entry).unwrap().lookup_uses_upper_bound;
        } else {
            // For now, the only lookup relation which is not a component is "Opcodes".
            assert_eq!(used_entry, OPCODES_RELATION_NAME);
            assert_eq!(num_uses, &1);
        }
    }

    let key = if let Some(OPCODES_RELATION_NAME) = compiled_entry.relation_name.as_deref() {
        compiled_entry.name.clone()
    } else {
        compiled_entry
            .relation_name
            .clone()
            .unwrap_or(compiled_entry.name.clone())
    };

    stat.insert(
        key,
        CompiledAirFnStat {
            trace_type: format!("{:?}", trace_type),
            num_state_cols,
            lookup_uses,
            lookup_yield,
            lookup_multiplicity,
            total_num_trace_cols,
            trace_cells_upper_bound,
            lookup_uses_upper_bound,
        },
    );
}

// Returns the number of times a lookup relation is used by the air function.
fn get_lookup_uses_count(deductions: Vec<TraceGenStep>) -> IndexMap<String, usize> {
    let mut lookup_uses = IndexMap::new();
    for deduction in deductions {
        if let TraceGenStep::LookupTerm(LookupTerm {
            relation_name,
            use_or_yield,
            ..
        }) = deduction
        {
            if use_or_yield == UseOrYield::Use {
                *lookup_uses.entry(relation_name).or_insert(0) += 1;
            }
        }
    }
    lookup_uses
}
