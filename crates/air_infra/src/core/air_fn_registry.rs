use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;

use super::air_fn::*;
use super::autogen_structs::*;
use super::state::*;
use super::variables::*;

// AirFnEntry describes everything we know about an Air function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirFnEntry {
    pub name: String,
    pub inst_def: BTreeMap<String, String>,
    pub input: GenericAirVar,
    pub output: GenericAirVar,
    pub air_body: Vec<AirBodyComponent>,
}

impl AirFnEntry {
    pub(super) fn new<I, O>(registry: &AirFnRegistry, air_fn: &dyn AirFn<In = I, Out = O>) -> Self
    where
        I: AirVar,
        O: AirVar,
    {
        let (air_builder, input, output) = registry.build_air(air_fn);
        let entry = Self {
            name: air_fn.name(),
            inst_def: air_fn.inst_def(),
            input: input.into(),
            output: output.into(),
            air_body: air_builder.air_body.clone(),
        };
        air_builder
            .registry
            .air_fns
            .borrow_mut()
            .insert(air_fn.name(), entry.clone());
        entry
    }
}

// AirFnRegistry is created for a specific air function. It keeps all the air function entries
// for the air function and its subroutines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirFnRegistry {
    pub air_fns: Rc<RefCell<BTreeMap<String, AirFnEntry>>>,
    #[serde(skip)]
    pub intermediate_vars_index: Rc<RefCell<usize>>,
}

impl AirFnRegistry {
    pub fn new<I, O>(air_fn: &dyn AirFn<In = I, Out = O>) -> (Self, AirFnEntry, AutogenLists)
    where
        I: AirVar,
        O: AirVar,
    {
        // Create the registry.
        let registry = Self {
            air_fns: Rc::new(RefCell::new(BTreeMap::new())),
            intermediate_vars_index: Rc::new(RefCell::new(0)),
        };

        // Add the function to the registry.
        let entry = AirFnEntry::new(&registry, air_fn);

        // Get the autogen lists.
        let lists = Self::get_autogen_lists(entry.air_body.clone(), entry.input.clone());
        (registry, entry, lists)
    }

    // Runs the air function on a given input and returns the resulting state and output.
    #[cfg(test)]
    pub fn run_air<I, O>(&self, air_fn: &dyn AirFn<In = I, Out = O>, input: I) -> (State, O)
    where
        I: AirVar,
        O: AirVar,
    {
        assert!(self.air_fns.borrow().get(&air_fn.name()).is_some());

        let mut air_builder = AirBuilder {
            state: State::default(),
            air_body: vec![],
            run: true,
            registry: self.clone(),
        };
        let output = air_fn.call(&mut air_builder, input);
        (air_builder.state, output)
    }

    // Builds the air function on a default input in order to create an air function entry for it.
    fn build_air<I, O>(&self, air_fn: &dyn AirFn<In = I, Out = O>) -> (AirBuilder, I, O)
    where
        I: AirVar,
        O: AirVar,
    {
        let mut input = I::new(format!("{}_input", air_fn.name()));
        let mut air_builder = AirBuilder {
            state: State::default(),
            air_body: vec![],
            #[cfg(test)]
            run: false,
            registry: self.clone(),
        };
        if air_fn.input_in_trace() {
            for felt_expr in input.as_felts() {
                air_builder.state.add(felt_expr);
            }
        }
        let output = air_fn.call(&mut air_builder, input.clone());
        (air_builder, input, output)
    }

    pub(super) fn get_intermediate_var_index(&self) -> String {
        let mut index = self.intermediate_vars_index.borrow_mut();
        let index_as_str = format!("{}", *index);
        *index += 1;
        index_as_str
    }

    // Dumps the registry to a file.
    pub fn dump_to_file(&self, file_name: &str) {
        let file = File::create(file_name).expect("Unable to create file");
        let mut writer = BufWriter::new(file);
        to_writer_pretty(&mut writer, self).expect("serialization failed");
        writer.flush().expect("flush failed");
        writer.write_all(b"\n").expect("write failed");
    }

    // Transforms the air body and input of an air function into the autogen format.
    fn get_autogen_lists(air_body: Vec<AirBodyComponent>, input: GenericAirVar) -> AutogenLists {
        let mut constraints = vec![];
        let mut deductions = vec![];

        for component in air_body {
            match component {
                AirBodyComponent::Constraint(constraint) => {
                    constraints.push(ConstraintOrIntermediate::Constraint(constraint.into()));
                }
                AirBodyComponent::Assignment {
                    constraint,
                    deduction,
                } => {
                    constraints.push(ConstraintOrIntermediate::Constraint(constraint.into()));
                    deductions.push(DeductionOrIntermediate::Deduction(deduction.into()));
                }
                AirBodyComponent::Deduction(deduction) => {
                    deductions.push(DeductionOrIntermediate::Deduction(deduction.into()));
                }
                AirBodyComponent::DeductionIntermediate(name, var) => {
                    deductions.push(DeductionOrIntermediate::Intermediate(name, var.into()));
                }
                AirBodyComponent::ConstraintIntermediate(name, var) => {
                    constraints.push(ConstraintOrIntermediate::Intermediate(name, var.into()));
                }
                AirBodyComponent::Subroutine(f) => {
                    let lists = Self::get_autogen_lists(f.air_body, f.input_arg);
                    constraints.extend(lists.constraints);
                    deductions.extend(lists.deductions);
                }
            }
        }

        AutogenLists {
            input: input.into(),
            constraints,
            deductions,
        }
    }
}
