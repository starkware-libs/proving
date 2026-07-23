use air_common::{TraceType, UseOrYield};
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::{const_expr, const_u32_expr};
use serde::Serialize;

use crate::casm::opcodes::blake::triple_sum32::*;
use crate::casm::opcodes::blake::xor_rot32::*;
use crate::circuit::ext_tables::*;

/// Gate that verifies Blake2s G function: G(a, b, c, d, x, y) = (a', b', c', d').
#[derive(Clone, Debug, Serialize)]
pub struct BlakeGGate {}

impl AirFn for BlakeGGate {
    type ExtIn = ();
    type In = [UInt32Expr; 10];
    type Out = ();

    fn call(
        &self,
        ab: &mut AirBuilder,
        _: (),
        [a, b, c, d, f0, f1, a_out, b_out, c_out, d_out]: Self::In,
    ) -> Self::Out {
        let a_tmp = ab.call(&TripleSum32 {}, [a.clone(), b.clone(), f0.clone()]);
        let d_tmp = ab.call(&XorRot32 { r: 16 }, [a_tmp.clone(), d.clone()]);
        let c_tmp = ab.call(&TripleSum32 {}, [c.clone(), d_tmp.clone(), const_u32_expr!(0)]);
        let b_tmp = ab.call(&XorRot32 { r: 12 }, [b.clone(), c_tmp.clone()]);

        // Since the output is in the state, we use verification air-fn to save trace-cells.
        ab.call(&VerifyTripleSum32 {}, [a_tmp, b_tmp.clone(), f1.clone(), a_out.clone()]);
        ab.call(&VerifyXorRot32 { r: 8 }, [a_out.clone(), d_tmp, d_out.clone()]);
        ab.call(&VerifyTripleSum32 {}, [c_tmp, d_out.clone(), const_u32_expr!(0), c_out.clone()]);
        ab.call(&VerifyXorRot32 { r: 7 }, [b_tmp, c_out.clone(), b_out.clone()]);

        // Add the lookup constraints to the gate relation.
        let input_addr_a = ab.call_external_table(&BlakeGGateInputAddrA {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_a.var, a.low().as_felt(), a.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );
        let input_addr_b = ab.call_external_table(&BlakeGGateInputAddrB {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_b.var, b.low().as_felt(), b.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );
        let input_addr_c = ab.call_external_table(&BlakeGGateInputAddrC {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_c.var, c.low().as_felt(), c.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );
        let input_addr_d = ab.call_external_table(&BlakeGGateInputAddrD {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_d.var, d.low().as_felt(), d.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );
        let input_addr_f0 = ab.call_external_table(&BlakeGGateInputAddrF0 {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_f0.var, f0.low().as_felt(), f0.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );
        let input_addr_f1 = ab.call_external_table(&BlakeGGateInputAddrF1 {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_f1.var, f1.low().as_felt(), f1.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );

        let mult = ab.call_external_table(&BlakeGGateMultiplicity {});
        let output_addr_a = ab.call_external_table(&BlakeGGateOutputAddrA {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![output_addr_a.var, a_out.low().as_felt(), a_out.high().as_felt()],
            UseOrYield::Yield,
            mult.clone(),
        );
        let output_addr_b = ab.call_external_table(&BlakeGGateOutputAddrB {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![output_addr_b.var, b_out.low().as_felt(), b_out.high().as_felt()],
            UseOrYield::Yield,
            mult.clone(),
        );
        let output_addr_c = ab.call_external_table(&BlakeGGateOutputAddrC {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![output_addr_c.var, c_out.low().as_felt(), c_out.high().as_felt()],
            UseOrYield::Yield,
            mult.clone(),
        );
        let output_addr_d = ab.call_external_table(&BlakeGGateOutputAddrD {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![output_addr_d.var, d_out.low().as_felt(), d_out.high().as_felt()],
            UseOrYield::Yield,
            mult,
        );
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("a".to_string()),
            Some("b".to_string()),
            Some("c".to_string()),
            Some("d".to_string()),
            Some("f0".to_string()),
            Some("f1".to_string()),
            Some("a_out".to_string()),
            Some("b_out".to_string()),
            Some("c_out".to_string()),
            Some("d_out".to_string()),
        ])
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Gate
    }
}
