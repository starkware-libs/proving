use std::path::PathBuf;

use xshell::{cmd, Shell};

pub fn project_root() -> PathBuf {
    std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
}

pub fn reformat_rust_code(code_text: String) -> String {
    // Since rustfmt is used with nightly features, it takes 2 runs to reach a fixed point.
    reformat_rust_code_inner(reformat_rust_code_inner(code_text))
}

pub fn reformat_rust_code_inner(code_text: String) -> String {
    let shell = Shell::new().unwrap();
    shell.set_var("RUSTUP_TOOLCHAIN", "nightly-2024-01-04");
    let rustfmt_toml = project_root().join("rustfmt.toml");
    let mut stdout = cmd!(shell, "rustfmt --config-path {rustfmt_toml}")
        .stdin(code_text)
        .read()
        .unwrap();
    if !stdout.ends_with('\n') {
        stdout.push('\n');
    }
    stdout
}

#[cfg(test)]
mod tests {
    use genco::lang::rust;
    use genco::quote;

    use super::reformat_rust_code;

    #[test]
    fn test_reformat_rust_code() {
        let mut code = rust::Tokens::new();
        code.extend(quote! {
            fn foo() {
                println!("Hello, world!");
            }
        });
        let code_text = reformat_rust_code(code.to_string().expect("Could not format Rust code."));
        println!("{}", code_text);
    }
}
