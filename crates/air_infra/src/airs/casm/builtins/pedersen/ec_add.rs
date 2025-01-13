use inst_def::InstDef;

use crate::airs::felt252_utils::add252::*;
use crate::airs::felt252_utils::div252::*;
use crate::airs::felt252_utils::mul252::*;
use crate::airs::felt252_utils::sub252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;

#[derive(Debug, InstDef)]
pub struct ECAdd {}

// Elliptic curve point addition.
// Assumes that the input felt252-s have range-checked limbs and range-checks
// the limbs of the result.
impl AirFn for ECAdd {
    type ExtIn = ();
    type In = [Felt252Expr; 4];
    type Out = [Felt252Expr; 2];

    fn call(&self, air_builder: &mut AirBuilder, _: (), [x1, y1, x2, y2]: Self::In) -> Self::Out {
        let x_diff = air_builder.call(&Sub252 {}, [x2.clone(), x1.clone()]);
        let x_sum = air_builder.call(&Add252 {}, [x2, x1.clone()]);
        let y_diff = air_builder.call(&Sub252 {}, [y2, y1.clone()]);
        let slope = air_builder.call(&Div252 {}, [y_diff, x_diff]);
        let slope_squared = air_builder.call(&Mul252 {}, [slope.clone(), slope.clone()]);
        let result_x = air_builder.call(&Sub252 {}, [slope_squared, x_sum]);

        // result_y = slope * (x1 - result_x) - y1.
        let tmp1 = air_builder.call(&Sub252 {}, [x1, result_x.clone()]);
        let tmp2 = air_builder.call(&Mul252 {}, [slope, tmp1]);
        let result_y = air_builder.call(&Sub252 {}, [tmp2, y1]);
        [result_x, result_y]
    }
}
