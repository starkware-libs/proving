use super::read_small_felt252::*;

use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::Expr;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::Felt252;

// Macros
use crate::const_expr;
use crate::felt252_expr;

#[test]
fn test_read_30_bits() {
    let addr = const_expr!(5);
    // Fill memory
    let memory_values = vec![(addr.clone(), felt252_expr!("op", 710034235, 0))];
    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new_with_data(memory_values);
    let read_30bit_felt = ReadSmallFelt252 {
        num_bits: 30,
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&read_30bit_felt);

    let (state, val) = registry.run_air(&read_30bit_felt, addr);
    assert_eq!(val.calc(), "(710034235, 0)");
    assert_eq!(state.calc(), ["315", "289", "148", "5"]);

    let deduction_vec = [
        "deduction_tmp_0 = Memory_81f75475e4cf34d6(ReadSmallFelt252_88bbc22de0781573_input)",
        "deduction_tmp_0.get_m31(const_0)",
        "deduction_tmp_0.get_m31(const_1)",
        "deduction_tmp_0.get_m31(const_2)",
        "deduction_tmp_0.get_m31(const_3)",
        "deduction_tmp_1 = RangeCheck3([state[3]])",
    ];
    assert_eq!(
        registry
            .get_compiled_air_fn(&read_30bit_felt)
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deduction_vec
    );

    let constraints_vec = vec![
        "Memory_81f75475e4cf34d6([ReadSmallFelt252_88bbc22de0781573_input]) == zero_extend([state[0], state[1], state[2], state[3]])",
        "RangeCheck3([state[3]]) == []"
    ];
    assert_eq!(
        registry
            .get_compiled_air_fn(&read_30bit_felt)
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints_vec
    );
}

#[test]
fn test_read_18_bits() {
    let addr = const_expr!(5);
    // Fill memory
    let memory_values = vec![(addr.clone(), felt252_expr!("op", 154293, 0))];
    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new_with_data(memory_values);
    let read_18bit_felt = ReadSmallFelt252 {
        num_bits: 18,
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&read_18bit_felt);

    let (state, val) = registry.run_air(&read_18bit_felt, addr);
    assert_eq!(val.calc(), "(154293, 0)");
    assert_eq!(state.calc(), ["181", "301"]);

    let deduction_vec = vec![
        "deduction_tmp_0 = Memory_81f75475e4cf34d6(ReadSmallFelt252_3eb8c0a330644873_input)",
        "deduction_tmp_0.get_m31(const_0)",
        "deduction_tmp_0.get_m31(const_1)",
    ];
    assert_eq!(
        registry
            .get_compiled_air_fn(&read_18bit_felt)
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deduction_vec
    );

    let constraints_vec =
        vec!["Memory_81f75475e4cf34d6([ReadSmallFelt252_3eb8c0a330644873_input]) == zero_extend([state[0], state[1]])"];
    assert_eq!(
        registry
            .get_compiled_air_fn(&read_18bit_felt)
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints_vec
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck4 on input 338")]
fn test_fail_too_small_read() {
    let addr = const_expr!(5);
    // Fill memory
    let memory_values = vec![(addr.clone(), felt252_expr!("op", 88754279, 0))];
    let memory = Memory::new_with_data(memory_values);
    let read_22bit_felt = ReadSmallFelt252 {
        num_bits: 22,
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&read_22bit_felt);
    registry.run_air(&read_22bit_felt, addr);
}
