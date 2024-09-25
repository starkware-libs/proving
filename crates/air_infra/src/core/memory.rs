use inst_def::InstDef;

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

// Memory is a simple key-value store that is passed to the relevant air builder functions.
#[derive(Clone, Debug, Default, InstDef)]
pub struct Memory<K, V>
where
    K: AirVar,
    V: AirVar,
{
    #[cfg(test)]
    #[instdef(skip)]
    pub(super) data: Rc<RefCell<HashMap<Vec<Felt>, V>>>,
    #[instdef(skip)]
    key_type: PhantomData<K>,
    #[instdef(skip)]
    value_type: PhantomData<V>,
}

impl<K, V> Memory<K, V>
where
    K: AirVar + Default,
    V: AirVar + Default,
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
    pub fn new_with_data(data: Vec<(K, V)>) -> Self {
        Self {
            data: Rc::new(RefCell::new(
                data.into_iter()
                    .map(|(k, v)| (k.to_values().expect("key has no values"), v))
                    .collect(),
            )),
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
    pub fn set(&self, key: K, value: V) {
        let actual_key = key.to_values().expect("key has no values");

        if !self.data.borrow().contains_key(&actual_key) {
            self.data.borrow_mut().insert(actual_key, value);
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

impl<K, V> AirFn for Memory<K, V>
where
    K: AirVar + Default,
    V: AirVar + Default,
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
        TraceType::Component
    }
}
