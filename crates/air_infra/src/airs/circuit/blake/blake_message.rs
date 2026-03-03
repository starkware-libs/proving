use air_common::{PaddingType, TraceType};
use serde::Serialize;

use crate::const_u32_expr;
use crate::core::air_fn::*;
use crate::core::constraint_connectedness_test;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;
#[cfg(test)]
use crate::core::variables::*;

#[derive(Debug, Serialize)]
pub struct BlakeMessage {
    #[serde(skip)]
    pub message: [UInt32Expr; 16],
}

impl AirFn for BlakeMessage {
    type ExtIn = ();
    type In = [FeltExpr; 2];
    type Out = UInt32Expr;

    fn call(&self, _ab: &mut AirBuilder, _: (), [_id, _index]: Self::In) -> Self::Out {
        // Blake message limbs are not connected by the constraints, only by the lookup to the
        // BlakeMessage relation.
        constraint_connectedness_test::exclude(self);

        #[allow(unused_mut)]
        let mut output = const_u32_expr!(0);

        #[cfg(test)]
        if _ab.is_run_mode() {
            output = self.message[_index.value().unwrap().0 as usize].clone();
        }

        output
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Relation
    }

    fn padding_type(&self) -> PaddingType {
        PaddingType::Multiplicity
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("id".to_string()), Some("index".to_string())])
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("message_limb".to_string())])
    }
}
