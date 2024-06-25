use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::variables::*;

/// Reads from memory a felt252, writes the lower <num_limbs> limbs to the trace
/// and constrains the rest to be zeros (so the felt252 has value < 2**(12*num_limbs)).
/// Returns the felt252.
#[derive(Debug)]
pub struct ReadSmallFelt252 {
    pub num_limbs: usize,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for ReadSmallFelt252 {
    type In = FeltExpr;

    type Out = Felt252Expr;

    fn call(
        &self,
        air_builder: &mut crate::core::air_fn::AirBuilder,
        address: Self::In,
    ) -> Self::Out {
        let mut value_from_memory = air_builder.get_from_memory(&self.memory, &address);
        let mut felts = value_from_memory.as_felts_mut();
        let mut expected_nonzero_limbs: Vec<FeltExpr> = vec![];

        for felt in felts.iter_mut().take(self.num_limbs) {
            expected_nonzero_limbs.push(air_builder.deduce(felt));
        }

        let expected_value = Felt252Expr::from(expected_nonzero_limbs);
        air_builder.set_in_memory(&self.memory, address, expected_value.clone());
        expected_value
    }

    fn inst_def(&self) -> std::collections::BTreeMap<String, String> {
        [("num_limbs".to_string(), self.num_limbs.to_string())].into()
    }
}
