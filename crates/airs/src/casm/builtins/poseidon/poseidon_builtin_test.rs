use air_infra::core::Felt;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::public_params::PublicParam;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};

use super::poseidon_builtin::*;

#[test]
fn simple_test_poseidon_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 600;

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(
                0x852357968577b1e386550ed6a9086133u128,
                0x79e8d1e78258000a28fc9d49e233bc6u128
            ),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(
                0xe5b5404b91ccaabca256154cbb6fb984u128,
                0x3840d003d0f3f96dbb796ff6aa6a63bu128
            ),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 5),
            const_felt252_expr!(
                0x9325a61fb2ef326e50b70eaa8a3c7cc7u128,
                0x1eb39da3f7d3b04142d0ac83d9da00cu128
            ),
        ),
    ]);

    let poseidon = PoseidonBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::PoseidonBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&poseidon);

    registry.run_air_with_row_number(&poseidon, (), (), 10);
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn simple_failed_test_poseidon_builtin() {
    let instance_number = const_expr!(10);
    let segment_start = 600;

    let memory = Felt252IdMemory::new_with_data(vec![
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 0),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 1),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 2),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 3),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 4),
            const_felt252_expr!(0, 0),
        ),
        (
            get_addr(const_expr!(segment_start), instance_number.clone(), 5),
            const_felt252_expr!(0, 0),
        ),
    ]);

    let poseidon = PoseidonBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::PoseidonBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&poseidon);

    registry.run_air_with_row_number(&poseidon, (), (), 10);
}

fn get_addr(segment_start: FeltExpr, instance_num: FeltExpr, offset: u32) -> FeltExpr {
    segment_start + instance_num * const_expr!(CELLS_PER_POSEIDON) + const_expr!(offset)
}
