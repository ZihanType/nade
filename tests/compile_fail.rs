#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/01_not_find_function_error.rs");
    t.compile_fail("tests/compile_fail/02_default_argument_unhygienic_error.rs");
}
