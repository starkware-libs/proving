use compiled_casm_air::compiled_structs::TraceType;
use serde::Serialize;

use crate::airs::felt252_utils::felt252_packing27::*;
use crate::airs::felt252_utils::mul252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252width27_expr::*;

/// Cubing of a 252-bit felt, as a lookup function.
/// The input and output are given in packed form. The function expands them to Felt252 and
/// explicitly range-checks the limbs after expansion.
/// The result is not constrained to be fully reduced, i.e. an assignement with y = ((x*x*x) % P) +
/// P could satisfy the constraints.
#[derive(Clone, Debug, Serialize)]
pub struct Cube252 {}

impl AirFn for Cube252 {
    type ExtIn = ();
    type In = Felt252Width27Expr;
    type Out = Felt252Width27Expr;

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), x: Self::In) -> Self::Out {
        let a = air_builder.call(
            &Felt252UnpackFrom27 {
                range_check_output: true,
            },
            x,
        );
        let a_squared = air_builder.call(&Mul252 {}, [a.clone(), a.clone()]);
        let a_cubed = air_builder.call(&Mul252 {}, [a, a_squared]);

        felt252_pack_into27(a_cubed)
    }

    fn deduce_output(&self) -> Option<String> {
        // TODO(DanC): Implement this in stwo-cairo
        Some(format!(
            "{}::deduce_output",
            self.relation_name().expect("Relation name not found")
        ))
    }
}
