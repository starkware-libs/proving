//! Runs a fibonacci PIE under the leaf simple bootloader and prints its output and preimage dump.

use std::error::Error;
use std::path::{Path, PathBuf};

use cairo_program_runner_lib::types::{
    HashFunc, LeafSimpleBootloaderInput, PrivacySimpleBootloaderInput, RunMode,
};
use cairo_program_runner_lib::{
    ProgramInput, SimpleBootloaderInput, Task, TaskSpec, cairo_run_program,
};
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::types::program::Program;
use stwo_cairo_utils::file_utils::repo_root;

fn prepare_fibonnaci_pie_task_spec(project_dir: &Path) -> Result<TaskSpec, Box<dyn Error>> {
    let fibonacci_compiled_program_path = project_dir.join("examples/fibonacci.json");
    let fibonnaci_program =
        Program::from_file(fibonacci_compiled_program_path.as_path(), Some("main"))?;

    let cairo_run_config =
        RunMode::Validation { layout: LayoutName::small, allow_missing_builtins: false }
            .create_config();
    let runner = cairo_run_program(&fibonnaci_program, None, cairo_run_config, None)?;
    let result_pie = runner.get_cairo_pie()?;
    let pie_task: Task = Task::Pie(result_pie);
    Ok(TaskSpec { task: pie_task.into(), program_hash_function: HashFunc::Blake })
}

fn main() -> Result<(), Box<dyn Error>> {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let pie_task_spec = prepare_fibonnaci_pie_task_spec(&project_dir)?;
    let preimage_dump = tempfile::NamedTempFile::new()?;
    let leaf_bootloader_input = LeafSimpleBootloaderInput {
        privacy_input: PrivacySimpleBootloaderInput {
            simple_bootloader_input: SimpleBootloaderInput {
                fact_topologies_path: None,
                single_page: true,
                tasks: vec![pie_task_spec],
            },
            output_preimage_dump_path: preimage_dump.path().to_path_buf(),
        },
        output_hash_func: HashFunc::Blake,
    };
    let leaf_bootloader_program = Program::from_file(
        &repo_root().join("compiled_cairo_programs/leaf_simple_bootloader_compiled.json"),
        Some("main"),
    )?;
    let cairo_run_config = RunMode::Proof {
        layout: LayoutName::all_cairo_stwo,
        dynamic_layout_params: None,
        disable_trace_padding: false,
        relocate_mem: true,
    }
    .create_config();

    let mut runner = cairo_run_program(
        &leaf_bootloader_program,
        Some(ProgramInput::Value(Box::new(leaf_bootloader_input))),
        cairo_run_config,
        None,
    )?;

    let mut output_buffer = "Program Output:\n".to_string();
    runner.vm.write_output(&mut output_buffer)?;
    print!("{output_buffer}");
    println!("Output preimage: {}", std::fs::read_to_string(preimage_dump.path())?);

    Ok(())
}
