use super::expressions::felt_expr::*;
use super::prover_types::*;
use super::state::*;
use super::variables::*;
// Macros
use crate::expr;

#[test]
fn test_state_elements() {
    let state = State::default();
    for x in [1, 2, 3] {
        let mut e = expr!("x", x);
        state.add(&mut e);

        assert!(e.in_state());
    }
    assert!(state.calc() == ["1", "2", "3"]);
}
