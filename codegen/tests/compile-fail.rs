extern crate compiletest_rs as compiletest;

use std::path::PathBuf;
use compiletest::common::Mode;

fn run_mode(mode: Mode) {
    let mut config = compiletest::Config::default();
    config.mode = mode;
    config.src_base = PathBuf::from(format!("tests/{}", mode));

    let flags = [
        "-L crate=../target/debug/",
        "-L dependency=../target/debug/deps/",
    ].join(" ");

    config.target_rustcflags = Some(flags);
    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode(Mode::CompileFail);
}
