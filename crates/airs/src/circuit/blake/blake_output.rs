use air_common::{GATE_RELATION_NAME, TraceType, UseOrYield};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::AirVar;
use serde::Serialize;

use crate::circuit::ext_tables::*;

#[derive(Clone, Debug, Serialize)]
pub struct BlakeOutput {}

impl AirFn for BlakeOutput {
    type ExtIn = ();
    type In = [UInt32Expr; 8];
    type Out = [FeltExpr; 8];

    fn call(&self, ab: &mut AirBuilder, _: (), final_state: Self::In) -> Self::Out {
        let mut output = vec![];
        for (i, state_limb) in final_state.iter().enumerate() {
            let mut output_limb =
                state_limb.low().as_felt() + (state_limb.high().as_felt() * const_expr!(1 << 16));
            output.push(ab.assign(&mut output_limb, &format!("output_limb{i}")));
        }

        let final_state_addr = ab.call_external_table(&FinalStateAddr {});
        let output0_mult = ab.call_external_table(&BlakeOutput0Multiplicity {});
        let output0_addr = ab.call_external_table(&BlakeOutput0Addr {});
        let output1_mult = ab.call_external_table(&BlakeOutput1Multiplicity {});
        let output1_addr = ab.call_external_table(&BlakeOutput1Addr {});

        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![final_state_addr.var].into_iter().chain(final_state.as_felts()).collect(),
            UseOrYield::Use,
            const_expr!(1),
        );

        ab.add_lookup_term(
            GATE_RELATION_NAME,
            vec![output0_addr.var].into_iter().chain(output[0..4].iter().cloned()).collect(),
            UseOrYield::Yield,
            output0_mult,
        );
        ab.add_lookup_term(
            GATE_RELATION_NAME,
            vec![output1_addr.var].into_iter().chain(output[4..8].iter().cloned()).collect(),
            UseOrYield::Yield,
            output1_mult,
        );

        output.try_into().expect("Output length should be 8")
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("final_state_limb0".to_string()),
            Some("final_state_limb1".to_string()),
            Some("final_state_limb2".to_string()),
            Some("final_state_limb3".to_string()),
            Some("final_state_limb4".to_string()),
            Some("final_state_limb5".to_string()),
            Some("final_state_limb6".to_string()),
            Some("final_state_limb7".to_string()),
        ])
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("output_limb0".to_string()),
            Some("output_limb1".to_string()),
            Some("output_limb2".to_string()),
            Some("output_limb3".to_string()),
            Some("output_limb4".to_string()),
            Some("output_limb5".to_string()),
            Some("output_limb6".to_string()),
            Some("output_limb7".to_string()),
        ])
    }

    fn relation_names(&self) -> Vec<String> {
        vec!["BlakeOutput".to_string()]
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Gate
    }
}
