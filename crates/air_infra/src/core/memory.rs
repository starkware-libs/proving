use std::any::type_name;

use indexmap::IndexMap;

#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
#[cfg(test)]
use std::rc::Rc;

use super::air_fn::*;
use super::expressions::felt_expr::*;
use super::expressions::var_expr::*;
use super::variables::*;

#[cfg(test)]
use super::Felt;

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

    #[allow(unused_variables)]
    pub fn get_state_value_for_constraints(&self, key: &K) -> V {
        #[cfg(test)]
        if let Some(value) = self.get(key) {
            return value;
        }

        // Cannot use V::default() here because it will return a const value.
        // The name is not important, because value will be used only in constraints.
        let mut value = V::new("".to_string()).let_(
            "".to_string(),
            IntermediateType {
                in_constraints: true,
                in_deductions: false,
            },
        );
        let i_start = K::default().as_felts().len();
        for (i, felt) in value.as_felts_mut().into_iter().enumerate() {
            *felt = FeltExpr::Var(VarExpr {
                name: format!("state[{}]", i_start + i),
                value: None,
                is_const: false,
                parent: None,
                complex_or_felt: ComplexOrFelt::Felt(StateInfo::StateIndex(i_start + i)),
                intermediate_type: None,
            });
        }
        value
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
        TraceType::Const
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        let mut k = type_name::<K>().to_string();
        k = k.rfind("::").map(|i| k[i + 2..].to_string()).unwrap_or(k);
        k = k.replace('>', "");

        let mut v = type_name::<V>().to_string();
        v = v.rfind("::").map(|i| v[i + 2..].to_string()).unwrap_or(v);
        v = v.replace('>', "");

        [("".to_string(), k), ("".to_string(), v)].into()
    }
}
