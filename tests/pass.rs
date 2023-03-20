#[test]
fn pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/01_find_function.rs");
    t.pass("tests/pass/02_default_argument_hygienic.rs");
}
