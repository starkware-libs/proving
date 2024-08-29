use super::range_check::*;
use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;

#[test]
fn test_range_check() {
    let deductions = [
        "tmp_0 = ()",
        "tmp_1 = external(Seq)",
        "tmp_5 = Memory((const_100 + tmp_1))",
        "tmp_5",
        "tmp_6 = Memory(state[0])",
        "tmp_6.get_m31(const_0)",
        "tmp_6.get_m31(const_1)",
        "tmp_6.get_m31(const_2)",
        "tmp_6.get_m31(const_3)",
        "tmp_7 = RangeCheck5([state[4]])",
    ];

    let constraints = [
        "tmp_1 = external(Seq)",
        "Memory([(const_100 + tmp_1)]) == [state[0]]",
        "RangeCheck5([state[4]]) == []",
        "Memory([state[0]]) == zero_extend([state[1], state[2], state[3], state[4]])",
    ];

    let memory = Felt252IdMemory::new_with_data(vec![(
        const_expr!(DUMMY_SEGMENT_START),
        const_felt252_expr!((1 << 17), 0),
    )]);

    let rc = RangeCheckBuiltin {
        bits: 32,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);
    let lists = registry.get_compiled_air_fn(&rc.name());

    assert_eq!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints
    );

    assert_eq!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deductions
    );
}

fn run_range_check(value: Felt252Expr, bits: usize) {
    let address = DUMMY_SEGMENT_START;
    let memory = Felt252IdMemory::new_with_data(vec![(const_expr!(address), value)]);

    let rc = RangeCheckBuiltin {
        bits,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);

    registry.run_air_with_row_number(&rc, (), 0);
}

#[test]
fn test_range_check_whole_limbs() {
    run_range_check(const_felt252_expr!(1 << 70, 0), 72);
}

#[test]
#[should_panic(expected = "Memory::set() failed")]
fn test_range_check_whole_limbs_fail() {
    run_range_check(const_felt252_expr!(1 << 74, 0), 72);
}

// Tests where <bits> is not divisible by 12
#[test]
fn test_range_check_partial_limbs() {
    run_range_check(const_felt252_expr!(1 << 127, 0), 128);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck2 on input 4")]
fn test_range_check_partial_limbs_fail() {
    run_range_check(const_felt252_expr!(0, 1), 128);
}
