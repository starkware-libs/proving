use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;
#[cfg(any(test, feature = "test"))]
use std::fmt::Display;
use std::rc::Rc;

use super::air_fn::*;
use super::expressions::felt_expr::*;
use super::variables::*;
use crate::const_expr;

/// The "context" that we carry while running / building a component
/// This is passed to, and updated by, all inline AirFns called by the component
#[derive(Clone, Debug)]
pub struct ComponentContext {
    // The state of the component. Contains all the values deduced so far.
    state: Rc<RefCell<State>>,
    // For each chain round component, how many times this component was called so far. Used to
    // assign each call a unique ID in chain_lookup_call.
    chain_call_counts: Rc<RefCell<HashMap<String, usize>>>,
    // For each chain round component, an intermediate for seq * number of chains. Used to assign
    // each call a unique ID in chain_lookup_call.
    chain_call_intermediates: Rc<RefCell<HashMap<String, FeltExpr>>>,
    pub enabler: FeltExpr,
}

impl ComponentContext {
    pub fn state(&self) -> Ref<'_, State> {
        self.state.borrow()
    }

    pub fn state_mut(&self) -> RefMut<'_, State> {
        self.state.borrow_mut()
    }

    // Returns a unique index for a call to the given chain round component.
    //
    // Each call returns a new index. For each chain round, the indices are sequential
    // and start from 0.
    pub fn get_chain_call_index<S>(&mut self, called_round: &dyn ChainRoundAirFn<S>) -> usize
    where
        S: AirVar,
        (ChainIdVar, RoundNumVar, S): AirVar,
    {
        let mut chain_call_counts = self.chain_call_counts.borrow_mut();
        let value_in_map = chain_call_counts.entry(called_round.name()).or_insert(0);
        let current_count = *value_in_map;
        assert!(
            current_count < called_round.number_of_chains(),
            "Chain round {} called more than its declared number_of_chains ({})",
            called_round.name(),
            called_round.number_of_chains()
        );
        *value_in_map = current_count + 1;
        current_count
    }

    pub fn get_chain_call_intermediate<S>(
        &self,
        called_round: &dyn ChainRoundAirFn<S>,
    ) -> Option<FeltExpr>
    where
        S: AirVar,
        (ChainIdVar, RoundNumVar, S): AirVar,
    {
        let chain_call_intermediates = self.chain_call_intermediates.borrow();
        chain_call_intermediates.get(&called_round.name()).cloned()
    }

    pub fn set_chain_call_intermediate<S>(
        &self,
        called_round: &dyn ChainRoundAirFn<S>,
        var: FeltExpr,
    ) where
        S: AirVar,
        (ChainIdVar, RoundNumVar, S): AirVar,
    {
        let mut chain_call_intermediates = self.chain_call_intermediates.borrow_mut();
        chain_call_intermediates.insert(called_round.name(), var);
    }
}

impl Default for ComponentContext {
    fn default() -> Self {
        Self {
            state: Default::default(),
            chain_call_counts: Default::default(),
            chain_call_intermediates: Default::default(),
            enabler: const_expr!(1),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StateCell(FeltExpr, Option<String>);

#[derive(Clone, Debug, Default)]
pub struct State {
    row: Vec<StateCell>,
}

impl State {
    pub(super) fn add(&mut self, expr: &mut FeltExpr, desc: &str) {
        let len = self.row.len();
        let desc = (!desc.is_empty()).then(|| desc.to_string());
        expr.set_value(ValueInfo::StateIndex(len, desc.clone()));
        self.row.push(StateCell(expr.clone(), desc));
    }

    pub fn get_felts(&self) -> Vec<FeltExpr> {
        self.row.iter().map(|cell| cell.0.clone()).collect()
    }

    pub fn get_cell_name(index: usize, desc: &Option<String>) -> String {
        match desc {
            Some(desc) => format!("{desc}_{STATE_VAR_SUFFIX}{index}"),
            None => format!("{STATE_VAR_SUFFIX}{index}"),
        }
    }

    pub fn get_state_names(&self) -> Vec<String> {
        self.row.iter().enumerate().map(|(i, cell)| Self::get_cell_name(i, &cell.1)).collect()
    }

    #[cfg(any(test, feature = "test"))]
    pub fn is_empty(&self) -> bool {
        self.row.is_empty()
    }
}

#[cfg(any(test, feature = "test"))]
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for cell in self.row.iter() {
            s.push_str(&format!(
                "({}, \"{}\"),\n",
                cell.0.calc(),
                cell.1.clone().unwrap_or_default()
            ));
        }
        write!(f, "{s}")
    }
}
