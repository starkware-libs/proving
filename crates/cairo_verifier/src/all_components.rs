// This file was created by the AIR team.

use circuits::ivalue::IValue;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use indexmap::IndexMap;

use crate::components;

pub fn all_components<Value: IValue>() -> IndexMap<&'static str, Box<dyn CircuitEval<Value>>> {
    IndexMap::from([
        (
            "add_opcode",
            Box::new(components::add_opcode::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "add_opcode_small",
            Box::new(components::add_opcode_small::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "add_ap_opcode",
            Box::new(components::add_ap_opcode::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "assert_eq_opcode",
            Box::new(components::assert_eq_opcode::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "assert_eq_opcode_imm",
            Box::new(components::assert_eq_opcode_imm::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "assert_eq_opcode_double_deref",
            Box::new(components::assert_eq_opcode_double_deref::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "blake_compress_opcode",
            Box::new(components::blake_compress_opcode::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "call_opcode_abs",
            Box::new(components::call_opcode_abs::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "call_opcode_rel_imm",
            Box::new(components::call_opcode_rel_imm::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "generic_opcode",
            Box::new(components::generic_opcode::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "jnz_opcode_non_taken",
            Box::new(components::jnz_opcode_non_taken::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "jnz_opcode_taken",
            Box::new(components::jnz_opcode_taken::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "jump_opcode_abs",
            Box::new(components::jump_opcode_abs::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "jump_opcode_double_deref",
            Box::new(components::jump_opcode_double_deref::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "jump_opcode_rel",
            Box::new(components::jump_opcode_rel::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "jump_opcode_rel_imm",
            Box::new(components::jump_opcode_rel_imm::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "mul_opcode",
            Box::new(components::mul_opcode::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "mul_opcode_small",
            Box::new(components::mul_opcode_small::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "qm_31_add_mul_opcode",
            Box::new(components::qm_31_add_mul_opcode::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "ret_opcode",
            Box::new(components::ret_opcode::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_instruction",
            Box::new(components::verify_instruction::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "blake_round",
            Box::new(components::blake_round::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        ("blake_g", Box::new(components::blake_g::Component {}) as Box<dyn CircuitEval<Value>>),
        (
            "blake_round_sigma",
            Box::new(components::blake_round_sigma::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "triple_xor_32",
            Box::new(components::triple_xor_32::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_12",
            Box::new(components::verify_bitwise_xor_12::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "add_mod_builtin",
            Box::new(components::add_mod_builtin::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "bitwise_builtin",
            Box::new(components::bitwise_builtin::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "mul_mod_builtin",
            Box::new(components::mul_mod_builtin::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "pedersen_builtin",
            Box::new(components::pedersen_builtin::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "pedersen_builtin_narrow_windows",
            Box::new(components::pedersen_builtin_narrow_windows::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "poseidon_builtin",
            Box::new(components::poseidon_builtin::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check96_builtin",
            Box::new(components::range_check96_builtin::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_builtin",
            Box::new(components::range_check_builtin::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "ec_op_builtin",
            Box::new(components::ec_op_builtin::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "partial_ec_mul_generic",
            Box::new(components::partial_ec_mul_generic::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "pedersen_aggregator_window_bits_18",
            Box::new(components::pedersen_aggregator_window_bits_18::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "partial_ec_mul_window_bits_18",
            Box::new(components::partial_ec_mul_window_bits_18::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "pedersen_points_table_window_bits_18",
            Box::new(components::pedersen_points_table_window_bits_18::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "pedersen_aggregator_window_bits_9",
            Box::new(components::pedersen_aggregator_window_bits_9::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "partial_ec_mul_window_bits_9",
            Box::new(components::partial_ec_mul_window_bits_9::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "pedersen_points_table_window_bits_9",
            Box::new(components::pedersen_points_table_window_bits_9::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "poseidon_aggregator",
            Box::new(components::poseidon_aggregator::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "poseidon_3_partial_rounds_chain",
            Box::new(components::poseidon_3_partial_rounds_chain::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "poseidon_full_round_chain",
            Box::new(components::poseidon_full_round_chain::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        ("cube_252", Box::new(components::cube_252::Component {}) as Box<dyn CircuitEval<Value>>),
        (
            "poseidon_round_keys",
            Box::new(components::poseidon_round_keys::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_252_width_27",
            Box::new(components::range_check_252_width_27::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_address_to_id",
            Box::new(components::memory_address_to_id::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big",
            Box::new(components::memory_id_to_big::Component { index: 0 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_1",
            Box::new(components::memory_id_to_big::Component { index: 1 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_2",
            Box::new(components::memory_id_to_big::Component { index: 2 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_3",
            Box::new(components::memory_id_to_big::Component { index: 3 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_4",
            Box::new(components::memory_id_to_big::Component { index: 4 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_5",
            Box::new(components::memory_id_to_big::Component { index: 5 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_6",
            Box::new(components::memory_id_to_big::Component { index: 6 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_7",
            Box::new(components::memory_id_to_big::Component { index: 7 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_8",
            Box::new(components::memory_id_to_big::Component { index: 8 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_9",
            Box::new(components::memory_id_to_big::Component { index: 9 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_10",
            Box::new(components::memory_id_to_big::Component { index: 10 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_11",
            Box::new(components::memory_id_to_big::Component { index: 11 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_12",
            Box::new(components::memory_id_to_big::Component { index: 12 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_13",
            Box::new(components::memory_id_to_big::Component { index: 13 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_14",
            Box::new(components::memory_id_to_big::Component { index: 14 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_big_15",
            Box::new(components::memory_id_to_big::Component { index: 15 })
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "memory_id_to_small",
            Box::new(components::memory_id_to_small::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_6",
            Box::new(components::range_check_6::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_8",
            Box::new(components::range_check_8::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_11",
            Box::new(components::range_check_11::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_12",
            Box::new(components::range_check_12::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_18",
            Box::new(components::range_check_18::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_20",
            Box::new(components::range_check_20::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_4_3",
            Box::new(components::range_check_4_3::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_4_4",
            Box::new(components::range_check_4_4::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_9_9",
            Box::new(components::range_check_9_9::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_7_2_5",
            Box::new(components::range_check_7_2_5::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_3_6_6_3",
            Box::new(components::range_check_3_6_6_3::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_4_4_4_4",
            Box::new(components::range_check_4_4_4_4::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "range_check_3_3_3_3_3",
            Box::new(components::range_check_3_3_3_3_3::Component {})
                as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_4",
            Box::new(components::verify_bitwise_xor_4::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_7",
            Box::new(components::verify_bitwise_xor_7::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_8",
            Box::new(components::verify_bitwise_xor_8::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_9",
            Box::new(components::verify_bitwise_xor_9::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
    ])
}
