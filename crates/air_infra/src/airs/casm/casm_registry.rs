use compiled_casm_air::compiled_structs::TraceType;
use compiled_casm_air::public_params::PublicParam;

// Builtins
use super::builtins::bitwise::BitwiseBuiltin;
use super::builtins::modulo::add_mod::AddModBuiltin;
use super::builtins::modulo::mul_mod::MulModBuiltin;
use super::builtins::pedersen::pedersen_builtin::PedersenBuiltin;
use super::builtins::poseidon::poseidon_builtin::PoseidonBuiltin;
use super::builtins::range_check::RangeCheckBuiltin;
use super::casm_state::CasmStateVar;
// Opcodes
use super::opcodes::add_ap_opcode::AddApOpcode;
use super::opcodes::add_opcode::AddOpcode;
use super::opcodes::assert_eq_opcode::AssertEqOpcode;
use super::opcodes::blake::blake_compress_opcode::BlakeCompressOpcode;
use super::opcodes::call_opcode::CallOpcode;
use super::opcodes::generic_opcode::generic_opcode::GenericOpcode;
use super::opcodes::jnz_opcode::JnzOpcode;
use super::opcodes::jump_opcode::JumpOpcode;
use super::opcodes::mul_opcode::MulOpcode;
use super::opcodes::qm31::qm31_add_mul_opcode::QM31AddMulOpcode;
use super::opcodes::ret_opcode::RetOpcode;
use crate::core::air_fn::AirFn;
use crate::core::air_fn_registry::AirFnRegistry;
use crate::core::felt252_id_memory::memory::Felt252IdMemory;
use crate::core::variables::{AirVar, ExtTable};

pub fn create_casm_registry() -> AirFnRegistry {
    let mut registry = AirFnRegistry::new_empty();

    for builtin in get_all_builtins() {
        registry.add_entry(builtin.as_ref());
    }

    for opcode in get_all_opcodes() {
        registry.add_entry(opcode.as_ref());
    }

    registry
}

/// Returns an array of all the air functions of opcodes.
pub fn get_all_opcodes() -> Vec<Box<dyn AirFn<ExtIn = (), In = CasmStateVar, Out = CasmStateVar>>> {
    vec![
        // Generic opcode
        Box::new(GenericOpcode::default()),
        // AddAp opcode
        Box::new(AddApOpcode {
            memory: Felt252IdMemory::default(),
        }),
        // Add opcode
        Box::new(AddOpcode {
            small: true,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(AddOpcode {
            small: false,
            memory: Felt252IdMemory::default(),
        }),
        // AssertEq opcode
        Box::new(AssertEqOpcode {
            double_deref: false,
            imm: false,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(AssertEqOpcode {
            double_deref: true,
            imm: false,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(AssertEqOpcode {
            double_deref: false,
            imm: true,
            memory: Felt252IdMemory::default(),
        }),
        // Call opcode
        Box::new(CallOpcode {
            rel_imm: false,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(CallOpcode {
            rel_imm: true,
            memory: Felt252IdMemory::default(),
        }),
        // Jnz opcode
        Box::new(JnzOpcode {
            taken: true,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(JnzOpcode {
            taken: false,
            memory: Felt252IdMemory::default(),
        }),
        // Jump opcode
        Box::new(JumpOpcode {
            rel: true,
            imm: true,
            double_deref: false,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(JumpOpcode {
            rel: true,
            imm: false,
            double_deref: false,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(JumpOpcode {
            rel: false,
            imm: false,
            double_deref: true,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(JumpOpcode {
            rel: false,
            imm: false,
            double_deref: false,
            memory: Felt252IdMemory::default(),
        }),
        // Mul opcode
        Box::new(MulOpcode {
            small: true,
            memory: Felt252IdMemory::default(),
        }),
        Box::new(MulOpcode {
            small: false,
            memory: Felt252IdMemory::default(),
        }),
        // Ret opcode
        Box::new(RetOpcode::default()),
        // QM31AddMul opcode
        Box::new(QM31AddMulOpcode {
            memory: Felt252IdMemory::default(),
        }),
        // Blake opcode
        Box::new(BlakeCompressOpcode::default()),
    ]
}

/// Returns an array of all the air functions of builtins.
pub fn get_all_builtins() -> Vec<Box<dyn AirFn<ExtIn = (), In = (), Out = ()>>> {
    vec![
        // Bitwise builtin
        Box::new(BitwiseBuiltin::default()),
        // RangeCheck 128 builtin
        Box::new(RangeCheckBuiltin {
            bits: 128,
            memory: Felt252IdMemory::default(),
            segment_start: PublicParam::RangeCheckBuiltinSegmentStart,
        }),
        // RangeCheck 96 builtin
        Box::new(RangeCheckBuiltin {
            bits: 96,
            memory: Felt252IdMemory::default(),
            segment_start: PublicParam::RangeCheck96BuiltinSegmentStart,
        }),
        // AddMod builtin
        Box::new(AddModBuiltin::default()),
        // MulMod builtin
        Box::new(MulModBuiltin::default()),
        // Poseidon builtin
        Box::new(PoseidonBuiltin::default()),
        // Pedersen builtin
        Box::new(PedersenBuiltin::<14>::default()),
    ]
}

pub fn get_sub_components<E, I, O>(air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>) -> Vec<String>
where
    E: ExtTable,
    I: AirVar,
    O: AirVar,
{
    let (registry, _) = AirFnRegistry::new(air_fn);
    let air_fns = registry.air_fns.borrow();
    air_fns
        .iter()
        .filter(|(_, entry)| {
            entry.trace_type != TraceType::Const && entry.trace_type != TraceType::Inline
        })
        .map(|(name, _)| name.clone())
        .collect()
}
