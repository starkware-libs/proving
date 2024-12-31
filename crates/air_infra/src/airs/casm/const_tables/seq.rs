use compiled_casm_air::const_tables::STWO_COMPONENT_TYPE_SEQ;
use inst_def::InstDef;

use crate::airs::casm::casm_state::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
#[cfg(test)]
use crate::core::expressions::var_expr::*;
use crate::core::variables::*;
#[cfg(test)]
use crate::core::Felt;

#[derive(Debug, InstDef, Default)]
pub struct Seq {}
impl ExtTable for Seq {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_SEQ;
    type T = FeltExpr;
}

/// A constant sequential column - row <i> contains the value <i>
impl AirFn for Seq {
    type ExtIn = ();
    type In = ();
    type Out = FeltExpr;

    fn name(&self) -> String {
        STWO_COMPONENT_TYPE_SEQ.to_string()
    }

    fn call(&self, _air_builder: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            return FeltExpr::Var(VarExpr::new(
                self.name(),
                Some(Felt::from(row_number as u32)),
                true,
                true,
                None,
            ));
        }

        Self::Out::default()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }
}

#[derive(Debug, InstDef, Default)]
pub struct SeqAddr {}
impl ExtTable for SeqAddr {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_SEQ;
    type T = CasmAddress;
}
