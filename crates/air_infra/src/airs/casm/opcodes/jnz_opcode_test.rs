use super::super::casm_state::*;
use super::super::common::*;
use super::jnz_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

use crate::const_expr;
use crate::const_felt252_expr;
use crate::expr;

fn build_and_test(
    [is_taken, dst_base_fp, ap_update_add_1]: [bool; 3],
    offset_dst: i16,
    dst_value: Felt252Expr,
    op1_value: i64,
    entry_file_name: Option<&str>,
    expected_state: Vec<u32>,
) {
    let [pc_value, ap_value, fp_value] = [50, 200, 150];
    let [pc, ap, fp] = [
        expr!("pc", pc_value),
        expr!("ap", ap_value),
        expr!("fp", fp_value),
    ];

    let mut jnz_opcode = JnzOpcode {
        is_taken,
        dst_base_fp,
        ap_update_add_1,
        memory: Felt252IdMemory::default(),
    };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(assemble_jnz(offset_dst, &jnz_opcode.get_flags()) as u128, 0),
    )];

    memory_values.push((const_expr!(pc_value + 1), const_felt252_expr!(op1_value)));

    if dst_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset_dst) as u32),
            dst_value.clone(),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset_dst) as u32),
            dst_value.clone(),
        ));
    }

    jnz_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let registry = AirFnRegistry::new(&jnz_opcode);
    let (state, next_state) =
        registry.run_air(&jnz_opcode, CasmStateVar::new(pc, ap.clone(), fp.clone()));

    // Check output
    if is_taken {
        assert_eq!(
            next_state.pc.calc(),
            (pc_value as i128 + op1_value as i128).to_string()
        );
    } else {
        assert_eq!(next_state.pc.calc(), (pc_value + 2).to_string());
    }

    if ap_update_add_1 {
        assert_eq!(next_state.ap.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_state.ap.calc(), ap_value.to_string());
    }

    assert_eq!(next_state.fp.calc(), fp_value.to_string());

    // Check state
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    // Check entry
    if let Some(entry_file_name) = entry_file_name {
        compare_test_json(
            &registry,
            &jnz_opcode.name(),
            &(TEST_JSONS_OPCODES_DIR.to_owned() + entry_file_name),
        );
    }
}

#[test]
fn test_jnz_not_taken_base_ap() {
    build_and_test(
        [false, false, false],
        -13,
        const_felt252_expr!(0, 0),
        15,
        Some("jnz_not_taken_base_ap.json"),
        vec![
            50, 200, 150, 499, 63, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    );
}

#[test]
fn test_jnz_taken_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        const_felt252_expr!(123, 456),
        15,
        Some("jnz_taken_base_ap.json"),
        vec![
            50, 200, 150, 499, 63, 0, 2, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 288, 3, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1955558780, 500077285, 1, 0, 0, 15, 0, 0,
        ],
    );
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_taken_zero_mismatch_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        const_felt252_expr!(0, 0),
        15,
        None,
        vec![],
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evalutate to 0)")]
fn test_not_taken_mismatch_base_ap() {
    build_and_test(
        [false, false, false],
        -13,
        const_felt252_expr!(123, 4567),
        15,
        None,
        vec![],
    );
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_taken_p_mismatch_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        const_felt252_expr!(1, 17 * u128::pow(2, 64) + u128::pow(2, 123)),
        15,
        None,
        vec![],
    );
}

#[test]
fn test_jnz_taken_negative_op1() {
    build_and_test(
        [true, true, false],
        -13,
        const_felt252_expr!(123, 456),
        -22,
        Some("jnz_taken_negative_op1.json"),
        vec![
            50, 200, 150, 499, 63, 0, 2, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 288, 3, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1955558780, 500077285, 1, 1, 1, 491, 511, 511,
        ],
    );
}

pub fn assemble_jnz(offset_dst: i16, flags: &Flags) -> u64 {
    assemble_instruction(offset_dst, -1, 1, flags.clone().into())
}
