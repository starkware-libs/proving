use super::super::common::*;
use super::add_ap_opcode::*;

use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

// Macros
use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

#[test]
fn test_add_ap() {
    // build the air function
    let mut add_ap_opcode = AddAp {
        memory: Memory::default(),
    };

    // Register values at opcode start
    let pc_value = 30;
    let ap_value = 11;
    let fp_value = 6;
    let immediate = 299;

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!(
            "op",
            assemble_instruction(-1, -1, 1, add_ap_opcode.get_flags().into()) as u128,
            0
        ),
    )];
    memory_values.push((
        const_expr!(pc_value + 1),
        felt252_expr!("imm", immediate, 0),
    ));
    add_ap_opcode.init_memory(&Memory::new_with_data(memory_values));

    // Run air function
    let registry = AirFnRegistry::new(&add_ap_opcode);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&add_ap_opcode, [pc, ap.clone(), fp.clone()]);

    // Check the output
    assert_eq!(next_pc.calc(), (pc_value + 2).to_string());
    assert_eq!(next_fp.calc(), (fp_value).to_string());
    assert_eq!(next_ap.calc(), (ap_value + immediate as u32).to_string());

    // Check the state
    let expected_state = [pc_value, ap_value, fp_value, immediate as u32];
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check the air body
    let entry = registry.get_air_fn_entry(&add_ap_opcode);
    assert_eq!(
        entry
            .air_body
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        vec![
            "tmp_0 = [\
                    AddAp_debc7bc8d7fbc47e_input[0], \
                    AddAp_debc7bc8d7fbc47e_input[1], \
                    AddAp_debc7bc8d7fbc47e_input[2]\
                ]",
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            "(\
                    [\
                        const_2147483646, \
                        const_2147483646, \
                        const_1\
                    ], [\
                        const_true, \
                        const_true, \
                        const_true, \
                        const_false, \
                        const_false, \
                        const_false, \
                        const_false, \
                        const_false, \
                        const_false, \
                        const_false, \
                        const_true, \
                        const_false, \
                        const_false, \
                        const_false, \
                        const_false\
                    ]\
                ) = DecodeInstruction_83cd6a5ed43aa52e(state[0])",
            "Felt252::from_m31_(zero_extend([state[3]])) = \
                    ReadSmallFelt252_cc824bd2f61c6ef6((state[0] + const_1))"
        ]
    );
}
