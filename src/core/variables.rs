use std::fmt::Debug;

/// Every input and output of an air function is an AirVar.
pub trait AirVar: Clone + Debug + Default {
    fn new(name: String) -> Self;
    fn create_intermediate_var(&self, name: String) -> Self;
    fn name(&self) -> String;
    fn description(&self) -> String {
        self.name()
    }
    // Returns whether the value of this AirVar is stored in a trace cell.
    // For example, an input to an air function is not in state when it is from the private input.
    fn in_state(&self) -> bool;
}

// Implements AirVar for arrays and tuples of air vars.
#[macro_export]
macro_rules! impl_air_var {
    ( [$s:ty;$n:literal] ) => {
        impl AirVar for [$s;$n] where $s: AirVar
        {
            fn name(&self) -> String {
                format!("[{}]", self.iter().map(|s| s.name()).collect::<Vec<String>>().join(", "))
            }
            fn in_state(&self) -> bool {
                self.iter().all(|s| s.in_state())
            }
            fn create_intermediate_var(&self, name: String) -> Self {
                let mut res = self.clone();
                for (i, s) in res.iter_mut().enumerate() {
                    *s = s.create_intermediate_var(format!("{}_{}", name, i));
                }
                res
            }
            fn new(name: String) -> Self {
                from_fn(|i| <$s>::new(format!("{}_{}", name, i)))
            }
        }
    };

    (($($s:ident),+)) => {
        impl AirVar for ($($s),+) where $($s: AirVar),+
        {
            fn name(&self) -> String {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                format!("({})", vec![$($s.name(), )+].join(", "))
            }
            fn in_state(&self) -> bool {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                $($s.in_state() &&)+ true
            }
            fn create_intermediate_var(&self, name: String) -> Self {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                let mut i = 0;
                ($($s.create_intermediate_var(format!("{}_{}", name, { i += 1; i })),)+)
            }
            fn new(name: String) -> Self {
                let mut i = 0;
                ($(<$s>::new(format!("{}_{}", name, { i += 1; i })),)+)
            }
        }
    };
}
