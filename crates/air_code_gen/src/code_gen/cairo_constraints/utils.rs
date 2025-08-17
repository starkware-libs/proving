use std::fs;
use std::path::Path;

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, ExternalState, PaddingType, TraceType, UseOrYield,
};
use genco::lang::rust;
use genco::quote;
use indexmap::IndexMap;

use super::component::generate_component_cairo_constraints_code;
use super::iniline_evaluate::generate_inline_cairo_constraints_code;
use crate::code_gen::utils::get_constraints_folder_path_suffix;

pub const QM31_N_TRACE_CELLTS: usize = 4;

pub fn dump_cairo_constraints_code(air_fn: &CompiledAirFn, path: &Path) {
    let cairo_code = generate_cairo_constraints_code(air_fn);
    let file_name = &format!("{}.cairo", air_fn.name);
    let suffix = get_constraints_folder_path_suffix(&air_fn.r#type, file_name);
    let path = path.join(suffix);
    fs::write(
        path.clone(),
        cairo_code
            .to_string()
            .expect("Unable to covert cairo code to string"),
    )
    .expect("Unable to write cairo code to file");
}

pub fn generate_cairo_constraints_code(air_fn: &CompiledAirFn) -> rust::Tokens {
    if air_fn.r#type == TraceType::Inline {
        generate_inline_cairo_constraints_code(air_fn)
    } else {
        generate_component_cairo_constraints_code(air_fn)
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
                .lookup_names
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
        let ExternalState {
            name,
            generic_param,
            args,
        } = air_fn.external_states.get_index(0).expect(
            "We assume that const-size components include at least one preprocessed column",
        );
        consts.extend(quote! {
            const SOME_COLUMN: PreprocessedColumn = PreprocessedColumn::$(name)$(*generic_param)(($(args.join(", "))));
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
        // For constant-size components, we don't have an easy way to know the
        // number of rows (it doesn't appear in the CompiledAirFn).
        // Therefore we rely on the `log_size` function that the verifier
        // implements for constant columns, and take the size of one of our
        // const columns (doesn't matter which, as component columns all
        // have the same size).
        //
        // We cannot have the size itself as a constant because Cairo doesn't
        // allow `PreprocessedColumn::SomeColumn(...).log_size()` as a constant
        // expression, so we store just the column as a constant and call
        // `.log_size()` every time.
        quote! { SOME_COLUMN.log_size() }
    } else if in_claim {
        quote! { *(self.log_size) }
    } else {
        quote! { *(self.claim.log_size) }
    }
}

// Currently, all components that have multiplicity other than verify instruction are of const size.
pub fn is_const_size_component(air_fn: &CompiledAirFn) -> bool {
    air_fn.padding_type == PaddingType::Multiplicity && air_fn.name != "verify_instruction"
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

    let n_lookup_terms = air_fn.lookup_names.len();
    let n_batches = n_lookup_terms.div_ceil(2);

    QM31_EXTENSION_DEGREE * n_batches
}

pub fn make_preprocessed_column(
    air_fn: &CompiledAirFn,
    external_state: &ExternalState,
) -> rust::Tokens {
    if external_state.name == "Seq" {
        quote! { PreprocessedColumn::Seq($(get_log_size(air_fn, false))) }
    } else {
        let generic_param = external_state
            .generic_param
            .map(|c| c.to_string())
            .unwrap_or_default();
        quote! { PreprocessedColumn::$(&external_state.name)$(generic_param)(($(external_state.args.join(", ")))) }
    }
}
