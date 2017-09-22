mod compiletest;

#[test]
fn ui() {
    compiletest::run(compiletest::Mode::Ui);
}
