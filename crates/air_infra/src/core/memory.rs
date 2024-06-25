#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
#[cfg(test)]
use std::rc::Rc;

use super::air_fn::*;
#[cfg(test)]
use super::prover_types::*;
use super::variables::*;

// Air functions (or lookup air functions) that need to access memory should implement this trait.
// This will allow the registry or other external code to initialize the same memory for all related air functions.
pub trait MemoryAirFn {
    type K: AirVar;
    type V: AirVar;

    fn init_memory(&mut self, memory: &Memory<Self::K, Self::V>);
}

// Memory is a simple key-value store that is passed to the relevant air builder functions.
#[derive(Clone, Debug, Default)]
pub struct Memory<K, V>
where
    K: AirVar,
    V: AirVar,
{
    #[cfg(test)]
    pub(super) data: Rc<RefCell<HashMap<Vec<Felt>, V>>>,
    key_type: PhantomData<K>,
    value_type: PhantomData<V>,
}

impl<K, V> Memory<K, V>
where
    K: AirVar,
    V: AirVar,
{
    #[allow(unused)]
    pub(super) fn new() -> Self {
        Self {
            #[cfg(test)]
            data: Rc::new(RefCell::new(HashMap::new())),
            key_type: PhantomData,
            value_type: PhantomData,
        }
    }

    #[cfg(test)]
    pub(super) fn get(&self, key: &K) -> Option<V> {
        let actual_key = key.to_values();
        self.data.borrow().get(&actual_key).cloned()
    }

    #[cfg(test)]
    pub(super) fn set(&self, key: K, value: V) {
        assert!(!value.is_const());

        let actual_key = key.to_values();
        if !self.data.borrow().contains_key(&actual_key) {
            self.data.borrow_mut().insert(actual_key, value);
        } else {
            let v = self.data.borrow().get(&actual_key).cloned().unwrap();
            assert_eq!(v.to_values(), value.to_values());
        }
    }
}

impl<K, V> AirFn for Memory<K, V>
where
    K: AirVar,
    V: AirVar,
{
    type In = K;
    type Out = V;

    #[allow(unused_variables)]
    fn call(&self, air_builder: &mut AirBuilder, key: Self::In) -> Self::Out {
        #[cfg(test)]
        if air_builder.run {
            return self.get(&key).unwrap();
        }

        Self::Out::default()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }
}
