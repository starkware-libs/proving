use super::common::*;

use crate::airs::casm::generic_opcode::*;
use crate::airs::casm::call_opcode::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::expressions::expr::*;

// Macros
use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

#[test]
fn test_genric_call() {
    // build the air function
    let mut generic_opcode = GenericOpcode {
        memory: Memory::default(),
    };
    let call_opcode = CallOpcode {
        is_rel: true,
        flag_op1_base_fp: false,
        memory: Memory::default(),
    };

    // Register values at opcode start
    let pc_value = 50;
    let ap_value = 200;
    let fp_value = 150;
    let immediate = 299;

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!(
            "current instruction",
            assemble_instruction(0, 1, 1, call_opcode.get_flags().into()) as u128,
            0
        ),
    )];
    memory_values.push((
        const_expr!(pc_value + 1),
        felt252_expr!("op1_imm", immediate as u128, 0),
    ));
    // Not in use
    memory_values.push((
        ap.clone(),
        felt252_expr!("fp", fp_value as u128, 0),
    ));
    memory_values.push((
        const_expr!(ap_value + 1),
        felt252_expr!("Next pc", (pc_value + 2) as u128, 0),
    ));

    generic_opcode.init_memory(&Memory::new_with_data(memory_values));

    // Run air function
    let registry = AirFnRegistry::new(&generic_opcode);
    let (state, [next_pc, next_ap, next_fp]) = registry.run_air(&generic_opcode, [pc, ap.clone(), fp.clone()]);

    // Check output
    assert_eq!(next_pc.calc(), (pc_value + immediate).to_string());
    assert_eq!(next_ap.calc(), (ap_value + 2).to_string());
    assert_eq!(next_fp.calc(), (ap_value + 2).to_string());

    // Check state
    let expected_state = vec!["50", "200", "150", "0", "64", "1", "0", "16", "1", "0", "4", "0", "0", "1", "0", "0", "0", "0", "0", "1", "0", "0", "0", "1", "0", "0", "150", "52", "299", "299", "0", "0", "349"];
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    // Check Air body
    let expected_air_body = [
       "tmp_0 = [GenericOpcode_7daeff5d541cd4fc_input[0], GenericOpcode_7daeff5d541cd4fc_input[1], \
        GenericOpcode_7daeff5d541cd4fc_input[2]]",
        "Deduction: tmp_0[0]",
        "Deduction: tmp_0[1]",
        "Deduction: tmp_0[2]",
        "([(((state[3] + (state[4] * const_512)) + const_0) - const_32768), (((state[5] + \
        (state[6] * const_4)) + (state[7] * const_2048)) - const_32768), (((state[8] + \
        (state[9] * const_16)) + (state[10] * const_8192)) - const_32768)], [Bool::from_m31(state[11]), \
        Bool::from_m31(state[12]), Bool::from_m31(state[13]), Bool::from_m31(state[14]), \
        Bool::from_m31(state[15]), Bool::from_m31(state[16]), Bool::from_m31(state[17]), \
        Bool::from_m31(state[18]), Bool::from_m31(state[19]), Bool::from_m31(state[20]), \
        Bool::from_m31(state[21]), Bool::from_m31(state[22]), Bool::from_m31(state[23]), \
        Bool::from_m31(state[24]), Bool::from_m31(state[25])]) = CheckInstruction_670dde07bf003f3a(state[0])",
        "Constraint: ((((const_1 - (((const_1 - state[13]) - state[14]) - state[15])) - state[13]) \
        - state[14]) - state[15])",
        "Constraint: (((const_1 - ((const_1 - state[16]) - state[17])) - state[16]) - state[17])",
        "Constraint: ((((const_1 - (((const_1 - state[18]) - state[19]) - state[20])) - state[18]) \
        - state[19]) - state[20])",
        "Constraint: ((((const_1 - state[21]) - state[22]) - state[23]) - (((const_1 - state[21]) - \
        state[22]) - state[23]))",
        "Constraint: (((const_1 - ((const_1 - state[23]) - state[24])) - state[23]) - state[24])",
        "Felt252::from_m31_(zero_extend([state[26]])) = ReadSmallFelt252_cc824bd2f61c6ef6((((state[11] * state[2]) \
        + ((const_1 - state[11]) * state[1])) + (((state[3] + (state[4] * const_512)) + const_0) - const_32768)))",
        "Felt252::from_m31_(zero_extend([state[27]])) = ReadSmallFelt252_cc824bd2f61c6ef6((((state[12] * state[2]) \
        + ((const_1 - state[12]) * state[1])) + (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768)))",
        "Felt252::from_m31_(zero_extend([state[28]])) = ReadSmallFelt252_cc824bd2f61c6ef6((((((state[14] * state[2]) \
        + (state[15] * state[1])) + (state[13] * state[0])) + ((((const_1 - state[13]) - state[14]) - state[15]) \
        * ((state[27] + const_0) + const_0))) + (((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) \
        - const_32768)))",
        "Assignment: (state[29] - (((((const_1 - state[16]) - state[17]) * state[28]) + (state[17] * (state[27] \
        * state[28]))) + (state[16] * (state[27] + state[28]))))",
        "Constraint: (state[25] * (state[29] - state[26]))",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[25] * const_0)",
        "Constraint: (state[24] * ((((state[3] + (state[4] * const_512)) + const_0) - const_32768) + const_2))",
        "Constraint: (state[24] * ((((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) - const_32768) + const_1))",
        "Constraint: (state[24] * ((((const_4 - state[18]) - state[11]) - state[14]) - ((const_1 - state[16]) - state[17])))",
        "Constraint: (state[23] * (((state[3] + (state[4] * const_512)) + const_0) - const_32768))",
        "Constraint: (state[23] * (const_1 - (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768)))",
        "Constraint: (state[23] * (state[12] + state[11]))",
        "Constraint: (state[23] * (state[26] - state[2]))",
        "Constraint: (state[23] * (state[27] - (state[0] + (const_1 + state[13]))))",
        "Assignment: (state[30] - (const_1 - ((const_1 // ((((((((((((((((((((((((((((const_0 + state[26]) + const_0) + \
        const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + \
        const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + \
        const_0) + const_0) + const_0) + const_0) + const_0) + const_0)) * ((((((((((((((((((((((((((((const_0 + state[26]) \
        + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + \
        const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + \
        const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0))))",
        "tmp_57 = (state[26] - const_1)",
        "tmp_58 = const_2147483511",
        "tmp_59 = const_2147483391",
        "Assignment: (state[31] - (const_1 - ((const_1 // ((((((((((((((((((((((((((((const_0 + (tmp_57 * tmp_57)) + \
        const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + \
        const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + \
        const_18496) + const_0) + const_0) + const_0) + const_0) + const_0) + const_65536)) * (((((((((((((((((((((\
        (((((((const_0 + (tmp_57 * tmp_57)) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) \
        + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + const_0) + \
        const_0) + const_0) + const_0) + const_18496) + const_0) + const_0) + const_0) + const_0) + const_0) + const_65536))))",
        "Assignment: (state[32] - (((state[30] + state[31]) * (state[0] + (const_1 + state[13]))) + (((const_1 - \
        state[30]) - state[31]) * (state[0] + ((state[28] + const_0) + const_0)))))",
    ];
    let air_body = registry.get_air_fn_entry(&generic_opcode).air_body;
    assert_eq!(
        air_body
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        expected_air_body
    );

}
