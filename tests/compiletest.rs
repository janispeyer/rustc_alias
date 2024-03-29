extern crate compiletest_rs as compiletest;

mod utils;

use std::{env, path::PathBuf};
use utils::*;

/// Taken from: https://github.com/viperproject/prusti-dev/blob/master/analysis/tests/test_analysis.rs
fn run_tests(mode: &str, path: &str, custom_args: Vec<String>) {
    let mut config = compiletest::Config::default();

    let mut flags = Vec::new();
    flags.push("--edition 2021".to_owned());
    // flags.push(format!("--sysroot {}", find_sysroot()));
    flags.extend(custom_args.into_iter());
    config.target_rustcflags = Some(flags.join(" "));

    config.mode = mode.parse().expect("Invalid mode");
    config.rustc_path = find_compiled_executable("rustc_alias");
    config.src_base = PathBuf::from("tests").join(path);
    assert!(config.src_base.is_dir());

    // Filter the tests to run
    if let Some(filter) = env::args().nth(1) {
        config.filters.push(filter);
    }

    if let Some(lib_path) = option_env!("RUSTC_LIB_PATH") {
        config.run_lib_path = PathBuf::from(lib_path);
        config.compile_lib_path = PathBuf::from(lib_path);
    }

    // config.bless = true;

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_tests("ui", "ui", Vec::new()); // mir-opt does not seem to work properly
}
