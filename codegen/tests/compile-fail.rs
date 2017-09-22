mod compiletest;

#[test]
fn compilefail() {
    compiletest::run(compiletest::Mode::CompileFail);
}
