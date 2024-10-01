use super::super::casm_state::*;
use super::super::common::*;
use super::jump_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;

fn test_jump_opcode(
    non_consts_flags: [bool; 6],
    op0: i64,
    op1: i64,
    offsets_value: [Option<i16>; 2],
    entry_file_name: Option<&str>,
    expected_state: Vec<&str>,
) {
    let [is_rel, is_imm, is_double_deref, op0_base_fp, op1_base_fp, ap_update_add_1] =
        non_consts_flags;
    // Create the air function
    let mut jump_opcode = JumpOpcode {
        is_rel,
        is_imm,
        is_double_deref,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let pc = 3;
    let ap = 11;
    let fp = 6;

    // Create the non-constant is_imm_jump
    let non_consts_flags = if is_imm {
        vec![ap_update_add_1]
    } else if is_double_deref {
        vec![op0_base_fp, ap_update_add_1]
    } else {
        vec![op1_base_fp, !op1_base_fp, ap_update_add_1]
    };

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_jump(
                offsets_value[0],
                offsets_value[1],
                jump_opcode
                    .get_flags()
                    .non_constants_to_arr(&non_consts_flags),
            ) as u128,
            0
        ),
    )];
    if is_imm {
        memory_values.push((const_expr!(pc + 1), const_felt252_expr!(op1)));
    } else if is_double_deref {
        memory_values.push((
            const_expr!((op0 as i32 + offsets_value[1].unwrap() as i32) as u32),
            const_felt252_expr!(op1),
        ));
        if op0_base_fp {
            memory_values.push((
                const_expr!((fp as i16 + offsets_value[0].unwrap()) as u32),
                const_felt252_expr!(op0),
            ));
        } else {
            memory_values.push((
                const_expr!((ap as i16 + offsets_value[0].unwrap()) as u32),
                const_felt252_expr!(op0),
            ));
        }
    } else if op1_base_fp {
        memory_values.push((
            const_expr!((fp as i16 + offsets_value[1].unwrap()) as u32),
            const_felt252_expr!(op1),
        ));
    } else {
        memory_values.push((
            const_expr!((ap as i16 + offsets_value[1].unwrap()) as u32),
            const_felt252_expr!(op1),
        ));
    }
    jump_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let registry = AirFnRegistry::new(&jump_opcode);
    let (state, next_state) = registry.run_air(
        &jump_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    if is_rel {
        assert_eq!(next_state.pc.calc(), (pc as i64 + op1).to_string());
    } else {
        assert_eq!(next_state.pc.calc(), op1.to_string());
    }
    assert_eq!(next_state.fp.calc(), fp.to_string());
    if ap_update_add_1 {
        assert_eq!(next_state.ap.calc(), (ap + 1).to_string());
    } else {
        assert_eq!(next_state.ap.calc(), ap.to_string());
    }

    // Check state
    assert_eq!(state.calc(), expected_state);

    // Check entry
    if let Some(entry_file_name) = entry_file_name {
        compare_json(
            &registry.get_air_fn_entry(&jump_opcode.name()),
            &(TEST_JSONS_OPCODES_DIR.to_owned() + entry_file_name),
        );
    }
}

#[test]
fn test_abs_jump_base_ap() {
    test_jump_opcode(
        [false, false, false, false, false, false],
        125,
        8,
        [None, Some(2)],
        Some("abs_jump_base_ap.json"),
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "32770", // offset2
            "0",     // flag op1_base_fp
            "1",     // flag op1_base_ap
            "0",     // flag ap_update_add_1
            "1",     // op1 id
            "8",     // op1
            "0",     // op1
            "0",     // op1
        ],
    );
}

#[test]
fn test_abs_jump_base_fp() {
    test_jump_opcode(
        [false, false, false, false, true, false],
        125,
        5,
        [None, Some(10)],
        None,
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "32778", // offset2
            "1",     // flag op1_base_fp
            "0",     // flag op1_base_ap
            "0",     // flag ap_update_add_1
            "1",     // op1 id
            "5",     // op1
            "0",     // op1
            "0",     // op1
        ],
    );
}

#[test]
fn test_abs_jump_base_ap_inc_ap() {
    test_jump_opcode(
        [false, false, false, false, false, true],
        125,
        8,
        [None, Some(2)],
        None,
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "32770", // offset2
            "0",     // flag op1_base_fp
            "1",     // flag op1_base_ap
            "1",     // flag ap_update_add_1
            "1",     // op1 id
            "8",     // op1
            "0",     // op1
            "0",     // op1
        ],
    );
}

#[test]
fn test_abs_jump_base_fp_inc_ap() {
    test_jump_opcode(
        [false, false, false, false, true, true],
        125,
        5,
        [None, Some(10)],
        None,
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "32778", // offset2
            "1",     // flag op1_base_fp
            "0",     // flag op1_base_ap
            "1",     // flag ap_update_add_1
            "1",     // op1 id
            "5",     // op1
            "0",     // op1
            "0",     // op1
        ],
    );
}

