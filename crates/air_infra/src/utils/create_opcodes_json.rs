use crate::airs::casm::assert_eq_opcode::*;
use crate::airs::casm::call_opcode::*;
use crate::airs::casm::jump_opcode::*;
use crate::airs::casm::ret_opcode::*;
use crate::core::air_fn_registry::*;
use crate::core::memory::Memory;

pub fn create_assert_equal_opcdode_json(arguments: Vec<bool>) {
    let registry = AirFnRegistry::new(&AssertEqOpcode {
        is_double_deref: arguments[0],
        is_immediate: arguments[1],
        memory: Memory::default(),
    });
    registry.dump_to_file("airs/casm/assert_equal_opcode.json");
}

pub fn create_call_opcdode_json(arguments: Vec<bool>) {
    let registry = AirFnRegistry::new(&CallOpcode {
        is_rel: arguments[0],
        flag_op1_base_fp: arguments[1],
        memory: Memory::default(),
    });
    registry.dump_to_file("airs/casm/call_opcode.json");
}

pub fn create_jump_opcdode_json(arguments: Vec<bool>) {
    let registry = AirFnRegistry::new(&JumpOpcode {
        is_rel: arguments[0],
        flag_op1_base_fp: arguments[1],
        flag_ap_update_add_1: arguments[2],
        memory: Memory::default(),
    });
    registry.dump_to_file("airs/casm/jump_opcode.json");
}

pub fn create_ret_opcdode_json() {
    let registry = AirFnRegistry::new(&RetOpcode {
        memory: Memory::default(),
    });
    registry.dump_to_file("airs/casm/ret_opcode.json");
}
