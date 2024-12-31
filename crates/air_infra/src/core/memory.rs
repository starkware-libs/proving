#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
#[cfg(test)]
use std::rc::Rc;

use super::air_fn::*;
use super::variables::*;
#[cfg(test)]
use super::Felt;

/// Describes an AirFn that is a memory component. This means:
/// 1. Each trace row contains a key (of type K), a value (of type V) and nothing else
/// 2. Writing the keys and values into the trace is not the responsibility of the AIR infra, but is
///    implemented directly in the prover. Therefore, call() doesn't write anything to the trace.
pub trait IsMemory<K, V>: AirFn<ExtIn = K, In = (), Out = V>
where
    K: ExtTable,
    V: AirVar,
{
    // Return the Memory object that contains the key-value pairs for this component
    fn mem(&self) -> &Memory<K::T, V>;

    fn mem_mut(&mut self) -> &mut Memory<K::T, V>;
}

/// Implements the common logic for all IsMemory components
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
    pub fn get(&self, key: &K) -> Option<V> {
        let actual_key = key.to_values();
        actual_key.and_then(|k| self.data.borrow().get(&k).cloned())
    }

    #[cfg(test)]
    pub fn set(&mut self, key: K, value: V) {
        let actual_key = key.to_values().expect("key has no values");
        assert!(value.is_const());

        if !self.data.borrow().contains_key(&actual_key) {
            self.data.borrow_mut().insert(actual_key.clone(), value);
        } else {
            let v = self.data.borrow().get(&actual_key).cloned().unwrap();
            assert!(
                v.to_values() == value.to_values(),
                "Memory::set() failed for key {:?}- given value != value in memory",
                actual_key
            );
        }
    }
}
