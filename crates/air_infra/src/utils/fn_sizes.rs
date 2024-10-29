use compiled_casm_air::compiled_structs::{CompiledAirFn, ConstraintEvalStep, TraceGenStep};

use crate::airs::casm::opcodes::call_opcode::*;
use crate::airs::casm::opcodes::jump_opcode::*;
use crate::airs::casm::opcodes::ret_opcode::*;
use crate::core::air_fn_registry::*;

fn print_statistics(air_fn_name: &str, compiled_fn: CompiledAirFn) {
    let mut trace_cells_used = 0;
    let mut num_regular_constraints = 0;
    let mut num_lookup_constraints = 0;

    for constraint in compiled_fn.constraints {
        match constraint {
            ConstraintEvalStep::Constraint(_, _, _) => num_regular_constraints += 1,
            ConstraintEvalStep::LookupData(_) => {
                num_lookup_constraints += 1;
            }
            ConstraintEvalStep::Intermediate(_, _) => {}
            ConstraintEvalStep::StartBlock(_) => {}
            ConstraintEvalStep::EndBlock => {}
        }
    }

    for deduction in compiled_fn.deductions {
        match deduction {
            TraceGenStep::Deduction(_) => trace_cells_used += 1,
            TraceGenStep::Intermediate(_, _) => {}
            TraceGenStep::LookupCall {
                fn_name: _,
                input: _,
                output_name: _,
            } => {}
            TraceGenStep::StartBlock(_) => {}
            TraceGenStep::EndBlock => {}
            TraceGenStep::LookupData(_) => {}
        }
    }

    println!(
        "{}: {} trace cells, {} constraints (out of them, {} lookups)",
        air_fn_name,
        trace_cells_used,
        num_regular_constraints + num_lookup_constraints,
        num_lookup_constraints
    )
}

pub fn print_fn_sizes() {
    let mut registry = AirFnRegistry::new_empty();
    let func = CallOpcode {
        is_rel: false,
        op1_base_fp: false,
        memory: Default::default(),
    };
    let compiled = registry.add_entry(&func).compile();
    print_statistics("call abs [ap]", compiled);

    let func = CallOpcode {
        is_rel: true,
        op1_base_fp: false,
        memory: Default::default(),
    };
    let compiled = registry.add_entry(&func).compile();
    print_statistics("call rel [ap]", compiled);

    let func = JumpOpcode {
        is_rel: false,
        is_imm: false,
        is_double_deref: false,
        memory: Default::default(),
    };
    let compiled = registry.add_entry(&func).compile();
    print_statistics("jump abs [ap/fp]", compiled);

    let func = JumpOpcode {
        is_rel: true,
        is_imm: true,
        is_double_deref: false,
        memory: Default::default(),
    };
    let compiled = registry.add_entry(&func).compile();
    print_statistics("jump rel [ap]", compiled);

    let func = RetOpcode {
        memory: Default::default(),
    };
    let compiled = registry.add_entry(&func).compile();
    print_statistics("ret", compiled);
}
