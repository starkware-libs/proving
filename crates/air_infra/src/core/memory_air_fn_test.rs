use super::air_fn::*;
use super::air_fn_registry::*;
use super::expressions::felt252_expr::*;
use super::expressions::felt_expr::*;
use super::memory::*;
use super::prover_types::*;
use super::variables::*;
#[cfg(test)]
use crate::core::expressions::expr::*;

// Macros
use crate::const_expr;
use crate::felt252_expr;

#[derive(Debug, Default)]
struct SimpleMemoryAirFn {
    memory: Memory<FeltExpr, Felt252Expr>,
}

impl MemoryAirFn for SimpleMemoryAirFn {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}

impl AirFn for SimpleMemoryAirFn {
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        let mut value = air_builder.get_from_memory(&self.memory, &input);

        value = air_builder.let_for_deduction(value);
        for f in value.as_felts() {
            air_builder.deduce(f);
        }

        air_builder.set_in_memory(&self.memory, &input + &const_expr!(1), value.clone());

        value.as_felts()[0].clone()
    }
}

#[test]
fn test_memory_air_fn() {
    let mut func = SimpleMemoryAirFn::default();

    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new();
    let k: FeltExpr = const_expr!(1000);
    memory.set(k.clone(), felt252_expr!("val", 3, 0));
    func.init_memory(&memory);

    let registry = AirFnRegistry::new(&func);
    let (_state, v) = registry.run_air(&func, k.clone());
    assert_eq!(v.calc(), "3".to_string());

    let (_state, v) = registry.run_air(&func, &k + &const_expr!(1));
    assert_eq!(v.calc(), "3".to_string());

    assert_eq!(memory.data.borrow().len(), 3);
    assert_eq!(
        memory.get(&const_expr!(1000)).unwrap().calc(),
        "(3, 0)".to_string()
    );
    assert_eq!(
        memory.get(&const_expr!(1001)).unwrap().calc(),
        "(3, 0)".to_string()
    );
    assert_eq!(
        memory.get(&const_expr!(1002)).unwrap().calc(),
        "(3, 0)".to_string()
    );
}
