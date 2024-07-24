use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::Expr;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::Felt252;
use crate::expr;
use crate::felt252_expr;

use super::read_small_felt252::*;

#[test]
fn test_read_30_bits() {
    let mut addr = expr!("addr", 5);
    addr.to_state(0);
    //fill memory
    let memory_values = vec![(addr.clone(), felt252_expr!("op", 710034235, 0))];
    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new_with_data(memory_values);
    let read_30bit_felt = ReadSmallFelt252 {
        num_bits: 30,
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&read_30bit_felt);
    let (state, val) = registry.run_air(&read_30bit_felt, addr);
    assert!(val.calc() == "(710034235, 0)");
    let deduction_vec = [
        "deduction_tmp_0 = Memory__FeltExpr__Felt252Expr(ReadSmallFelt252__30_input)",
        "deduction_tmp_0.get_m31(const_0)",
        "deduction_tmp_0.get_m31(const_1)",
        "deduction_tmp_0.get_m31(const_2)",
        "deduction_tmp_1 = RangeCheck6(state[2])",
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
        "Memory__FeltExpr__Felt252Expr([ReadSmallFelt252__30_input]) == zero_extend([state[0], state[1], state[2]])",
        "RangeCheck6([state[2]]) == []"
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
    let state_vec = vec!["827", "1316", "42"];
    assert_eq!(state.calc(), state_vec);
}

#[test]
fn test_read_24_bits() {
    let mut addr = expr!("addr", 5);
    addr.to_state(0);
    //fill memory
    let memory_values = vec![(addr.clone(), felt252_expr!("op", 9874755, 0))];
    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new_with_data(memory_values);
    let read_24bit_felt = ReadSmallFelt252 {
        num_bits: 24,
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&read_24bit_felt);
    let (state, val) = registry.run_air(&read_24bit_felt, addr);
    assert!(val.calc() == "(9874755, 0)");
    let deduction_vec = vec![
        "deduction_tmp_0 = Memory__FeltExpr__Felt252Expr(ReadSmallFelt252__24_input)",
        "deduction_tmp_0.get_m31(const_0)",
        "deduction_tmp_0.get_m31(const_1)",
    ];
    assert_eq!(
        registry
            .get_compiled_air_fn(&read_24bit_felt)
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deduction_vec
    );
    let constraints_vec =
        vec!["Memory__FeltExpr__Felt252Expr([ReadSmallFelt252__24_input]) == zero_extend([state[0], state[1]])"];
    assert_eq!(
        registry
            .get_compiled_air_fn(&read_24bit_felt)
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints_vec
    );
    let state_vec = vec!["3395", "2410"];
    assert_eq!(state.calc(), state_vec);
}

#[test]
#[should_panic(expected = "RangeCheck4 failed (input 42)")]
fn test_fail_too_small_read() {
    let mut addr = expr!("addr", 5);
    addr.to_state(0);
    //fill memory
    let memory_values = vec![(addr.clone(), felt252_expr!("op", 710034235, 0))];
    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new_with_data(memory_values);
    let read_28bit_felt = ReadSmallFelt252 {
        num_bits: 28,
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&read_28bit_felt);
    let (_, val) = registry.run_air(&read_28bit_felt, addr);
    assert!(val.calc() == "(710034235, 0)");
}
