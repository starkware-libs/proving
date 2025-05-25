use std::fs;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

use compiled_casm_air::compiled_structs::{CompiledAirFn, PaddingType, TraceType};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;

use crate::code_gen::cairo_constraints::component::generate_cairo_constraints_code;
use crate::code_gen::utils::{get_constraints_folder_path_suffix, project_root};

pub fn get_git_rev(directory: &Path) -> String {
    let git_show_output = Command::new("git")
        .args([
            "-C",
            directory
                .to_str()
                .expect("The directory should be valid UTF-8"),
            "describe",
            "--always",
            "--dirty",
        ])
        .output()
        .unwrap();
    from_utf8(git_show_output.stdout.as_slice())
        .expect("Git output is valid UTF-8")
        .trim()
        .to_string()
}

pub fn dump_component_cairo_constraints_code(air_fn: &CompiledAirFn) {
    const CONSTRAINTS_DIR: &str = "../code_gen_regression/cairo_air/src/components";
    let cairo_code = generate_cairo_constraints_code(air_fn);
    let file_name = &format!("{}.cairo", air_fn.name);
    let suffix = get_constraints_folder_path_suffix(&air_fn.r#type, file_name);
    let path = project_root().join(CONSTRAINTS_DIR).join(suffix);
    fs::write(path.clone(), cairo_code.to_string().unwrap()).unwrap();
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
    }

    if is_const_size_component(air_fn) {
        consts.extend(quote! {
            pub const LOG_SIZE: u32 = $(air_fn.name.to_case(Case::Constant))_LOG_SIZE;
        });
    }

    consts
}

pub fn gen_imports(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    code.append(quote! {
        use core::num::traits::Zero;
        use stwo_constraint_framework::{
            PreprocessedColumn, PreprocessedColumnSet, PreprocessedMaskValues, PreprocessedMaskValuesImpl,
            PreprocessedColumnSetImpl, LookupElementsImpl,
        };
        use stwo_verifier_core::circle::CirclePointQM31AddCirclePointM31Trait;
        use stwo_verifier_core::circle::CirclePointIndexTrait;
        use stwo_verifier_core::channel::{Channel, ChannelTrait};
        use stwo_verifier_core::circle::CirclePoint;
        use stwo_verifier_core::fields::Invertible;
        use stwo_verifier_core::fields::m31::{m31, M31};
        use stwo_verifier_core::fields::qm31::{qm31_const, QM31, QM31Impl, QM31Serde, QM31Zero};
        use stwo_verifier_core::poly::circle::CanonicCosetImpl;
        use stwo_verifier_core::utils::{ArrayImpl, pow2};
        use stwo_verifier_core::{ColumnArray, ColumnSpan, TreeArray};
        use crate::components::CairoComponent;
        use crate::utils::U32Impl;
    });

    if is_const_size_component(air_fn) {
        code.append(quote! {
            use crate::components::$(air_fn.name.to_case(Case::Constant))_LOG_SIZE;
        });
    }

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

// Currently, all components that have multiplicity other than verify instruction are of const size.
pub fn is_const_size_component(air_fn: &CompiledAirFn) -> bool {
    air_fn.padding_type == PaddingType::Multiplicity && air_fn.name != "verify_instruction"
}

pub fn has_enabler_or_mult_column(air_fn: &CompiledAirFn) -> bool {
    // TODO(AnatG): Support both enabler and multiplicity columns in the same component.
    air_fn.padding_type == PaddingType::Enabler || air_fn.padding_type == PaddingType::Multiplicity
}

pub fn n_logup_columns(air_fn: &CompiledAirFn) -> usize {
    const QM31_EXTENSION_DEGREE: usize = 4;

    let n_lookup_terms = air_fn.lookup_names.len();
    let n_batches = n_lookup_terms.div_ceil(2);

    QM31_EXTENSION_DEGREE * n_batches
}