#[test]
fn test_abs_big_op1() {
    test_jump_opcode(
        [false, false, false, false, false, false],
        125,
        1684685,
        [None, Some(402)],
        None,
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "33170", // offset2
            "0",     // flag op1_base_fp
            "1",     // flag op1_base_ap
            "0",     // flag ap_update_add_1
            "1",     // op1 id
            "205",   // op1
            "218",   // op1
            "6",     // op1
        ],
    );
}

#[test]
fn test_abs_jump_negativ_offset() {
    test_jump_opcode(
        [false, false, false, false, false, false],
        125,
        9,
        [None, Some(-9)],
        None,
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "32759", // offset2
            "0",     // flag op1_base_fp
            "1",     // flag op1_base_ap
            "0",     // flag ap_update_add_1
            "1",     // op1 id
            "9",     // op1
            "0",     // op1
            "0",     // op1
        ],
    );
}

#[test]
fn test_rel_jump() {
    test_jump_opcode(
        [true, true, false, false, false, false],
        125,
        100,
        [None, None],
        Some("rel_jump.json"),
        vec![
            "3",   // pc
            "11",  // ap
            "6",   // fp
            "0",   // flag ap_update_add_1
            "1",   // op1 id
            "0",   // op1 (sign)
            "0",   // op1 (sign)
            "100", // op1
            "0",   // op1
            "0",   // op1
        ],
    );
}

#[test]
fn test_rel_jump_inc_ap() {
    test_jump_opcode(
        [true, true, false, false, false, true],
        125,
        3,
        [None, None],
        None,
        vec![
            "3",  // pc
            "11", // ap
            "6",  // fp
            "1",  // ap_update_add_1
            "1",  // op1 id
            "0",  // op1 (sign)
            "0",  // op1 (sign)
            "3",  // op1
            "0",  // op1
            "0",  // op1
        ],
    );
}

#[test]
fn test_rel_big_op1() {
    test_jump_opcode(
        [true, true, false, false, false, false],
        125,
        54687687,
        [None, None],
        None,
        vec![
            "3",   // pc
            "11",  // ap
            "6",   // fp
            "0",   // ap_update_add_1
            "1",   // op1 id
            "0",   // op1 (sign)
            "0",   // op1 (sign)
            "455", // op1
            "315", // op1
            "208", // op1
        ],
    );
}

#[test]
fn test_rel_negative_imm() {
    test_jump_opcode(
        [true, true, false, false, false, false],
        125,
        -2,
        [None, None],
        None,
        vec![
            "3",   // pc
            "11",  // ap
            "6",   // fp
            "0",   // ap_update_add_1
            "1",   // op1 id
            "1",   // op1 (sign)
            "1",   // op1 (sign)
            "511", // op1
            "511", // op1
            "511", // op1
        ],
    );
}

#[test]
fn test_rel_negative_op1() {
    test_jump_opcode(
        [true, false, false, false, false, false],
        125,
        -2,
        [None, Some(333)],
        None,
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "33101", // offset2
            "0",     // flag op1_base_fp
            "1",     // flag op1_base_ap
            "0",     // ap_update_add_1
            "1",     // op1 id
            "1",     // op1 (sign)
            "1",     // op1 (sign)
            "511",   // op1
            "511",   // op1
            "511",   // op1
        ],
    );
}

#[test]
fn test_rel_deref_base_fp() {
    test_jump_opcode(
        [true, false, false, false, true, true],
        125,
        16584,
        [None, Some(12345)],
        Some("rel_jump_deref_base_fp.json"),
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "45113", // offset2
            "1",     // flag op1_base_fp
            "0",     // flag op1_base_ap
            "1",     // flag ap_update_add_1
            "1",     // op1 id
            "0",     // op1(sign)
            "0",     // op1(sign)
            "200",   // op1
            "32",    //op1
            "0",     // op1
        ],
    );
}

#[test]
fn test_abs_double_deref() {
    test_jump_opcode(
        [false, false, true, true, true, true],
        125,
        16584,
        [Some(4654), Some(12345)],
        Some("abs_jump_double_deref.json"),
        vec![
            "3",     // pc
            "11",    // ap
            "6",     // fp
            "37422", // offset1
            "45113", // offset2
            "1",     // flag op0_base_fp
            "1",     // ap_update_add_1
            "2",     // op0 id
            "125",   // op0
            "0",     // op0
            "0",     // op0
            "1",     // op1 id
            "200",   // op1
            "32",    // op1
            "0",     // op1
        ],
    );
}

#[test]
#[should_panic(expected = "Immediate jump must be relative")]
fn test_abs_immediate() {
    test_jump_opcode(
        [false, true, false, false, false, true],
        125,
        16584,
        [Some(4654), Some(12345)],
        None,
        vec![],
    );
}

#[test]
#[should_panic(expected = "Double deref jump must be absolute")]
fn test_rel_double_deref() {
    test_jump_opcode(
        [true, false, true, true, false, false],
        125,
        16584,
        [Some(4654), Some(12345)],
        None,
        vec![],
    );
}

pub fn assemble_jump(op0_off: Option<i16>, op1_off: Option<i16>, flags: [bool; 15]) -> u64 {
    let off0 = op0_off.map_or(-1, |v| v);
    let off1 = op1_off.map_or(1, |v| v);
    assemble_instruction(-1, off0, off1, flags)
}
