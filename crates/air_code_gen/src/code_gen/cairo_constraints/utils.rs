use std::fs;

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, ExternalState, PaddingType, TraceType, UseOrYield,
};
use convert_case::{Case, Casing};
use eval_air_fn_constraints::SampleEvaluation;
use genco::lang::rust;
use genco::quote;
use indexmap::IndexMap;
use tempfile::tempdir;
use xshell::{cmd, Shell};

use super::component::generate_component_cairo_constraints_code;
use super::iniline_evaluate::generate_inline_cairo_constraints_code;
use crate::code_gen::utils::is_const_size_component;

pub const QM31_N_TRACE_CELLTS: usize = 4;

const MINIMAL_SCARB_TOML: &str = "[package]
name = \"scarb_fmt_testing\"
version = \"1.2.3\"
edition = \"2024_07\"
";

pub fn generate_cairo_constraints_code(
    air_fn: &CompiledAirFn,
    sample_evaluation: Option<&SampleEvaluation>,
) -> rust::Tokens {
    if air_fn.r#type == TraceType::Inline {
        generate_inline_cairo_constraints_code(air_fn)
    } else {
        generate_component_cairo_constraints_code(
            air_fn,
            &sample_evaluation
                .unwrap_or_else(|| panic!("Missing sample evaluation {}", air_fn.name))
                .assignment,
        )
    }
}

pub fn gen_consts(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut consts = rust::Tokens::new();

    if air_fn.r#type != TraceType::Inline {
        if has_enabler_or_mult_column(air_fn) {
            consts.extend(quote! {
                pub const N_TRACE_COLUMNS: usize = $(air_fn.state_names.len() + 1);
            });
        } else {
            consts.extend(quote! {
                pub const N_TRACE_COLUMNS: usize = $(air_fn.state_names.len());
            });
        }

        if !is_const_size_component(air_fn) {
            let uses = air_fn
                .constraint_lookups
                .iter()
                .filter(|(_, use_or_yield)| matches!(use_or_yield, UseOrYield::Use))
                .collect::<Vec<_>>();
            let mut uses_count = IndexMap::new();
            for (relation, _) in &uses {
                *(uses_count.entry(relation.clone()).or_insert(0)) += 1;
            }
            consts.extend(quote! {
                pub const RELATION_USES_PER_ROW: [(felt252, u32); $(uses_count.keys().len())] = [
                    $(uses_count.iter().map(|(relation, count)| {
                        format!(r#"('{}', {})"#, relation, count)
                    }).collect::<Vec<_>>().join(", "))
                ];
            });
        }
    }

    if is_const_size_component(air_fn) {
        consts.extend(quote! {
            const LOG_SIZE: u32 = $(air_fn.log_height);
        });
    }

    consts
}

pub fn gen_imports(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = quote! {
        use crate::prelude::*;
    };

    for (inline_fn, _) in &air_fn.inline_calls {
        code.append(quote! {
            use crate::components::subroutines::$(inline_fn)::$(inline_fn)_evaluate;
        });
    }

    code
}

/// Create an expression that evaluates to the log size of the current AirFn
/// air_fn - The AirFn
/// in_claim - If true, return code that works inside the generated `Claim` struct (where the type
/// of `self` is `Claim`). Otherwise, return code that works inside the generated `Eval` struct
/// (where the type of `self` is `Eval`).
pub fn get_log_size(air_fn: &CompiledAirFn, in_claim: bool) -> rust::Tokens {
    if is_const_size_component(air_fn) {
        quote! { LOG_SIZE }
    } else if in_claim {
        quote! { *(self.log_size) }
    } else {
        quote! { *(self.claim.log_size) }
    }
}

pub fn has_enabler_or_mult_column(air_fn: &CompiledAirFn) -> bool {
    // TODO(AnatG): Support both enabler and multiplicity columns in the same component.
    air_fn.padding_type == PaddingType::Enabler || air_fn.padding_type == PaddingType::Multiplicity
}

pub fn is_chain(air_fn: &CompiledAirFn) -> bool {
    // All components that are part of a chain.
    air_fn.r#type == TraceType::ChainRound || air_fn.r#type == TraceType::Opcode
}

pub fn n_logup_columns(air_fn: &CompiledAirFn) -> usize {
    const QM31_EXTENSION_DEGREE: usize = 4;

    let n_lookup_terms = air_fn.constraint_lookups.len();
    let n_batches = n_lookup_terms.div_ceil(2);

    QM31_EXTENSION_DEGREE * n_batches
}

pub fn make_preprocessed_column(
    external_state: &ExternalState,
    log_size_expr: &rust::Tokens,
) -> rust::Tokens {
    if external_state.name == "Seq" {
        quote! { seq_column_idx($(log_size_expr)) }
    } else {
        // TODO(adar): Once we represent external states using string IDs instead of
        // (name, generic_param, args) tuples, this should be just
        // quote! { preprocessed_columns::$(external_state.to_case(Case::Upper))_IDX }
        let generic_param = external_state
            .generic_param
            .map(|c| c.to_string())
            .unwrap_or_default();
        let args_str = external_state
            .args
            .join("_")
            .replace("[", "")
            .replace("]", "")
            .replace(",", "_");
        quote! { $(&external_state.name.to_case(Case::Constant))_$(generic_param)_$(args_str)_IDX }
    }
}

pub fn format_cairo_code(code_text: String) -> String {
    // Currently, `scarb fmt` requires the input to be from file, so we create a temporary
    // workspace with Scarb.toml and the code to format.
    let scarb_workspace = tempdir().unwrap();
    let scarb_workspace = scarb_workspace.path();

    let manifest_path = scarb_workspace.join("Scarb.toml");
    let manifest_path = manifest_path
        .to_str()
        .expect("Invalid temporary manifest path");
    fs::write(manifest_path, MINIMAL_SCARB_TOML).unwrap();

    let code_path = scarb_workspace.join("code.cairo");
    let code_path = code_path.to_str().expect("Invalid temporary manifest path");
    fs::write(code_path, code_text).unwrap();

    let shell = Shell::new().unwrap();
    let mut stdout = cmd!(
        shell,
        "scarb --manifest-path {manifest_path} fmt -e stdout {code_path}"
    )
    .ignore_status() // "scarb fmt" returns error code if the input file wasn't already formatted
    .read()
    .unwrap();

    if !stdout.ends_with('\n') {
        stdout.push('\n');
    }

    stdout
}

pub(super) fn lookup_elements_field(relation_name: &str) -> String {
    format!("{}_lookup_elements", relation_name.to_case(Case::Snake))
}
