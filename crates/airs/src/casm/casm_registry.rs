use air_common::TraceType;
use air_compile::compiled_structs::CompiledAirFn;
use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn::AirFn;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::public_params::PublicParam;
use air_infra::core::variables::{AirVar, ExtTable};
use air_infra::felt252_id_memory::id_to_small::MemoryIdToSmall;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use indexmap::IndexMap;

// Builtins
use super::builtins::bitwise::BitwiseBuiltin;
use super::builtins::ec_op::ec_op_builtin::ECOpBuiltin;
use super::builtins::modulo::add_mod::AddModBuiltin;
use super::builtins::modulo::mul_mod::MulModBuiltin;
use super::builtins::pedersen::pedersen_builtin::PedersenBuiltin;
use super::builtins::poseidon::poseidon_builtin::PoseidonBuiltin;
use super::builtins::range_check::RangeCheckBuiltin;
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

// The casm registry should contain all the builtins and opcodes used by Stwo for the casm vm.
// Note that other components used by an opcode or a builtin will be added to the registry
// automatically.
pub fn create_casm_registry() -> AirFnRegistry {
    let mut registry = AirFnRegistry::new_empty();

    for builtin in get_all_builtins() {
        registry.add_entry(builtin.as_ref());
    }

    for opcode in get_all_opcodes() {
        registry.add_entry(opcode.as_ref());
    }

    // Memory id to small need to be added manually.
    registry.add_entry(&MemoryIdToSmall::default());

    registry
}

// Returns a CASM registry filtered to exclude inline and const AIR functions,
// ordered for code generation.
pub fn create_components_casm_registry_reversed() -> IndexMap<String, CompiledAirFn> {
    let registry = create_casm_registry();
    let mut compiled_registry: IndexMap<String, CompiledAirFn> = registry
        .compile()
        .into_iter()
        .filter(|(_, compiled_air_fn)| {
            compiled_air_fn.r#type != TraceType::Const
                && compiled_air_fn.r#type != TraceType::Inline
        })
        .collect();

    // Ensure each component appears before all its dependencies.
    compiled_registry.reverse();
    compiled_registry
}

// Returns a CASM registry filtered to exclude inline and const AIR functions,
// ordered according to the arbitrary order defined in stwo-cairo.
// TODO(Stav): delete after the components are auto-generated.
pub fn create_casm_registry_ordered_by_stwo_cairo() -> IndexMap<String, CompiledAirFn> {
    let registry = create_casm_registry();
    let mut compiled_registry: IndexMap<String, CompiledAirFn> = registry
        .compile()
        .into_iter()
        .filter(|(_, compiled_air_fn)| {
            compiled_air_fn.r#type != TraceType::Const
                && compiled_air_fn.r#type != TraceType::Inline
        })
        .collect();

    // Define the order
    let order: Vec<&str> = vec![
        // Opcodes
        "add_opcode",
        "add_opcode_small",
        "add_ap_opcode",
        "assert_eq_opcode",
        "assert_eq_opcode_imm",
        "assert_eq_opcode_double_deref",
        "blake_compress_opcode",
        "call_opcode_abs",
        "call_opcode_rel_imm",
        "generic_opcode",
        "jnz_opcode_non_taken",
        "jnz_opcode_taken",
        "jump_opcode_abs",
        "jump_opcode_double_deref",
        "jump_opcode_rel",
        "jump_opcode_rel_imm",
        "mul_opcode",
        "mul_opcode_small",
        "qm_31_add_mul_opcode",
        "ret_opcode",
        // verify_instruction
        "verify_instruction",
        // Blake context components
        "blake_round",
        "blake_g",
        "blake_round_sigma",
        "triple_xor_32",
        "verify_bitwise_xor_12",
        // Builtins
        "add_mod_builtin",
        "bitwise_builtin",
        "mul_mod_builtin",
        "pedersen_builtin",
        "pedersen_builtin_narrow_windows",
        "poseidon_builtin",
        "range_check96_builtin",
        "range_check_builtin",
        "ec_op_builtin",
        // ECOp components
        "partial_ec_mul_generic",
        // Pedersen context components
        "pedersen_aggregator_window_bits_18",
        "partial_ec_mul_window_bits_18",
        "pedersen_points_table_window_bits_18",
        // Pedersen narrow windows context components
        "pedersen_aggregator_window_bits_9",
        "partial_ec_mul_window_bits_9",
        "pedersen_points_table_window_bits_9",
        // Poseidon context components
        "poseidon_aggregator",
        "poseidon_3_partial_rounds_chain",
        "poseidon_full_round_chain",
        "cube_252",
        "poseidon_round_keys",
        "range_check_252_width_27",
        // Memory components
        "memory_address_to_id",
        "memory_id_to_big",
        "memory_id_to_small",
        // Range checks
        "range_check_6",
        "range_check_8",
        "range_check_11",
        "range_check_12",
        "range_check_18",
        "range_check_20",
        "range_check_4_3",
        "range_check_4_4",
        "range_check_9_9",
        "range_check_7_2_5",
        "range_check_3_6_6_3",
        "range_check_4_4_4_4",
        "range_check_3_3_3_3_3",
        // Verify bitwise xor components
        "verify_bitwise_xor_4",
        "verify_bitwise_xor_7",
        "verify_bitwise_xor_8",
        "verify_bitwise_xor_9",
    ];

    // Add components in the specified order.
    let mut ordered = IndexMap::new();
    for name in order {
        if let Some(component) = compiled_registry.get(name) {
            ordered.insert(name.to_string(), component.clone());
            compiled_registry.swap_remove(name);
        } else {
            panic!("Component {name} not found in registry");
        }
    }

    assert!(compiled_registry.is_empty(), "Some components were not added to the ordered registry");
    ordered
}

/// Returns an array of all the air functions of opcodes.
pub fn get_all_opcodes() -> Vec<Box<dyn AirFn<ExtIn = (), In = CasmStateVar, Out = CasmStateVar>>> {
    vec![
        // Generic opcode
        Box::new(GenericOpcode::default()),
        // AddAp opcode
        Box::new(AddApOpcode { memory: Felt252IdMemory::default() }),
        // Add opcode
        Box::new(AddOpcode { small: true, memory: Felt252IdMemory::default() }),
        Box::new(AddOpcode { small: false, memory: Felt252IdMemory::default() }),
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
        Box::new(CallOpcode { rel_imm: false, memory: Felt252IdMemory::default() }),
        Box::new(CallOpcode { rel_imm: true, memory: Felt252IdMemory::default() }),
        // Jnz opcode
        Box::new(JnzOpcode { taken: true, memory: Felt252IdMemory::default() }),
        Box::new(JnzOpcode { taken: false, memory: Felt252IdMemory::default() }),
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
        Box::new(MulOpcode { small: true, memory: Felt252IdMemory::default() }),
        Box::new(MulOpcode { small: false, memory: Felt252IdMemory::default() }),
        // Ret opcode
        Box::new(RetOpcode::default()),
        // QM31AddMul opcode
        Box::new(QM31AddMulOpcode { memory: Felt252IdMemory::default() }),
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
        // Pedersen builtin narrow windows
        Box::new(PedersenBuiltin::<28>::default()),
        // ECOp builtin
        Box::new(ECOpBuiltin::default()),
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
