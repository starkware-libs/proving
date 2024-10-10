use inst_def::InstDef;

use super::air_fn::*;
use super::air_fn_registry::*;
use super::expressions::felt252_expr::*;
use super::expressions::felt_expr::*;
use super::memory::*;
use super::variables::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;

#[derive(Debug, Default, InstDef)]
struct SimpleMemoryAirFn {
    #[instdef(skip)]
    memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for SimpleMemoryAirFn {
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        let mut value = air_builder.mem_read_unverified(&self.memory, &input);
        for f in value.as_felts_mut() {
            air_builder.deduce(f, "");
        }

        air_builder.mem_verify(&self.memory, &(input + const_expr!(1)), value.clone());

        value.get_felt(0)
    }
}

#[test]
fn test_memory_air_fn() {
    let mut func = SimpleMemoryAirFn::default();

    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new();
    let k: FeltExpr = const_expr!(1000);
    memory.set(k.clone(), const_felt252_expr!(3, 0));
    func.memory = memory.clone();

    let (registry, _) = AirFnRegistry::new(&func);
    let (_state, v) = registry.run_air(&func, k.clone());
    assert_eq!(v.calc(), "3".to_string());

    let (_state, v) = registry.run_air(&func, k + const_expr!(1));
    assert_eq!(v.calc(), "3".to_string());

    assert_eq!(memory.data.borrow().len(), 3);
    assert_eq!(
        memory.get(&const_expr!(1000)).unwrap().calc(),
        "[3, 0, 0, 0]".to_string()
    );
    assert_eq!(
        memory.get(&const_expr!(1001)).unwrap().calc(),
        "[3, 0, 0, 0]".to_string()
    );
    assert_eq!(
        memory.get(&const_expr!(1002)).unwrap().calc(),
        "[3, 0, 0, 0]".to_string()
    );
}
