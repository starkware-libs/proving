use air_common::{PaddingType, TraceType, UseOrYield};
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::constraint_connectedness_test;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::AirVar;
use air_infra::seq::Seq;
use serde::Serialize;

use super::blake_output::*;
use super::blake_round::*;
use super::create_blake_round_input::*;
use super::qm31_into_u32::*;
use crate::casm::opcodes::blake::blake_compress_opcode::BLAKE_NUM_ROUNDS;
use crate::casm::opcodes::blake::create_blake_output::*;
use crate::circuit::ext_tables::*;

#[derive(Clone, Debug, Serialize)]
pub struct BlakeGate {}

impl AirFn for BlakeGate {
    type ExtIn = ();
    type In = ([[UInt32Expr; 8]; 2], [FeltExpr; 16]);
    type Out = ();

    fn call(
        &self,
        ab: &mut AirBuilder,
        _: (),
        ([state_before, state_after], message): Self::In,
    ) -> Self::Out {
        // Blake message limbs are not connected by the constraints, only by the lookup to the
        // BlakeMessage relation.
        constraint_connectedness_test::exclude(self);

        let finalize_flag = ab.call_external_table(&FinalizeFlag {});

        let state = ab.call(&CreateBlakeRoundInput {}, (state_before.clone(), finalize_flag));

        // Yields the message to the BlakeMessage relation.
        let message_id = ab.call_external_table(&Seq {});
        let new_message = ab.call(&QM31IntoU32 {}, (message.clone(), message_id.clone()));

        let (new_state, _) = ab.chain_lookup_call(
            &CircuitBlakeRound { message: new_message.clone() },
            (state, message_id),
            0,
            BLAKE_NUM_ROUNDS,
        );

        let expected_h_after =
            ab.call(&CreateBlakeOutput {}, (state_before.clone(), new_state.clone()));
        for i in 0..8 {
            ab.constrain(
                expected_h_after[i].low().as_felt() - state_after[i].low().as_felt(),
                &format!("Blake output h[{i}].low() matches expected"),
            );
            ab.constrain(
                expected_h_after[i].high().as_felt() - state_after[i].high().as_felt(),
                &format!("Blake output h[{i}].high() matches expected"),
            );
        }

        ab.registry.add_entry(&BlakeOutput {});
        let enabler = ab.call_external_table(&BlakeGateEnabler {});
        let state_before_addr = ab.call_external_table(&StateBeforeAddr {});
        ab.add_lookup_term(
            "BlakeOutput",
            vec![state_before_addr.var].into_iter().chain(state_before.as_felts()).collect(),
            UseOrYield::Use,
            enabler.clone(),
        );
        let state_after_addr = ab.call_external_table(&StateAfterAddr {});
        ab.add_lookup_term(
            "BlakeOutput",
            vec![state_after_addr.var].into_iter().chain(state_after.as_felts()).collect(),
            UseOrYield::Yield,
            enabler.clone(),
        );

        let message_addr = ab.call_external_table(&Message0Addr {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![message_addr.var.clone()]
                .into_iter()
                .chain(message[0..4].iter().cloned())
                .collect(),
            UseOrYield::Use,
            enabler.clone(),
        );
        let message_addr = ab.call_external_table(&Message1Addr {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![message_addr.var.clone()]
                .into_iter()
                .chain(message[4..8].iter().cloned())
                .collect(),
            UseOrYield::Use,
            enabler.clone(),
        );
        let message_addr = ab.call_external_table(&Message2Addr {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![message_addr.var.clone()]
                .into_iter()
                .chain(message[8..12].iter().cloned())
                .collect(),
            UseOrYield::Use,
            enabler.clone(),
        );
        let message_addr = ab.call_external_table(&Message3Addr {});
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![message_addr.var.clone()]
                .into_iter()
                .chain(message[12..16].iter().cloned())
                .collect(),
            UseOrYield::Use,
            enabler.clone(),
        );
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("state_before_limb0".to_string()),
            Some("state_before_limb1".to_string()),
            Some("state_before_limb2".to_string()),
            Some("state_before_limb3".to_string()),
            Some("state_before_limb4".to_string()),
            Some("state_before_limb5".to_string()),
            Some("state_before_limb6".to_string()),
            Some("state_before_limb7".to_string()),
            Some("state_after_limb0".to_string()),
            Some("state_after_limb1".to_string()),
            Some("state_after_limb2".to_string()),
            Some("state_after_limb3".to_string()),
            Some("state_after_limb4".to_string()),
            Some("state_after_limb5".to_string()),
            Some("state_after_limb6".to_string()),
            Some("state_after_limb7".to_string()),
            Some("message_limb0".to_string()),
            Some("message_limb1".to_string()),
            Some("message_limb2".to_string()),
            Some("message_limb3".to_string()),
            Some("message_limb4".to_string()),
            Some("message_limb5".to_string()),
            Some("message_limb6".to_string()),
            Some("message_limb7".to_string()),
            Some("message_limb8".to_string()),
            Some("message_limb9".to_string()),
            Some("message_limb10".to_string()),
            Some("message_limb11".to_string()),
            Some("message_limb12".to_string()),
            Some("message_limb13".to_string()),
            Some("message_limb14".to_string()),
            Some("message_limb15".to_string()),
        ])
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Gate
    }

    fn padding_type(&self) -> PaddingType {
        PaddingType::Enabler
    }
}
