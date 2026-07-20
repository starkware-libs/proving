use air_infra::core::expressions::felt252_expr::Felt252Expr;
use stwo_cairo_common::prover_types::cpu::Felt252;

pub type ECPoint = [Felt252Expr; 2];

#[derive(Clone)]
pub struct CurvePoint {
    pub x: Felt252,
    pub y: Felt252,
}

const THREE: Felt252 = Felt252 { limbs: [3, 0, 0, 0] };
const TWO: Felt252 = Felt252 { limbs: [2, 0, 0, 0] };
#[cfg(test)]
const ZERO: Felt252 = Felt252 { limbs: [0, 0, 0, 0] };

// The alpha parameter in the STARK elliptic curve: y^2 = x^3 + alpha * x + beta (mod P_252)
const CURVE_A: Felt252 = Felt252 { limbs: [1, 0, 0, 0] };

pub const P_SHIFT: CurvePoint = CurvePoint {
    x: Felt252 {
        limbs: [6133865585316620292, 8172638112721434419, 67021315040190230, 332954023760060423],
    },
    y: Felt252 {
        limbs: [
            15006096378035185290,
            5648094899004790465,
            17529932370477190867,
            273045013732771549,
        ],
    },
};

#[cfg(test)]
pub const P_0: CurvePoint = CurvePoint {
    x: Felt252 {
        limbs: [
            1189180620722136187,
            10351543583799438092,
            10852677322373114341,
            158796407618141823,
        ],
    },
    y: Felt252 {
        limbs: [7984924534557791765, 10664864617948026060, 2200172416265571897, 265807971118313394],
    },
};

#[cfg(test)]
pub const P_1: CurvePoint = CurvePoint {
    x: Felt252 {
        limbs: [
            13233426379687240568,
            11027519718089961496,
            4584008176878587699,
            358694723999251891,
        ],
    },
    y: Felt252 {
        limbs: [5866207792187422029, 6208038334140916660, 9301024774522029057, 286551992207264227],
    },
};

#[cfg(test)]
pub const P_2: CurvePoint = CurvePoint {
    x: Felt252 {
        limbs: [4225347253695244695, 4667577249380821263, 8523362424349652811, 340669115356057068],
    },
    y: Felt252 {
        limbs: [5194092000597189020, 14611301832847634044, 5446855563628374111, 18067299449795871],
    },
};

#[cfg(test)]
pub const P_3: CurvePoint = CurvePoint {
    x: Felt252 {
        limbs: [15235661236934115842, 3217347257548387109, 7945700668643894203, 379149940513229852],
    },
    y: Felt252 {
        limbs: [
            9772191581856343078,
            14886839300189935130,
            12918580351824604709,
            123703021930624260,
        ],
    },
};

#[cfg(test)]
pub fn ec_neg(p: &CurvePoint) -> CurvePoint {
    CurvePoint { x: p.x, y: ZERO - p.y }
}

pub fn ec_add(a: &CurvePoint, b: &CurvePoint) -> CurvePoint {
    let slope = if a.x == b.x {
        assert!(
            a.y == b.y,
            "Cannot add a point to its negation, the result will be the point at infinity"
        );
        (THREE * a.x * a.x + CURVE_A) / (TWO * a.y)
    } else {
        (b.y - a.y) / (b.x - a.x)
    };
    let result_x = slope * slope - a.x - b.x;
    let result_y = slope * (a.x - result_x) - a.y;
    CurvePoint { x: result_x, y: result_y }
}

pub fn ec_mul(p: &CurvePoint, mut k: usize) -> CurvePoint {
    assert!(k > 0);

    // The zero point doesn't have a representation as (x,y) pair. Therefore we initialize the
    // result accumulator with <p> and add <p> another <k-1> times.
    let mut result = p.clone();
    let mut shifted = p.clone();
    k -= 1;

    while k > 0 {
        if k & 1 == 1 {
            result = ec_add(&result, &shifted);
        }
        shifted = ec_add(&shifted, &shifted);
        k >>= 1;
    }

    result
}

// Combines an add and mul operations.
#[cfg(test)]
pub fn ec_add_mul(a: &CurvePoint, b: &CurvePoint, k: usize) -> CurvePoint {
    if k == 0 { a.clone() } else { ec_add(a, &ec_mul(b, k)) }
}

/// Compute `p` * (2 ** `amount`)
#[cfg(test)]
pub fn ec_shift(p: &CurvePoint, amount: usize) -> CurvePoint {
    let mut result = p.clone();
    for _ in 0..amount {
        result = ec_add(&result, &result);
    }
    result
}
