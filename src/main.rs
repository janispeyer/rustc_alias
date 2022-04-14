#![feature(rustc_private)]
#![feature(let_else)]

extern crate rustc_driver;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_mir_transform;
extern crate rustc_session;

mod alias;

use alias::Alias;
use rustc_driver::{catch_with_exit_code, Callbacks, RunCompiler};
use rustc_mir_transform::MIR_PASS_INJECTION;

use std::process::exit;

struct AliasCallbacks;

impl Callbacks for AliasCallbacks {}

fn find_sysroot() -> String {
    // Taken from https://github.com/rust-lang/miri/blob/84b058ac47e2ea5f29887a4ed5ae286e37b22194/benches/helpers/miri_helper.rs
    // Taken from https://github.com/rust-lang/rust-clippy/pull/911
    let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
    let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
    match (home, toolchain) {
        (Some(home), Some(toolchain)) => format!("{}/toolchains/{}", home, toolchain),
        _ => option_env!("RUST_SYSROOT")
            .expect("need to specify RUST_SYSROOT env var or use rustup or multirust")
            .to_owned(),
    }
}

/// Injects [Alias] as a pass into rustc_mir_transform optimisations.
fn inject_alias_pass() {
    let mut injection = MIR_PASS_INJECTION.lock().unwrap();
    *injection = Some(Box::new(Alias));
}

fn compiler_args() -> Vec<String> {
    let mut compiler_args = Vec::new();
    let mut _analysis_args = Vec::new();
    for arg in std::env::args() {
        if arg.starts_with("--analysis") {
            _analysis_args.push(arg);
        } else {
            compiler_args.push(arg);
        }
    }
    // compiler_args.push("-Zalways-encode-mir".to_owned());
    // compiler_args.push("-Zcrate-attr=feature(register_tool)".to_owned());
    // compiler_args.push("-Zcrate-attr=register_tool(analyzer)".to_owned());

    // Needed sysroot because otherwise it results in the following error: error[E0463]: can't find crate for `std`
    // https://github.com/rust-lang/rust/issues/11792#issuecomment-33296069
    compiler_args.push("--sysroot".to_string());
    compiler_args.push(find_sysroot());

    compiler_args
}

fn run_compiler(compiler_args: &[String]) -> i32 {
    catch_with_exit_code(|| RunCompiler::new(compiler_args, &mut AliasCallbacks).run())
}

/// Run an analysis by calling it like rustc.
///
/// Give arguments to the analyser by prefixing them with '--analysis'.
/// All other arguments are directly forwarded to rustc.
///
/// Source: https://github.com/viperproject/prusti-dev/blob/8678e8faf214768535677c72b4f50b22abc6b83b/analysis/src/bin/analysis-driver.rs
fn main() {
    env_logger::init();
    rustc_driver::init_rustc_env_logger();

    inject_alias_pass();

    let compiler_args = compiler_args();
    let exit_code = run_compiler(&compiler_args);
    exit(exit_code)
}
