pub mod framework_gen;
pub mod packed_types;
pub mod simd_prover_gen;
pub mod trace_gen;
pub mod utils;

#[cfg(test)]
mod tests {
    use air_infra::airs::examples::fibonacci::narrow_fib::NarrowFib;
    use air_infra::airs::examples::fibonacci::wide_fib::WideFib;
    use air_infra::core::air_fn::AirFn;

    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    fn generate_component_code(air_fn: &impl AirFn) {
        const COMPONENTS_DIR: &str = "../generated_components/src/";
        let folder_path = project_root().join(COMPONENTS_DIR);
        compare_contents_or_fix_with_path(air_fn, &folder_path);
    }

    // TODO(Ohad): consider moving these next to the corresponding infra code, when/if they are in a
    // separate crate.
    #[test]
    fn narrow_fib_gen() {
        let air_fn = NarrowFib { num_steps: 20 };
        generate_component_code(&air_fn);
    }

    #[test]
    fn wide_fib_code_gen() {
        let air_fn = WideFib {
            num_narrow: 8,
            narrow_size: 20,
        };
        generate_component_code(&air_fn);
    }
}
